# Istio Kubernetes Example

This example demonstrates how to use the WASM TCP filter with Istio in a Kubernetes cluster using kind.

## Overview

This setup demonstrates a transparent TCP proxy using WASM filters:

- **WASM filter** acts as a transparent proxy for HTTPS traffic, intercepting TCP connections on port 443
- **go-client** makes HTTP/HTTPS requests to httpbin.org, with different behaviors:
  - **HTTPS requests**: Intercepted by WASM filter and routed through egress-router1/egress-router2 on port 8080
  - **HTTP requests**: Use Istio's HTTP filter chain, bypass WASM filter, and are blocked by NetworkPolicy since only egress routers are allowed internet access
- **egress-router1/egress-router2** act as TCP bridges, forwarding intercepted traffic to httpbin.org (54.198.84.155:443)
- **NetworkPolicy** blocks direct internet access, only allowing traffic to egress routers and DNS resolution
- This creates a controlled egress path where all HTTPS traffic is transparently routed through monitored egress points

## Quick Start

From the repository root:

```bash
# 1. Create kind cluster and install Istio
kind create cluster --name istio-wasm
istioctl install --set profile=demo -y
kubectl label namespace default istio-injection=enabled --overwrite

# 2. Build the WASM plugin (from repository root)
cd ../../  # Go back to repository root if you're in examples/istio-k8s
cargo build --target=wasm32-unknown-unknown --release

# 3. Create ConfigMap with WASM binary
kubectl create configmap tcp-wasm-filter \
  --from-file=wasmstreamcontext.wasm=target/wasm32-unknown-unknown/release/wasmstreamcontext.wasm \
  -n default

# 4. Build and load Docker images
cd examples/istio-k8s
docker build -t go-client:latest -f Dockerfile.client .
docker build -t egress-router:latest -f Dockerfile.egress-router .
kind load docker-image go-client:latest --name istio-wasm
kind load docker-image egress-router:latest --name istio-wasm

# 5. Deploy Kubernetes resources
cd k8s
kubectl apply -f egress-router1-deployment.yaml
kubectl apply -f egress-router2-deployment.yaml
kubectl apply -f client-deployment.yaml
kubectl apply -f envoyfilter.yaml
kubectl apply -f network-policies.yaml
```

## Expected Output

The go-client will make both HTTP and HTTPS requests to httpbin.org. You should see output like:

```
HTTPS httpbin.org 213.55.242.101 200
HTTP httpbin.org - 503
HTTPS httpbin.org 213.55.242.101 200
HTTP httpbin.org - 503
```

This shows:

- **HTTP requests (port 80)**: Go through Istio HTTP connection manager, bypassing WASM filter. Since NetworkPolicy only allows egress routers to access the internet, HTTP traffic fails with 503 responses
- **HTTPS requests (port 443)**: Intercepted by WASM filter acting as transparent proxy and routed through egress-router (IP 213.55.242.101)

Check WASM filter logs in the Envoy sidecar:

```bash
kubectl logs -l app=go-client -c istio-proxy --tail=20 | grep "TCP WASM"
```

You should see WASM filter logs showing the routing decisions:

```
[TCP WASM] Source address: 10.244.0.64:34028
[TCP WASM] Source IP last octet: 64, intercepting ALL traffic
[TCP WASM] Routing to egress-router1
[TCP WASM] set_envoy_filter_state status (envoy.tcp_proxy.cluster): Ok(None)
[TCP WASM] Rerouting to egress-router1 via filter state
```

## How It Works

### WASM Filter Behavior

The WASM TCP filter successfully intercepts:

- ✅ **HTTPS connections (port 443)** to external sites - Uses TCP proxy filter chain
- ✅ **TCP connections** to internal services (egress-router:8080)
- ✅ **Any non-standard ports** that Istio doesn't auto-detect as HTTP

The WASM filter does NOT intercept:

- ❌ **HTTP connections (port 80)** - Uses HTTP connection manager instead of TCP proxy
- ❌ **HTTP client library requests** - Go through HTTP filter chains

### Why Port 80 vs 443 Behave Differently

Istio automatically detects well-known ports and routes them through different filter chains:

**Port 443 (HTTPS) - Uses TCP Proxy Filter Chain:**

- `istio.stats`
- `envoy.filters.network.wasm` ← **Your WASM filter is here!**
- `envoy.filters.network.tcp_proxy`

