# Envoy Docker Example

This example demonstrates how to use the WASM TCP filter with standalone Envoy running in Docker Compose.

## Quick Start

From the repository root:

```bash
# 1. Build the WASM plugin (run from repository root)
cd ../../
cargo build --target=wasm32-unknown-unknown --release

# 2. Start the example
cd examples/envoy-docker
docker-compose up
```

## Test the Routing

In separate terminals, test the routing behavior:

```bash
# Even IP (last octet 10) → routes to egress1
docker run --rm -it --network envoy-docker_envoymesh --ip 172.21.0.10 curlimages/curl curl http://proxy:10000 -H "Host: www.ipconfig.io"

# Odd IP (last octet 11) → routes to egress2
docker run --rm -it --network envoy-docker_envoymesh --ip 172.21.0.11 curlimages/curl curl http://proxy:10000 -H "Host: www.ipconfig.io"
```

## Expected Output

Check the Docker Compose logs to see the WASM filter in action:

```
proxy-1  | [TCP WASM] Source address: 172.21.0.10:58762
proxy-1  | [TCP WASM] Source IP last octet: 10, intercepting ALL traffic
proxy-1  | [TCP WASM] Routing to egress1
proxy-1  | [TCP WASM] set_envoy_filter_state status (envoy.tcp_proxy.cluster): Ok(None)
proxy-1  | [TCP WASM] Rerouting to egress1 via filter state
proxy-1  | [2025-06-15T19:48:34.841Z] cluster=egress1 src=172.21.0.10:58762 dst=172.21.0.2:10000 -> 104.21.16.1:80
```

## Architecture

- **Envoy Proxy**: Listens on port 10000, applies the WASM filter
- **WASM Filter**: Routes traffic based on source IP last octet
- **egress1/egress2 Clusters**: Target external services (ipconfig.io)

## Configuration Files

- [`docker-compose.yaml`](docker-compose.yaml): Defines the services and network
- [`envoy/envoy.yaml`](envoy/envoy.yaml): Envoy configuration with WASM filter

## Cleanup

```bash
docker-compose down
```
