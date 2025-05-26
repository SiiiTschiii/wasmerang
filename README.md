# Envoy WASM Stream Context - a Proxy WASM filter written in Rust implementing the StreamContext to access the tcp connection

Most of the WASM filter examples are HTTP filters, but sometimes you need to access the TCP connection directly. This is where the StreamContext comes in handy. This filter is a simple example of how to use the StreamContext to access the TCP connection and log the source and destination IP addresses of the connection.

This project is insipired by https://github.com/otomato-gh/proxy-wasm-rust

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
proxy-1  | [2025-05-26 20:41:47.581][29][info][wasm] [source/extensions/common/wasm/context.cc:1137] wasm log tcp_dest_ip_logger tcp_dest_ip_logger vm.tcp_dest_ip_logger: [TCP WASM] Source address: 172.20.0.1:58900
proxy-1  | [2025-05-26 20:41:47.581][29][info][wasm] [source/extensions/common/wasm/context.cc:1137] wasm log tcp_dest_ip_logger tcp_dest_ip_logger vm.tcp_dest_ip_logger: [TCP WASM] Destination address: 172.20.0.2:10000
```

This shows that the filter is applied before the connection is established to the upstream service (www.example.com)
