# Byra Scale

## Build

> Requires rust & docker to be installed.

`cross build --target arm-unknown-linux-gnueabihf`


### Usage

#### Config
```
dout = 23
dt_sck = 24
offset = 521703
calibration = 545351
backoff = 3
retry = 3
```

#### Calibrate
```
./scale calibrate
```

#### Run

```
./scale
```


### Copy binary to raspberry

> Assuming you've setup ssh over USB & added device to network.

* `scp target/arm-unknown-linux-gnueabihf/{release}/scale pi@192.168.7.2:~/scale`

```
sudo ip addr add 192.168.7.1/24 dev enp0s20f0u1 # Depending on how you've configured ssh over usb
cross build --target arm-unknown-linux-gnueabihf && scp target/arm-unknown-linux-gnueabihf/debug/scale pi@192.168.7.2:~/scale
```
