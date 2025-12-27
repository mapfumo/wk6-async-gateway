# Week 6 Troubleshooting Guide

Common issues and solutions for the async gateway service.

---

## Build Errors

### Error: `the trait bound 'Stdio: From<fn() -> Stdio {...}>' is not satisfied`

**Symptom**:
```
error[E0277]: the trait bound `Stdio: From<fn() -> Stdio {Stdio::inherit}>` is not satisfied
   --> src/main.rs:212:17
    |
212 |         .stderr(Stdio::inherit) // Wrong!
```

**Cause**: Missing parentheses on `Stdio::inherit`

**Fix**:
```rust
// Wrong:
.stderr(Stdio::inherit)

// Correct:
.stderr(Stdio::inherit())
```

---

## Runtime Errors

### Error: `Failed to spawn probe-rs process`

**Symptom**:
```
Error: Failed to spawn probe-rs process

Caused by:
    No such file or directory (os error 2)
```

**Causes**:
1. probe-rs not installed
2. probe-rs not in PATH
3. Incorrect firmware path

**Diagnosis**:
```bash
# Check if probe-rs is available
which probe-rs

# Try running manually
probe-rs run --probe 0483:374b:066DFF3833584B3043115433 \
  --chip STM32F446RETx \
  ../wk5-gateway-firmware/target/thumbv7em-none-eabihf/release/wk5-gateway-firmware
```

**Fix**:
```bash
# Install probe-rs if missing
cargo install probe-rs-tools --locked

# Or fix PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

---

### Error: `No probe was found`

**Symptom** (from probe-rs stderr):
```
Error: No probe was found.
```

**Causes**:
1. Nucleo board not connected
2. USB cable is power-only (no data)
3. Wrong probe ID
4. USB permissions issue (Linux)

**Diagnosis**:
```bash
# List available probes
probe-rs list

# Expected output:
# The following debug probes were found:
# [0]: STLink V2-1 -- 0483:374b:066DFF3833584B3043115433 (ST-LINK)
```

**Fix for Linux Permissions**:
```bash
# Add udev rules
sudo tee /etc/udev/rules.d/70-st-link.rules > /dev/null <<EOF
SUBSYSTEM=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", MODE="0666"
EOF

# Reload udev
sudo udevadm control --reload-rules
sudo udevadm trigger

