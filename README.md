# Week 6: Async Rust Gateway Service

![Week 6](image.png)

**Complete IIoT telemetry system in a single Cargo workspace.**

## Introduction

This project represents Week 6 of a 12-week Industrial IoT (IIoT) systems engineer transition plan. It demonstrates **professional-grade async Rust programming** for embedded systems integration, bridging the gap between resource-constrained microcontrollers and cloud infrastructure.

The system implements a **dual-sensor wireless telemetry pipeline** using industry-standard protocols and modern Rust async patterns. Real-world sensor data flows from embedded devices through LoRa radio links, gets parsed and structured on a local gateway service, and is prepared for cloud ingestion—all with production-quality error handling, structured logging, and graceful degradation.

### What This Project Demonstrates

- **Async Rust Mastery**: Tokio runtime, async/await patterns, channels, and concurrent task management
- **Embedded Integration**: Subprocess management, stdout parsing, and hardware abstraction
- **Production Patterns**: Structured logging, error handling, graceful shutdown, and backpressure
- **Real Hardware**: Not a simulation—runs on actual STM32 Nucleo boards with sensors and radios
- **Future-Ready Architecture**: Clean separation of concerns designed for Week 7+ cloud integration

### System Components

This self-contained workspace includes:

- **Node 1 Firmware** (`node1-firmware/`): Remote sensor node with BME680 + SHT31-D environmental sensors
- **Node 2 Firmware** (`node2-firmware/`): LoRa gateway with BMP280 local sensor and JSON output
- **Gateway Service** (`gateway-service/`): Tokio-based async service for telemetry aggregation

## 🚀 Quick Start

**See [QUICKSTART.md](QUICKSTART.md) for the fastest way to get running.**

For workspace details and architecture, see [WORKSPACE_README.md](WORKSPACE_README.md).

---

## Technology Stack

### Embedded Firmware (STM32F446)

| Technology | Purpose | Version |
|------------|---------|---------|
| **Rust (nightly)** | Systems programming language | 1.92.0-nightly |
| **RTIC** | Real-Time Interrupt-driven Concurrency | 1.1.4 |
| **defmt** | Efficient embedded logging | 0.3.100 |
| **embedded-hal** | Hardware Abstraction Layer | 0.2.7 / 1.0.0 |
| **stm32f4xx-hal** | STM32F4 peripheral drivers | 0.23.0 |
| **LoRa (RYLR998)** | 915MHz wireless module | AT commands |
| **BME680** | Temp, humidity, gas sensor | I2C |
| **SHT31-D** | Humidity & temp sensor | I2C |
| **BMP280** | Barometric pressure sensor | I2C |
| **SSD1306** | 128x64 OLED display | I2C |

### Gateway Service (Async Rust)

| Technology | Purpose | Version |
|------------|---------|---------|
| **Tokio** | Async runtime | 1.42 (full features) |
| **serde/serde_json** | JSON ser/de | 1.0 |
| **tracing** | Structured logging | 0.1 |
| **tracing-subscriber** | Log aggregation | 0.3 |
| **anyhow** | Error handling | 1.0 |
| **probe-rs** | Embedded debug/flash | subprocess |

### Development Tools

- **Cargo Workspaces**: Multi-package project organization
- **Make**: Build automation and convenience targets
- **probe-rs**: Firmware flashing via ST-Link and RTT logging
- **Git**: Version control and collaboration

## Architecture

