//! Brew coffee from the command-line!
//!
//! # Details
//!
//! Longshot is an API and command-line application to brew coffee from the command-line (or whatever
//! front-end is built). At this time it supports DeLonghi ECAM-based Bluetooth-Low-Energy devices, and has only been tested on the
//! Dinamica Plus over Bluetooth.
//!
//! The protocol for status and monitoring has been mostly decoded, but at this time is only available in
//! source form.
//!
//! # Command-Line Examples
//!
//! Monitor a given device:
//! ```text
//! $ longshot monitor --device-name (device)
//! Dispensing... [###############################===========]
//! ```
//! Get the brew information for a given beverage:
//!
//! ```text
//! $ longshot brew  --device-name (device) --beverage regularcoffee
//! ...
//! ```
//!
//! Brew a beverage:
//!
//! ```text
//! $ longshot brew  --device-name (device) --beverage regularcoffee --coffee 180 --taste strong
//! Fetching recipe for RegularCoffee...
//! Fetching recipes...
//! Brewing RegularCoffee with --coffee=180 --taste strong
//! ```
//!
//! # API Examples
//!
//! ```no_run
//! # use longshot::{ecam::*, protocol::*};
//! # let _ = async {
//! # let device_name = "00000000-0000-0000-0000-000000000000";
//! let ecam = ecam_lookup(device_name).await?;
//! let req = Request::BeverageDispensingMode(
//!     EcamBeverageId::LongCoffee.into(),
//!     EcamOperationTrigger::Start.into(),
//!     vec![RecipeInfo::new(EcamIngredients::Coffee, 250)],
//!     EcamBeverageTasteType::Prepare.into(),
//! );
//! ecam.write_request(req).await?;
//! # Result::<(), EcamError>::Ok(())
//! # };
//! ```
//!
//! # Demo
//!
//! ![Demo of brewing a cappuccino](https://user-images.githubusercontent.com/512240/200137316-a09304e8-b34a-41ff-a847-af71af521ef8.gif)

pub mod display;
pub mod ecam;
pub mod logging;
pub mod operations;
mod prelude;
pub mod protocol;