# Replug device
```

---

### Warning: `Failed to parse JSON`

**Symptom**:
```
WARN Failed to parse JSON error="trailing characters at line 1 column 116" json="{...}\n (wk5_gateway_firmware ...)"
```

**Cause**: Parser not handling defmt source location suffix correctly

**Check**: Look at the `json` field in the warning - does it have extra text after the closing `}`?

**Fix**: Update `extract_json_from_log_line` function to split on ` (`:

```rust
let without_location = json_str
    .split(" (")
    .next()
    .unwrap_or(json_str)
    .trim();
```

---

### Error: `Parser task ended unexpectedly`

**Symptom**:
```
WARN Parser task ended unexpectedly
WARN probe-rs process ended (EOF on stdout)
```

**Causes**:
1. probe-rs crashed
2. Firmware panic
3. Node 2 board disconnected
4. Week 5 firmware not flashed

**Diagnosis**:
```bash
# Run Week 5 firmware manually to see error
cd ../wk5-gateway-firmware
probe-rs run --probe 0483:374b:066DFF3833584B3043115433 \
  --chip STM32F446RETx \
  target/thumbv7em-none-eabihf/release/wk5-gateway-firmware
```

**Common fixes**:
- Re-flash Week 5 firmware
- Check USB connection
- Restart both boards

---

## No Telemetry Received

### Symptom: Service runs but no packets logged

**Check list**:

1. **Is Node 1 running?**
   ```bash
   # In separate terminal
   cd ../wk3-binary-protocol
   probe-rs run --probe 0483:374b:0671FF3833554B3043164817 \
     --chip STM32F446RETx \
     target/thumbv7em-none-eabihf/release/wk3-binary-protocol
   ```

2. **Is Node 2 receiving LoRa packets?**
   - Check probe-rs output (stderr) for:
     ```
     [INFO] Binary RX - T:27.6 H:54.3 ...
     ```

3. **Is JSON being logged?**
   - Look for:
     ```
     [INFO] JSON sent via VCP: {...}
     ```

4. **Check LoRa configuration match**:
   - Both nodes must use same:
     - Network ID (18)
     - Frequency (915 MHz)
     - Parameters (7,9,1,7)

---

## Channel Backpressure Issues

### Symptom: Parser blocks/hangs

**Possible causes**:
1. Processor task crashed (channel full, no receiver)
2. Processing too slow (packets arriving faster than 100/interval)

**Diagnosis**:
```rust
// Add to main.rs for debugging
info!("Channel capacity: {}", rx.capacity());
```

**Fixes**:
- Increase channel capacity (100 → 1000)
- Speed up processor (remove slow operations)
- Add timeout on send:
  ```rust
  tokio::time::timeout(
      Duration::from_secs(5),
      tx.send(packet)
  ).await??;
  ```

---

## Compilation Warnings

### Warning: `unused import: 'regex::Regex'`

**Cause**: Imported regex but didn't use it

**Fix**: Remove from imports:
```rust
// Remove this line:
use regex::Regex;
```

---

## Memory/Performance Issues

### High CPU Usage

**Normal**: <5% average (spikes to 10-15% during packet processing)

**If >20% sustained**:
1. Check for busy loops
2. Look for blocking operations in async context
3. Profile with `cargo flamegraph`

### Memory Leak

**Diagnosis**:
```bash
# Monitor with
top -p $(pgrep wk6-async-gateway)

# Or use valgrind
valgrind --leak-check=full target/debug/wk6-async-gateway
```

**Common causes**:
- Unbounded data structure growth
- Not draining channel
- Circular references with Rc/Arc

---

## Shutdown Issues

### Service doesn't stop on Ctrl+C

**Symptom**: Have to use `Ctrl+\` or `kill -9`

**Cause**: Signal handler not working

**Check**:
```rust
// Ensure this is present in main:
tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        info!("Received Ctrl+C");
    }
    ...
}
```

**Platform-specific**: On Windows, Ctrl+C behavior differs

---

### Zombie probe-rs processes

**Symptom**: Multiple probe-rs processes after stopping

**Diagnosis**:
```bash
ps aux | grep probe-rs
```

**Fix**: Ensure proper cleanup:
```rust
// In shutdown path:
child.kill().await.ok();
```

**Manual cleanup**:
```bash
pkill -9 probe-rs
```

---

## JSON Schema Mismatches

### Error: `missing field 'n1'`

**Cause**: Week 5 firmware changed JSON format

**Fix**: Update `TelemetryPacket` struct to match Week 5 output

**Debug**: Log the raw JSON:
```rust
warn!(raw_json = %json_str, "Schema mismatch");
```

### Error: `invalid type: floating point, expected u32`

**Cause**: Type mismatch in struct definition

**Example**:
```rust
// If JSON has: "g": 84190
// But struct expects:
pub struct Node1Data {
    g: f32,  // Wrong! Should be u32
}
```

**Fix**: Match Week 5's [main.rs](../wk5-gateway-firmware/src/main.rs) types exactly

---

## Testing Issues

### Tests pass but real data fails

**Cause**: Test data doesn't match real defmt output

**Fix**: Copy actual log line from probe-rs output into test:
```rust
#[test]
fn test_real_log_line() {
    let line = r#"[INFO] JSON sent via VCP: {"ts":12000,"id":"N2","n1":{"t":27.6,"h":54.1,"g":84190},"n2":{},"sig":{"rssi":-39,"snr":13},"sts":{"rx":1,"err":0}}\n (wk5_gateway_firmware wk5-gateway-firmware/src/main.rs:573)"#;

    let result = extract_json_from_log_line(line);
    assert!(result.is_some());

    let json = result.unwrap();
    assert!(serde_json::from_str::<TelemetryPacket>(&json).is_ok());
}
```

---

## Getting Help

If issues persist:

1. **Enable debug logging**:
   ```bash
   RUST_LOG=debug cargo run 2>&1 | tee debug.log
   ```

2. **Check versions**:
   ```bash
   rustc --version
   probe-rs --version
   cargo tree | grep tokio
   ```

3. **Minimal reproduction**:
   - Test Week 5 firmware alone
   - Test probe-rs subprocess separately
   - Test JSON parsing with sample data

4. **File an issue** with:
   - Full error message
   - `cargo --version` output
   - Platform (Linux/Windows/macOS)
   - Logs with `RUST_LOG=trace`

---

*Last Updated*: 2025-12-27
