use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter,
    ValueNotification,
};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use std::pin::Pin;
use std::result::Result;
use std::time::Duration;
use stream_cancel::{StreamExt as _, Tripwire};
use tokio::time;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::command::Response;
use crate::ecam::{EcamDriver, EcamError, EcamOutput, EcamPacketReceiver};
use crate::packet::packetize;

const SERVICE_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000300);
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000301);

/// The concrete peripheral type to avoid going crazy here managaing an unsized trait.
type Peripheral = <Adapter as Central>::Peripheral;

pub struct EcamBT {
    peripheral: Peripheral,
    characteristic: Characteristic,
    notifications: EcamPacketReceiver,
}

impl EcamBT {
    /// Send a packet to the ECAM
    pub async fn send(&self, data: Vec<u8>) -> Result<(), EcamError> {
        let (peripheral, characteristic) = (self.peripheral.clone(), self.characteristic.clone());
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
    pub async fn stream(&self) -> Result<impl Stream<Item = Vec<u8>> + Send, EcamError> {
        let (peripheral, characteristic) = (self.peripheral.clone(), self.characteristic.clone());
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

impl EcamDriver for EcamBT {
    fn read<'a>(
        &'a self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<EcamOutput>, EcamError>> + Send + 'a>>
    {
        Box::pin(self.notifications.recv())
    }

    fn write<'a>(
        &'a self,
        data: Vec<u8>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), EcamError>> + Send + 'a>> {
        Box::pin(self.send(data))
    }

    fn scan<'a>(
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(String, Uuid), EcamError>> + Send + 'a>>
    {
        Box::pin(scan())
    }
}

async fn scan() -> Result<(String, Uuid), EcamError> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    for adapter in adapter_list.into_iter() {
        if let Ok(Some((s, p, _c))) = get_ecam_from_adapter(&adapter).await {
            // Icky, but we don't have a PeripheralId to UUID function
            let uuid = format!("{:?}", p.id())[13..49].to_owned();
            return Ok((
                s,
                Uuid::parse_str(&uuid).expect("failed to parse UUID from debug string"),
            ));
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
) -> Result<EcamPacketReceiver, EcamError> {
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
    let f = |m: ValueNotification| {
        EcamOutput::Packet(Response::decode(&m.value[2..m.value.len() - 2].to_vec()))
    };
    let n = peripheral
        .notifications()
        .await?
        .map(f)
        .take_until_if(tripwire);
    Ok(EcamPacketReceiver::from_stream(n, true))
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
                    let characteristic = Characteristic {
                        uuid: CHARACTERISTIC_UUID,
                        service_uuid: SERVICE_UUID,
                        properties: CharPropFlags::WRITE
                            | CharPropFlags::READ
                            | CharPropFlags::INDICATE,
                    };
                    let n = get_notifications_from_peripheral(&peripheral, &characteristic).await?;
                    // Ignore errors here -- we just want the first peripheral that connects
                    let _ = tx
                        .send(EcamBT {
                            peripheral,
                            characteristic,
                            notifications: n,
                        })
                        .await;
                    break;
                }
            }
            Result::<_, EcamError>::Ok(())
        })
        .await;
    }

    Ok(rx.recv().await.expect("Failed to receive anything"))
}

async fn get_ecam_from_adapter(
    adapter: &Adapter,
) -> Result<Option<(String, Peripheral, Characteristic)>, EcamError> {
    println!("Starting scan on {}...", adapter.adapter_info().await?);
    let filter = ScanFilter {
        services: vec![SERVICE_UUID],
    };
    adapter
        .start_scan(filter)
        .await
        .expect("Can't scan BLE adapter for connected devices...");

    for _ in 0..10 {
        time::sleep(Duration::from_secs(1)).await;
        let peripherals = adapter.peripherals().await?;
        for peripheral in peripherals.iter() {
            let r = validate_peripheral(peripheral).await?;
            if let Some((local_name, characteristic)) = r {
                return Result::Ok(Some((
                    local_name,
                    peripheral.clone(),
                    characteristic,
                )));
            }
        }
    }

    Result::Err(EcamError::NotFound)
}

async fn validate_peripheral(
    peripheral: &Peripheral,
) -> Result<Option<(String, Characteristic)>, EcamError> {
    let properties = peripheral.properties().await?;
    let is_connected = peripheral.is_connected().await?;
    let properties = properties.unwrap();
    if let Some(local_name) = properties.local_name {
        if !is_connected {
            peripheral.connect().await?
        }
        peripheral.is_connected().await?;
        peripheral.discover_services().await?;
        for service in peripheral.services() {
            for characteristic in service.characteristics {
                if characteristic.uuid == CHARACTERISTIC_UUID {
                    return Result::Ok(Some((local_name, characteristic)));
                }
            }
        }
        return Result::Ok(None);
    }
    Result::Ok(None)
}
