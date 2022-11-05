

# longshot [![docs.rs](https://docs.rs/longshot/badge.svg)](https://docs.rs/longshot) [![crates.io](https://img.shields.io/crates/v/longshot.svg)](https://crates.io/crates/longshot)

Brew coffee from the command-line!

## Examples

Monitor a given device:

```
$ longshot monitor --device-name (device)
Dispensing... [###############################===========]
```

Get the brew information for a given beverage:

```
$ longshot brew  --device-name (device) --beverage regularcoffee
...
```

Brew a beverage:

```
$ longshot brew  --device-name (device) --beverage regularcoffee --coffee 180 --taste strong
Fetching recipe for RegularCoffee...
Fetching recipes...
Brewing RegularCoffee with --coffee=180 --taste strong
```

# Demo

![Demo of brewing a cappuccino](https://user-images.githubusercontent.com/512240/200137316-a09304e8-b34a-41ff-a847-af71af521ef8.gif)
