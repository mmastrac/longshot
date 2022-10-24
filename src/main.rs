use clap::{arg, command, value_parser, ArgAction};
use std::future::{self, Future};
use std::sync::Mutex;
use std::time::Duration;
use std::{error::Error, sync::Arc};
use stream_cancel::{StreamExt as _, Tripwire};
use tokio::try_join;
use tokio_stream::{Stream, StreamExt as _};
use tuples::*;

mod command;
mod ecam_bt;
mod ecam_subprocess;
mod packet;
mod packet_stream;

use ecam_bt::EcamError;

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

async fn pipe(device: String) -> Result<(), Box<dyn Error>> {
    let ecam = ecam_bt::get_ecam().await?;
    let (trigger, tripwire) = Tripwire::new();
    let trigger1 = Arc::new(Mutex::new(Some(trigger)));
    let trigger2 = trigger1.clone();

    let mut bt_in = ecam.stream().await?.take_until_if(tripwire.clone());
    let mut bt_out = Box::pin(packet_stream::packet_stdio_stream().take_until_if(tripwire.clone()));

    let a = tokio::spawn(async move {
        while let Some(value) = bt_out.next().await {
            ecam.send(value).await?;
        }
        println!("a done");
        trigger1.lock().unwrap().take();
        Result::<(), EcamError>::Ok(())
    });

    let b = tokio::spawn(async move {
        while let Some(value) = bt_in.next().await {
            println!(
                "R: {}",
                value
                    .iter()
                    .map(|n| format!("{:02x}", n))
                    .collect::<String>()
            );
        }
        println!("b done");
        trigger2.lock().unwrap().take();
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let device_name = arg!(--"device-name" <name>).help("Provides the name of the device");
    let matches = command!()
        .subcommand(
            command!("brew")
                .about("Brew a coffee")
                .arg(device_name.clone()),
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

            let mut r = Box::pin(ecam.read().await?);
            while let Some(s) = r.next().await {
                println!("{:?}", s);
            }
        }
        Some(("list", cmd)) => {
            println!("{:?}", cmd);
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
