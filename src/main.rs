use clap::{arg, command, value_parser, ArgAction};
use std::error::Error;
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};

mod command;
mod ecam_bt;
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
    let mut bt_in = ecam.stream().await?;
    let packet_out = packet_stream::packet_stdio_stream();
    let update_stream = get_update_packet_stream(Duration::from_millis(250));
    let mut bt_out = Box::pin(packet_out.merge(update_stream));

    let a = tokio::spawn(async move {
        while let Some(value) = bt_out.next().await {
            ecam.send(value).await?;
        }
        println!("a closed");
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
        println!("b closed");
        Result::<(), EcamError>::Ok(())
    });

    a.await?;
    b.await?;

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
