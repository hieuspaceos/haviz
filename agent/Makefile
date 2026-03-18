.PHONY: all build helpers clean run

all: helpers build

# Compile Swift helper for AX API reading
helpers:
	@echo "Compiling zalo_reader..."
	swiftc helpers/zalo_reader.swift -o helpers/zalo_reader \
		-framework Cocoa -framework ApplicationServices
	@echo "Done: helpers/zalo_reader"

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
