# Week 6: Async Rust Gateway Service

**Complete IIoT telemetry system in a single Cargo workspace.**

This is a self-contained project with all components needed to run the full hardware-to-cloud pipeline:

- Node 1 firmware (sensor transmitter)
- Node 2 firmware (LoRa gateway)
- Async gateway service (Tokio-based parser/processor)

## 🚀 Quick Start

**See [QUICKSTART.md](QUICKSTART.md) for the fastest way to get running.**

For workspace details and architecture, see [WORKSPACE_README.md](WORKSPACE_README.md).

---

## Overview

This workspace contains a Tokio-based async service that bridges embedded hardware to cloud infrastructure:

- **Subprocess management**: Spawns probe-rs to run Node 2 gateway firmware
- **stdout parsing**: Extracts JSON telemetry from defmt logging output
- **Channel architecture**: Producer-consumer pattern with bounded channels
- **Structured logging**: tracing framework with contextual information
- **Graceful shutdown**: Clean Ctrl+C handling with resource cleanup

## What's New in Week 6

### From Week 5 (Embedded Gateway)

```
Node 1 → LoRa → Node 2 Gateway → defmt/RTT → probe-rs terminal
                                     ↓
                            JSON logged to stdout
```

### Week 6 (Async Service)

```
Node 1 → LoRa → Node 2 (spawned by Week 6) → probe-rs stdout
                                                ↓
                                        Week 6 Parser
                                                ↓
                                        Tokio Channel
                                                ↓
                                        Processor Task
                                                ↓
                                    (Future: MQTT, InfluxDB)
```

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────┐
│  Week 6 Async Gateway Service (Tokio Runtime)          │
│                                                         │
│  ┌──────────────┐       ┌──────────────┐              │
│  │   Main Task  │──────>│ probe-rs run │              │
│  └──────────────┘       └───────┬──────┘              │
│                                  │ stdout               │
│                                  ▼                      │
│                         ┌─────────────────┐            │
│                         │  Parser Task    │            │
│                         │ (BufReader)     │            │
│                         └────────┬────────┘            │
│                                  │ TelemetryPacket     │
│                                  ▼                      │
│                         ┌─────────────────┐            │
│                         │ mpsc::channel   │            │
│                         │  (capacity=100) │            │
│                         └────────┬────────┘            │
│                                  │                      │
│                                  ▼                      │
│                         ┌─────────────────┐            │
│                         │ Processor Task  │            │
│                         │ (logs for now)  │            │
│                         └─────────────────┘            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**Main Task**:
- Spawns probe-rs subprocess with Node 2 firmware
- Creates parser and processor tasks
- Waits for Ctrl+C signal
- Coordinates graceful shutdown

**Parser Task** (`parse_probe_rs_output`):
- Reads probe-rs stdout line-by-line (async)
- Extracts JSON from defmt log lines
- Deserializes with serde_json
- Sends to channel (with error handling)

**Processor Task** (`process_telemetry`):
- Receives packets from channel
- Logs structured telemetry data
- **TODO Week 7**: Publish to MQTT
- **TODO Week 7**: Write to InfluxDB

## Hardware Configuration

Uses the same setup from Week 5:

### Node 1 - Remote Sensor
- **ST-Link Probe**: `0483:374b:0671FF3833554B3043164817`
- Runs `wk3-binary-protocol` firmware
- Transmits sensor data via LoRa every 10 seconds

### Node 2 - Gateway (Spawned by Week 6 service)
- **ST-Link Probe**: `0483:374b:066DFF3833584B3043115433`
- Runs `wk5-gateway-firmware` (via probe-rs subprocess)
- Receives LoRa data, outputs JSON via defmt

## Building

```bash
# Build everything
cargo build --release --workspace

# Build individual packages
cargo build --package node1-firmware --release
cargo build --package node2-firmware --release
cargo build --package wk6-async-gateway --release

# Using Makefile
make clean    # Clean build artifacts
make build    # Build all packages
```

## Running

### Prerequisites

1. **Both Nucleo boards connected** via USB

### Start the Service

**Easiest method** - Use the Makefile:

```bash
# Terminal 1: Start Node 1 (sensor transmitter)
make n1

# Terminal 2: Start Gateway (auto-spawns Node 2 and parses telemetry)
make gateway
```

**Alternative** - Use shell scripts:

```bash
# Terminal 1
./build-n1.sh

# Terminal 2
./run-gateway.sh
```

**Manual** - Use cargo directly:

```bash
# Terminal 1: Node 1
cargo build --package node1-firmware --release
probe-rs run --probe 0483:374b:0671FF3833554B3043164817 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/node1-firmware

# Terminal 2: Gateway service (spawns Node 2)
cargo run --package wk6-async-gateway
```

The service will:
1. Spawn probe-rs to run Node 2 firmware
2. Start parsing stdout for JSON
3. Log received telemetry packets

### Expected Output

```
INFO Week 6 Async Gateway Service starting
INFO Spawning probe-rs subprocess probe="0483:374b:066DFF3833584B3043115433" ...
INFO Service running. Press Ctrl+C to stop.
INFO Starting probe-rs output parser
INFO Starting telemetry processor

# When packets arrive (Node 1 remote sensor data):
INFO Telemetry packet received node_id="N2" timestamp_ms=12000 temp_c=27.6 humidity_pct=54.1 rssi_dbm=-39
INFO Processing telemetry packet timestamp_ms=12000 node_id="N2" n1_temperature=27.6 n1_humidity=54.1 n1_gas_resistance=84190 rssi=-39 snr=13 packets_received=1 crc_errors=0

# Node 2 local BMP280 sensor data:
INFO Gateway local sensor (BMP280) n2_temperature=Some(25.3) n2_pressure=Some(1013.2)
```

