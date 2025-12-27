# Workspace Setup Complete

This document describes the Week 6 workspace restructuring that was completed on 2025-12-27.

## What Was Done

Converted Week 6 from a single-package project into a self-contained Cargo workspace containing:

1. **node1-firmware** - STM32 sensor node (from Week 3)
2. **node2-firmware** - STM32 gateway (from Week 5)
3. **gateway-service** - Async Rust service (Week 6 core)

## Benefits

### Self-Sufficiency
- All firmware and software in one location
- No need to navigate to ../wk3-binary-protocol or ../wk5-gateway-firmware
- Shared `target/` directory for efficient builds

### Convenient Build System
- **Makefile** with simple targets: `make n1`, `make gateway`, `make clean`
- **Shell scripts**: `build-n1.sh`, `run-gateway.sh`
- **Cargo workspace**: Standard Rust workspace commands work

### Proper Toolchain Management
- Each firmware package has its own `rust-toolchain.toml` (nightly)
- Gateway service uses stable Rust automatically
- Added `build.rs` to both firmware packages to handle memory.x in workspace builds

## File Structure

```
wk6-async-gateway/           # Workspace root
├── Cargo.toml               # Workspace manifest
├── Cargo.lock               # Shared dependency lock
├── rust-toolchain.toml      # ← NEW: Workspace uses nightly (for firmware)
├── target/                  # Shared build output
│
├── gateway-service/         # Async service (Tokio)
│   ├── Cargo.toml
│   └── src/main.rs
│
├── node1-firmware/          # Sensor node
│   ├── Cargo.toml
│   ├── build.rs            # ← NEW: Copies memory.x to OUT_DIR
│   ├── rust-toolchain.toml # (inherits from workspace)
│   ├── memory.x
│   ├── .cargo/
│   └── src/main.rs
│
├── node2-firmware/          # Gateway firmware
│   ├── Cargo.toml
│   ├── build.rs            # ← NEW: Copies memory.x to OUT_DIR
│   ├── rust-toolchain.toml # (inherits from workspace)
│   ├── memory.x
│   ├── .cargo/
│   └── src/main.rs
│
├── Makefile                 # Convenience targets
├── build-n1.sh
├── build-n2.sh
├── run-gateway.sh
│
├── README.md                # Main documentation
├── QUICKSTART.md            # Quick start guide
├── WORKSPACE_README.md      # Workspace architecture
├── NOTES.md
├── TROUBLESHOOTING.md
└── TODO.md
```

## Key Technical Challenges Solved

### 1. Cargo Workspace with Mixed Toolchains

**Problem**: Firmware requires nightly Rust, but when building the workspace, cargo needs a single toolchain.

**Solution**:
- Added workspace-level `rust-toolchain.toml` specifying nightly
- Gateway service (Tokio) compiles fine on nightly too
- Individual package toolchain files are kept for documentation

### 2. Linker Script (memory.x) Not Found

**Problem**: In a workspace, builds run from the workspace root, but cortex-m-rt expects memory.x in the current directory.

**Solution**: Added `build.rs` to both firmware packages:

```rust
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
}
```

This embeds memory.x at compile time and places it in the build output directory.

### 3. Package Naming

Gateway service package is named `wk6-async-gateway` (historical name from initial development).

All scripts updated to use:
```bash
cargo run --package wk6-async-gateway
```

## Build Verification

All three packages build successfully:

```bash
$ cargo build --package node1-firmware --release
   Compiling node1-firmware v0.1.0
    Finished `release` profile [optimized] target(s) in 0.53s

$ cargo build --package node2-firmware --release
   Compiling node2-firmware v0.1.0
    Finished `release` profile [optimized] target(s) in 0.53s

$ cargo build --package wk6-async-gateway --release
   Compiling wk6-async-gateway v0.1.0
    Finished `release` profile [optimized] target(s) in 0.96s
```

## Usage Examples

### Using Makefile (Easiest)
```bash
# Terminal 1
make n1

# Terminal 2
make gateway
```

### Using Scripts
```bash
# Terminal 1
./build-n1.sh

# Terminal 2
./run-gateway.sh
```

### Using Cargo Directly
```bash
# Build all packages
cargo build --release --workspace

# Run specific package
cargo run --package wk6-async-gateway
```

### Using Your Aliases

From the workspace root:

```bash
alias n1='clear && cargo build --package node1-firmware --release && probe-rs run --probe 0483:374b:0671FF3833554B3043164817 --chip STM32F446RETx target/thumbv7em-none-eabihf/release/node1-firmware'

alias n2='clear && cargo build --package node2-firmware --release && probe-rs run --probe 0483:374b:066DFF3833584B3043115433 --chip STM32F446RETx target/thumbv7em-none-eabihf/release/node2-firmware'

alias gw='clear && cargo run --package wk6-async-gateway'
```

Then simply:
```bash
n1   # Terminal 1
gw   # Terminal 2
```

## Notes

- Warnings about "profiles for non-root package will be ignored" are expected and harmless
- The workspace uses resolver = "2" for modern dependency resolution
- All firmware uses `thumbv7em-none-eabihf` target (Cortex-M4F)
- Gateway service outputs to `target/release/wk6-async-gateway`
- Firmware outputs to `target/thumbv7em-none-eabihf/release/{node1,node2}-firmware`

## Next Steps

This workspace is ready for Week 7 integration (MQTT + InfluxDB). The gateway service already has placeholders for:

```rust
// TODO Week 7: Publish to MQTT
// TODO Week 7: Write to InfluxDB
```

---

*Week 6 Workspace Setup Completed: 2025-12-27*
