#!/bin/bash
# Build and run Node 2 (gateway firmware)
clear
cargo build --package node2-firmware --release --target thumbv7em-none-eabihf && \
probe-rs run --probe 0483:374b:066DFF3833584B3043115433 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/node2-firmware
