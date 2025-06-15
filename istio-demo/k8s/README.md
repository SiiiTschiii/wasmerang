# Istio WASM Filter Demo (kind + Istio)

This demo shows how to use your WASM TCP filter with Istio in a local kind cluster. It includes a Go client that makes HTTP requests and two Go servers (egress1, egress2) that receive traffic based on the last octet of the source IP, as determined by the WASM filter.

## Structure

- `go-client/`: Go HTTP client app
- `go-server/`: Go HTTP server app (used for both egress1 and egress2)
- `Dockerfile.client`, `Dockerfile.server`: Dockerfiles for building the apps
- `k8s/`: Kubernetes manifests for deployments, services, ServiceEntry, and (to be added) EnvoyFilter

## Prerequisites

1. **Install kind** (Kubernetes in Docker):

   ```sh
   go install sigs.k8s.io/kind@v0.29.0
   # Add kind to your PATH if not already present
   export PATH="$PATH:$(go env GOPATH)/bin"
   # Optionally, add the above line to your ~/.zshrc for persistence
   ```

2. **Install istioctl** (Istio CLI):
   ```sh
   curl -sL https://istio.io/downloadIstioctl | sh -
   export PATH="$HOME/.istioctl/bin:$PATH"
   # Optionally, add the above line to your ~/.zshrc for persistence
   ```

## Quickstart

1. **Create kind cluster and install Istio**:

   ```sh
   kind create cluster --name istio-wasm
   istioctl install --set profile=demo -y
   # Enable automatic sidecar injection in the default namespace
   kubectl label namespace default istio-injection=enabled --overwrite
   ```

2. **Build WASM filter** (from repo root):

   ```sh
   cargo build --target=wasm32-unknown-unknown --release
   # Make wasm file available to Istio (ConfigMap or HTTP server)
   kubectl delete configmap tcp-wasm-filter -n default
   kubectl create configmap tcp-wasm-filter --from-file=wasmstreamcontext.wasm=target/wasm32-unknown-unknown/release/wasmstreamcontext.wasm -n default
   ```

3. **Build Go images** (from `istio-demo/`):

   ```sh
   docker build -t go-client:latest -f Dockerfile.client .
   docker build -t go-server:latest -f Dockerfile.server .
   # Load images into kind so the cluster can use them
   kind load docker-image go-client:latest --name istio-wasm
   kind load docker-image go-server:latest --name istio-wasm
   # If using a remote registry, push images and update manifests
   ```

4. **Deploy demo apps**:

   ```sh
   kubectl apply -f k8s/egress1-deployment.yaml
   kubectl apply -f k8s/egress2-deployment.yaml
   kubectl apply -f k8s/client-deployment.yaml
   kubectl apply -f k8s/serviceentry.yaml
   # (Apply EnvoyFilter manifest after editing for your WASM filter)
   ```

5. **Test**:
   ```sh
   kubectl get pods
   kubectl logs deploy/go-client
   # You should see output from egress1 or egress2 depending on the pod IP
   ```

## Notes

- Only the go-client deployment has `istio-injection: enabled` in its pod template labels. The egress1 and egress2 deployments have `istio-injection: disabled` to prevent sidecar injection.
- Update the EnvoyFilter manifest to point to your WASM file (ConfigMap or HTTP URL).
- You can exec into the client pod to run more requests if needed.
- For production, use a proper image registry and update image references in the manifests.

## WASM Filter Behavior Analysis

### Working Cases ✅

The WASM TCP filter successfully intercepts:

1. **TCP connections to internal services** (egress1:8080, egress2:8080)
2. **HTTPS connections** (port 443) to external sites
3. **Any non-standard ports** that Istio doesn't auto-detect as HTTP

### Not Working Cases ❌

The WASM filter does NOT intercept:

1. **HTTP connections on port 80** - Uses HTTP connection manager instead of TCP proxy
2. **HTTP client library requests** - Go through HTTP filter chains

### Why Port 80 vs 443 Behave Differently

Istio automatically detects well-known ports and routes them through different filter chains:

**Port 443 (HTTPS) - Uses TCP Proxy Filter Chain:**

```bash
kubectl exec <go-client-pod> -c istio-proxy -- curl -s localhost:15000/config_dump | \
  jq '.configs[] | select(.["@type"] | contains("Listener")) | .dynamic_listeners[] |
      select(.name == "0.0.0.0_443") | .active_state.listener.filter_chains[0].filters[] | .name'
```

Result:

- `"istio.stats"`
- `"envoy.filters.network.wasm"` ← **Our WASM filter is here!**
- `"envoy.filters.network.tcp_proxy"`

**Port 80 (HTTP) - Uses HTTP Connection Manager:**

```bash
kubectl exec <go-client-pod> -c istio-proxy -- curl -s localhost:15000/config_dump | \
  jq '.configs[] | select(.["@type"] | contains("Listener")) | .dynamic_listeners[] |
      select(.name == "0.0.0.0_80") | .active_state.listener.filter_chains[0].filters[] | .name'
```

Result:

- `"envoy.filters.network.http_connection_manager"` ← **No WASM filter here!**

### Debugging Filter Chains

To investigate which filter chain handles specific ports:

1. **List all listeners:**

```bash
kubectl exec <go-client-pod> -c istio-proxy -- curl -s localhost:15000/config_dump | \
  jq '.configs[] | select(.["@type"] | contains("Listener")) | .dynamic_listeners[] | .name'
```

2. **Check filter chain for specific port:**

```bash
kubectl exec <go-client-pod> -c istio-proxy -- curl -s localhost:15000/config_dump | \
  jq '.configs[] | select(.["@type"] | contains("Listener")) | .dynamic_listeners[] |
      select(.name == "0.0.0.0_<PORT>") | .active_state.listener.filter_chains[0].filters[] | .name'
```

### Summary

The WASM TCP filter works correctly for actual TCP traffic:

- ✅ **Port 443 (HTTPS)**: Uses TCP proxy filter chains and gets intercepted
- ❌ **Port 80 (HTTP)**: Uses HTTP connection manager and bypasses TCP filters
- ✅ **Non-standard ports**: Would use TCP proxy filter chains and get intercepted

This behavior is by design in Istio/Envoy - HTTP traffic gets HTTP-specific features like routing, retries, and circuit breakers, while raw TCP traffic goes through TCP proxy filter chains.

## Troubleshooting

- **Check WASM filter logs:** `kubectl logs <go-client-pod> -c istio-proxy | grep "TCP WASM"`
- **Check filter chains:** Use the debugging commands above
- **Verify WASM file:** `kubectl exec <go-client-pod> -c istio-proxy -- ls -la /etc/wasm/`
- **Check Envoy config:** `kubectl exec <go-client-pod> -c istio-proxy -- curl localhost:15000/config_dump`
