#!/bin/bash

set -e

TARGET=byra-pkg

# Building --release, cause readings from the HX711 to become corrupt not sure why atm.
cross build --target arm-unknown-linux-gnueabihf

rm -rf ${TARGET}
mkdir ${TARGET}

cp target/arm-unknown-linux-gnueabihf/debug/elva-byra-scale ${TARGET}
cp target/arm-unknown-linux-gnueabihf/debug/elva-byra-iot-worker ${TARGET}
cp elva-byra-scale/byra.service ${TARGET}
cp elva-byra-iot-worker/byra-iot.service ${TARGET}

zip -r ${TARGET}.zip ${TARGET}

rm -r byra-pkg
