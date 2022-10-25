use async_stream::stream;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, CharPropFlags};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::future::FutureExt;
use tokio::sync::Mutex;
use std::pin::Pin;
use std::result::Result;
use std::sync::{Arc};
use std::time::Duration;
use stream_cancel::{StreamExt as _, Tripwire};
use tokio::sync::mpsc::{self, Receiver};
use tokio::time;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::command::Response;
use crate::ecam::{Ecam, EcamError, EcamOutput};
use crate::packet::{self, packetize};

const SERVICE_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000300);
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000301);

/// The concrete peripheral type to avoid going crazy here managaing an unsized trait.
type Peripheral = <Adapter as Central>::Peripheral;

pub struct EcamBT {
    peripheral: Peripheral,
    characteristic: Characteristic,
    notifications: Arc<Mutex<Pin<Box<Receiver<EcamOutput>>>>>,
}

impl EcamBT {
    /// Send a packet to the ECAM
    pub async fn send(self: &Self, data: Vec<u8>) -> Result<(), EcamError> {
        let (peripheral, characteristic) 
            = (self.peripheral.clone(), self.characteristic.clone());
        Result::Ok(
            peripheral
                .write(
                    &characteristic,
                    &packetize(&data),
                    btleplug::api::WriteType::WithoutResponse,
                )
                .await?,
        )
    }

    /// Create a stream that outputs the packets from the ECAM
    pub async fn stream(self: &Self) -> Result<impl Stream<Item = Vec<u8>> + Send, EcamError> {
        let (peripheral, characteristic) 
            = (self.peripheral.clone(), self.characteristic.clone());
        peripheral.subscribe(&characteristic).await?;
        let (trigger, tripwire) = Tripwire::new();
        let peripheral2 = peripheral.clone();
        tokio::spawn(async move {
            while peripheral2.is_connected().await.unwrap_or_default() {}
            drop(trigger);
        });
        // Trim the header and CRC
        Result::Ok(
            peripheral
                .notifications()
                .await?
                .map(|m| m.value[2..m.value.len() - 2].to_vec())
                .take_until_if(tripwire),
        )
    }
}

impl Ecam for EcamBT {
    fn read<'a>(
        self: &'a Self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<EcamOutput>, EcamError>> + Send + 'a>>
    {
        Box::pin(async { Result::Ok(self.notifications.lock().await.recv().await) })
    }

    fn write<'a>(
        self: &'a Self,
        data: Vec<u8>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), EcamError>> + Send + 'a>> {
        Box::pin(self.send(data))
    }

    fn scan<'a>(
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Uuid, EcamError>> + Send + 'a>> {
        Box::pin(scan())
    }
}

async fn scan() -> Result<Uuid, EcamError> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    for adapter in adapter_list.into_iter() {
        if let Ok(Some((p, c))) = get_ecam_from_adapter(&adapter).await {
            // Icky, but we don't have a PeripheralId to UUID function
            let uuid = format!("{:?}", p.id())[13..49].to_owned();
            return Ok(Uuid::parse_str(&uuid).expect("failed to parse UUID from debug string"))
        }
    }

    Err(EcamError::NotFound)
}

pub async fn get_ecam(uuid: Uuid) -> Result<EcamBT, EcamError> {
    let manager = Manager::new().await?;
    get_ecam_from_manager(&manager, uuid).await
}

async fn get_notifications_from_peripheral(
    peripheral: &Peripheral,
    characteristic: &Characteristic,
) -> Result<Receiver<EcamOutput>, EcamError> {
    peripheral.subscribe(characteristic).await?;
    let peripheral2 = peripheral.clone();
    let (trigger, tripwire) = Tripwire::new();
    tokio::spawn(async move {
        while peripheral2.is_connected().await.unwrap_or_default() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        println!("disconnected");
        drop(trigger);
    });

    // Use a forwarding task to make this stream Sync
    let mut n = peripheral.notifications().await?.take_until_if(tripwire);
    let (tx, mut rx) = mpsc::channel(100);
    tokio::spawn(async move {
        tx.send(EcamOutput::Ready)
            .await
            .expect("Failed to forward notification");
        while let Some(m) = n.next().await {
            tx.send(EcamOutput::Packet(Response::decode(
                &m.value[2..m.value.len() - 2].to_vec(),
            )))
            .await
            .expect("Failed to forward notification");
        }
        tx.send(EcamOutput::Done)
            .await
            .expect("Failed to forward notification");
    });

    Result::Ok(rx)
}

async fn get_ecam_from_manager(manager: &Manager, uuid: Uuid) -> Result<EcamBT, EcamError> {
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        return Result::Err(EcamError::NotFound);
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    for adapter in adapter_list.into_iter() {
        adapter.start_scan(ScanFilter::default()).await?;
        let tx = tx.clone();
        let _ = tokio::spawn(async move {
            println!("Looking for peripheral {}", uuid);
            loop {
                if let Ok(peripheral) = adapter.peripheral(&PeripheralId::from(uuid)).await {
                    println!("Got peripheral");
                    peripheral.connect().await?;
                    println!("Connected");
                    let characteristic = Characteristic { uuid: CHARACTERISTIC_UUID, service_uuid: SERVICE_UUID, properties: CharPropFlags::WRITE | CharPropFlags::READ | CharPropFlags::NOTIFY };
                    let n = Box::pin(get_notifications_from_peripheral(&peripheral, &characteristic).await?);
                    // Ignore errors here -- we just want the first peripheral that connects
                    let _ = tx.send(EcamBT {
                        peripheral,
                        characteristic,
                        notifications: Arc::new(Mutex::new(n)),
                    }).await;
                    break;
                }
            }
            Result::<_, EcamError>::Ok(())
        }).await;
    }

    Ok(rx.recv().await.expect("Failed to receive anything"))
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
    time::sleep(Duration::from_secs(2)).await;
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
