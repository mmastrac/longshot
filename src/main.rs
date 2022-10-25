use clap::{arg, command, value_parser, ArgAction};
use command::{Request, StateRequest};
use ecam_bt::EcamBT;
use std::future::{self, Future};
use std::time::Duration;
use std::{error::Error, sync::Arc};
use stream_cancel::{StreamExt as _, Tripwire};
use tokio::sync::Mutex;
use tokio::try_join;
use tokio_stream::{Stream, StreamExt as _};
use tuples::*;
use uuid::Uuid;

mod command;
mod ecam;
mod ecam_bt;
mod ecam_subprocess;
mod packet;
mod packet_stream;

use ecam::{Ecam, EcamError};

fn get_update_packet_stream(d: Duration) -> impl Stream<Item = Vec<u8>> {
    let mut interval = tokio::time::interval(d);
    let update_stream = async_stream::stream! {
        loop {
            interval.tick().await;
            yield command::Request::Monitor(command::MonitorRequestVersion::V2).encode();
        }
    };
    update_stream
}

async fn pipe(device_name: String) -> Result<(), Box<dyn Error>> {
    let uuid = Uuid::parse_str(&device_name).expect("Failed to parse UUID");
    let mut ecam = ecam_bt::get_ecam(uuid).await?;
    let (trigger, tripwire) = Tripwire::new();
    let trigger1 = Arc::new(Mutex::new(Some(trigger)));
    let trigger2 = trigger1.clone();

    let mut bt_in = ecam.stream().await?.take_until_if(tripwire.clone());
    let mut bt_out = Box::pin(packet_stream::packet_stdio_stream().take_until_if(tripwire.clone()));

    let a = tokio::spawn(async move {
        while let Some(value) = bt_out.next().await {
            ecam.send(value).await?;
        }
        println!("Packet stream done.");
        trigger1.lock().await.take();
        Result::<(), EcamError>::Ok(())
    });

    let b = tokio::spawn(async move {
        while let Some(value) = bt_in.next().await {
            println!("R: {}", packet::stringify(&value));
        }
        println!("Device stream done.");
        trigger2.lock().await.take();
        Result::<(), EcamError>::Ok(())
    });

    // iterator_try_collect will probably simplify this
    try_join!(a, b)?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    // TODO: Figure out where tokio is getting stuck and failing to terminate the process
    // std::process::exit(0);
    Result::Ok(())
}

async fn monitor(turn_on: bool, device_name: String) -> Result<(), EcamError> {
    let ecam = Arc::new(ecam_subprocess::connect(&device_name).await?);
    let timeout = Duration::from_millis(100);
    if turn_on {
        ecam.write(Request::State(StateRequest::TurnOn).encode())
            .await?;
    }
    let ecam2 = ecam.clone();
    let a = tokio::spawn(async move {
        loop {
            match tokio::time::timeout(timeout, ecam2.write(vec![0x75, 0x0f])).await {
                Ok(x) => {}
                Err(x) => {
                    println!("timeout");
                }
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        // Result::<(), EcamError>::Ok(())
    });

    loop {
        match tokio::time::timeout(timeout, ecam.read()).await {
            Ok(Ok(Some(x))) => {
                println!("{:?}", x);
            }
            Err(x) => {}
            x => {
                println!("{:?}", x);
                break;
            }
        }
    }

    a.abort();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let device_name = arg!(--"device-name" <name>).help("Provides the name of the device");
    let turn_on = arg!(--"turn-on").help("Turn on the machine before running this operation");
    let matches = command!()
        .subcommand(
            command!("brew")
                .about("Brew a coffee")
                .arg(device_name.clone())
                .arg(turn_on.clone()),
        )
        .subcommand(
            command!("monitor")
                .about("Monitor the status of the device")
                .arg(device_name.clone())
                .arg(turn_on.clone()),
        )
        .subcommand(command!("list").about("List all supported devices"))
        .subcommand(
            command!("x-internal-pipe")
                .about("Used to communicate with the device")
                .hide(true)
                .arg(device_name.clone()),
        )
        .get_matches();

    let subcommand = matches.subcommand();

    match subcommand {
        Some(("brew", cmd)) => {
            println!("{:?}", cmd);
            let mut ecam = ecam_subprocess::connect(
                &cmd.get_one::<String>("device-name")
                    .expect("Device name required")
                    .clone(),
            )
            .await?;

            while let Some(s) = ecam.read().await? {
                println!("{:?}", s);
            }
        }
        Some(("monitor", cmd)) => {
            monitor(
                cmd.get_flag("turn-on"),
                cmd.get_one::<String>("device-name")
                    .expect("Device name required")
                    .clone(),
            )
            .await?;
        }
        Some(("list", cmd)) => {
            let (s, uuid) = EcamBT::scan().await?;
            println!("{}  {}", s, uuid);
        }
        Some(("x-internal-pipe", cmd)) => {
            pipe(
                cmd.get_one::<String>("device-name")
                    .expect("Device name required")
                    .clone(),
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}