### System-Level Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Week 6 Architecture                         │
│                                                                     │
│  ┌──────────────┐     LoRa      ┌─────────────────┐                │
│  │   Node 1     │    915 MHz    │     Node 2      │                │
│  │ (Remote)     │ ────────────> │   (Gateway)     │                │
│  │              │               │                 │                │
│  │ • BME680     │               │ • BMP280        │                │
│  │ • SHT31-D    │               │ • LoRa RX       │                │
│  │ • LoRa TX    │               │ • JSON output   │                │
│  │ • OLED       │               │ • OLED          │                │
│  └──────────────┘               └────────┬────────┘                │
│   STM32F446                              │                         │
│                                          │ defmt/RTT               │
│                                          ▼                         │
│                                   ┌─────────────┐                  │
│                                   │  probe-rs   │                  │
│                                   │  (spawned)  │                  │
│                                   └──────┬──────┘                  │
│                                          │ stdout                  │
│                                          ▼                         │
│  ┌────────────────────────────────────────────────────────┐       │
│  │        Gateway Service (Tokio Async Runtime)           │       │
│  │                                                         │       │
│  │  ┌──────────────┐       ┌──────────────────┐          │       │
│  │  │ Parser Task  │──────>│  MPSC Channel    │          │       │
│  │  │ (JSON extract)│      │  (bounded: 100)  │          │       │
│  │  └──────────────┘       └────────┬─────────┘          │       │
│  │                                   │                     │       │
│  │                                   ▼                     │       │
│  │                         ┌──────────────────┐           │       │
│  │                         │ Processor Task   │           │       │
│  │                         │ • Structured logs│           │       │
│  │                         │ • Week 7: MQTT   │           │       │
│  │                         │ • Week 7: InfluxDB│          │       │
│  │                         └──────────────────┘           │       │
│  └────────────────────────────────────────────────────────┘       │
│   Rust async/await with backpressure                             │
└────────────────────────────────────────────────────────────────────┘
```

### Core Capabilities

- **Subprocess Management**: Spawns and monitors probe-rs to run Node 2 gateway firmware
- **Stdout Parsing**: Extracts JSON telemetry from defmt logging output with robust error handling
- **Channel Architecture**: Producer-consumer pattern with bounded MPSC channels (capacity: 100)
- **Structured Logging**: Tracing framework with contextual key-value pairs for observability
- **Graceful Shutdown**: Clean Ctrl+C handling with proper resource cleanup
- **Backpressure Handling**: Natural flow control through bounded channels prevents memory exhaustion

### Component Responsibilities

**Main Task**:
- Spawns probe-rs subprocess with Node 2 firmware
- Creates parser and processor tasks
- Waits for Ctrl+C signal (tokio::signal::ctrl_c)
- Coordinates graceful shutdown and resource cleanup

**Parser Task** (`parse_probe_rs_output`):
- Reads probe-rs stdout line-by-line using async BufReader
- Extracts JSON from defmt log lines (handles source location suffixes)
- Deserializes with serde_json into TelemetryPacket struct
- Sends to bounded MPSC channel with error handling

**Processor Task** (`process_telemetry`):
- Receives telemetry packets from channel
- Logs structured data with tracing (Node 1 + Node 2 sensors)
- **Week 7 TODO**: Publish to MQTT broker
- **Week 7 TODO**: Write to InfluxDB time-series database

## Hardware Configuration

### Node 1 - Remote Sensor Transmitter
- **Board**: STM32F446 Nucleo-64
- **ST-Link Probe**: `0483:374b:0671FF3833554B3043164817`
- **Sensors**: BME680 (temp, humidity, gas), SHT31-D (redundant temp/humidity)
- **Radio**: RYLR998 LoRa module (915 MHz, 10dBm)
- **Display**: SSD1306 128x64 OLED (I2C)
- **Function**: Reads sensors every 10 seconds, transmits via LoRa with binary protocol

### Node 2 - Gateway with Local Sensor
- **Board**: STM32F446 Nucleo-64
- **ST-Link Probe**: `0483:374b:066DFF3833584B3043115433`
- **Sensor**: BMP280 (barometric pressure + temperature)
- **Radio**: RYLR998 LoRa module (receive mode)
- **Display**: SSD1306 128x64 OLED (I2C)
- **Function**: Receives LoRa packets, validates CRC, sends ACKs, outputs JSON telemetry

## Building

```bash
# Build everything (gateway service only by default)
cargo build --release

# Build all workspace members explicitly
cargo build --release --workspace

# Build individual packages with correct targets
cargo build --package node1-firmware --release --target thumbv7em-none-eabihf
cargo build --package node2-firmware --release --target thumbv7em-none-eabihf
cargo build --package wk6-async-gateway --release

# Using Makefile
make clean    # Clean build artifacts
make build    # Build all packages
```

## Running

### Prerequisites

1. **Both Nucleo boards connected** via USB
2. **probe-rs installed**: `cargo install probe-rs-tools --locked`

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
cargo build --package node1-firmware --release --target thumbv7em-none-eabihf
probe-rs run --probe 0483:374b:0671FF3833554B3043164817 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/node1-firmware

# Terminal 2: Gateway service (spawns Node 2)
cargo run --package wk6-async-gateway
```

