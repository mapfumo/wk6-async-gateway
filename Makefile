.PHONY: help n1 n2 gateway clean

help:
	@echo "Week 6 Workspace - Make Commands"
	@echo ""
	@echo "Usage:"
	@echo "  make n1        - Build and run Node 1 (sensor)"
	@echo "  make n2        - Build and run Node 2 (gateway firmware)"
	@echo "  make gateway   - Run gateway service (spawns Node 2)"
	@echo "  make clean     - Clean build artifacts"
	@echo ""
	@echo "Typical usage:"
	@echo "  Terminal 1: make n1"
	@echo "  Terminal 2: make gateway"

n1:
	@echo "Building Node 1 firmware..."
	cd node1-firmware && cargo build --release --target thumbv7em-none-eabihf
	@echo "Running Node 1 on probe 0483:374b:0671FF3833554B3043164817..."
	probe-rs run --probe 0483:374b:0671FF3833554B3043164817 \
	  --chip STM32F446RETx \
	  target/thumbv7em-none-eabihf/release/node1-firmware

n2:
	@echo "Building Node 2 firmware..."
	cd node2-firmware && cargo build --release --target thumbv7em-none-eabihf
	@echo "Running Node 2 on probe 0483:374b:066DFF3833584B3043115433..."
	probe-rs run --probe 0483:374b:066DFF3833584B3043115433 \
	  --chip STM32F446RETx \
	  target/thumbv7em-none-eabihf/release/node2-firmware

gateway:
	@echo "Building Node 2 firmware..."
	cd node2-firmware && cargo build --release --target thumbv7em-none-eabihf
	@echo "Starting gateway service..."
	cargo run --package wk6-async-gateway --release

clean:
	cargo clean
