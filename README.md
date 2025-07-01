# Wasmerang - A WASM TCP Filter for Istio (and Envoy)

[![CI](https://github.com/SiiiTschiii/wasmerang/workflows/CI/badge.svg)](https://github.com/SiiiTschiii/wasmerang/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/target-wasm32--unknown--unknown-green.svg)](https://webassembly.org/)
[![Envoy](https://img.shields.io/badge/envoy-proxy--wasm-purple.svg)](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/wasm)

A WebAssembly (WASM) filter for Istio (Envoy) that operates at the TCP layer using the StreamContext. This filter demonstrates how to create a **transparent proxy** that dynamically reroutes TCP traffic at runtime by intercepting the `network.tcp_proxy` filter chain and using Envoy's `set_envoy_filter_state` API to override cluster routing decisions.

Most WASM filter examples focus on HTTP, but this project shows how to work at the TCP/IP level for transparent network control without requiring application-level changes.

## Features

- **Transparent TCP proxy**: Intercepts and reroutes traffic without client knowledge
- **Runtime routing decisions**: Uses `set_envoy_filter_state` to dynamically override cluster destinations
- **TCP-level filtering**: Operates in the `network.tcp_proxy` filter chain.

## Quick Start

### Prerequisites

1. **Rust with WebAssembly target**:

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. **Choose your deployment method**:
   - **Docker**: Docker and Docker Compose
   - **Kubernetes**: kind, kubectl, istioctl

### Build the WASM Plugin

```bash
cargo build --target=wasm32-unknown-unknown --release
```

> **Note:** Protocol buffer code in `src/generated/` is pre-generated and committed to avoid requiring `protoc` for basic builds. To regenerate: `make regen-proto`

### Run Examples

**Standalone Envoy (Docker):**

```bash
cd examples/envoy-docker
```

See [examples/envoy-docker/README.md](examples/envoy-docker/README.md) for detailed instructions.

_Note: The Docker example demonstrates WASM-based dynamic routing but does not include transparent traffic interception. Clients must explicitly connect to the Envoy proxy._

**Istio/Kubernetes:**

```bash
cd examples/istio-k8s
```

See [examples/istio-k8s/README.md](examples/istio-k8s/README.md) for detailed instructions.

_Note: The Istio example provides full transparent proxy capabilities, where clients make normal requests but traffic is automatically intercepted and rerouted by the sidecar proxy._

## Architecture

The WASM filter acts as a **transparent proxy** by inserting itself into Envoy's `network.tcp_proxy` filter chain. When a TCP connection is established, the filter:

1. **Intercepts the connection** in the `on_new_connection()` callback
2. **Looks at the tcp connection** to make a routing decision (based on the source IP address)
3. **Uses `set_envoy_filter_state`** to dynamically override the cluster destination at runtime

**Routing Logic:**

- **Even last octet** (0, 2, 4, 6, 8) → Route to `egress1`
- **Odd last octet** (1, 3, 5, 7, 9) → Route to `egress2`

This creates a transparent proxy where clients connect to one destination but are seamlessly routed to different backend clusters based on their source IP, all without any client-side configuration or awareness.

### HTTP-to-TCP Override for Unified Traffic Handling

**Problem**: Istio/Envoy routes HTTP (port 80) through `http_connection_manager` and HTTPS (port 443) through `tcp_proxy` filter chains. Our WASM filter using StreamContext only sees TCP traffic, so HTTP bypasses it entirely.

**Solution**: Use an EnvoyFilter to force HTTP traffic through the TCP proxy chain by removing the auto-generated HTTP listener and adding a custom TCP listener with the WASM filter.

**Result**: Single WASM filter handles both HTTP and HTTPS traffic transparently using the same StreamContext routing logic. Trade-off: HTTP loses application-layer features but gains unified TCP-level proxying.

## Limitations & Future Improvements

1. **Pass destination to egress router**:

   - Implement PROXY protocol support to forward original destination IP:Port to egress routers

## Development & CI/CD

### Running Tests Locally

```bash
# Quick development checks
make dev

# Run all CI checks locally
make ci

# Individual commands
make test          # Run unit tests
make fmt           # Format code
make lint          # Run clippy lints
make build-wasm    # Build WASM module
make wasm-size     # Check WASM file size
make check-readme  # Verify all README files are linked

# Manual cargo commands
cargo test --verbose
cargo llvm-cov --html  # Requires: cargo install cargo-llvm-cov
cargo fmt --check
cargo clippy -- -D warnings
cargo audit            # Requires: cargo install cargo-audit
```

**Simple, fast, and reliable!** 🚀

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all CI checks pass
5. Submit a pull request

## Resources

- [Proxy-Wasm ABI specification](https://github.com/proxy-wasm/spec)
- [proxy-wasm-rust-sdk StreamContext](https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259)
- [Envoy WASM documentation](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/wasm)
- [set_envoy_filter_state discussion](https://github.com/envoyproxy/envoy/issues/28128)
