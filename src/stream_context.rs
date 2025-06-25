//
// The following structs and functions are the core logic of the router.
// They are plain Rust and have no dependency on the proxy-wasm crate,
// making them easy to test.
//

pub mod set_envoy_filter_state {
    include!("generated/envoy.source.extensions.common.wasm.rs");
}

/// Represents the parsed configuration for the filter.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Config {
    pub is_istio: bool,
}

impl Config {
    /// Parses the configuration from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let config_str = std::str::from_utf8(bytes).unwrap_or("");
        Config {
            is_istio: config_str.contains("istio"),
        }
    }
}

/// The core routing logic.
#[derive(Debug, Clone)]
pub struct Router {
    config: Config,
}

impl Router {
    /// Creates a new Router with the given configuration.
    pub fn new(config: Config) -> Self {
        Router { config }
    }

    /// Determines the names of the two egress clusters based on the configuration.
    pub fn get_cluster_names(&self) -> (String, String) {
        if self.config.is_istio {
            (
                "outbound|8080||egress-router1.default.svc.cluster.local".to_string(),
                "outbound|8080||egress-router2.default.svc.cluster.local".to_string(),
            )
        } else {
            ("egress-router1".to_string(), "egress-router2".to_string())
        }
    }

    /// Given a source IP address, decides which cluster to route to.
    /// Returns `None` if the IP address is invalid.
    pub fn decide_route_cluster(&self, source_address: &str) -> Option<String> {
        let last_octet = extract_last_octet(source_address)?;
        let (cluster1, cluster2) = self.get_cluster_names();

        if last_octet % 2 == 0 {
            Some(cluster1)
        } else {
            Some(cluster2)
        }
    }
}

/// Extracts the last octet from an IP address string (v4 or v6).
/// Returns `None` if parsing fails.
fn extract_last_octet(ip_address: &str) -> Option<u8> {
    // Handles "[ipv6]:port" by taking the part between the brackets
    let ip_part = if ip_address.starts_with('[') {
        ip_address.split(']').next()?.strip_prefix('[')?
    } else {
        // Handles "ipv4:port"
        ip_address.split(':').next()?
    };

    // Handles ipv4-mapped ipv6 "::ffff:1.2.3.4"
    let ipv4_part = ip_part.split(':').next_back()?;

    ipv4_part.split('.').next_back()?.parse::<u8>().ok()
}

//
// The following code is the "glue" that connects the core logic to the
// proxy-wasm environment. It is gated by `#[cfg(target_arch = "wasm32")]`
// and is not included in the host unit tests.
//

#[cfg(target_arch = "wasm32")]
mod wasm_glue {
    use super::{Config, Router}; // Import the logic from the parent module.
    use log::{info, warn};
    use proxy_wasm::traits::*;
    use proxy_wasm::types::*;

    // --- Root Context ---
    #[derive(Default)]
    #[allow(dead_code)] // This struct is only used within the wasm_glue module.
    struct PluginRootContext {
        config: Config,
    }

    impl Context for PluginRootContext {}

    impl RootContext for PluginRootContext {
        fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
            if let Some(config_bytes) = self.get_plugin_configuration() {
                self.config = Config::from_bytes(&config_bytes);
                info!(
                    "[TCP WASM] New configuration: is_istio={}",
                    self.config.is_istio
                );
            } else {
                warn!("[TCP WASM] No configuration found");
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

    // --- Stream Context ---
    #[allow(dead_code)] // This struct is only used within the wasm_glue module.
    struct SourceBasedRouter {
        router: Router,
    }

    impl Context for SourceBasedRouter {}

    impl StreamContext for SourceBasedRouter {
        fn on_new_connection(&mut self) -> Action {
            if let Some(source_addr_bytes) = self.get_property(vec!["source", "address"]) {
                if let Ok(source_addr) = std::str::from_utf8(&source_addr_bytes) {
                    info!("[TCP WASM] Source address: {}", source_addr);
                    if let Some(cluster) = self.router.decide_route_cluster(source_addr) {
                        info!("[TCP WASM] Routing to {}", &cluster);

                        // Set dynamic metadata for rerouting using the proper Envoy mechanism
                        use crate::set_envoy_filter_state::{
                            LifeSpan, SetEnvoyFilterStateArguments,
                        };

                        let args = SetEnvoyFilterStateArguments {
                            path: "envoy.tcp_proxy.cluster".to_string(),
                            value: cluster.clone(),
                            span: LifeSpan::FilterChain as i32,
                        };
                        let mut buf = Vec::new();
                        prost::Message::encode(&args, &mut buf).unwrap();

                        // Use the Envoy-specific filter state mechanism
                        // https://github.com/envoyproxy/envoy/issues/28128
                        let status =
                            self.call_foreign_function("set_envoy_filter_state", Some(&buf));
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

    // --- Wasm Entry Point ---
    #[no_mangle]
    pub fn _start() {
        proxy_wasm::set_log_level(LogLevel::Info);
        proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
            Box::new(PluginRootContext::default())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_last_octet, Config, Router};

    #[test]
    fn test_config_parsing() {
        let config = Config::from_bytes(b"istio");
        assert!(config.is_istio);

        let config = Config::from_bytes(b"standalone");
        assert!(!config.is_istio);

        let config = Config::from_bytes(b"");
        assert!(!config.is_istio);
    }

    #[test]
    fn test_cluster_name_generation() {
        let istio_router = Router::new(Config { is_istio: true });
        let (c1, c2) = istio_router.get_cluster_names();
        assert_eq!(
            c1,
            "outbound|8080||egress-router1.default.svc.cluster.local"
        );
        assert_eq!(
            c2,
            "outbound|8080||egress-router2.default.svc.cluster.local"
        );

        let standalone_router = Router::new(Config { is_istio: false });
        let (c1, c2) = standalone_router.get_cluster_names();
        assert_eq!(c1, "egress-router1");
        assert_eq!(c2, "egress-router2");
    }

    #[test]
    fn test_routing_decision_logic() {
        let router = Router::new(Config { is_istio: false });
        let (cluster1, cluster2) = router.get_cluster_names();

        // Even octet -> cluster1
        assert_eq!(
            router.decide_route_cluster("10.0.0.2:12345"),
            Some(cluster1.clone())
        );
        // Odd octet -> cluster2
        assert_eq!(
            router.decide_route_cluster("192.168.1.1:54321"),
            Some(cluster2.clone())
        );
        // IPv6 even octet
        assert_eq!(
            router.decide_route_cluster("[::ffff:192.168.1.100]:8080"),
            Some(cluster1.clone())
        );
        // IPv6 odd octet
        assert_eq!(
            router.decide_route_cluster("[::ffff:192.168.1.101]:8080"),
            Some(cluster2.clone())
        );
    }

    #[test]
    fn test_extract_last_octet() {
        assert_eq!(extract_last_octet("1.2.3.4:5678"), Some(4));
        assert_eq!(extract_last_octet("10.244.0.94:33198"), Some(94));
        assert_eq!(extract_last_octet("127.0.0.1"), Some(1));
        assert_eq!(extract_last_octet("[::ffff:192.168.1.100]:8080"), Some(100));
        assert_eq!(extract_last_octet("not-an-ip"), None);
        assert_eq!(extract_last_octet("1.2.3.256"), None);
        assert_eq!(extract_last_octet(""), None);
    }
}
