# Quick Start Guide

## The Easiest Way to Run Everything

### Option 1: Using Make (Recommended)

**Terminal 1** - Start Node 1:
```bash
make n1
```

**Terminal 2** - Start Gateway:
```bash
make gateway
```

That's it! The gateway automatically spawns Node 2.

### Option 2: Using Shell Scripts

**Terminal 1**:
```bash
./build-n1.sh
```

**Terminal 2**:
```bash
./run-gateway.sh
```

### Option 3: Define Aliases in Your Shell

Add to your `~/.bashrc` or `~/.zshrc`:

```bash
# Navigate to workspace
alias wk6='cd ~/dev/4-month-plan/wk6-async-gateway'

# From workspace root:
alias n1='cargo build --package node1-firmware --release --target thumbv7em-none-eabihf && probe-rs run --probe 0483:374b:0671FF3833554B3043164817 --chip STM32F446RETx target/thumbv7em-none-eabihf/release/node1-firmware'

alias n2='cargo build --package node2-firmware --release --target thumbv7em-none-eabihf && probe-rs run --probe 0483:374b:066DFF3833584B3043115433 --chip STM32F446RETx target/thumbv7em-none-eabihf/release/node2-firmware'

alias gw='cargo run --package wk6-async-gateway --release'
```

Then:
```bash
wk6          # Go to workspace
n1           # Terminal 1
gw           # Terminal 2
```

## What You'll See

### Terminal 1 (Node 1):
```
[INFO] Configuring LoRa module...
[INFO] LoRa module configured
[INFO] Binary TX [AUTO]: 10 bytes sent, packet #1
[INFO] ACK received for packet #1
[INFO] Binary TX [AUTO]: 10 bytes sent, packet #2
...
```

### Terminal 2 (Gateway):
```
INFO Week 6 Async Gateway Service starting
INFO Spawning probe-rs subprocess...
INFO Telemetry packet received node_id="N2" timestamp_ms=12000 temp_c=27.6 humidity_pct=54.1 rssi_dbm=-39
INFO Processing telemetry packet n1_temperature=27.6 n1_humidity=54.1 n1_gas_resistance=84190 rssi=-39 snr=13
INFO Gateway local sensor (BMP280) n2_temperature=Some(25.3) n2_pressure=Some(1013.2)
```

## Stopping

Press `Ctrl+C` in each terminal. Everything shuts down gracefully.

## Troubleshooting

**"No probe was found"**:
```bash
probe-rs list  # Check both boards are connected
```

**Build errors**:
```bash
make clean     # Clean and rebuild
```

**Can't find firmware**:
```bash
# Build Node 2 first with correct target:
cd node2-firmware && cargo build --release --target thumbv7em-none-eabihf
```

## What's Next?

See [WORKSPACE_README.md](WORKSPACE_README.md) for full documentation.
