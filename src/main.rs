#![warn(clippy::all)]
use clap::builder::{PossibleValue, PossibleValuesParser};
use clap::{Arg, ArgMatches, arg, command};

mod app;

embed_plist::embed_info_plist!("Info.plist");

use longshot::ecam::{
    Ecam, EcamBT, EcamError, EcamId, ecam_lookup, ecam_scan, get_ecam_simulator, pipe_stdin,
};
use longshot::{operations::*, protocol::*};

fn enum_value_parser<T: MachineEnumerable<T> + 'static>() -> PossibleValuesParser {
    PossibleValuesParser::new(T::all().map(|x| PossibleValue::new(x.to_arg_string())))
}

struct DeviceCommon {
    device_id: EcamId,
    dump_packets: bool,
    turn_on: bool,
    allow_off: bool,
}

impl DeviceCommon {
    fn args() -> [Arg; 4] {
        [
            arg!(--"device-name" <name>)
                .help("Provides the name of the device")
                .required(true),
            arg!(--"dump-packets").help("Dumps decoded packets to the terminal for debugging"),
            arg!(--"turn-on")
                .help("Turn on the machine before running this operation")
                .conflicts_with("allow-off"),
            arg!(--"allow-off")
                .hide(true)
                .help("Allow brewing while machine is off")
                .conflicts_with("turn-on"),
        ]
    }

    fn parse(cmd: &ArgMatches) -> Self {
        Self {
            device_id: cmd
                .get_one::<String>("device-name")
                .expect("Device name required")
                .into(),
            dump_packets: cmd.get_flag("dump-packets"),
            turn_on: cmd.get_flag("turn-on"),
            allow_off: cmd.get_flag("allow-off"),
        }
    }
}

async fn ecam(cmd: &ArgMatches, allow_off_and_alarms: bool) -> Result<Ecam, EcamError> {
    let device_common = DeviceCommon::parse(cmd);
    let ecam = ecam_lookup(&device_common.device_id, device_common.dump_packets).await?;
    if !power_on(
        ecam.clone(),
        device_common.allow_off | allow_off_and_alarms,
        allow_off_and_alarms,
        device_common.turn_on,
    )
    .await?
    {
        longshot::display::shutdown();
        std::process::exit(1);
    }
    Ok(ecam)
}

