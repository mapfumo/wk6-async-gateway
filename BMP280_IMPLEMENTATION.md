# BMP280 Sensor Implementation

**Date**: 2025-12-27
**Status**: ✅ Complete

## Overview

Implemented BMP280 barometric pressure sensor reading on Node 2 (gateway) firmware to provide local environmental data alongside the remote Node 1 sensor data.

## What Was Added

### 1. Node 2 Firmware (`node2-firmware/src/main.rs`)

**Lines 389-405**: BMP280 sensor reading in timer interrupt (2Hz):

```rust
// Read BMP280 sensor (gateway local sensor)
cx.shared.bmp280.lock(|bmp_opt| {
    if let Some(bmp) = bmp_opt {
        // Read temperature and pressure from BMP280
        let temp = bmp.temp() as f32;      // Convert f64 to f32
        let pressure = bmp.pressure() as f32;  // Pressure in Pa

        // Convert pressure from Pa to hPa (more common for weather)
        let pressure_hpa = pressure / 100.0;

        // Store in shared state
        cx.shared.gateway_temp.lock(|t| *t = Some(temp));
        cx.shared.gateway_pressure.lock(|p| *p = Some(pressure_hpa));

        defmt::debug!("BMP280 read: T={}°C, P={}hPa", temp, pressure_hpa);
    }
});
```

### 2. Gateway Service (`gateway-service/src/main.rs`)

**Lines 163-189**: Updated telemetry processor to log BMP280 data:

```rust
while let Some(packet) = rx.recv().await {
    // Log Node 1 (remote sensor) data
    info!(
        timestamp_ms = packet.ts,
        node_id = %packet.id,
        n1_temperature = packet.n1.t,
        n1_humidity = packet.n1.h,
        n1_gas_resistance = packet.n1.g,
        rssi = packet.sig.rssi,
        snr = packet.sig.snr,
        packets_received = packet.sts.rx,
        crc_errors = packet.sts.err,
        "Processing telemetry packet"
    );

    // Log Node 2 (gateway local sensor) data if available
    if packet.n2.t.is_some() || packet.n2.p.is_some() {
        info!(
            n2_temperature = ?packet.n2.t,
            n2_pressure = ?packet.n2.p,
            "Gateway local sensor (BMP280)"
        );
    }

    // TODO Week 7: Publish to MQTT
    // TODO Week 7: Write to InfluxDB
}
```

## Sensor Details

### BMP280 Specifications

- **Interface**: I2C (shared bus with SSD1306 OLED display)
- **I2C Address**: 0x76
- **Temperature Range**: -40°C to +85°C
- **Pressure Range**: 300 to 1100 hPa
- **Reading Frequency**: Every 500ms (2 Hz, matching the timer)

### Data Format

The BMP280 provides:
- **Temperature**: Floating point in °C
- **Pressure**: Converted from Pascals to hectopascals (hPa / mbar)

### Integration with Existing System

The BMP280 was already:
- ✅ Wired to the I2C bus
- ✅ Initialized in firmware (`BMP280::new()`)
- ✅ Included in JSON schema (`n2` object)

What was missing:
- ❌ Actual sensor reading code (was commented out as TODO)
- ❌ Gateway service logging of `n2` data

## JSON Output

### Before Implementation
```json
{
  "ts": 12000,
  "id": "N2",
  "n1": { "t": 27.6, "h": 54.1, "g": 84190 },
  "n2": {},  // Empty - no BMP280 data
  "sig": { "rssi": -39, "snr": 13 },
  "sts": { "rx": 42, "err": 1 }
}
```

### After Implementation
```json
{
  "ts": 12000,
  "id": "N2",
  "n1": { "t": 27.6, "h": 54.1, "g": 84190 },
  "n2": { "t": 25.3, "p": 1013.2 },  // BMP280 data populated!
  "sig": { "rssi": -39, "snr": 13 },
  "sts": { "rx": 42, "err": 1 }
}
```

## Gateway Service Output

### Before
```
INFO Processing telemetry packet timestamp_ms=12000 node_id="N2" temperature=27.6 humidity=54.1 gas_resistance=84190 rssi=-39 snr=13
```

### After
```
INFO Processing telemetry packet timestamp_ms=12000 node_id="N2" n1_temperature=27.6 n1_humidity=54.1 n1_gas_resistance=84190 rssi=-39 snr=13
INFO Gateway local sensor (BMP280) n2_temperature=Some(25.3) n2_pressure=Some(1013.2)
```

## Technical Implementation Notes

### BMP280 Crate API

Using `bmp280-ehal` version 0.0.6:

```rust
// Temperature reading
let temp: f64 = bmp.temp();

// Pressure reading (returns Pascals)
let pressure: f64 = bmp.pressure();
```

**Key Points**:
- Methods return `f64` (converted to `f32` for storage)
- Pressure is in Pascals (Pa), converted to hectopascals (hPa): `pressure / 100.0`
- Reading blocks briefly on I2C bus (acceptable in timer context)

### Error Handling

The BMP280 sensor is optional:
- Initialization creates `Option<BMP280>`
- If sensor not found at 0x76: `None` (logged as warning, continues without local sensor)
- Reading code checks `if let Some(bmp) = bmp_opt` before attempting reads

### Performance

- **Reading frequency**: 500ms (2 Hz)
- **I2C transaction time**: ~1-2ms (negligible impact)
- **Memory**: 2 additional `Option<f32>` in shared state (8 bytes)

## Testing

To verify BMP280 is working:

1. **Check firmware logs** (Node 2 probe-rs output):
   ```
   [INFO] BMP280 found at 0x76!
   [DEBUG] BMP280 read: T=25.3°C, P=1013.2hPa
   ```

2. **Check gateway service logs**:
   ```
   INFO Gateway local sensor (BMP280) n2_temperature=Some(25.3) n2_pressure=Some(1013.2)
   ```

3. **Verify JSON parsing** works end-to-end

## Why This Was Needed

The BMP280 hardware was connected but unused because:
- Week 5 focused on LoRa gateway functionality (higher priority)
- BMP280 API documentation wasn't immediately available
- Reading code was left as TODO: "API TBD"

Now completed:
- ✅ Hardware fully utilized
- ✅ Dual sensor architecture (remote + local)
- ✅ Enables temperature comparison between Node 1 and Node 2
- ✅ Provides barometric pressure for weather monitoring

## Future Enhancements

Week 7+ could add:
- **MQTT topics** for BMP280 data: `iiot/gateway/temperature`, `iiot/gateway/pressure`
- **InfluxDB fields** for time-series storage
- **Grafana dashboards** comparing Node 1 vs Node 2 temperatures
- **Altitude calculation** from pressure readings
- **Trend analysis** for pressure changes (weather forecasting)

## Documentation Updated

- ✅ [README.md](README.md) - Updated expected output examples
- ✅ [QUICKSTART.md](QUICKSTART.md) - Added BMP280 to output examples
- ✅ [TODO.md](TODO.md) - Marked BMP280 implementation as complete
- ✅ Created this BMP280_IMPLEMENTATION.md

---

*BMP280 Implementation Completed: 2025-12-27*
