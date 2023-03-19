# Byra Scale

## Build

> Requires rust & docker to be installed.

`cross build --target arm-unknown-linux-gnueabihf`

### Copy binary to raspberry

> Assuming you've setup ssh over USB & added device to network.

* `scp target/arm-unknown-linux-gnueabihf/{release}/scale pi@192.168.7.2:~/scale`

