use async_stream::stream;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use tokio::sync::mpsc;
use std::pin::Pin;
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use stream_cancel::{StreamExt as _, Tripwire};
use tokio::time;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use uuid::Uuid;
use futures::future::FutureExt;

use crate::ecam::{Ecam, EcamError, EcamOutput};
use crate::packet::{self, packetize};

const SERVICE_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000300);
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000301);

/// The concrete peripheral type to avoid going crazy here managaing an unsized trait.
type Peripheral = <Adapter as Central>::Peripheral;

pub struct EcamBT {
    peripheral: Peripheral,
    characteristic: Characteristic,
    notifications: Pin<Box<dyn Stream<Item = EcamOutput> + Send + Sync>>,
}

impl EcamBT {
    /// Send a packet to the ECAM
    pub async fn send(self: &Self, data: Vec<u8>) -> Result<(), EcamError> {
        Result::Ok(
            self.peripheral
                .write(
                    &self.characteristic,
                    &packetize(&data),
                    btleplug::api::WriteType::WithoutResponse,
                )
                .await?,
        )
    }

    /// Create a stream that outputs the packets from the ECAM
    pub async fn stream(self: &Self) -> Result<impl Stream<Item = Vec<u8>> + Send, EcamError> {
        self.peripheral.subscribe(&self.characteristic).await?;
        let peripheral = self.peripheral.clone();
        let (trigger, tripwire) = Tripwire::new();
        tokio::spawn(async move {
            while peripheral.is_connected().await.unwrap_or_default() {}
            drop(trigger);
        });
        // Trim the header and CRC
        Result::Ok(
            self.peripheral
                .notifications()
                .await?
                .map(|m| m.value[2..m.value.len() - 2].to_vec())
                .take_until_if(tripwire),
        )
    }
}

impl Ecam for EcamBT {
    fn read(self: &Self) -> Pin<Box<dyn std::future::Future<Output = Result<Option<EcamOutput>, EcamError>> + Send>> {
        let x = self.notifications.get_mut().expect("mutex failure").next();
        let y = x.map(|x| x);
        unimplemented!()
    }

    fn send(self: &Self, data: Vec<u8>) -> Pin<Box<dyn std::future::Future<Output = Result<(), EcamError>> + Send>> {
        // Box::pin(self.send(data))
        unimplemented!()
    }
}

pub async fn get_ecam() -> Result<EcamBT, EcamError> {
    let manager = Manager::new().await?;
    get_ecam_from_manager(&manager).await
}

async fn get_notifications_from_peripheral(peripheral: &Peripheral, characteristic: &Characteristic) -> Result<impl Stream<Item = EcamOutput> + Send + Sync, EcamError> {
    peripheral.subscribe(characteristic).await?;
    let peripheral2 = peripheral.clone();
    let (trigger, tripwire) = Tripwire::new();
    tokio::spawn(async move {
        while peripheral2.is_connected().await.unwrap_or_default() {}
        drop(trigger);
    });

    // Use a forwarding task to make this stream Sync
    let mut n = peripheral.notifications().await?.take_until_if(tripwire);
    let (tx, mut rx) = mpsc::channel(100);
    tokio::spawn(async move {
        while let Some(m) = n.next().await {
            tx.send(m).await.expect("Failed to forward notification");
        }
    });

    // Then yield the rest
    Result::Ok(stream! {
        yield EcamOutput::Ready;
        while let Some(m) = rx.recv().await {
            yield EcamOutput::Packet(m.value[2..m.value.len() - 2].to_vec());
        }
        yield EcamOutput::Done;
    })
}

async fn get_ecam_from_manager(manager: &Manager) -> Result<EcamBT, EcamError> {
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        return Result::Err(EcamError::NotFound);
    }

    for adapter in adapter_list.iter() {
        let res = get_ecam_from_adapter(adapter).await?;
        if let Some((p, c)) = res {
            let n = Box::pin(get_notifications_from_peripheral(&p, &c).await?);
            return Ok(EcamBT {
                peripheral: p,
                characteristic: c,
                notifications: n,
            });
        }
    }

    Result::Err(EcamError::NotFound)
}

async fn get_ecam_from_adapter(
    adapter: &Adapter,
) -> Result<Option<(Peripheral, Characteristic)>, EcamError> {
    println!("Starting scan on {}...", adapter.adapter_info().await?);
    let filter = ScanFilter {
        services: vec![SERVICE_UUID],
    };
    adapter
        .start_scan(filter)
        .await
        .expect("Can't scan BLE adapter for connected devices...");
    time::sleep(Duration::from_secs(10)).await;
    let peripherals = adapter.peripherals().await?;
    for peripheral in peripherals.iter() {
        let r = validate_peripheral(peripheral).await?;
        if let Some(characteristic) = r {
            return Result::Ok(Some((peripheral.clone(), characteristic.clone())));
        }
    }

    Result::Err(EcamError::NotFound)
}

async fn validate_peripheral(peripheral: &Peripheral) -> Result<Option<Characteristic>, EcamError> {
    let properties = peripheral.properties().await?;
    let is_connected = peripheral.is_connected().await?;
    let properties = properties.unwrap();
    if let Some(local_name) = properties.local_name {
        println!(
            "Peripheral {:?} is connected: {:?}",
            local_name, is_connected
        );
        if !is_connected {
            println!("Connecting to peripheral {:?}...", &local_name);
            peripheral.connect().await?
        }
        let is_connected = peripheral.is_connected().await?;
        println!(
            "Now connected ({:?}) to peripheral {:?}...",
            is_connected, &local_name
        );
        peripheral.discover_services().await?;
        for service in peripheral.services() {
            for characteristic in service.characteristics {
                if characteristic.uuid == CHARACTERISTIC_UUID {
                    return Result::Ok(Some(characteristic));
                }
            }
        }
        return Result::Ok(None);
    }
    Result::Ok(None)
}
