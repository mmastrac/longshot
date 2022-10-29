use crate::prelude::*;
use std::collections::HashMap;

use clap::{arg, command};

mod ecam;
mod logging;
mod operations;
mod prelude;
mod protocol;

use ecam::{
    ecam_scan, get_ecam_bt, get_ecam_simulator, get_ecam_subprocess, pipe_stdin, Ecam, EcamDriver,
    EcamError, EcamOutput, EcamStatus,
};
use enum_iterator::Sequence;
use operations::RecipeAccumulator;
use protocol::*;
use uuid::Uuid;

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
        ecam.write_request(Request::AppControl(AppControl::TurnOn))
            .await?;
    }
    // ecam.write_request(Request::ProfileNameRead(1, 3)).await?;
    // tokio::time::sleep(Duration::from_millis(250)).await;
    // ecam.write_request(Request::RecipeNameRead(1, 3)).await?;
    // tokio::time::sleep(Duration::from_millis(250)).await;
    // ecam.write_request(Request::RecipeNameRead(4, 6)).await?;
    // tokio::time::sleep(Duration::from_millis(250)).await;
    // ecam.write(EcamPacket::from_represenation(Request::Profile(
    // ProfileRequest::GetRecipeNames(1, 3),
    // )))
    // .await?;
    // tokio::time::sleep(Duration::from_millis(250)).await;
    // ecam.write(EcamPacket::from_undecodeable_bytes(&[176, 0xf0, 1]))
    //     .await?;
    // tokio::time::sleep(Duration::from_millis(250)).await;

    loop {
        // Poll for current state
        let _ = ecam.current_state().await?;
    }

    let _ = handle.await;

    Ok(())
}

