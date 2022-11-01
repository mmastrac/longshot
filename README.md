# longshot

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

