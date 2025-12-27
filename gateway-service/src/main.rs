//! Week 6: Async Gateway Service
//!
//! This service:
//! - Spawns probe-rs as a subprocess to run the Week 5 gateway firmware
//! - Captures stdout and parses JSON telemetry
//! - Demonstrates Tokio async patterns and structured logging
//!
//! Architecture: probe-rs → stdout → parser → channel → processor

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Telemetry packet from Node 2 gateway (matches Week 5 JSON format)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TelemetryPacket {
    /// Timestamp in milliseconds since boot
    ts: u32,
    /// Node ID (should be "N2" for gateway)
    id: String,
    /// Node 1 sensor data (remote sensor via LoRa)
    n1: Node1Data,
    /// Node 2 sensor data (gateway local sensor)
    n2: Node2Data,
    /// Signal quality metrics
    sig: SignalQuality,
    /// Statistics
    sts: Statistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node1Data {
    /// Temperature in °C
    t: f32,
    /// Humidity in %
    h: f32,
    /// Gas resistance in ohms
    g: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node2Data {
    /// Temperature in °C (optional, BMP280 may not be reading yet)
    #[serde(skip_serializing_if = "Option::is_none")]
    t: Option<f32>,
    /// Pressure in hPa (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    p: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignalQuality {
    /// RSSI in dBm
    rssi: i16,
    /// SNR in dB
    snr: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Statistics {
    /// Packets received
    rx: u32,
    /// CRC errors
    err: u32,
}

/// Extract JSON from probe-rs log line
///
/// Example input: `[INFO] JSON sent via VCP: {"ts":12000,...}\n`
/// Returns: `{"ts":12000,...}`
fn extract_json_from_log_line(line: &str) -> Option<String> {
    // Look for the JSON marker in the log
    if let Some(start_idx) = line.find("JSON sent via VCP: ") {
        let json_start = start_idx + "JSON sent via VCP: ".len();
        let json_str = &line[json_start..];

        // Remove the escaped \n and defmt source location suffix
        // Format: {...}\n (wk5_gateway_firmware src/main.rs:573)
        let without_location = json_str
            .split(" (")  // Split on defmt source location
            .next()       // Take everything before the location
            .unwrap_or(json_str)
            .trim();

        // Remove both escaped \\n and actual \n characters
        let json_clean = without_location
            .trim_end_matches("\\n")
            .trim_end_matches('\n')
            .trim();

        Some(json_clean.to_string())
    } else {
        None
    }
}

/// Parse probe-rs stdout and send telemetry packets to channel
async fn parse_probe_rs_output(
    mut reader: BufReader<tokio::process::ChildStdout>,
    tx: mpsc::Sender<TelemetryPacket>,
) -> Result<()> {
    let mut line_buf = String::new();

    info!("Starting probe-rs output parser");

    loop {
        line_buf.clear();

        match reader.read_line(&mut line_buf).await {
            Ok(0) => {
                warn!("probe-rs process ended (EOF on stdout)");
                break;
            }
            Ok(_) => {
                // Try to extract JSON from this line
                if let Some(json_str) = extract_json_from_log_line(&line_buf) {
                    match serde_json::from_str::<TelemetryPacket>(&json_str) {
                        Ok(packet) => {
                            info!(
                                node_id = %packet.id,
                                timestamp_ms = packet.ts,
                                temp_c = packet.n1.t,
                                humidity_pct = packet.n1.h,
                                rssi_dbm = packet.sig.rssi,
                                "Telemetry packet received"
                            );

                            if let Err(e) = tx.send(packet).await {
                                error!(error = %e, "Failed to send packet to channel");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, json = %json_str, "Failed to parse JSON");
                        }
                    }
                } else {
                    // Not a JSON line, just pass through for debugging
                    // (Could filter these to only show important logs)
                    if line_buf.contains("[INFO]") || line_buf.contains("[WARN]") || line_buf.contains("[ERROR]") {
                        print!("{}", line_buf); // Pass through defmt logs
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Error reading from probe-rs stdout");
                break;
            }
        }
    }

    Ok(())
}

/// Process telemetry packets (placeholder for Week 7 MQTT publishing)
async fn process_telemetry(mut rx: mpsc::Receiver<TelemetryPacket>) {
    info!("Starting telemetry processor");

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

    info!("Telemetry processor stopped");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("Week 6 Async Gateway Service starting");

    // Configuration for probe-rs (from your alias)
    let probe_id = "0483:374b:066DFF3833584B3043115433"; // Node 2
    let chip = "STM32F446RETx";
    let firmware_path = "target/thumbv7em-none-eabihf/release/node2-firmware";

    info!(
        probe = probe_id,
        chip = chip,
        firmware = firmware_path,
        "Spawning probe-rs subprocess"
    );

    // Spawn probe-rs as subprocess
    let mut child = Command::new("probe-rs")
        .args(&[
            "run",
            "--probe",
            probe_id,
            "--chip",
            chip,
            firmware_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()) // Pass through stderr for errors
        .spawn()
        .context("Failed to spawn probe-rs process")?;

    let stdout = child
        .stdout
        .take()
        .context("Failed to capture probe-rs stdout")?;

    // Create channel for telemetry packets
    let (tx, rx) = mpsc::channel::<TelemetryPacket>(100);

    // Spawn parser task
    let parser_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        if let Err(e) = parse_probe_rs_output(reader, tx).await {
            error!(error = %e, "Parser task failed");
        }
    });

    // Spawn processor task
    let processor_handle = tokio::spawn(process_telemetry(rx));

    // Wait for Ctrl+C
    info!("Service running. Press Ctrl+C to stop.");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully");
        }
        _ = parser_handle => {
            warn!("Parser task ended unexpectedly");
        }
    }

    // Kill probe-rs subprocess
    info!("Killing probe-rs subprocess");
    child.kill().await.ok();

    // Wait for processor to finish
    processor_handle.await.ok();

    info!("Week 6 Async Gateway Service stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_log_line() {
        let line = r#"[INFO] JSON sent via VCP: {"ts":12000,"id":"N2"}\n"#;
        let result = extract_json_from_log_line(line);
        assert_eq!(result, Some(r#"{"ts":12000,"id":"N2"}"#.to_string()));
    }

    #[test]
    fn test_extract_json_no_match() {
        let line = "[INFO] Some other log message";
        let result = extract_json_from_log_line(line);
        assert_eq!(result, None);
    }
}
