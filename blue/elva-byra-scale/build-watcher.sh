#!/bin/bash

while inotifywait -r target/arm-unknown-linux-gnueabihf/debug/elva-byra-scale; do 
	cp target/arm-unknown-linux-gnueabihf/debug/elva-byra-scale build
done
