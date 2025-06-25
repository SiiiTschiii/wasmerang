# Makefile for wasmerang

.PHONY: help build build-wasm test fmt lint clean check-readme check-deps ci install-tools regen-proto

# Default target
help:
	@echo "  build      - Build the project for host target"
	@echo "  build-wasm - Build the WASM module"
	@echo "  test       - Run unit tests"
	@echo "  fmt        - Format code with rustfmt"
	@echo "  lint       - Run clippy lints"
	@echo "  clean      - Clean build artifacts"
	@echo "  check-readme - Verify all README files are linked"
	@echo "  check-deps - Check required dependencies (protoc, rust)"
	@echo "  regen-proto - Regenerate protocol buffer code"
	@echo "  ci         - Run all CI checks locally"
	@echo "  install-tools - Install development tools"

.PHONY: help build build-wasm test fmt lint clean check-readme check-deps ci install-tools regen-proto

# Check dependencies
check-deps:
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

# Check prerequisites
check-deps:
	@echo "Checking prerequisites..."
	@command -v protoc >/dev/null 2>&1 || { echo >&2 "protoc is required but not installed. Run 'make install-tools' for instructions."; exit 1; }
	@echo "✅ protoc found: $$(protoc --version)"
	@command -v rustc >/dev/null 2>&1 || { echo >&2 "Rust is required but not installed."; exit 1; }
	@echo "✅ Rust found: $$(rustc --version)"

# Run all CI checks locally
ci: check-deps fmt-check lint test build-wasm check-readme
	@echo "All CI checks passed!"

# Install development tools
install-tools:
	rustup target add wasm32-unknown-unknown
	cargo install cargo-audit
	cargo install cargo-deny
	cargo install cargo-license
	cargo install cargo-geiger
	cargo install cargo-llvm-cov
	@echo ""
	@echo "Also install protoc (Protocol Buffers compiler):"
	@echo "  macOS: brew install protobuf"
	@echo "  Ubuntu/Debian: sudo apt-get install protobuf-compiler"
	@echo "  Arch: sudo pacman -S protobuf"

# Regenerate protocol buffer code (requires protoc)
regen-proto: check-deps
	@echo "Regenerating protocol buffer code..."
	cargo clean
	cargo build --target wasm32-unknown-unknown --release
	@echo "✅ Protocol buffer code regenerated"

# Show WASM file size
wasm-size: build-wasm
	@ls -lh target/wasm32-unknown-unknown/release/wasmstreamcontext.wasm | awk '{print "WASM file size:", $$5}'

# Quick development cycle
dev: fmt lint test
	@echo "Development checks passed!"