### Expected Output

```
INFO Week 6 Async Gateway Service starting
INFO Spawning probe-rs subprocess probe="0483:374b:066DFF3833584B3043115433" ...
INFO Service running. Press Ctrl+C to stop.
INFO Starting probe-rs output parser
INFO Starting telemetry processor

# When packets arrive (Node 1 remote sensor data):
INFO Telemetry packet received node_id="N2" timestamp_ms=12000 temp_c=27.6 humidity_pct=54.1 rssi_dbm=-39
INFO Processing telemetry packet timestamp_ms=12000 node_id="N2" n1_temperature=27.6 n1_humidity=54.1 n1_gas_resistance=88797 rssi=-39 snr=13 packets_received=1 crc_errors=0

# Node 2 local BMP280 sensor data:
INFO Gateway local sensor (BMP280) n2_temperature=Some(25.3) n2_pressure=Some(1013.2)
```

### Stopping

Press `Ctrl+C` to trigger graceful shutdown:
- Service kills probe-rs subprocess
- Processor task drains remaining packets
- All resources cleaned up
- Exit code 0

## Configuration

Hard-coded in [gateway-service/src/main.rs](gateway-service/src/main.rs):

```rust
let probe_id = "0483:374b:066DFF3833584B3043115433"; // Node 2
let chip = "STM32F446RETx";
let firmware_path = "target/thumbv7em-none-eabihf/release/node2-firmware";
```

**Channel capacity**: 100 packets (bounded MPSC for backpressure)

## JSON Telemetry Schema

Comprehensive dual-sensor telemetry format:

```json
{
  "ts": 12000,           // Timestamp in milliseconds since boot
  "id": "N2",            // Node ID (gateway identifier)
  "n1": {                // Node 1 sensor data (via LoRa from remote)
    "t": 27.6,           // Temperature (°C) - BME680 + SHT31-D average
    "h": 54.1,           // Humidity (%) - BME680 + SHT31-D average
    "g": 88797           // Gas resistance (ohms) - BME680 only
  },
  "n2": {                // Node 2 local sensor data (BMP280, read every 500ms)
    "t": 25.3,           // Temperature (°C)
    "p": 1013.2          // Pressure (hPa - hectopascals/millibars)
  },
  "sig": {               // LoRa signal quality metrics
    "rssi": -39,         // RSSI in dBm (Received Signal Strength Indicator)
    "snr": 13            // SNR in dB (Signal-to-Noise Ratio)
  },
  "sts": {               // Communication statistics
    "rx": 42,            // Total packets successfully received
    "err": 1             // CRC validation errors encountered
  }
}
```

## Logging

Uses `tracing` for production-grade structured logging:

```bash
# Default: INFO level
cargo run

# Debug level (includes BMP280 readings)
RUST_LOG=debug cargo run

# Trace level (very verbose, shows all parsing)
RUST_LOG=trace cargo run

# JSON output (for log aggregators like Datadog, ELK)
RUST_LOG=info cargo run --features json-logs
```

## Testing

Run unit tests:

```bash
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_extract_json_from_log_line
```

Current test coverage:
- ✅ `test_extract_json_from_log_line`: Validates JSON extraction from defmt logs
- ✅ `test_extract_json_no_match`: Ensures non-JSON lines are ignored

## Dependencies

### Gateway Service Key Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| **tokio** | 1.42 | Async runtime (full features: I/O, process, signals, channels) |
| **serde** | 1.0 | Serialization framework |
| **serde_json** | 1.0 | JSON parsing and generation |
| **tracing** | 0.1 | Structured, composable logging |
| **tracing-subscriber** | 0.3 | Log aggregation with env filtering |
| **anyhow** | 1.0 | Error handling with context chains |

See [Cargo.toml](gateway-service/Cargo.toml) for complete dependency list.

## Key Learnings

### Tokio Subprocess Management

