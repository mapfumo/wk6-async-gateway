#!/bin/bash
# Build and run Node 1 (sensor firmware)
clear
cargo build --package node1-firmware --release --target thumbv7em-none-eabihf && \
probe-rs run --probe 0483:374b:0671FF3833554B3043164817 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/node1-firmware
