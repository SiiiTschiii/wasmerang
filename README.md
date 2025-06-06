# Wasmerang - A WASM network filter for Envoy using the StreamContext for TCP of the WASM Rust SDK

Most of the WASM filter examples are HTTP filters, but what if you want to avoid the application layer and stay on TCP/IP? This is where the StreamContext comes in handy. This filter is an example of how to use the StreamContext to access the TCP connection information and then re-route tcp traffic based on it.

This project is insipired by https://github.com/otomato-gh/proxy-wasm-rust. Thanks to the author https://github.com/antweiss!

Helpful resources:

- [Proxy-Wasm ABI specification](https://github.com/proxy-wasm/spec)
- [proxy-wasm-rust-sdk StreamContext](https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259)
- [Envoy WASM documentation](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/wasm)
- [set_envoy_filter_state thread 1](https://github.com/envoyproxy/envoy/issues/15148#issuecomment-2913718379)
- [set_envoy_filter_state thread 2](https://github.com/envoyproxy/envoy/issues/28128)

## Building and running:

1. clone this repo
2. `rustup target add wasm32-unknown-unknown`, to build for WebAssembly, only needed once per machine
3. `cargo build --target=wasm32-unknown-unknown --release`
4. `docker-compose up --build`

## Testing it Works

```bash
# source ip with even last octet are routed via egress1
docker run --rm -it --network wasmerang_envoymesh --ip 172.21.0.10 curlimages/curl curl http://proxy:10000 -H "Host: www.ipconfig.io"

# source ip with even last octet are routed via egress1
docker run --rm -it --network wasmerang_envoymesh --ip 172.21.0.11 curlimages/curl curl http://proxy:10000 -H "Host: www.ipconfig.io"
```

Check the logs to see the source IP as seen by the WASM filter and which egress it was routed to:

```
proxy-1  | [2025-06-06 08:30:29.357][31][info][wasm] [source/extensions/common/wasm/context.cc:1137] wasm log tcp_dest_ip_logger tcp_dest_ip_logger vm.tcp_dest_ip_logger: [TCP WASM] Source address: 172.21.0.10:58762
proxy-1  | [2025-06-06T08:30:29.356Z] cluster=egress1 src=172.21.0.10:58762 dst=172.21.0.2:10000 -> 104.21.48.1:80

proxy-1  | [2025-06-06 08:31:13.384][31][info][wasm] [source/extensions/common/wasm/context.cc:1137] wasm log tcp_dest_ip_logger tcp_dest_ip_logger vm.tcp_dest_ip_logger: [TCP WASM] Source address: 172.21.0.11:34498
proxy-1  | [2025-06-06T08:31:13.384Z] cluster=egress2 src=172.21.0.11:34498 dst=172.21.0.2:10000 -> 104.21.48.1:80
```
