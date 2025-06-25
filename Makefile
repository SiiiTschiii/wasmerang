# Makefile for wasmerang

.PHONY: help build build-wasm test fmt lint clean check-readme ci install-tools

# Default target
help:
	@echo "Available targets:"
	@echo "  build      - Build the project for host target"
	@echo "  build-wasm - Build the WASM module"
	@echo "  test       - Run unit tests"
	@echo "  fmt        - Format code with rustfmt"
	@echo "  lint       - Run clippy lints"
	@echo "  clean      - Clean build artifacts"
	@echo "  check-readme - Verify all README files are linked"
	@echo "  ci         - Run all CI checks locally"
	@echo "  install-tools - Install development tools"

# Build for host target
build:
	cargo build

# Build WASM module
build-wasm:
	cargo build --target wasm32-unknown-unknown --release

# Run tests
test:
	cargo test

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Run clippy
lint:
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Check README links
check-readme:
	python3 scripts/check-readme-links.py

# Run all CI checks locally
ci: fmt-check lint test build-wasm check-readme
	@echo "All CI checks passed!"

# Install development tools
install-tools:
	rustup target add wasm32-unknown-unknown
	cargo install cargo-audit
	cargo install cargo-deny
	cargo install cargo-license
	cargo install cargo-geiger
	cargo install cargo-llvm-cov

# Show WASM file size
wasm-size: build-wasm
	@ls -lh target/wasm32-unknown-unknown/release/wasmstreamcontext.wasm | awk '{print "WASM file size:", $$5}'

# Quick development cycle
dev: fmt lint test
	@echo "Development checks passed!"
