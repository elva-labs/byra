# Byra Scale

Weight cli for the HX711 module (used on pi zero w), the binary outputs scale information 
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

#### Calibrate
```bash
elva-byra-scale calibrate
```

#### Run

```bash
elva-byra-scale
```

## Build

> Requires rust & docker to be installed.

`cross build --target arm-unknown-linux-gnueabihf`

## Contribute / Dev


### Copy binary to raspberry

> Assuming you've setup ssh over USB & added device to network.

```bash
sudo ip link set dev <interface> down
sudo ip addr add 192.168.7.1/24 dev <interface>
sudo ip link set dev <interface> up
```

* `scp target/arm-unknown-linux-gnueabihf/{release}/scale pi@192.168.7.2:~/scale`

```bash
sudo ip addr add 192.168.7.1/24 dev enp0s20f0u1 # Depending on how you've configured ssh over usb
cross build --target arm-unknown-linux-gnueabihf && scp target/arm-unknown-linux-gnueabihf/debug/scale pi@192.168.7.2:~/scale
```

## Docs

```bash
cargo doc --open --no-deps
```
