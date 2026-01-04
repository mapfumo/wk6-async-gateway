# Week 6: Async Rust Gateway Service - Bridging Embedded and Cloud

**Status**: ✅ Complete  
**Focus**: Tokio async runtime, subprocess management, structured logging  
**Key Achievement**: Production-quality async Rust service that bridges embedded firmware to cloud infrastructure

---

## Series Navigation

- [Week 1: RTIC LoRa Basics](https://github.com/mapfumo/wk1_rtic_lora) | [Blog Post](https://www.mapfumo.net/posts/building-deterministic-iiot-systems-with-embedded-rust-and-rtic/)
- [Week 2: Sensor Fusion & Constraints](https://github.com/mapfumo/wk2-lora-sensor-fusion) | [Blog Post](https://www.mapfumo.net/posts/lora-sensor-fusion-when-simple-becomes-reliable/)
- [Week 3: Binary Protocols & CRC](https://github.com/mapfumo/wk3-binary-protocol)
- [Week 5: Gateway Firmware](https://github.com/mapfumo/wk5-gateway-firmware) | [Blog Post](https://www.mapfumo.net/posts/gateway-firmware-from-wireless-to-desktop-wk5/)
- **Week 6: Async Gateway Service** (You are here) | [Blog Post](https://www.mapfumo.net/posts/async-rust-gateway-from-embedded-firmware-to-cloud-infrastructure/)
- Week 7: MQTT & InfluxDB (Coming soon)

---

## Table of Contents

- [Overview](#overview)
- [Week 6 Focus: Async Rust for Real Systems](#week-6-focus-async-rust-for-real-systems)
- [Architecture](#architecture)
- [Key Technical Achievements](#key-technical-achievements)
- [Hardware Configuration](#hardware-configuration)
- [Building & Running](#building--running)
- [Performance Characteristics](#performance-characteristics)
- [Current Status](#current-status)

---

## Overview

![Week 6](image.png)

Week 6 transforms the isolated firmware components (Weeks 3 and 5) into a **unified, production-ready system** using modern async Rust patterns. This is no longer just "getting data from A to B" - it's about building **observable, maintainable, cloud-ready infrastructure**.

**What Changed from Week 5**:

- ✅ Unified Cargo workspace (firmware + service in one repository)
- ✅ Tokio async runtime for concurrent task management
- ✅ Subprocess management (probe-rs spawning and monitoring)
- ✅ Structured logging with tracing (not just println!)
- ✅ Producer-consumer architecture with bounded channels
- ✅ Graceful shutdown and resource cleanup
- ✅ Foundation for Week 7 MQTT/InfluxDB integration

**Why This Matters**: Week 6 is where the project crosses from "embedded hobby project" to "professional IIoT system architecture."

---

## Week 6 Focus: Async Rust for Real Systems

### The Challenge

Up through Week 5, we had:

- Node 1: Sensor → LoRa transmit
- Node 2: LoRa receive → JSON output via defmt/RTT

But **defmt/RTT output only exists in the probe-rs terminal**. There was no way to:

- Store data in databases
- Publish to MQTT brokers
- Build dashboards
- Alert on anomalies
- Do anything "cloud-like"

**Week 6 solves this** by creating a desktop service that:

1. **Spawns Node 2 firmware** as a subprocess (via probe-rs)
2. **Parses JSON telemetry** from probe-rs stdout
3. **Structures and processes** the data asynchronously
4. **Prepares for cloud integration** (Week 7)

### Why Async Rust?

| Requirement               | Sync Rust         | Async Rust (Chosen)       |
| ------------------------- | ----------------- | ------------------------- |
| **Concurrent I/O**        | Threads (heavy)   | Tasks (lightweight)       |
| **Subprocess monitoring** | Blocking polls    | Non-blocking awaits       |
| **Channel operations**    | Locks everywhere  | Lock-free patterns        |
| **Future extensibility**  | Complex threading | Natural async composition |
| **Resource efficiency**   | ~2 MB per thread  | ~2 KB per task            |

**Decision**: Use Tokio because this project will eventually have:

- MQTT client (network I/O)
- InfluxDB writer (network I/O)
- Metrics HTTP endpoint (network I/O)
- Multiple concurrent tasks

Async Rust is the **right tool for this job**.

---

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

### Workspace Structure

This is a **Cargo workspace** with three packages:

```
wk6-async-gateway/
├── Cargo.toml               # Workspace configuration
├── gateway-service/         # Tokio async service (THIS IS NEW!)
│   ├── Cargo.toml
│   └── src/main.rs          # 275 lines
├── node1-firmware/          # From Week 3 (sensor node)
│   ├── Cargo.toml
│   ├── memory.x
│   └── src/main.rs
└── node2-firmware/          # From Week 5 (gateway firmware)
    ├── Cargo.toml
    ├── memory.x
    └── src/main.rs
```

**Why a workspace?**:

- ✅ **Self-contained**: Everything needed to run the system in one repo
- ✅ **Shared dependencies**: Common crates don't duplicate
- ✅ **Unified builds**: `cargo build --workspace` handles everything
- ✅ **Clean separation**: Firmware (no_std) vs service (std)

### Task Architecture

The gateway service uses **three Tokio tasks**:

```rust
┌────────────────┐
│   Main Task    │  (Coordinator)
│  • Spawns tasks│
│  • Waits Ctrl+C│
│  • Cleanup     │
└───┬────────┬───┘
    │        │
    ▼        ▼
┌────────────────┐    Channel    ┌────────────────┐
│  Parser Task   │───────────────>│ Processor Task │
│                │  (bounded:100) │                │
│ • Read stdout  │                │ • Log metrics  │
│ • Extract JSON │                │ • TODO: MQTT   │
│ • Deserialize  │                │ • TODO: InfluxDB│
│ • Send packet  │                │                │
└────────────────┘                └────────────────┘
```

**Pattern**: Producer-consumer with **bounded channel** for backpressure.

---

## Key Technical Achievements

### Achievement 1: Async Subprocess Management

#### The Pattern

```rust
use tokio::process::Command;

let mut child = Command::new("probe-rs")
    .args(&["run", "--probe", probe_id, "--chip", chip, firmware_path])
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())  // ← Function call, not constant!
    .spawn()
    .context("Failed to spawn probe-rs process")?;

let stdout = child.stdout.take()
    .context("Failed to capture stdout")?;
```

#### Why This Matters

**DON'T use `std::process::Command`**:

```rust
use std::process::Command;  // ❌ This blocks Tokio!
let child = Command::new("probe-rs").spawn()?;
// Blocks the entire async runtime on I/O
```

**DO use `tokio::process::Command`**:

```rust
use tokio::process::Command;  // ✅ Async-aware
let child = Command::new("probe-rs").spawn()?;
// Registers with Tokio, doesn't block
```

#### The Gotcha

**Common mistake**:

```rust
.stderr(Stdio::inherit)  // ❌ Compile error!
```

**Correct**:

```rust
.stderr(Stdio::inherit())  // ✅ It's a function!
```

**Error message**:

```
error[E0277]: the trait bound `Stdio: From<fn() -> Stdio {...}>` is not satisfied
```

This was **Week 6's first compile error** - a reminder to read the docs!

### The Lesson

> **Async runtimes require async-aware primitives. Using std::process in Tokio is like using blocking I/O in an async function - it defeats the purpose.**

---

### Achievement 2: Robust stdout Parsing

#### The Challenge

probe-rs output includes:

- defmt log lines with timestamps and levels
- Source location suffixes
- Escaped newline characters
- Actual newline characters

Example actual output:

```
[INFO] JSON sent via VCP: {"ts":12000,"id":"N2",...}\n (wk5_gateway_firmware src/main.rs:573)
```

Contains:

1. Log level prefix: `[INFO]`
2. Message marker: `JSON sent via VCP: `
3. The JSON: `{"ts":12000,...}`
4. Escaped newline: `\n` (as text characters)
5. Source location: ` (wk5_gateway_firmware src/main.rs:573)`
6. Actual newline: `\n` (the EOL character)

#### The Solution (Evolved Through Baby Steps)

```rust
fn extract_json_from_log_line(line: &str) -> Option<String> {
    // Find the JSON marker
    if let Some(start_idx) = line.find("JSON sent via VCP: ") {
        let json_start = start_idx + "JSON sent via VCP: ".len();
        let json_str = &line[json_start..];

        // Remove source location: split on " ("
        let without_location = json_str
            .split(" (")
            .next()
            .unwrap_or(json_str)
            .trim();

        // Remove both escaped \\n and actual \n
        let json_clean = without_location
            .trim_end_matches("\\n")  // Escaped backslash-n
            .trim_end_matches('\n')   // Actual newline
            .trim();

        Some(json_clean.to_string())
    } else {
        None
    }
}
```

#### Evolution of the Parser

**Attempt 1** (naive):

```rust
let json = json_str.trim();  // ❌ Still has \\n and source location
```

**Attempt 2** (close):

```rust
let json = json_str.trim_end_matches('\n');  // ❌ Still has \\n as text
```

**Attempt 3** (closer):

```rust
let json = json_str.trim_end_matches("\\n");  // ❌ Still has source location
```

**Attempt 4** (working!):

```rust
let without_location = json_str.split(" (").next().unwrap_or(json_str);
let json = without_location.trim_end_matches("\\n").trim_end_matches('\n');
// ✅ Works!
```

#### The Lesson

> **Parse errors teach you the exact format of the data. Looking at "trailing characters at column 116" tells you exactly where the problem is.**

**Debugging approach**:

1. Log the raw string: `warn!(raw = %json_str, "Parse failed")`
2. Count characters to find column 116
3. See what's there (in our case: ` (wk5_gateway_firmware...`)
4. Fix the parser incrementally

This took ~10 minutes using the "baby steps" approach.

---

### Achievement 3: Bounded Channels with Backpressure

#### The Design

```rust
let (tx, rx) = mpsc::channel::<TelemetryPacket>(100);
```

Capacity: **100 packets**

#### Why Bounded?

**Alternative 1: Unbounded channel**:

```rust
let (tx, rx) = mpsc::unbounded_channel();
```

**Problems**:

- ❌ Grows without limit if processor is slow
- ❌ Memory leak risk (can exhaust RAM)
- ❌ No backpressure signal to producer

**Alternative 2: Synchronous channel (crossbeam)**:

```rust
let (tx, rx) = crossbeam::channel::bounded(100);
```

**Problems**:

- ❌ Blocking sends (defeats async)
- ❌ Can't integrate with Tokio select!
- ❌ Requires spawning OS threads

**Chosen: Tokio bounded MPSC**:

```rust
let (tx, rx) = tokio::sync::mpsc::channel(100);
```

**Benefits**:

- ✅ **Backpressure**: `send().await` blocks when full
- ✅ **Bounded memory**: Maximum 100 × ~200 bytes = 20 KB
- ✅ **Producer paces**: Parser slows down if processor can't keep up
- ✅ **Async-friendly**: Plays nice with Tokio select!

#### Sizing the Capacity

**Calculation**:

- Node 1 transmits: ~1 packet per 10 seconds
- Capacity of 100 packets = ~16 minutes of buffering
- Processing time: <1 ms per packet

**Scenarios**:

| Situation         | Outcome                                                   |
| ----------------- | --------------------------------------------------------- |
| Normal operation  | Channel ~empty (packets processed immediately)            |
| Processor slow    | Channel fills, parser waits (backpressure works)          |
| Processor crashes | Channel fills to 100, parser blocks, graceful degradation |

#### The Pattern in Action

**Producer (parser task)**:

```rust
match tx.send(packet).await {
    Ok(()) => { /* Sent successfully */ }
    Err(e) => {
        error!(error = %e, "Channel closed, processor stopped");
        break;  // Exit task gracefully
    }
}
```

**Consumer (processor task)**:

```rust
while let Some(packet) = rx.recv().await {
    process_telemetry(packet);
}
info!("Channel closed, parser stopped");
```

#### The Lesson

> **Bounded channels are backpressure made explicit. Unbounded channels hide the problem until you run out of memory.**

In production systems, **always bound your queues**. Infinite buffers are infinite problems.

---

### Achievement 4: Structured Logging with tracing

#### Why Not println! or log crate?

**Option 1: println!**:

```rust
println!("Packet received: temp={}, humidity={}", temp, humidity);
```

**Problems**:

- ❌ No log levels (can't filter INFO vs DEBUG)
- ❌ No timestamps
- ❌ No structured fields (can't query/aggregate)
- ❌ Mixes with application output

**Option 2: log crate**:

```rust
info!("Packet received: temp={}, humidity={}", temp, humidity);
```

**Better, but**:

- ❌ Format strings only (not structured)
- ❌ Hard to parse in log aggregators
- ❌ No context propagation
- ❌ Can't change output format easily

**Option 3: tracing (chosen)**:

```rust
info!(
    node_id = %packet.id,
    timestamp_ms = packet.ts,
    temperature = packet.n1.t,
    humidity = packet.n1.h,
    rssi = packet.sig.rssi,
    "Packet received"
);
```

**Benefits**:

- ✅ **Structured data**: Key-value pairs, not just text
- ✅ **Filterable**: `RUST_LOG=debug` without recompiling
- ✅ **Multiple formats**: Text for dev, JSON for production
- ✅ **Context propagation**: Spans track request flow
- ✅ **Efficient**: Can disable at compile time

#### Configuration

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    )
    .with_target(false)
    .with_thread_ids(true)
    .init();
```

**Environment control**:

```bash
RUST_LOG=info cargo run        # Default
RUST_LOG=debug cargo run       # More verbose
RUST_LOG=wk6_async_gateway=trace cargo run  # Module-specific
```

#### Example Output

```
INFO Telemetry packet received node_id="N2" timestamp_ms=12000 temp_c=27.6 humidity_pct=54.1 rssi_dbm=-39
INFO Processing telemetry packet n1_temperature=27.6 n1_humidity=54.1 n1_gas_resistance=84190 rssi=-39 snr=13
INFO Gateway local sensor (BMP280) n2_temperature=Some(25.3) n2_pressure=Some(1013.2)
```

**Compared to println!**:

```
Packet received: temp=27.6, humidity=54.1
Processing...
Gateway sensor: 25.3, 1013.2
```

Which would you want to parse in a log aggregator?

#### The Lesson

> **Structured logging is the difference between hobbyist scripts and professional services. Invest in tracing from day one.**

Week 7 will add JSON output for log aggregation. The foundation is already there.

---

### Achievement 5: Graceful Shutdown

#### The Pattern

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ... initialization ...

    // Spawn tasks
    let parser_handle = tokio::spawn(parse_probe_rs_output(reader, tx));
    let processor_handle = tokio::spawn(process_telemetry(rx));

    info!("Service running. Press Ctrl+C to stop.");

    // Wait for shutdown signal OR task completion
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully");
        }
        _ = parser_handle => {
            warn!("Parser task ended unexpectedly");
        }
    }

    // Clean up resources
    info!("Killing probe-rs subprocess");
    child.kill().await.ok();

    // Wait for processor to drain channel
    processor_handle.await.ok();

    info!("Week 6 Async Gateway Service stopped");
    Ok(())
}
```

#### Why This Matters

**Without graceful shutdown**:

```rust
// BAD: Just exit on Ctrl+C
ctrl_c().await?;
std::process::exit(0);  // ❌ Orphaned subprocess!
```

**Problems**:

- ❌ probe-rs subprocess becomes zombie
- ❌ Channel data lost (packets in-flight discarded)
- ❌ Resources not cleaned up
- ❌ Logs cut off mid-message

**With graceful shutdown**:

```rust
// GOOD: Coordinate shutdown
ctrl_c().await?;
child.kill().await.ok();  // ✅ Kill subprocess
processor_handle.await.ok();  // ✅ Wait for channel drain
```

**Benefits**:

- ✅ **No zombies**: Subprocess killed explicitly
- ✅ **Data preserved**: Processor finishes in-flight packets
- ✅ **Clean logs**: Shutdown message logged
- ✅ **Testable**: Can mock Ctrl+C in tests

#### The select! Macro

**What it does**:

```rust
tokio::select! {
    result1 = future1 => { /* future1 completed first */ }
    result2 = future2 => { /* future2 completed first */ }
}
```

**In our case**:

- Wait for **either** Ctrl+C **or** parser task exit
- Whichever happens first triggers shutdown
- Clean up afterward

**Why this works**:

- If user presses Ctrl+C: Graceful shutdown path
- If probe-rs crashes: Detect and log, still clean up
- If parser panics: Catch and handle

#### The Lesson

> **Graceful shutdown isn't optional in production systems. Test it by pressing Ctrl+C during a run.**

**Verification**:

```bash
# Start service
cargo run

# Press Ctrl+C

# Check logs:
INFO Received Ctrl+C, shutting down gracefully
INFO Killing probe-rs subprocess
INFO Telemetry processor stopped
INFO Week 6 Async Gateway Service stopped

# Verify no zombie processes:
ps aux | grep probe-rs  # Should be empty
```

---

## Hardware Configuration

### Node 1 - Remote Sensor Transmitter

| Component         | Specification                                    | Notes                  |
| ----------------- | ------------------------------------------------ | ---------------------- |
| **Board**         | STM32F446 Nucleo-64                              | Cortex-M4F @ 84 MHz    |
| **ST-Link Probe** | `0483:374b:0671FF3833554B3043164817`             | For flashing/debugging |
| **Sensors**       | BME680, SHT31-D                                  | Temp, humidity, gas    |
| **Radio**         | RYLR998 LoRa                                     | 915 MHz, 10dBm         |
| **Display**       | SSD1306 128x64 OLED                              | Real-time metrics      |
| **Function**      | Sample sensors → Binary protocol → LoRa transmit |

### Node 2 - Gateway with Local Sensor

| Component         | Specification                                          | Notes                             |
| ----------------- | ------------------------------------------------------ | --------------------------------- |
| **Board**         | STM32F446 Nucleo-64                                    | Cortex-M4F @ 84 MHz               |
| **ST-Link Probe** | `0483:374b:066DFF3833584B3043115433`                   | For flashing/debugging            |
| **Sensor**        | BMP280                                                 | Barometric pressure + temperature |
| **Radio**         | RYLR998 LoRa                                           | 915 MHz, receive mode             |
| **Display**       | SSD1306 128x64 OLED                                    | Real-time metrics                 |
| **Function**      | LoRa receive → CRC validate → ACK → JSON via defmt/RTT |

### Desktop - Gateway Service

| Component    | Specification                                          | Notes                 |
| ------------ | ------------------------------------------------------ | --------------------- |
| **Runtime**  | Tokio async                                            | Full features enabled |
| **Language** | Rust (stable)                                          | 1.82+                 |
| **Function** | Spawn Node 2 firmware → Parse JSON → Process telemetry |

---

## Building & Running

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install probe-rs for flashing embedded firmware
cargo install probe-rs-tools --locked

# Verify both Nucleo boards are connected
probe-rs list
# Should show both:
# [0]: STLink V2-1 -- 0483:374b:0671FF3833554B3043164817
# [1]: STLink V2-1 -- 0483:374b:066DFF3833584B3043115433
```

### Quick Start (Recommended)

**Terminal 1 - Node 1**:

```bash
./build-n1.sh
```

**Terminal 2 - Gateway Service** (spawns Node 2 automatically):

```bash
./run-gateway.sh
```

**Or use Make**:

```bash
# Terminal 1
make n1

# Terminal 2
make gateway
```

### Manual Build & Run

```bash
# Build everything
cargo build --release --workspace

# Flash Node 1 (Terminal 1)
cargo build --package node1-firmware --release --target thumbv7em-none-eabihf
probe-rs run --probe 0483:374b:0671FF3833554B3043164817 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/node1-firmware

# Run gateway service (Terminal 2 - spawns Node 2 automatically)
cargo run --package wk6-async-gateway --release
```

### Expected Output

**Terminal 1 (Node 1)**:

```
[INFO] Configuring LoRa module...
[INFO] LoRa module configured
[INFO] Binary TX [AUTO]: 10 bytes sent, packet #1
[INFO] ACK received for packet #1
[INFO] Binary TX [AUTO]: 10 bytes sent, packet #2
```

**Terminal 2 (Gateway Service)**:

```
INFO Week 6 Async Gateway Service starting
INFO Spawning probe-rs subprocess probe=0483:374b:066DFF3833584B3043115433
INFO Service running. Press Ctrl+C to stop.
INFO Telemetry packet received node_id="N2" timestamp_ms=12000 temp_c=27.6 humidity_pct=54.1 rssi_dbm=-39
INFO Processing telemetry packet n1_temperature=27.6 n1_humidity=54.1 n1_gas_resistance=84190 rssi=-39 snr=13
INFO Gateway local sensor (BMP280) n2_temperature=Some(25.3) n2_pressure=Some(1013.2)
```

---

## Performance Characteristics

### Latency

**End-to-end** (Node 1 transmit → Gateway service log):

| Stage             | Time            | Notes                                 |
| ----------------- | --------------- | ------------------------------------- |
| LoRa transmission | ~300 ms         | RF propagation + module processing    |
| Node 2 processing | ~50 ms          | CRC validation, ACK send, JSON format |
| probe-rs output   | immediate       | RTT is fast                           |
| JSON parsing      | <1 ms           | serde_json deserialize                |
| Channel send      | <1 µs           | In-memory operation                   |
| **Total**         | **~350-400 ms** | LoRa is the bottleneck (expected)     |

### Memory Usage

**Gateway service process** (measured with `top`):

| Metric                 | Value  | Notes                    |
| ---------------------- | ------ | ------------------------ |
| **Resident (RSS)**     | ~15 MB | Actual RAM usage         |
| **Virtual (VSZ)**      | ~50 MB | Address space            |
| **Tokio runtime**      | ~5 MB  | Core async machinery     |
| **Channel buffer**     | ~20 KB | 100 packets × ~200 bytes |
| **BufReader**          | 8 KB   | Default buffer size      |
| **tracing subscriber** | ~3 MB  | Log formatting machinery |

**Compared to alternatives**:

- Python equivalent: ~80 MB RSS
- Node.js equivalent: ~60 MB RSS
- Rust wins on efficiency!

### CPU Usage

| Condition                      | Usage      | Notes                           |
| ------------------------------ | ---------- | ------------------------------- |
| **Idle** (waiting for packets) | <1%        | Tokio reactor efficiently waits |
| **Active** (processing packet) | 2-3% spike | JSON parse + log formatting     |
| **Average**                    | <5%        | Very low overhead               |

**Measured on**: 4-core Intel i5 system

---

## Current Status

### Completed (Week 6 Core)

- [x] Cargo workspace setup (gateway-service + 2 firmware packages)
- [x] Tokio async runtime integration
- [x] probe-rs subprocess spawning and management
- [x] Async stdout line reading with BufReader
- [x] JSON extraction from defmt log lines
- [x] serde_json deserialization into TelemetryPacket
- [x] Bounded MPSC channel (capacity: 100)
- [x] Parser task (stdout → channel)
- [x] Processor task (channel → logs)
- [x] Structured logging with tracing
- [x] Graceful shutdown (Ctrl+C handling)
- [x] Unit tests for JSON extraction
- [x] End-to-end hardware testing
- [x] BMP280 sensor integration (Node 2 local sensor)
- [x] Comprehensive documentation

### Performance Metrics

| Metric                 | Value     | Target  | Status |
| ---------------------- | --------- | ------- | ------ |
| **End-to-end latency** | ~350 ms   | <500 ms | ✅     |
| **Memory usage**       | 15 MB RSS | <50 MB  | ✅     |
| **CPU usage (avg)**    | <5%       | <10%    | ✅     |
| **Packet loss**        | 0%        | <1%     | ✅     |
| **Parse errors**       | 0%        | <0.1%   | ✅     |

### Documentation

- [x] [README.md](README.md) - This file (comprehensive overview)
- [x] [QUICKSTART.md](QUICKSTART.md) - Get running in 2 minutes
- [x] [WORKSPACE_README.md](WORKSPACE_README.md) - Workspace architecture
- [x] [NOTES.md](NOTES.md) - Technical learning notes
- [x] [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues
- [x] [TODO.md](TODO.md) - Completed tasks and future work
- [x] [BMP280_IMPLEMENTATION.md](BMP280_IMPLEMENTATION.md) - Sensor integration details

---

## Next Steps (Week 7 Preview)

Week 7 will add **cloud integration** to the gateway service:

### MQTT Publishing

```rust
use rumqttc::{AsyncClient, MqttOptions, QoS};

// In process_telemetry:
let topic = format!("iiot/node1/temperature");
client.publish(topic, QoS::AtLeastOnce, false, temperature).await?;
```

**Topics**:

- `iiot/node1/temperature`
- `iiot/node1/humidity`
- `iiot/node1/gas_resistance`
- `iiot/node2/temperature`
- `iiot/node2/pressure`
- `iiot/signal/rssi`
- `iiot/signal/snr`

### InfluxDB Writing

```rust
use influxdb2::Client;
use influxdb2::models::DataPoint;

let point = DataPoint::builder("sensor_data")
    .tag("node_id", "N1")
    .field("temperature", temperature)
    .field("humidity", humidity)
    .build()?;

client.write("iiot-bucket", stream::iter(vec![point])).await?;
```

### Configuration Management

```toml
# config.toml
[mqtt]
broker = "mqtt://localhost:1883"
client_id = "wk6-gateway"
username = "sensor_gateway"

[influxdb]
url = "http://localhost:8086"
org = "iiot-lab"
bucket = "sensor-data"
```

**Everything is ready** for this integration - the architecture is designed for it.

---

## Why Week 6 Matters

Week 6 is where the project **crosses the chasm** from embedded hobby to professional IIoT system.

### Technical Achievements

1. ✅ **Async Rust mastery**: Tokio runtime, tasks, channels, select!
2. ✅ **Subprocess management**: Spawning, monitoring, cleanup
3. ✅ **Robust parsing**: Handled all edge cases incrementally
4. ✅ **Structured logging**: Professional observability
5. ✅ **Graceful degradation**: Backpressure, shutdown, error handling

### Architectural Achievements

1. ✅ **Unified workspace**: Firmware + service in one repo
2. ✅ **Clean separation**: Embedded (no_std) vs service (std)
3. ✅ **Extensible design**: Ready for MQTT, InfluxDB, metrics
4. ✅ **Production patterns**: Bounded channels, structured logs, graceful shutdown

### The Meta-Lesson

> **Week 6 demonstrates that embedded and cloud aren't separate worlds - they're two parts of one system, connected by async Rust.**

The firmware (Weeks 1-5) collected data.  
The gateway service (Week 6) bridges it to infrastructure.  
Next week (Week 7) connects it to the cloud.

**This is how real IIoT systems work.**

---

## References

### Code Repository

- [Week 6 Source Code](https://github.com/mapfumo/wk6-async-gateway)

### Related Projects

- [Week 1: RTIC LoRa Basics](https://github.com/mapfumo/wk1_rtic_lora)
- [Week 2: Sensor Fusion](https://github.com/mapfumo/wk2-lora-sensor-fusion)
- [Week 3: Binary Protocols](https://github.com/mapfumo/wk3-binary-protocol)
- [Week 5: Gateway Firmware](https://github.com/mapfumo/wk5-gateway-firmware)

### Technical Documentation

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [tracing Documentation](https://docs.rs/tracing/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [serde_json Documentation](https://docs.rs/serde_json/)

---

**Author**: Antony (Tony) Mapfumo  
**Part of**: 4-Month Embedded Rust Learning Roadmap  
**Week**: 6 of 16