fn command() -> clap::Command {
    command!()
        .arg(arg!(--"trace").help("Trace packets to/from device"))
        .subcommand(
            command!("brew")
                .about("Brew a coffee")
                .args(DeviceCommon::args())
                .arg(
                    arg!(--"beverage" <name>)
                        .required(true)
                        .help("The beverage to brew")
                        .value_parser(enum_value_parser::<EcamBeverageId>()),
                )
                .arg(
                    arg!(--"coffee" <amount>)
                        .help("Amount of coffee to brew")
                        .value_parser(0..=2500),
                )
                .arg(
                    arg!(--"milk" <amount>)
                        .help("Amount of milk to steam/pour")
                        .value_parser(0..=2500),
                )
                .arg(
                    arg!(--"hotwater" <amount>)
                        .help("Amount of hot water to pour")
                        .value_parser(0..=2500),
                )
                .arg(
                    arg!(--"taste" <taste>)
                        .help("The strength of the beverage")
                        .value_parser(enum_value_parser::<EcamBeverageTaste>()),
                )
                .arg(
                    arg!(--"temperature" <temperature>)
                        .help("The temperature of the beverage")
                        .value_parser(enum_value_parser::<EcamTemperature>()),
                )
                .arg(
                    arg!(--"allow-defaults")
                        .help("Allow brewing if some parameters are not specified"),
                )
                .arg(arg!(--"force").help("Allow brewing with parameters that do not validate"))
                .arg(
                    arg!(--"skip-brew")
                        .hide(true)
                        .help("Does everything except actually brew the beverage"),
                ),
        )
        .subcommand(
            command!("monitor")
                .about("Monitor the status of the device")
                .args(DeviceCommon::args()),
        )
        .subcommand(
            command!("status")
                .about("Print the status of the device and then exit")
                .args(DeviceCommon::args()),
        )
        .subcommand(
            command!("read-parameter")
                .about("Read a parameter from the device")
                .args(DeviceCommon::args())
                .arg(
                    arg!(--"parameter" <parameter>)
                        .required(true)
                        .help("The parameter ID"),
                )
                .arg(
                    arg!(--"length" <length>)
                        .required(true)
                        .help("The parameter length"),
                ),
        )
        .subcommand(
            command!("read-statistic")
                .about("Read a statistic from the device")
                .args(DeviceCommon::args())
                .arg(
                    arg!(--"statistic" <statistic>)
                        .required(true)
                        .help("The statistic ID"),
                )
                .arg(
                    arg!(--"length" <length>)
                        .required(true)
                        .help("The statistic length"),
                ),
        )
        .subcommand(
            command!("read-statistics")
                .about("Read all statistics from the device")
                .args(DeviceCommon::args()),
        )
        .subcommand(
            command!("read-parameter-memory")
                .about("Read the parameter memory from the device")
                .args(DeviceCommon::args()),
        )
        .subcommand(
            command!("list-recipes")
                .about("List recipes stored in the device")
                .args(DeviceCommon::args())
                .arg(arg!(--"detail").help("Show detailed ingredient information"))
                .arg(arg!(--"raw").help("Show raw ingredient information")),
        )
        .subcommand(command!("list").about("List all supported devices"))
        .subcommand(
            command!("x-internal-pipe")
                .about("Used to communicate with the device")
                .hide(true)
                .args(DeviceCommon::args()),
        )
        .subcommand(
            command!("app-control")
                .about("Send a custom app-control command to the device (potentially dangerous)")
                .args(DeviceCommon::args())
                .arg(arg!(--"a" <a>).help("The first byte of the command"))
                .arg(arg!(--"b" <b>).help("The second byte of the command")),
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let matches = command().get_matches();

    if matches.get_flag("trace") {
        longshot::logging::enable_tracing();
    }

    let subcommand = matches.subcommand();
    match subcommand {
        Some(("brew", cmd)) => {
            let skip_brew = cmd.get_flag("skip-brew");
            let allow_defaults = cmd.get_flag("allow-defaults");
            let force = cmd.get_flag("force");

            let beverage: EcamBeverageId = EcamBeverageId::lookup_by_name_case_insensitive(
                cmd.get_one::<String>("beverage").unwrap(),
            )
            .expect("Beverage required");

            let mut ingredients = vec![];
            for arg in ["coffee", "milk", "hotwater", "taste", "temperature"] {
                if let Some(value) = cmd.get_raw(arg) {
                    // Once clap has had a chance to validate the args, we go back to the underlying OsStr to parse it
                    let value = value.into_iter().next().unwrap().to_str().unwrap();
                    if let Some(ingredient) = BrewIngredientInfo::from_arg(arg, value) {
                        ingredients.push(ingredient);
                    } else {
                        eprintln!("Invalid value '{}' for argument '{}'", value, arg);
                        return Ok(());
                    }
                }
            }

            let mode = match (allow_defaults, force) {
                (_, true) => IngredientCheckMode::Force,
                (true, false) => IngredientCheckMode::AllowDefaults,
                (false, false) => IngredientCheckMode::Strict,
            };
            let ecam = ecam(cmd, false).await?;
            let recipe = validate_brew(ecam.clone(), beverage, ingredients, mode).await?;
            brew(ecam.clone(), skip_brew, beverage, recipe).await?;
        }
        Some(("monitor", cmd)) => {
            let ecam = ecam(cmd, true).await?;
            monitor(ecam).await?;
        }
        Some(("status", cmd)) => {
            let ecam = ecam(cmd, true).await?;
            eprintln!("Status = {:?}", ecam.current_state().await?);
        }
        Some(("list", _cmd)) => {
            let (s, uuid) = ecam_scan().await?;
            longshot::info!("{}  {}", s, uuid);
        }
        Some(("list-recipes", cmd)) => {
            let ecam = ecam(cmd, true).await?;
            let detailed = cmd.get_flag("detail");
            let raw = cmd.get_flag("raw");
            if detailed {
                list_recipes_detailed(ecam).await?;
            } else if raw {
                list_recipes_raw(ecam).await?;
            } else {
                list_recipes(ecam).await?;
            }
        }
        Some(("read-parameter", cmd)) => {
            let parameter = cmd
                .get_one::<String>("parameter")
                .map(|s| s.parse::<u16>().expect("Invalid number"))
                .expect("Required");
            let length = cmd
                .get_one::<String>("length")
                .map(|s| s.parse::<u8>().expect("Invalid number"))
                .expect("Required");
            let ecam = ecam(cmd, true).await?;
            read_parameter(ecam, parameter, length).await?;
        }
        Some(("read-statistics", cmd)) => {
            let ecam = ecam(cmd, true).await?;
            read_statistics(ecam).await?;
        }
        Some(("read-statistic", cmd)) => {
            let parameter = cmd
                .get_one::<String>("statistic")
                .map(|s| s.parse::<u16>().expect("Invalid number"))
                .expect("Required");
            let length = cmd
                .get_one::<String>("length")
                .map(|s| s.parse::<u8>().expect("Invalid number"))
                .expect("Required");
            let ecam = ecam(cmd, true).await?;
            read_statistic(ecam, parameter, length).await?;
        }
        Some(("read-parameter-memory", cmd)) => {
            let ecam = ecam(cmd, true).await?;
            read_parameter_memory(ecam).await?;
        }
        Some(("app-control", cmd)) => {
            let ecam = ecam(cmd, true).await?;
            let a = cmd
                .get_one::<String>("a")
                .map(|s| s.parse::<u8>().expect("Invalid number"))
                .expect("Required");
            let b = cmd
                .get_one::<String>("b")
                .map(|s| s.parse::<u8>().expect("Invalid number"))
                .expect("Required");
            app_control(ecam, a, b).await?;
        }
        Some(("x-internal-pipe", cmd)) => match DeviceCommon::parse(cmd).device_id {
            id @ EcamId::Simulator(..) => {
                let ecam = get_ecam_simulator(&id).await?;
                pipe_stdin(ecam).await?;
            }
            id => {
                let ecam = EcamBT::get(id).await?;
                pipe_stdin(ecam).await?;
            }
        },
        _ => {
            command().print_help()?;
        }
    }

    longshot::display::shutdown();
    Ok(())
}