**Port 80 (HTTP) - Uses HTTP Connection Manager:**

- `envoy.filters.network.http_connection_manager` ← **No WASM filter here!**

### NetworkPolicy Enforcement

The NetworkPolicy blocks direct internet access by only allowing:

- DNS resolution (port 53)
- Traffic to egress-router pods (port 8080)
- Istio control plane communication (ports 15xxx)

This results in different behavior for HTTP vs HTTPS:

- **HTTP traffic** goes through Istio's HTTP connection manager (PassthroughCluster) but is ultimately blocked by NetworkPolicy, resulting in 503 responses
- **HTTPS traffic** gets intercepted by WASM acting as a transparent proxy and redirected to allowed egress-router destinations

## Prerequisites

1. **Install kind** (Kubernetes in Docker):

   ```bash
   go install sigs.k8s.io/kind@v0.29.0
   export PATH="$PATH:$(go env GOPATH)/bin"
   ```

2. **Install istioctl** (Istio CLI):
   ```bash
   curl -sL https://istio.io/downloadIstioctl | sh -
   export PATH="$HOME/.istioctl/bin:$PATH"
   ```

## Architecture

- **go-client**: Makes HTTP/HTTPS requests, has Istio sidecar with WASM filter
- **egress-router1/egress-router2**: TCP bridge servers that forward traffic to httpbin.org (54.198.84.155:443)
- **EnvoyFilter**: Configures the WASM plugin in Istio sidecars

## Configuration Files

- [`k8s/envoyfilter.yaml`](k8s/envoyfilter.yaml): Istio EnvoyFilter configuration for WASM plugin
- [`k8s/client-deployment.yaml`](k8s/client-deployment.yaml): Client application deployment with Istio sidecar
- [`k8s/egress-router1-deployment.yaml`](k8s/egress-router1-deployment.yaml): Egress router 1 deployment
- [`k8s/egress-router2-deployment.yaml`](k8s/egress-router2-deployment.yaml): Egress router 2 deployment
- [`k8s/network-policies.yaml`](k8s/network-policies.yaml): NetworkPolicy to block direct internet access

## Debugging Filter Chains

To investigate which filter chain handles specific ports:

```bash
# Get the go-client pod name
POD=$(kubectl get pod -l app=go-client -o jsonpath='{.items[0].metadata.name}')

# Check filter chain for port 80 (HTTP)
kubectl exec $POD -c istio-proxy -- curl -s localhost:15000/config_dump | \
  jq '.configs[] | select(.["@type"] | contains("Listener")) | .dynamic_listeners[] |
      select(.name == "0.0.0.0_80") | .active_state.listener.filter_chains[0].filters[] | .name'

# Check filter chain for port 443 (HTTPS)
kubectl exec $POD -c istio-proxy -- curl -s localhost:15000/config_dump | \
  jq '.configs[] | select(.["@type"] | contains("Listener")) | .dynamic_listeners[] |
      select(.name == "0.0.0.0_443") | .active_state.listener.filter_chains[0].filters[] | .name'
```

## Updating the WASM Plugin

To update the WASM plugin after making changes:

```bash
# From repository root
cargo build --target=wasm32-unknown-unknown --release

# Update ConfigMap
kubectl create configmap tcp-wasm-filter \
  --from-file=wasmstreamcontext.wasm=target/wasm32-unknown-unknown/release/wasmstreamcontext.wasm \
  -n default --dry-run=client -o yaml | kubectl apply -f -

# Restart client deployment to pick up new WASM binary
kubectl rollout restart deployment/go-client -n default
kubectl rollout status deployment/go-client -n default
```

## Cleanup

```bash
kubectl delete -f k8s/
kubectl delete configmap tcp-wasm-filter -n default
kind delete cluster --name istio-wasm
```

## Troubleshooting

- **Check application logs:** `kubectl logs -l app=go-client -c go-client --tail=10`
- **Check WASM filter logs:** `kubectl logs -l app=go-client -c istio-proxy | grep "TCP WASM"`
- **Check NetworkPolicy:** `kubectl get networkpolicy -o yaml`
- **Verify WASM file:** `kubectl exec $POD -c istio-proxy -- ls -la /etc/wasm/`
- **Check Envoy config:** `kubectl exec $POD -c istio-proxy -- curl localhost:15000/config_dump`
