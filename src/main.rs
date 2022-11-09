use clap::{arg, command};

use longshot::ecam::{ecam_lookup, ecam_scan, get_ecam_simulator, pipe_stdin, EcamBT};
use longshot::{operations::*, protocol::*};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    longshot::display::initialize_display();

    let device_name = arg!(--"device-name" <name>).help("Provides the name of the device");
    let turn_on = arg!(--"turn-on").help("Turn on the machine before running this operation");
    let dump_packets =
        arg!(--"dump-packets").help("Dumps decoded packets to the terminal for debugging");
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
                )
                .arg(dump_packets.clone()),
        )
        .subcommand(
            command!("monitor")
                .about("Monitor the status of the device")
                .arg(device_name.clone())
                .arg(turn_on.clone())
                .arg(dump_packets.clone()),
        )
        .subcommand(
            command!("read-parameter")
                .about("Read a parameter from the device")
                .arg(device_name.clone())
                .arg(turn_on.clone())
                .arg(dump_packets.clone())
                .arg(arg!(--"parameter" <parameter>).help("The parameter ID"))
                .arg(arg!(--"length" <length>).help("The parameter length")),
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
        longshot::logging::enable_tracing();
    }

    let subcommand = matches.subcommand();
    match subcommand {
        Some(("brew", cmd)) => {
            let turn_on = cmd.get_flag("turn-on");
            let skip_brew = cmd.get_flag("skip-brew");
            let allow_off = cmd.get_flag("allow-off");
            let dump_packets = cmd.get_flag("dump-packets");
            let allow_defaults = cmd.get_flag("allow-defaults");
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();

            let beverage: EcamBeverageId = EcamBeverageId::lookup_by_name_case_insensitive(
                cmd.get_one::<String>("beverage").unwrap_or(&"".to_owned()),
            )
            .expect("Beverage required");

            let parse_u16 = |s: &String| s.parse::<u16>().expect("Invalid number");
            let ingredients = vec![
                cmd.get_one("coffee")
                    .map(parse_u16)
                    .map(BrewIngredientInfo::Coffee),
                cmd.get_one("milk")
                    .map(parse_u16)
                    .map(BrewIngredientInfo::Milk),
                cmd.get_one("hotwater")
                    .map(parse_u16)
                    .map(BrewIngredientInfo::HotWater),
                cmd.get_one("taste")
                    .map(|s: &String| {
                        EcamBeverageTaste::lookup_by_name_case_insensitive(s)
                            .expect("Invalid taste")
                    })
                    .map(BrewIngredientInfo::Taste),
                cmd.get_one("temperature")
                    .map(|s: &String| {
                        EcamTemperature::lookup_by_name_case_insensitive(s)
                            .expect("Invalid temperature")
                    })
                    .map(BrewIngredientInfo::Temperature),
            ]
            .into_iter()
            .filter_map(std::convert::identity)
            .collect();
            let ecam = ecam_lookup(device_name).await?;
            let recipe = validate_brew(
                ecam.clone(),
                beverage,
                ingredients,
                IngredientCheckMode::Strict,
            )
            .await?;
            brew(
                ecam,
                turn_on,
                allow_off,
                skip_brew,
                dump_packets,
                beverage,
                recipe,
            )
            .await?;
        }
        Some(("monitor", cmd)) => {
            let turn_on = cmd.get_flag("turn-on");
            let dump_packets = cmd.get_flag("dump-packets");
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();
            let ecam = ecam_lookup(device_name).await?;
            monitor(ecam, turn_on, dump_packets).await?;
        }
        Some(("list", _cmd)) => {
            let (s, uuid) = ecam_scan().await?;
            longshot::info!("{}  {}", s, uuid);
        }
        Some(("list-recipes", cmd)) => {
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();
            let ecam = ecam_lookup(device_name).await?;
            list_recipes(ecam).await?;
        }
        Some(("read-parameter", cmd)) => {
            let device_name = &cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .clone();
            let ecam = ecam_lookup(device_name).await?;
            let parameter = cmd
                .get_one::<String>("parameter")
                .map(|s| s.parse::<u16>().expect("Invalid number"))
                .expect("Required");
            let length = cmd
                .get_one::<String>("length")
                .map(|s| s.parse::<u8>().expect("Invalid number"))
                .expect("Required");
            read_parameter(ecam, parameter, length).await?;
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
                let ecam = EcamBT::get(uuid).await?;
                pipe_stdin(ecam).await?;
            }
        }
        _ => {}
    }

    longshot::display::clear_status();
    Ok(())
}