```rust
let mut child = Command::new("probe-rs")
    .args(&["run", "--probe", probe_id, ...])
    .stdout(Stdio::piped())      // Capture stdout for parsing
    .stderr(Stdio::inherit())    // Pass through stderr for errors
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
    line_buf.clear();  // Reuse buffer to avoid allocations
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

defmt adds source location metadata to log output:

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
- Prevents unbounded memory growth
- Graceful degradation under load

## Project Structure

```
wk6-async-gateway/           # Cargo workspace root
├── Cargo.toml               # Workspace configuration
├── rust-toolchain.toml      # Nightly (for firmware embedded-hal features)
├── gateway-service/         # Async Rust service (Week 6 core)
│   ├── Cargo.toml
│   └── src/main.rs          # 275 lines - parser, processor, main
├── node1-firmware/          # STM32 sensor node (from Week 3)
│   ├── Cargo.toml
│   ├── build.rs             # Embeds memory.x for workspace builds
│   ├── rust-toolchain.toml  # Nightly
│   ├── memory.x             # STM32F446 linker script
│   ├── .cargo/config.toml   # Target and runner configuration
│   └── src/main.rs          # RTIC-based firmware
├── node2-firmware/          # STM32 gateway (from Week 5)
│   ├── Cargo.toml
│   ├── build.rs             # Embeds memory.x for workspace builds
│   ├── rust-toolchain.toml  # Nightly
│   ├── memory.x             # STM32F446 linker script
│   ├── .cargo/config.toml   # Target and runner configuration
│   └── src/main.rs          # RTIC-based gateway firmware
├── Makefile                 # Convenient build targets (n1, n2, gateway)
├── build-n1.sh              # Build & run Node 1
├── build-n2.sh              # Build & run Node 2
├── run-gateway.sh           # Run gateway service
├── README.md                # This file
├── QUICKSTART.md            # Quick start guide
├── WORKSPACE_README.md      # Workspace architecture details
├── WORKSPACE_SETUP.md       # Technical setup documentation
├── BMP280_IMPLEMENTATION.md # BMP280 sensor integration details
├── NOTES.md                 # Technical learning notes
├── TROUBLESHOOTING.md       # Common issues and solutions
└── TODO.md                  # Completed tasks and future enhancements
```

## Future Enhancements

### Week 7: Cloud Integration (MQTT + InfluxDB)

**Objective**: Publish telemetry to cloud infrastructure for real-time monitoring and time-series storage

- [ ] **MQTT Client** (rumqttc crate)
  - [ ] Connect to Mosquitto broker (TLS support)
  - [ ] Topic hierarchy design:
    - `iiot/node1/temperature`
    - `iiot/node1/humidity`
    - `iiot/node1/gas_resistance`
    - `iiot/node2/temperature`
    - `iiot/node2/pressure`
    - `iiot/signal/rssi`
    - `iiot/signal/snr`
    - `iiot/stats/packets_received`
    - `iiot/stats/crc_errors`
  - [ ] QoS level 1 (at least once delivery)
  - [ ] Offline buffering (queue to disk if broker unavailable)
  - [ ] Reconnection logic with exponential backoff
  - [ ] Last Will and Testament (LWT) for gateway health

- [ ] **InfluxDB Writer** (influxdb2 crate)
  - [ ] Convert telemetry to line protocol format
  - [ ] Batched writes (buffer 10-100 points before flush)
  - [ ] Tag design: `node_id`, `sensor_type`, `location`
  - [ ] Field design: all numeric values as f64
  - [ ] Timestamp: convert from milliseconds to RFC3339
  - [ ] Error handling for write failures
  - [ ] Retry logic with circuit breaker

- [ ] **Configuration Management**
  - [ ] TOML configuration file (`config.toml`)
  - [ ] Environment variable overrides
  - [ ] Secrets management (MQTT password, InfluxDB token from env)

### Week 8: Observability & Visualization

**Objective**: Build comprehensive monitoring dashboards and alerting

- [ ] **Grafana Dashboards**
  - [ ] Real-time sensor value gauges (Node 1 + Node 2)
  - [ ] Temperature comparison chart (Node 1 vs Node 2)
  - [ ] Humidity trend over time
  - [ ] Gas resistance heatmap
  - [ ] LoRa signal quality metrics (RSSI, SNR)
  - [ ] CRC error rate graph
  - [ ] System uptime and packet statistics

- [ ] **Prometheus Metrics** (prometheus crate)
  - [ ] Expose `/metrics` HTTP endpoint
  - [ ] Counter: `telemetry_packets_total{node}`
  - [ ] Counter: `telemetry_crc_errors_total`
  - [ ] Gauge: `telemetry_temperature_celsius{node}`
  - [ ] Gauge: `telemetry_humidity_percent{node}`
  - [ ] Gauge: `lora_rssi_dbm`
  - [ ] Histogram: `telemetry_processing_duration_seconds`

- [ ] **Alert Rules**
  - [ ] Node offline (no packets for > 60 seconds)
  - [ ] High CRC error rate (> 10% over 5 minutes)
  - [ ] Temperature out of range (< 0°C or > 50°C)
  - [ ] Pressure rapid change (> 5 hPa in 1 hour - storm warning)
  - [ ] Low LoRa signal quality (RSSI < -100 dBm)

- [ ] **Distributed Tracing** (OpenTelemetry)
  - [ ] Trace packet journey: sensor → LoRa → gateway → MQTT → InfluxDB
  - [ ] Jaeger integration for trace visualization
  - [ ] Performance bottleneck identification

### Week 9: OPC-UA Integration

**Objective**: Expose sensor data via OPC-UA protocol for industrial integration

- [ ] **OPC-UA Server** (opcua crate)
  - [ ] Information model design (namespace, nodes, variables)
  - [ ] Expose sensor values as OPC-UA variables
  - [ ] Historical data access
  - [ ] Subscription support for real-time updates
  - [ ] Security: user authentication, encryption

- [ ] **Testing**
  - [ ] UAExpert client connectivity
  - [ ] Integration with SCADA systems
  - [ ] Performance benchmarking

### Advanced Features (Week 10+)

- [ ] **Web UI Dashboard**
  - [ ] React + Vite frontend
  - [ ] WebSocket for real-time updates
  - [ ] Historical data visualization
  - [ ] Node configuration interface

- [ ] **REST API**
  - [ ] Axum web framework
  - [ ] GET /api/telemetry/latest
  - [ ] GET /api/telemetry/history?start={ts}&end={ts}
  - [ ] GET /api/nodes/{id}/status
  - [ ] OpenAPI/Swagger documentation

- [ ] **Multi-Node Support**
  - [ ] Dynamic node registration
  - [ ] Support for N nodes (not just 2)
  - [ ] Node discovery protocol
  - [ ] Load balancing across gateways

- [ ] **Database Configuration**
  - [ ] PostgreSQL for persistent state
  - [ ] Node configuration table
  - [ ] Alert rules table
  - [ ] User management

- [ ] **Deployment & Operations**
  - [ ] Docker containerization (multi-stage builds)
  - [ ] Docker Compose for full stack (gateway + Mosquitto + InfluxDB + Grafana)
  - [ ] Kubernetes deployment manifests
  - [ ] Helm charts for k8s
  - [ ] CI/CD pipeline (GitHub Actions)
  - [ ] Automated testing (unit, integration, E2E)

- [ ] **Security Hardening**
  - [ ] TLS for all network connections
  - [ ] Certificate management (Let's Encrypt)
  - [ ] API authentication (JWT tokens)
  - [ ] Rate limiting
  - [ ] Input validation and sanitization

- [ ] **Performance Optimization**
  - [ ] Zero-copy parsing where possible
  - [ ] Connection pooling
  - [ ] Batch processing
  - [ ] Profiling with flamegraph
  - [ ] Benchmarking with Criterion

## Related Repositories

This workspace includes firmware from previous weeks:

- **Week 3**: [wk3-binary-protocol](https://github.com/mapfumo/wk3-binary-protocol) - Original Node 1 sensor firmware (binary protocol with CRC)
- **Week 5**: [wk5-gateway-firmware](https://github.com/mapfumo/wk5-gateway-firmware) - Original Node 2 gateway firmware (LoRa receiver + JSON output)
- **Week 7**: wk7-mqtt-influx (coming next) - MQTT and InfluxDB integration

## Contributing

This is a learning project, but feedback is welcome! Areas for improvement:

- Code review and Rust best practices
- Architecture suggestions for scalability
- Security hardening recommendations
- Testing strategies for embedded integration

## License

MIT

---

_Part of the 12-Week IIoT Systems Engineer Transition Plan_
_Week 6 of 12 - Async Rust Gateway Service_

**Author**: Antony (Tony) Mapfumo
**Date**: December 2025
