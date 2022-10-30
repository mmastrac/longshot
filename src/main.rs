use std::time::Instant;

use crate::prelude::*;

use clap::{arg, command};

mod display;
mod ecam;
mod logging;
mod operations;
mod prelude;
mod protocol;

use display::{BasicStatusDisplay, ColouredStatusDisplay, StatusDisplay};
use ecam::{
    ecam_scan, get_ecam_bt, get_ecam_simulator, get_ecam_subprocess, pipe_stdin, Ecam, EcamDriver,
    EcamError, EcamOutput, EcamStatus,
};
use enum_iterator::Sequence;
use operations::{RecipeAccumulator, RecipeList};
use protocol::*;
use uuid::Uuid;

async fn monitor(ecam: Ecam, turn_on: bool) -> Result<(), EcamError> {
    let mut tap = ecam.packet_tap().await?;
    let ecam = ecam.clone();
    let handle = tokio::spawn(async move {
        while let Some(packet) = tap.next().await {
            // println!("{:?}", packet);
            if packet == EcamOutput::Done {
                break;
            }
        }
    });

    let mut display: Box<dyn StatusDisplay> = Box::new(ColouredStatusDisplay::new(80));
    let mut state = ecam.current_state().await?;
    display.display(state);
    if turn_on && state == EcamStatus::StandBy {
        ecam.write_request(Request::AppControl(AppControl::TurnOn))
            .await?;
    }

    let mut debounce = Instant::now();
    loop {
        // Poll for current state
        let next_state = ecam.current_state().await?;
        if next_state != state || debounce.elapsed() > Duration::from_millis(250) {
            // println!("{:?}", next_state);
            display.display(next_state);
            state = next_state;
            debounce = Instant::now();
        }
    }

    let _ = handle.await;

    Ok(())
}

async fn list_recipies_for(
    ecam: Ecam,
    recipes: Option<Vec<EcamBeverageId>>,
) -> Result<RecipeList, EcamError> {
    // Get the tap we'll use for reading responses
    let mut tap = ecam.packet_tap().await?;
    let mut recipes = if let Some(recipes) = recipes {
        RecipeAccumulator::limited_to(recipes)
    } else {
        RecipeAccumulator::new()
    };
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
    Ok(recipes.take())
}

async fn list_recipes(ecam: Ecam) -> Result<(), EcamError> {
    // Wait for device to settle
    ecam.wait_for_connection().await?;
    let list = list_recipies_for(ecam, None).await?;

    for recipe in list.recipes {
        println!("{:?} {:?}", recipe.beverage, recipe.fetch_ingredients());
    }

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
                .arg(arg!(--"temperature" <temperature>).help("The temperature of the beverage"))
                .arg(
                    arg!(--"allow-off")
                        .hide(true)
                        .help("Allow brewing while machine is off"),
                )
                .arg(
                    arg!(--"skip-brew")
                        .hide(true)
                        .help("Does everything except actually brew the beverage"),
                ),
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
            let skip_brew = cmd.get_flag("skip-brew");
            let allow_off = cmd.get_flag("allow-off");
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();

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

            let ecam: Box<dyn EcamDriver> = Box::new(get_ecam_subprocess(device_name).await?);
            let ecam = Ecam::new(ecam).await;
            match ecam.current_state().await? {
                EcamStatus::Ready => {}
                EcamStatus::StandBy => {
                    if allow_off {
                        println!("Machine is off, but --allow-off will allow us to proceed")
                    } else {
                        if !turn_on {
                            println!(
                                "Machine is not on, pass --turn-on to turn it on before operation"
                            );
                            return Ok(());
                        }
                        println!("Waiting for the machine to turn on...");
                        ecam.write_request(Request::AppControl(AppControl::TurnOn))
                            .await?;
                        ecam.wait_for_state(ecam::EcamStatus::Ready).await?;
                    }
                }
                s => {
                    println!(
                        "Machine is in state {:?}, so we will cowardly refuse to brew coffee",
                        s
                    );
                    return Ok(());
                }
            }

            println!(
                "{:?} {:?} {:?} {:?} {:?} {:?}",
                beverage, coffee, milk, hotwater, taste, temp
            );

            println!("Fetching recipe for {:?}...", beverage);
            let recipe_list = list_recipies_for(ecam.clone(), Some(vec![beverage])).await?;
            let recipe = recipe_list.find(beverage);
            if let Some(recipe) = recipe {
                println!("{:?}", recipe.fetch_ingredients());
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

                if skip_brew {
                    println!("--skip-brew was passed, so we aren't going to brew anything")
                } else {
                    ecam.write_request(req).await?;
                }
                monitor(ecam, false).await?;
            } else {
                println!("I wasn't able to fetch the recipe for {:?}. Perhaps this machine can't make it?", beverage);
            }
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