### Stopping

Press `Ctrl+C` to trigger graceful shutdown:
- Service kills probe-rs subprocess
- Processor task drains remaining packets
- All resources cleaned up

## Configuration

Hard-coded in [gateway-service/src/main.rs](gateway-service/src/main.rs):

```rust
let probe_id = "0483:374b:066DFF3833584B3043115433"; // Node 2
let chip = "STM32F446RETx";
let firmware_path = "target/thumbv7em-none-eabihf/release/node2-firmware";
```

**Channel capacity**: 100 packets (bounded MPSC)

## JSON Schema

Matches Week 5 output format:

```json
{
  "ts": 12000,           // Timestamp in milliseconds
  "id": "N2",            // Node ID (gateway)
  "n1": {                // Node 1 sensor data (via LoRa)
    "t": 27.6,           // Temperature (°C)
    "h": 54.1,           // Humidity (%)
    "g": 84190           // Gas resistance (ohms)
  },
  "n2": {                // Node 2 local sensor (BMP280) - read every 500ms
    "t": 25.3,           // Temperature (°C)
    "p": 1013.2          // Pressure (hPa - hectopascals)
  },
  "sig": {               // LoRa signal quality
    "rssi": -39,         // RSSI in dBm
    "snr": 13            // SNR in dB
  },
  "sts": {               // Statistics
    "rx": 42,            // Packets received
    "err": 1             // CRC errors
  }
}
```

## Logging

Uses `tracing` for structured logging:

```bash
# Default: INFO level
cargo run

# Debug level
RUST_LOG=debug cargo run

# Trace level (very verbose)
RUST_LOG=trace cargo run
```

## Testing

Run unit tests:

```bash
cargo test
```

Current tests:
- `test_extract_json_from_log_line`: Validates JSON extraction from defmt logs
- `test_extract_json_no_match`: Ensures non-JSON lines are ignored

## Dependencies

Key crates:

- **tokio** (1.42): Async runtime with full features
- **serde/serde_json**: JSON serialization/deserialization
- **tracing/tracing-subscriber**: Structured logging framework
- **anyhow**: Error handling with context

See [Cargo.toml](Cargo.toml) for complete list.

## Key Learnings

### Tokio Subprocess Management

```rust
let mut child = Command::new("probe-rs")
    .args(&["run", "--probe", probe_id, ...])
    .stdout(Stdio::piped())      // Capture stdout
    .stderr(Stdio::inherit())    // Pass through stderr
    .spawn()?;

let stdout = child.stdout.take().unwrap();
```

**Critical**: Must call `.stdout.take()` to move ownership to parser task.

### Async Line-by-Line Reading

```rust
use tokio::io::{AsyncBufReadExt, BufReader};

let reader = BufReader::new(stdout);
let mut line_buf = String::new();

loop {
    line_buf.clear();
    match reader.read_line(&mut line_buf).await {
        Ok(0) => break,  // EOF
        Ok(_) => {
            // Process line
        }
        Err(e) => { /* Handle error */ }
    }
}
```

### defmt Log Format Parsing

defmt adds source location to log output:

```
[INFO] JSON sent via VCP: {...}\n (wk5_gateway_firmware src/main.rs:573)
```

Parser must handle:
1. The `JSON sent via VCP: ` prefix
2. Escaped `\n` characters in the JSON string
3. Actual newline after the JSON
4. Source location suffix in parentheses

**Solution**:
```rust
json_str
    .split(" (")              // Remove source location
    .next()
    .trim_end_matches("\\n")  // Remove escaped \n
    .trim_end_matches('\n')   // Remove actual newline
    .trim()
```

### Channel Backpressure

```rust
let (tx, rx) = mpsc::channel::<TelemetryPacket>(100);
```

With capacity=100:
- Parser blocks if channel is full (natural backpressure)
- Processor controls consumption rate
- No unbounded memory growth

## Project Structure

```
wk6-async-gateway/           # Cargo workspace root
├── Cargo.toml               # Workspace configuration
├── gateway-service/         # Async Rust service (Week 6 core)
│   ├── Cargo.toml
│   └── src/main.rs
├── node1-firmware/          # STM32 sensor node (from Week 3)
│   ├── Cargo.toml
│   ├── src/main.rs
│   ├── memory.x
│   └── .cargo/
├── node2-firmware/          # STM32 gateway (from Week 5)
│   ├── Cargo.toml
│   ├── src/main.rs
│   ├── memory.x
│   └── .cargo/
├── Makefile                 # Convenient build targets
├── build-n1.sh              # Build & run Node 1
├── build-n2.sh              # Build & run Node 2
├── run-gateway.sh           # Run gateway service
├── README.md                # This file
├── QUICKSTART.md            # Quick start guide
├── WORKSPACE_README.md      # Workspace architecture
├── NOTES.md                 # Technical learning notes
├── TROUBLESHOOTING.md       # Common issues and solutions
└── TODO.md                  # Future enhancements
```

## Next Steps (Week 7)

- [ ] Add MQTT client (rumqttc)
- [ ] Publish telemetry to topics
- [ ] Add InfluxDB writer
- [ ] Implement offline buffering
- [ ] Add reconnection logic
- [ ] Configuration file (TOML)

## Related Repositories

This workspace includes firmware from:

- **Week 3**: [wk3-binary-protocol](../wk3-binary-protocol) - Original Node 1 sensor firmware
- **Week 5**: [wk5-gateway-firmware](../wk5-gateway-firmware) - Original Node 2 gateway firmware
- **Week 7**: wk7-mqtt-influx (coming next) - MQTT and InfluxDB integration

## License

MIT

---

*Part of the 12-Week IIoT Systems Engineer Transition Plan*
*Week 6 of 12 - Async Rust Gateway Service*
