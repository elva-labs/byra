# Byra Scale

> This module / implementation exist for learning purposes

HX711 module (used on pi zero w), the binary outputs scale information 
to stdout or file on a configured interval.

### Usage

#### Config
```toml
dout = 23
dt_sck = 24
offset = 521703
calibration = 545351
backoff = 3
retry = 3
```

#### Run

```bash
elva-byra-scale -v
```

## Build

> Requires rust & docker to be installed.

`cross build --target arm-unknown-linux-gnueabihf`
