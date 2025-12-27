# Week 6 TODO List

## ✅ Completed

- [x] Set up Cargo project with Tokio
- [x] Add dependencies (tokio, serde, tracing, anyhow)
- [x] Define `TelemetryPacket` struct matching Week 5 JSON
- [x] Implement `extract_json_from_log_line()` parser
- [x] Spawn probe-rs subprocess with correct arguments
- [x] Implement async stdout line reader
- [x] Create bounded MPSC channel (capacity=100)
- [x] Implement parser task (`parse_probe_rs_output`)
- [x] Implement processor task (`process_telemetry`)
- [x] Add structured logging with tracing
- [x] Implement graceful shutdown (Ctrl+C handler)
- [x] Fix JSON parsing (handle defmt source location)
- [x] Test with live hardware (Node 1 + Node 2)
- [x] Verify end-to-end telemetry flow
- [x] Write unit tests for JSON extraction
- [x] Create comprehensive README.md
- [x] Create NOTES.md with learnings
- [x] Create TROUBLESHOOTING.md
- [x] Create this TODO.md
- [x] Create Cargo workspace with node1-firmware, node2-firmware, and gateway-service
- [x] Implement BMP280 sensor reading in Node 2 firmware
- [x] Update gateway service to log Node 2 BMP280 data
- [x] Update all documentation with BMP280 implementation

## 📋 Week 6 Enhancements (Optional)

### Code Quality
- [ ] Refactor into modules (separate files):
  - [ ] `parser.rs` - JSON extraction and parsing logic
  - [ ] `telemetry.rs` - Data structures (TelemetryPacket, etc.)
  - [ ] `subprocess.rs` - probe-rs process management
  - [ ] `processor.rs` - Telemetry processing logic
- [ ] Add more unit tests:
  - [ ] Test JSON parsing edge cases (empty fields, missing fields)
  - [ ] Test channel backpressure behavior
  - [ ] Test shutdown cleanup
- [ ] Add integration test:
  - [ ] Mock subprocess with known JSON output
  - [ ] Verify end-to-end flow without hardware
- [ ] Improve error types:
  - [ ] Custom error enum with thiserror
  - [ ] Better error context with anyhow chains
- [ ] Add rustdoc comments to all public items

### Configuration
- [ ] Create `config.toml` file:
  ```toml
  [probe]
  id = "0483:374b:066DFF3833584B3043115433"
  chip = "STM32F446RETx"
  firmware_path = "../wk5-gateway-firmware/target/..."

  [channel]
  capacity = 100

  [logging]
  level = "info"
  ```
- [ ] Add `config` crate for TOML parsing
- [ ] Support environment variable overrides
- [ ] Add `--config` CLI argument

### Monitoring
- [ ] Add metrics:
  - [ ] Packets received counter
  - [ ] Parse error counter
  - [ ] Channel depth gauge
  - [ ] Processing latency histogram
- [ ] Expose /metrics endpoint (Prometheus format)
- [ ] Add health check endpoint

### Robustness
- [ ] Handle probe-rs crashes:
  - [ ] Detect subprocess exit
  - [ ] Attempt restart (with backoff)
  - [ ] Log restart attempts
- [ ] Add timeout on subprocess spawn (don't wait forever)
- [ ] Implement watchdog timer (restart if no packets for N seconds)
- [ ] Add packet deduplication (track sequence numbers)

### Performance
- [ ] Benchmark JSON parsing overhead
- [ ] Profile with `cargo flamegraph`
- [ ] Optimize hot paths if needed
- [ ] Consider zero-copy parsing (if beneficial)

### Developer Experience
- [ ] Add CLI arguments:
  - [ ] `--probe-id <ID>` - Override probe ID
  - [ ] `--firmware <PATH>` - Override firmware path
  - [ ] `--log-level <LEVEL>` - Override log level
  - [ ] `--json-output` - Output JSON logs (for aggregators)
- [ ] Add `--version` flag
- [ ] Add `--dry-run` mode (parse config, don't spawn subprocess)
- [ ] Improve startup messages (ASCII art logo?)

## 🚀 Week 7 Integration (Next Week)

### MQTT Client
- [ ] Add `rumqttc` dependency
- [ ] Create MQTT connection manager
- [ ] Design topic hierarchy:
  - [ ] `iiot/node1/temperature`
  - [ ] `iiot/node1/humidity`
  - [ ] `iiot/node1/gas_resistance`
  - [ ] `iiot/gateway/rssi`
  - [ ] `iiot/gateway/snr`
  - [ ] `iiot/stats/packets_received`
  - [ ] `iiot/stats/crc_errors`
- [ ] Implement publish in `process_telemetry`
- [ ] Add TLS support for MQTT broker
- [ ] Implement reconnection logic with exponential backoff
- [ ] Add offline buffering (queue to disk if broker down)

### InfluxDB Writer
- [ ] Add `influxdb2` dependency
- [ ] Convert telemetry to line protocol format
- [ ] Implement batched writes (buffer N points before flush)
- [ ] Add tags (node_id, sensor_type, location)
- [ ] Add fields (all numeric values)
- [ ] Add timestamp (from packet.ts)
- [ ] Error handling for write failures
- [ ] Retry logic with backoff

### Configuration Updates
- [ ] Add MQTT config section:
  ```toml
  [mqtt]
  broker = "mqtt://localhost:1883"
  client_id = "wk6-gateway"
  username = "sensor_gateway"
  password_env = "MQTT_PASSWORD"
  tls_ca_cert = "/path/to/ca.crt"
  ```
- [ ] Add InfluxDB config section:
  ```toml
  [influxdb]
  url = "http://localhost:8086"
  org = "iiot-lab"
  bucket = "sensor-data"
  token_env = "INFLUXDB_TOKEN"
  ```

### Testing with Infrastructure
- [ ] Install Mosquitto broker locally
- [ ] Install InfluxDB locally
- [ ] Test MQTT publish with `mosquitto_sub`
- [ ] Test InfluxDB writes with UI
- [ ] Verify data flow end-to-end

## 📚 Documentation Updates

### For Week 7
- [ ] Update README with MQTT section
- [ ] Update README with InfluxDB section
- [ ] Add architecture diagram showing full pipeline
- [ ] Document topic hierarchy
- [ ] Document InfluxDB schema (tags, fields)
- [ ] Add example queries for InfluxDB

## 🐛 Known Issues

*(None currently - add issues as they're discovered)*

## 💡 Ideas for Future Weeks

### Week 8: Observability
- Grafana dashboards consuming InfluxDB
- Prometheus metrics scraping
- Alert rules (high CRC error rate, node offline)
- Distributed tracing with OpenTelemetry

### Week 9: OPC-UA Server
- Expose sensor data via OPC-UA protocol
- Information model design
- UAExpert client testing

### Later Enhancements
- Web UI for real-time monitoring
- REST API for querying historical data
- Multi-node support (more than 2 sensors)
- Database for configuration (instead of files)
- Docker containerization
- Kubernetes deployment (overkill but good learning!)

---

## Priority for This Week

**If time allows, focus on**:
1. Configuration file support (makes Week 7 easier)
2. Basic error metrics (packets received, parse errors)
3. Code refactoring into modules (better organization)

**Don't over-engineer**:
- Week 6 core is done and working
- Focus should shift to Week 7 integration
- Polish can happen during Week 12 (portfolio assembly)

---

*Last Updated*: 2025-12-27
*Status*: Week 6 core complete, ready to proceed to Week 7
