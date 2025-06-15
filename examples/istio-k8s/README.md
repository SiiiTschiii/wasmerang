# Istio Kubernetes Example

This example demonstrates how to use the WASM TCP filter with Istio in a Kubernetes cluster using kind.

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
docker build -t go-server:latest -f Dockerfile.server .
kind load docker-image go-client:latest --name istio-wasm
kind load docker-image go-server:latest --name istio-wasm

# 5. Deploy Kubernetes resources
cd k8s
kubectl apply -f egress1-deployment.yaml
kubectl apply -f egress2-deployment.yaml
kubectl apply -f client-deployment.yaml
kubectl apply -f envoyfilter.yaml
```

## Test the Deployment

Check the logs to see the WASM filter in action:

```bash
# Check application logs
kubectl logs -l app=go-client -c go-client --tail=10

# Check WASM filter logs in Envoy sidecar
kubectl logs -l app=go-client -c istio-proxy --tail=20 | grep "TCP WASM"
```

## Expected Output

You should see WASM filter logs showing the routing decisions:

```
[TCP WASM] Source address: 10.244.0.64:34028
[TCP WASM] Source IP last octet: 64, intercepting ALL traffic
[TCP WASM] Routing to egress1
[TCP WASM] set_envoy_filter_state status (envoy.tcp_proxy.cluster): Ok(None)
[TCP WASM] Rerouting to egress1 via filter state
```

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
- **egress1/egress2**: TCP echo servers that receive routed traffic
- **EnvoyFilter**: Configures the WASM plugin in Istio sidecars

## Configuration Files

- [`k8s/envoyfilter.yaml`](k8s/envoyfilter.yaml): Istio EnvoyFilter configuration
- [`k8s/client-deployment.yaml`](k8s/client-deployment.yaml): Client application deployment
- [`k8s/egress1-deployment.yaml`](k8s/egress1-deployment.yaml): Egress1 server deployment
- [`k8s/egress2-deployment.yaml`](k8s/egress2-deployment.yaml): Egress2 server deployment

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

## Notes

- The WASM filter only intercepts TCP traffic (port 443 HTTPS works, port 80 HTTP uses HTTP filters)
- The go-client makes requests that get intercepted and routed to egress1/egress2 instead of external sites
- Connection errors are expected since egress1/egress2 are TCP echo servers, not HTTP servers
