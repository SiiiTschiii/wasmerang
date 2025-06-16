# Wasmerang - A WASM TCP Filter for Envoy

A WebAssembly (WASM) filter for Envoy that operates at the TCP layer using the StreamContext. This filter demonstrates how to create a **transparent proxy** that dynamically reroutes TCP traffic at runtime by intercepting the `network.tcp_proxy` filter chain and using Envoy's `set_envoy_filter_state` API to override cluster routing decisions.

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

### Run Examples

**Standalone Envoy (Docker):**

```bash
cd examples/envoy-docker
# See examples/envoy-docker/README.md for detailed instructions
```

_Note: The Docker example demonstrates WASM-based dynamic routing but does not include transparent traffic interception. Clients must explicitly connect to the Envoy proxy._

**Istio/Kubernetes:**

```bash
cd examples/istio-k8s
# See examples/istio-k8s/README.md for detailed instructions
```

_Note: The Istio example provides full transparent proxy capabilities, where clients make normal requests but traffic is automatically intercepted and rerouted by the sidecar proxy._

## Architecture

The WASM filter acts as a **transparent proxy** by inserting itself into Envoy's `network.tcp_proxy` filter chain. When a TCP connection is established, the filter:

1. **Intercepts the connection** in the `on_new_connection()` callback
2. **Analyzes the source IP** address to make routing decisions
3. **Uses `set_envoy_filter_state`** to dynamically override the cluster destination at runtime
4. **Allows the connection to proceed** transparently to the new destination

**Routing Logic:**

- **Even last octet** (0, 2, 4, 6, 8) → Route to `egress1`
- **Odd last octet** (1, 3, 5, 7, 9) → Route to `egress2`

This creates a transparent proxy where clients connect to one destination but are seamlessly routed to different backend clusters based on their source IP, all without any client-side configuration or awareness.

## Limitations & Future Improvements

### Current Limitations

1. **No destination information forwarding**: The egress routers currently don't receive information about the original destination (IP:Port) that the client intended to reach
2. **HTTPS-only in Istio**: Only works for HTTPS traffic (port 443) in Istio environments, as HTTP traffic (port 80) uses the HTTP connection manager filter chain instead of the TCP proxy filter chain

### Planned Enhancements

1. **Pass destination to egress router**:

   - Implement PROXY protocol support to forward original destination IP:Port to egress routers
   - This would allow egress routers to make intelligent routing decisions based on the intended destination
   - Alternative: Use custom headers or metadata to pass destination information

2. **Support HTTP traffic routing**:
   - Extend the filter to work with HTTP connection manager filter chain
   - Implement HTTP-level WASM filter in addition to TCP-level filtering
   - This would enable transparent proxying for all traffic types, not just HTTPS

These improvements would make the transparent proxy more complete and production-ready for diverse network environments.

## Resources

- [Proxy-Wasm ABI specification](https://github.com/proxy-wasm/spec)
- [proxy-wasm-rust-sdk StreamContext](https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259)
- [Envoy WASM documentation](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/wasm)
- [set_envoy_filter_state discussion](https://github.com/envoyproxy/envoy/issues/28128)
