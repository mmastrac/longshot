# longshot [![docs.rs](https://docs.rs/longshot/badge.svg)](https://docs.rs/longshot) [![crates.io](https://img.shields.io/crates/v/longshot.svg)](https://crates.io/crates/longshot)

Brew coffee from the command-line!

## Details

Longshot is an API and command-line application to brew coffee from the command-line (or whatever
front-end is built). At this time it supports DeLonghi ECAM-based Bluetooth-Low-Energy devices, and has only been tested on the
Dinamica Plus over Bluetooth.

The protocol for status and monitoring has been mostly decoded, but at this time is only available in
source form.

## Command-Line Examples

Monitor the given device (will continue until you press Ctrl+C):

```console
$ longshot monitor --device-name (device)
Dispensing... [###############################===========]
```

Get the brew information for a given beverage:

```console
$ longshot brew  --device-name (device) --beverage regularcoffee
...
```

Brew a beverage:

```console
$ longshot brew  --device-name (device) --beverage regularcoffee --coffee 180 --taste strong
Fetching recipe for RegularCoffee...
Fetching recipes...
Brewing RegularCoffee...
```

## API Examples

Brew a long coffee with 250 impulses of water (approximately the size of an average North American coffee mug, or slightly more).

```rust
let ecam = ecam_lookup(device_name).await?;
let req = Request::BeverageDispensingMode(
    EcamBeverageId::LongCoffee.into(),
    EcamOperationTrigger::Start.into(),
    vec![RecipeInfo::new(EcamIngredients::Coffee, 250)],
    EcamBeverageTasteType::Prepare.into(),
);
ecam.write_request(req).await?;
```

## Demo

![Demo of brewing a cappuccino](https://user-images.githubusercontent.com/512240/200137316-a09304e8-b34a-41ff-a847-af71af521ef8.gif)
