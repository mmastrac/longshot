use bluster::{
    gatt::{
        characteristic,
        characteristic::*,
        descriptor,
        descriptor::Descriptor,
        event::{Event, EventSender, Response},
        service::Service,
    },
    Peripheral, SdpShortUuid,
};
use futures::{channel::mpsc::channel, prelude::*};
use std::{collections::HashSet, time::Duration};
use uuid_bluster::Uuid;

const SERVICE_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000300);
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00035b03_58e6_07dd_021a_08123a000301);
const DESCRIPTOR_UUID: Uuid = Uuid::from_u128(0x00002902_0000_1000_8000_00805f9b34fb);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let peripheral = Peripheral::new().await?;
    let mut characteristics = HashSet::new();
    let (tx, mut rx) = channel(1);
    let (tx2, mut rx2) = channel(1);
    let (tx3, mut rx3) = channel(1);
    let (tx4, mut rx4) = channel(1);

    tokio::spawn(async move {
        while let Some(x) = rx.next().await {
            println!("{:?}", x);
        }
    });

    tokio::spawn(async move {
        while let Some(x) = rx2.next().await {
            println!("{:?}", x);
        }
    });

    tokio::spawn(async move {
        while let Some(x) = rx3.next().await {
            println!("{:?}", x);
        }
    });

    let properties = Properties::new(
        Some(Read(Secure::Insecure(tx))),
        Some(Write::WithoutResponse(tx2)),
        Some(tx4),
        None,
    );
    let mut descriptors = HashSet::new();
    descriptors.insert(Descriptor::new(
        DESCRIPTOR_UUID,
        descriptor::Properties::new(
            Some(descriptor::Read(descriptor::Secure::Insecure(tx3.clone()))),
            Some(descriptor::Write(descriptor::Secure::Insecure(tx3))),
        ),
        None,
    ));
    characteristics.insert(Characteristic::new(
        CHARACTERISTIC_UUID,
        properties,
        None,
        descriptors,
    ));
    peripheral.add_service(&Service::new(SERVICE_UUID, true, characteristics))?;
    while !peripheral.is_powered().await? {}
    println!("Peripheral powered on");
    peripheral.register_gatt().await?;
    peripheral.start_advertising("D123456", &[]).await?;
    println!("Peripheral started advertising");
    let timeout = tokio::time::sleep(Duration::from_secs(60)).await;
    while peripheral.is_advertising().await.unwrap() {}
    Ok(())
}
