use crate::{ecam::EcamOutput, prelude::*};

use clap::{arg, command};

use tokio::select;
use uuid::Uuid;

mod command;
mod ecam;
mod packet;
mod packet_stream;
mod prelude;

use command::*;
use ecam::{ecam_scan, get_ecam_bt, get_ecam_subprocess, Ecam, EcamDriver, EcamError, EcamStatus};

async fn pipe(device_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let uuid = Uuid::parse_str(&device_name).expect("Failed to parse UUID");
    let ecam = get_ecam_bt(uuid).await?;

    let mut bt_in = ecam.stream().await?;
    let mut bt_out = Box::pin(packet_stream::packet_stdio_stream());

    loop {
        select! {
            input = bt_in.next() => {
                if let Some(value) = input {
                    println!("R: {}", packet::stringify(&value));
                } else {
                    println!("Device closed");
                    break;
                }
            },
            out = bt_out.next() => {
                if let Some(value) = out {
                    ecam.send(value).await?;
                } else {
                    println!("Input closed");
                    break;
                }
            }
        }
    }

    Result::Ok(())
}

async fn monitor(ecam: Ecam, turn_on: bool) -> Result<(), EcamError> {
    let mut tap = ecam.packet_tap().await?;
    let ecam = ecam.clone();
    let handle = tokio::spawn(async move {
        while let Some(packet) = tap.next().await {
            println!("{:?}", packet);
            if packet == EcamOutput::Done {
                break;
            }
        }
    });
    let state = ecam.current_state().await?;
    if turn_on && state == EcamStatus::StandBy {
        ecam.write(Request::State(StateRequest::TurnOn)).await?;
    }

    let _ = handle.await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            let turn_on = cmd.get_flag("turn-on");
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();
            let ecam: Box<dyn EcamDriver> = Box::new(get_ecam_subprocess(device_name).await?);
            let ecam = Ecam::new(ecam).await;
            match ecam.current_state().await? {
                EcamStatus::Ready => {}
                EcamStatus::StandBy => {
                    if !turn_on {
                        println!(
                            "Machine is not on, pass --turn-on to turn it on before operation"
                        );
                        return Ok(());
                    }
                    ecam.write(Request::State(StateRequest::TurnOn)).await?;
                    ecam.wait_for_state(ecam::EcamStatus::Ready).await?;
                }
                s => {
                    println!(
                        "Machine is in state {:?}, so we will cowardly refuse to brew coffee",
                        s
                    );
                    return Ok(());
                }
            }
            println!("Waiting for ready...");
            ecam.wait_for_state(ecam::EcamStatus::Ready).await?;
            println!("Waiting for ready done...");
        }
        Some(("monitor", cmd)) => {
            let turn_on = cmd.get_flag("turn-on");
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();
            let ecam: Box<dyn EcamDriver> = Box::new(get_ecam_subprocess(device_name).await?);
            let ecam = Ecam::new(ecam).await;

            monitor(ecam, turn_on).await?;
        }
        Some(("list", _cmd)) => {
            let (s, uuid) = ecam_scan().await?;
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
