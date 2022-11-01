use clap::{arg, command};

mod display;
mod ecam;
mod logging;
mod operations;
mod prelude;
mod protocol;

use ecam::{
    ecam_scan, get_ecam_bt, get_ecam_simulator, get_ecam_subprocess, pipe_stdin, Ecam, EcamDriver,
};
use operations::*;
use protocol::*;
use uuid::Uuid;

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
                    arg!(--"allow-defaults")
                        .help("Allow brewing if some parameters are not specified"),
                )
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
            let turn_on = cmd.get_flag("turn-on");
            let skip_brew = cmd.get_flag("skip-brew");
            let allow_off = cmd.get_flag("allow-off");
            let allow_defaults = cmd.get_flag("allow-defaults");
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();

            let beverage: EcamBeverageId = EcamBeverageId::lookup_by_name_case_insensitive(
                cmd.get_one::<String>("beverage").unwrap_or(&"".to_owned()),
            )
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
                EcamBeverageTaste::lookup_by_name_case_insensitive(
                    cmd.get_one::<String>("taste").unwrap_or(&"".to_owned()),
                );
            let temp: Option<EcamTemperature> = EcamTemperature::lookup_by_name_case_insensitive(
                cmd.get_one::<String>("temperature")
                    .unwrap_or(&"".to_owned()),
            );

            let ecam: Box<dyn EcamDriver> = Box::new(get_ecam_subprocess(device_name).await?);
            let ecam = Ecam::new(ecam).await;
            let ingredients = BrewIngredients {
                beverage,
                coffee,
                milk,
                hotwater,
                taste,
                temp,
                allow_defaults,
            };
            brew(ecam, turn_on, allow_off, skip_brew, ingredients).await?;
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
                let uuid = Uuid::parse_str(device_name).expect("Failed to parse UUID");
                let ecam = get_ecam_bt(uuid).await?;
                pipe_stdin(ecam).await?;
            }
        }
        _ => {}
    }

    Ok(())
}