async fn list_recipes(ecam: Ecam) -> Result<(), EcamError> {
    // Wait for device to settle
    ecam.wait_for_connection().await?;

    // Get the tap we'll use for reading responses
    let mut tap = ecam.packet_tap().await?;
    let mut recipes = RecipeAccumulator::new();
    for i in 0..3 {
        if i == 0 {
            println!("Fetching recipes...");
        } else {
            if recipes.get_remaining_beverages().len() > 0 {
                println!(
                    "Fetching potentially missing recipes... {:?}",
                    recipes.get_remaining_beverages()
                );
            }
        }
        'outer: for beverage in recipes.get_remaining_beverages() {
            'inner: for packet in vec![
                Request::RecipeMinMaxSync(MachineEnum::Value(beverage)),
                Request::RecipeQuantityRead(1, MachineEnum::Value(beverage)),
            ] {
                let request_id = packet.ecam_request_id();
                ecam.write_request(packet).await?;
                let now = std::time::Instant::now();
                while now.elapsed() < Duration::from_millis(500) {
                    match tokio::time::timeout(Duration::from_millis(50), tap.next()).await {
                        Err(_) => {}
                        Ok(None) => {}
                        Ok(Some(x)) => {
                            if let Some(packet) = x.take_packet() {
                                let response_id = packet.ecam_request_id();
                                recipes.accumulate_packet(beverage, packet);
                                // If this recipe is totally complete, move to the next one
                                if recipes.is_complete(beverage) {
                                    continue 'outer;
                                }
                                // If we got a response for the given request, move to the next packet/beverage
                                if response_id == request_id {
                                    continue 'inner;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let list = recipes.take();
    for recipe in list.recipes {
        println!("{:?} {:?}", recipe.beverage, recipe.fetch_ingredients());
    }
    // 'outer: for beverage in enum_iterator::all() {
    //     let response1 = m.get(&beverage);
    //     println!("{:?}", response1);
    //     let response = m2.get(&beverage);
    //     let mut s = format!("brew {:?}", beverage).to_owned();
    //     if let Some(response) = response {
    //         if response.is_empty() {
    //             continue;
    //         }
    //         for r in response.iter() {
    //             if r.ingredient == MachineEnum::Value(EcamIngredients::Visible) && r.value == 0 {
    //                 continue 'outer;
    //             }
    //             if matches!(r.ingredient.into(), Some(EcamIngredients::Visible | EcamIngredients::Programmable | EcamIngredients::IndexLength | EcamIngredients::Accessorio)) {
    //                 continue;
    //             }
    //             if r.min == 0 && r.max == 1 {
    //                 s += &format!(" --{} (true|false)", format!("{:?}", r.ingredient).to_ascii_lowercase());
    //             } else {
    //                 s += &format!(" --{} ({}<={}<={})", format!("{:?}", r.ingredient).to_ascii_lowercase(), r.min, r.value, r.max);
    //             }
    //         }
    //         println!("{}", s);
    //     }
    // }

    Ok(())
}

fn enum_lookup<T: Sequence + std::fmt::Debug>(s: &str) -> Option<T> {
    for e in enum_iterator::all() {
        println!("{:?} {:?}", e, s);
        if format!("{:?}", e).to_ascii_lowercase() == s.to_ascii_lowercase() {
            return Some(e);
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let device_name = arg!(--"device-name" <name>).help("Provides the name of the device");
    let turn_on = arg!(--"turn-on").help("Turn on the machine before running this operation");
    let matches = command!()
        .arg(arg!(--"trace").help("Trace packets to/from device"))
        .subcommand(
            command!("brew")
                .about("Brew a coffee")
                .arg(device_name.clone())
                .arg(turn_on.clone())
                .arg(
                    arg!(--"beverage" <name>)
                        .required(true)
                        .help("The beverage to brew"),
                )
                .arg(arg!(--"coffee" <amount>).help("Amount of coffee to brew"))
                .arg(arg!(--"milk" <amount>).help("Amount of milk to steam/pour"))
                .arg(arg!(--"hotwater" <amount>).help("Amount of hot water to pour"))
                .arg(arg!(--"taste" <taste>).help("The strength of the beverage"))
                .arg(arg!(--"temperature" <temperature>).help("The temperature of the beverage")),
        )
        .subcommand(
            command!("monitor")
                .about("Monitor the status of the device")
                .arg(device_name.clone())
                .arg(turn_on.clone()),
        )
        .subcommand(
            command!("list-recipes")
                .about("List recipes stored in the device")
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

    if matches.get_flag("trace") {
        crate::logging::TRACE_ENABLED.store(true, std::sync::atomic::Ordering::Relaxed);
    }

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
                    ecam.write_request(Request::AppControl(AppControl::TurnOn))
                        .await?;
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

            let beverage: EcamBeverageId =
                enum_lookup(cmd.get_one::<String>("beverage").unwrap_or(&"".to_owned()))
                    .expect("Beverage required");
            let coffee = cmd
                .get_one::<String>("coffee")
                .map(|s| s.parse::<u16>().expect("Invalid number"));
            let milk = cmd
                .get_one::<String>("milk")
                .map(|s| s.parse::<u16>().expect("Invalid number"));
            let hotwater = cmd
                .get_one::<String>("hotwater")
                .map(|s| s.parse::<u16>().expect("Invalid number"));
            let taste: Option<EcamBeverageTaste> =
                enum_lookup(cmd.get_one::<String>("taste").unwrap_or(&"".to_owned()));
            let temp: Option<EcamTemperature> = enum_lookup(
                cmd.get_one::<String>("temperature")
                    .unwrap_or(&"".to_owned()),
            );

            println!(
                "{:?} {:?} {:?} {:?} {:?} {:?}",
                beverage, coffee, milk, hotwater, taste, temp
            );

            let recipe = vec![
                RecipeInfo::new(EcamIngredients::Coffee, 240),
                RecipeInfo::new(
                    EcamIngredients::Taste,
                    <u8>::from(EcamBeverageTaste::ExtraStrong) as u16,
                ),
            ];
            let req = Request::BeverageDispensingMode(
                MachineEnum::Value(beverage),
                MachineEnum::Value(EcamOperationTrigger::Start),
                recipe,
                MachineEnum::Value(EcamBeverageTasteType::Prepare),
            );

            ecam.write_request(req).await?;
            monitor(ecam, false).await?;
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
        Some(("list-recipes", cmd)) => {
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();
            let ecam: Box<dyn EcamDriver> = Box::new(get_ecam_subprocess(device_name).await?);
            let ecam = Ecam::new(ecam).await;
            list_recipes(ecam).await?;
        }
        Some(("x-internal-pipe", cmd)) => {
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();
            if device_name == "simulate" {
                let ecam = get_ecam_simulator().await?;
                pipe_stdin(ecam).await?;
            } else {
                let uuid = Uuid::parse_str(&device_name).expect("Failed to parse UUID");
                let ecam = get_ecam_bt(uuid).await?;
                pipe_stdin(ecam).await?;
            }
        }
        _ => {}
    }

    Ok(())
}
