# Wasmerang - A WASM network filter for Envoy using the StreamContext of the WASM Rust SDK to access the tcp connection

Most of the WASM filter examples are HTTP filters, but sometimes you want to access the TCP connection directly. This is where the StreamContext comes in handy. This filter is a simple example of how to use the StreamContext to access the TCP connection and log the source and destination IP addresses of the connection.

This project is insipired by https://github.com/otomato-gh/proxy-wasm-rust. Thanks to the author https://github.com/antweiss!

Helpful resources:

- [Proxy-Wasm ABI specification](https://github.com/proxy-wasm/spec)
- [proxy-wasm-rust-sdk StreamContext](https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259)
- [Envoy WASM documentation](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/wasm)

## Building and running:

1. clone this repo
2. `rustup target add wasm32-unknown-unknown`, to build for WebAssembly, only needed once per machine
3. `cargo build --target=wasm32-unknown-unknown --release`
4. `docker-compose up --build`

## Testing it Works

```bash
curl "http://localhost:19000/" -H "Host: www.example.com"
```

Check the logs of the envoy container to see the source and destination IP addresses of the connection

```
proxy-1  ... [TCP WASM] Source address: 172.20.0.1:58900
proxy-1  ... [TCP WASM] Destination address: 172.20.0.2:10000
```

This shows that the filter is applied before the connection is established to the upstream service (`www.example.com`).


https://github.com/envoyproxy/envoy/issues/15148#issuecomment-2913718379
https://github.com/envoyproxy/envoy/issues/28128
