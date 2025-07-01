//! WASM filter implementation for Envoy
//!
//! This module contains all the Envoy/WASM-specific code that integrates
//! the core routing logic with the proxy-wasm environment.

use crate::core::{Config, Router};
use log::{info, warn};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

// Include the generated protobuf code
pub mod set_envoy_filter_state {
    include!("generated/envoy.source.extensions.common.wasm.rs");
}

/// Root context for the WASM filter
///
/// This handles configuration parsing and creates stream contexts for new connections.
#[derive(Default)]
#[allow(dead_code)] // This struct is only used within the WASM context
struct PluginRootContext {
    config: Config,
}

impl Context for PluginRootContext {}

impl RootContext for PluginRootContext {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            self.config = Config::from_bytes(&config_bytes);
            info!(
                "[TCP WASM] Configuration: {:?}",
                std::str::from_utf8(&config_bytes).unwrap_or("invalid UTF-8")
            );
            info!(
                "[TCP WASM] Environment: {}",
                if self.config.is_istio {
                    "Istio/Kubernetes"
                } else {
                    "Standalone Envoy"
                }
            );
        } else {
            warn!("[TCP WASM] Failed to parse configuration as UTF-8");
        }
        true
    }

    fn create_stream_context(&self, _context_id: u32) -> Option<Box<dyn StreamContext>> {
        Some(Box::new(SourceBasedRouter {
            router: Router::new(self.config.clone()),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::StreamContext)
    }
}

/// Stream context for handling individual connections
///
/// This uses the core Router logic to make routing decisions and communicates
/// them to Envoy via the filter state mechanism.
#[allow(dead_code)] // This struct is only used within the WASM context
struct SourceBasedRouter {
    router: Router,
}

impl Context for SourceBasedRouter {}

impl StreamContext for SourceBasedRouter {
    fn on_new_connection(&mut self) -> Action {
        if let Some(source_addr_bytes) = self.get_property(vec!["source", "address"]) {
            if let Ok(source_addr) = std::str::from_utf8(&source_addr_bytes) {
                info!("[TCP WASM] Source address: {}", source_addr);

                // Get destination address to determine the port
                let dest_port = if let Some(dest_addr_bytes) =
                    self.get_property(vec!["destination", "address"])
                {
                    if let Ok(dest_addr) = std::str::from_utf8(&dest_addr_bytes) {
                        info!("[TCP WASM] Destination address: {}", dest_addr);
                        // Extract port from destination address (format: "ip:port")
                        if let Some(port_str) = dest_addr.split(':').last() {
                            port_str.parse::<u16>().unwrap_or(443) // Default to HTTPS
                        } else {
                            443 // Default to HTTPS
                        }
                    } else {
                        info!("[TCP WASM] Destination address: (non-UTF8)");
                        443 // Default to HTTPS
                    }
                } else {
                    info!("[TCP WASM] Destination address not found, defaulting to HTTPS port 443");
                    443
                };

                info!("[TCP WASM] Detected destination port: {}", dest_port);

                if let Some(cluster) = self
                    .router
                    .decide_route_cluster_with_dest(source_addr, dest_port)
                {
                    info!("[TCP WASM] Routing to {}", &cluster);

                    // Set dynamic metadata for rerouting using the proper Envoy mechanism
                    use set_envoy_filter_state::{LifeSpan, SetEnvoyFilterStateArguments};

                    let args = SetEnvoyFilterStateArguments {
                        path: "envoy.tcp_proxy.cluster".to_string(),
                        value: cluster.clone(),
                        span: LifeSpan::FilterChain as i32,
                    };
                    let mut buf = Vec::new();
                    if let Err(e) = prost::Message::encode(&args, &mut buf) {
                        warn!("[TCP WASM] Failed to encode filter state: {}", e);
                        return Action::Continue;
                    }

                    // Use the Envoy-specific filter state mechanism
                    // https://github.com/envoyproxy/envoy/issues/28128
                    let status = self.call_foreign_function("set_envoy_filter_state", Some(&buf));
                    info!(
                        "[TCP WASM] set_envoy_filter_state status (envoy.tcp_proxy.cluster): {:?}",
                        status
                    );
                    info!("[TCP WASM] Rerouting to {} via filter state", cluster);
                }
            }
        }
        Action::Continue
    }
}

/// WASM entry point
///
/// This function is called when the WASM module is loaded by Envoy.
#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(PluginRootContext::default())
    });
}
