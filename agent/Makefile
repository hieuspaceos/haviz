.PHONY: all build helpers clean run

all: helpers build

# Compile Swift helpers for AX API reading
helpers:
	@echo "Compiling zalo_reader (Desktop)..."
	swiftc helpers/zalo_reader.swift -o helpers/zalo_reader \
		-framework Cocoa -framework ApplicationServices
	@echo "Compiling zalo_web_reader (Browser)..."
	swiftc helpers/zalo_web_reader.swift -o helpers/zalo_web_reader \
		-framework Cocoa -framework ApplicationServices
	@echo "Done: helpers/zalo_reader + helpers/zalo_web_reader"

# Build Rust agent
build: helpers
	cargo build

# Build release
release: helpers
	cargo build --release

# Run agent in dev mode
run: helpers
	cargo run

# Clean
clean:
	cargo clean
	rm -f helpers/zalo_reader
