use log::{info};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_stream_context(|context_id, _| -> Box<dyn StreamContext> {
        Box::new(DestIpLogger { context_id })
    });
}

struct DestIpLogger {
    context_id: u32,
}

impl Context for DestIpLogger {}

impl StreamContext for DestIpLogger {
     // See https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259
    fn on_new_connection(&mut self) -> Action {
        info!("[TCP WASM] New connection established (context #{})", self.context_id);
        // Log source address
        // Property names based on the Proxy-Wasm ABI version 0.2.1
        // https://github.com/proxy-wasm/spec/tree/main/abi-versions/v0.2.1#downstream-connection-properties
        if let Some(val) = self.get_property(vec!["source", "address"]) {
            if let Ok(s) = String::from_utf8(val) {
                info!("[TCP WASM] Source address: {}", s);
            } else {
                info!("[TCP WASM] Source address: (non-UTF8)");
            }
        } else {
            info!("[TCP WASM] Source address not found");
        }
        // Log destination address and set reroute metadata
        let mut reroute_cluster: Option<&str> = None;
        if let Some(val) = self.get_property(vec!["destination", "address"]) {
            if let Ok(s) = String::from_utf8(val) {
                info!("[TCP WASM] Destination address: {}", s);
                // Parse last octet
                if let Some(ip_part) = s.split(':').next() {
                    if let Some(last_octet) = ip_part.split('.').last() {
                        if let Ok(num) = last_octet.parse::<u8>() {
                            if num % 2 == 0 {
                                // Even last octet, reroute to egress2
                                reroute_cluster = Some("egress2");
                            } else {
                                // Odd last octet, reroute to egress1
                                reroute_cluster = Some("egress1");
                            }
                        }
                    }
                }
            } else {
                info!("[TCP WASM] Destination address: (non-UTF8)");
            }
        } else {
            info!("[TCP WASM] Destination address not found");
        }
        // Set dynamic metadata for rerouting if needed
        if let Some(cluster) = reroute_cluster {
            // Use prost-generated protobuf struct for filter state
            use set_envoy_filter_state::{SetEnvoyFilterStateArguments, LifeSpan};
            use prost::Message;

            let args = SetEnvoyFilterStateArguments {
                path: "envoy.tcp_proxy.cluster".to_string(),
                value: cluster.to_string(),
                span: LifeSpan::FilterChain as i32, // or LifeSpan::DownstreamConnection if preferred
            };
            let mut buf = Vec::new();
            args.encode(&mut buf).unwrap();
            // se background to set_envoy_filter_state
            // https://github.com/envoyproxy/envoy/issues/28128
            // https://github.com/envoyproxy/envoy/issues/28128
            let status = self.call_foreign_function("set_envoy_filter_state", Some(&buf));
            info!("[TCP WASM] set_envoy_filter_state status (envoy.tcp_proxy.cluster): {:?}", status);
            info!("[TCP WASM] Rerouting to {} via filter state", cluster);
        }
        Action::Continue
    }

}pub mod set_envoy_filter_state {
    include!("generated/envoy.source.extensions.common.wasm.rs");
}