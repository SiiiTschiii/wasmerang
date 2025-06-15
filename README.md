# Wasmerang - A WASM TCP Filter for Envoy

A WebAssembly (WASM) filter for Envoy that operates at the TCP layer using the StreamContext. This filter demonstrates how to access TCP connection information and dynamically route traffic based on the source IP address.

Most WASM filter examples focus on HTTP, but this project shows how to work at the TCP/IP level for more fundamental network control.

## Features

- **Environment-agnostic**: Works with both standalone Envoy and Istio/Kubernetes
- **Dynamic routing**: Routes traffic based on source IP last octet (even → egress1, odd → egress2)
- **TCP-level filtering**: Operates below the application layer for maximum flexibility

## How It Works

The filter uses configuration to determine the environment:

- **Docker/Standalone Envoy**: Configure with `"standalone"` - uses simple cluster names (`egress1`, `egress2`)
- **Istio/Kubernetes**: Configure with `"istio"` - uses full Istio cluster names (`outbound|{port}||egress1.default.svc.cluster.local`)

The environment is specified through the WASM filter configuration in the Envoy config files.

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

**Istio/Kubernetes:**

```bash
cd examples/istio-k8s
# See examples/istio-k8s/README.md for detailed instructions
```

## Architecture

The WASM filter intercepts TCP connections and examines the source IP address. Based on the last octet of the source IP:

- **Even numbers** (0, 2, 4, 6, 8) → Route to `egress1`
- **Odd numbers** (1, 3, 5, 7, 9) → Route to `egress2`

The filter uses Envoy's `set_envoy_filter_state` API to dynamically override the cluster assignment at runtime.

## Resources

- [Proxy-Wasm ABI specification](https://github.com/proxy-wasm/spec)
- [proxy-wasm-rust-sdk StreamContext](https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259)
- [Envoy WASM documentation](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/wasm)
- [set_envoy_filter_state discussion](https://github.com/envoyproxy/envoy/issues/28128)
