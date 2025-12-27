# Week 6 Workspace - Self-Sufficient Project

This is a Cargo workspace containing all components needed to run the complete IIoT telemetry system.

## Project Structure

```
wk6-async-gateway/
├── gateway-service/      # Async Rust service (Tokio-based)
│   └── src/main.rs
├── node1-firmware/       # STM32 sensor node firmware
│   ├── src/main.rs
│   ├── memory.x
│   └── .cargo/
├── node2-firmware/       # STM32 gateway firmware
│   ├── src/main.rs
│   ├── memory.x
│   └── .cargo/
├── build-n1.sh          # Build & run Node 1
├── build-n2.sh          # Build & run Node 2
└── run-gateway.sh       # Run gateway service
```

## Quick Start

### Option 1: Run Everything (Recommended)

**Terminal 1** - Node 1 (Sensor):
```bash
./build-n1.sh
```

**Terminal 2** - Gateway Service (spawns Node 2 automatically):
```bash
./run-gateway.sh
```

### Option 2: Manual Control

**Terminal 1** - Node 1:
```bash
cargo build --package node1-firmware --release
probe-rs run --probe 0483:374b:0671FF3833554B3043164817 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/node1-firmware
```

**Terminal 2** - Node 2:
```bash
cargo build --package node2-firmware --release
probe-rs run --probe 0483:374b:066DFF3833584B3043115433 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/node2-firmware
```

**Terminal 3** - Gateway Service:
```bash
cargo run --package wk6-async-gateway
```

## Your Aliases (For Convenience)

You can also use your existing shell aliases from the workspace root:

```bash
# Node 1
alias n1='clear && cargo build --package node1-firmware --release && probe-rs run --probe 0483:374b:0671FF3833554B3043164817 --chip STM32F446RETx target/thumbv7em-none-eabihf/release/node1-firmware'

# Node 2
alias n2='clear && cargo build --package node2-firmware --release && probe-rs run --probe 0483:374b:066DFF3833584B3043115433 --chip STM32F446RETx target/thumbv7em-none-eabihf/release/node2-firmware'

# Gateway service
alias gw='clear && cargo run --package wk6-async-gateway'
```

## What Each Component Does

### Node 1 (node1-firmware)
- Reads BME680 + SHT31-D sensors
- Transmits data via LoRa (RYLR998)
- Binary protocol with CRC
- ACK/retry state machine
- **Probe**: `0483:374b:0671FF3833554B3043164817`

### Node 2 (node2-firmware)
- Receives LoRa packets from Node 1
- Validates CRC, sends ACK
- Outputs JSON telemetry via defmt/RTT
- **Probe**: `0483:374b:066DFF3833584B3043115433`

### Gateway Service (gateway-service)
- Spawns Node 2 firmware via probe-rs subprocess
- Parses JSON from probe-rs stdout
- Tokio-based async architecture
- Structured logging with tracing
- **TODO Week 7**: Publishes to MQTT/InfluxDB

## Building

```bash
# Build everything
cargo build --release --workspace

# Build individual packages
cargo build --package node1-firmware --release
cargo build --package node2-firmware --release
cargo build --package gateway-service --release
```

## Testing

```bash
# Test gateway service
cargo test --package gateway-service

# No tests for firmware (embedded)
```

## Expected Output

When running the complete system:

**Node 1 Terminal**:
```
[INFO] Configuring LoRa module...
[INFO] Binary TX [AUTO]: 10 bytes sent, packet #1
[INFO] ACK received for packet #1
```

**Gateway Service Terminal**:
```
INFO Week 6 Async Gateway Service starting
INFO Telemetry packet received node_id="N2" timestamp_ms=12000 temp_c=27.6 ...
INFO Processing telemetry packet temperature=27.6 humidity=54.1 gas_resistance=84190 ...
```

## Troubleshooting

### "No probe was found"
- Check USB connections for both Nucleo boards
- Run `probe-rs list` to see connected probes

### Build fails with feature errors
- Each firmware package has its own `rust-toolchain.toml` (nightly)
- Gateway service uses stable Rust
- Workspace handles this automatically

### Gateway can't find firmware
- Ensure Node 2 firmware is built first:
  ```bash
  cargo build --package node2-firmware --release
  ```
- Check firmware path in gateway-service/src/main.rs

## Documentation

- [Gateway Service README](gateway-service/README.md) - Detailed docs
- [NOTES.md](NOTES.md) - Technical learnings
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues
- [TODO.md](TODO.md) - Future enhancements

---

*Part of the 12-Week IIoT Systems Engineer Transition Plan*
*Week 6 - Complete Async Gateway System*
