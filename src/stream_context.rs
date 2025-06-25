use log::{info, warn};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

struct PluginRootContext {
    is_istio: bool,
}

impl PluginRootContext {
    fn new() -> Self {
        Self {
            is_istio: false, // Default to standalone Envoy
        }
    }
}

impl Context for PluginRootContext {}

impl RootContext for PluginRootContext {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::StreamContext)
    }

    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            if let Ok(config_str) = String::from_utf8(config_bytes) {
                let config_str = config_str.trim();
                info!("[TCP WASM] Configuration: '{}'", config_str);

                // Simple string-based configuration
                self.is_istio = config_str == "istio" || config_str == "kubernetes";

                info!(
                    "[TCP WASM] Environment: {}",
                    if self.is_istio {
                        "Istio/Kubernetes"
                    } else {
                        "Standalone Envoy"
                    }
                );
            } else {
                warn!("[TCP WASM] Failed to parse configuration as UTF-8");
            }
        } else {
            info!("[TCP WASM] No configuration provided, using default (standalone Envoy)");
        }
        true
    }

    fn create_stream_context(&self, context_id: u32) -> Option<Box<dyn StreamContext>> {
        Some(Box::new(SourceBasedRouter {
            context_id,
            is_istio: self.is_istio,
        }))
    }
}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(PluginRootContext::new())
    });
}

struct SourceBasedRouter {
    context_id: u32,
    is_istio: bool,
}

impl SourceBasedRouter {
    /// Returns the appropriate cluster names based on the environment configuration
    fn get_cluster_names(&self) -> (String, String) {
        if self.is_istio {
            // Istio/Kubernetes environment - use full cluster names with port 8080
            info!("[TCP WASM] Using Istio cluster names with port 8080");
            (
                "outbound|8080||egress-router1.default.svc.cluster.local".to_string(),
                "outbound|8080||egress-router2.default.svc.cluster.local".to_string(),
            )
        } else {
            // Standalone Envoy - use simple names
            info!("[TCP WASM] Using standalone Envoy cluster names");
            ("egress1".to_string(), "egress2".to_string())
        }
    }
}

impl Context for SourceBasedRouter {}

impl StreamContext for SourceBasedRouter {
    // See https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259
    fn on_new_connection(&mut self) -> Action {
        info!(
            "[TCP WASM] New connection established (context #{})",
            self.context_id
        );
        // Log destination address
        // Property names based on the Proxy-Wasm ABI version 0.2.1
        // https://github.com/proxy-wasm/spec/tree/main/abi-versions/v0.2.1#downstream-connection-properties
        if let Some(val) = self.get_property(vec!["destination", "address"]) {
            if let Ok(s) = String::from_utf8(val) {
                info!("[TCP WASM] Destination address: {}", s);
            } else {
                info!("[TCP WASM] Destination address: (non-UTF8)");
            }
        } else {
            info!("[TCP WASM] Destination address not found");
        }
        // Log source address and set reroute metadata
        let mut reroute_cluster: Option<String> = None;

        // Remove destination port logic since we always forward to port 8080

        if let Some(val) = self.get_property(vec!["source", "address"]) {
            if let Ok(s) = String::from_utf8(val) {
                info!("[TCP WASM] Source address: {}", s);

                // Parse last octet for routing decision - intercept ALL traffic
                if let Some(ip_part) = s.split(':').next() {
                    if let Some(last_octet) = ip_part.split('.').next_back() {
                        if let Ok(num) = last_octet.parse::<u8>() {
                            info!(
                                "[TCP WASM] Source IP last octet: {}, intercepting ALL traffic",
                                num
                            );

                            // Determine cluster name based on environment (always port 8080)
                            let (egress1_cluster, egress2_cluster) = self.get_cluster_names();

                            if num % 2 == 0 {
                                // Even last octet, reroute to egress-router1
                                reroute_cluster = Some(egress1_cluster);
                                info!("[TCP WASM] Routing to egress-router1");
                            } else {
                                // Odd last octet, reroute to egress-router2
                                reroute_cluster = Some(egress2_cluster);
                                info!("[TCP WASM] Routing to egress-router2");
                            }
                        }
                    }
                }
            } else {
                info!("[TCP WASM] Source address: (non-UTF8)");
            }
        } else {
            info!("[TCP WASM] Source address not found");
        }
        // Set dynamic metadata for rerouting if needed
        if let Some(cluster) = reroute_cluster {
            // Use prost-generated protobuf struct for filter state
            use prost::Message;
            use set_envoy_filter_state::{LifeSpan, SetEnvoyFilterStateArguments};

            let args = SetEnvoyFilterStateArguments {
                path: "envoy.tcp_proxy.cluster".to_string(),
                value: cluster.clone(),
                span: LifeSpan::FilterChain as i32, // or LifeSpan::DownstreamConnection if preferred
            };
            let mut buf = Vec::new();
            args.encode(&mut buf).unwrap();
            // se background to set_envoy_filter_state
            // https://github.com/envoyproxy/envoy/issues/28128
            // https://github.com/envoyproxy/envoy/issues/28128
            let status = self.call_foreign_function("set_envoy_filter_state", Some(&buf));
            info!(
                "[TCP WASM] set_envoy_filter_state status (envoy.tcp_proxy.cluster): {:?}",
                status
            );
            info!("[TCP WASM] Rerouting to {} via filter state", cluster);
        }
        Action::Continue
    }
}
pub mod set_envoy_filter_state {
    include!("generated/envoy.source.extensions.common.wasm.rs");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_root_context_creation() {
        let context = PluginRootContext::new();
        assert!(!context.is_istio); // Default should be false (standalone Envoy)
    }

    #[test]
    fn test_source_based_router_creation() {
        let router = SourceBasedRouter {
            context_id: 42,
            is_istio: false,
        };
        assert_eq!(router.context_id, 42);
        assert!(!router.is_istio);
    }

    #[test]
    fn test_get_cluster_names_standalone_envoy() {
        let router = SourceBasedRouter {
            context_id: 1,
            is_istio: false,
        };
        let (egress1, egress2) = router.get_cluster_names();
        assert_eq!(egress1, "egress1");
        assert_eq!(egress2, "egress2");
    }

    #[test]
    fn test_get_cluster_names_istio() {
        let router = SourceBasedRouter {
            context_id: 1,
            is_istio: true,
        };
        let (egress1, egress2) = router.get_cluster_names();
        assert_eq!(
            egress1,
            "outbound|8080||egress-router1.default.svc.cluster.local"
        );
        assert_eq!(
            egress2,
            "outbound|8080||egress-router2.default.svc.cluster.local"
        );
    }

    #[test]
    fn test_plugin_root_context_get_type() {
        let context = PluginRootContext::new();
        assert_eq!(context.get_type(), Some(ContextType::StreamContext));
    }

    #[test]
    fn test_router_naming_makes_sense() {
        // This test documents why we renamed from DestIpLogger to SourceBasedRouter
        let router = SourceBasedRouter {
            context_id: 1,
            is_istio: false,
        };

        // The router examines SOURCE IP, not destination
        // It routes traffic based on the source IP's last octet
        // It's not just logging - it's actively routing traffic

        // Test the routing logic conceptually (even though we can't easily test
        // the full on_new_connection without mocking the proxy-wasm context)
        let (egress1, egress2) = router.get_cluster_names();

        // For even last octets -> egress1
        // For odd last octets -> egress2
        assert_ne!(egress1, egress2);
        assert!(!egress1.is_empty());
        assert!(!egress2.is_empty());
    }

    #[test]
    fn test_ip_octet_routing_logic() {
        // Test the IP parsing and routing logic that would be used in on_new_connection

        // Simulate the routing decision based on last octet
        fn determine_cluster_for_ip(ip: &str, is_istio: bool) -> Option<String> {
            if let Some(ip_part) = ip.split(':').next() {
                if let Some(last_octet) = ip_part.split('.').next_back() {
                    if let Ok(num) = last_octet.parse::<u8>() {
                        let router = SourceBasedRouter {
                            context_id: 1,
                            is_istio,
                        };
                        let (egress1, egress2) = router.get_cluster_names();

                        if num % 2 == 0 {
                            return Some(egress1);
                        } else {
                            return Some(egress2);
                        }
                    }
                }
            }
            None
        }

        // Test even last octets go to egress1
        assert_eq!(
            determine_cluster_for_ip("192.168.1.2:12345", false),
            Some("egress1".to_string())
        );
        assert_eq!(
            determine_cluster_for_ip("10.0.0.4", false),
            Some("egress1".to_string())
        );
        assert_eq!(
            determine_cluster_for_ip("172.16.1.100:8080", false),
            Some("egress1".to_string())
        );

        // Test odd last octets go to egress2
        assert_eq!(
            determine_cluster_for_ip("192.168.1.3:12345", false),
            Some("egress2".to_string())
        );
        assert_eq!(
            determine_cluster_for_ip("10.0.0.5", false),
            Some("egress2".to_string())
        );
        assert_eq!(
            determine_cluster_for_ip("172.16.1.101:8080", false),
            Some("egress2".to_string())
        );

        // Test with Istio cluster names
        assert_eq!(
            determine_cluster_for_ip("192.168.1.2", true),
            Some("outbound|8080||egress-router1.default.svc.cluster.local".to_string())
        );
        assert_eq!(
            determine_cluster_for_ip("192.168.1.3", true),
            Some("outbound|8080||egress-router2.default.svc.cluster.local".to_string())
        );

        // Test invalid IPs return None
        assert_eq!(determine_cluster_for_ip("invalid.ip", false), None);
        assert_eq!(determine_cluster_for_ip("192.168.1.abc", false), None);
    }

    #[test]
    fn test_edge_cases_for_ip_parsing() {
        // Test edge cases in IP parsing logic

        fn extract_last_octet(ip: &str) -> Option<u8> {
            if let Some(ip_part) = ip.split(':').next() {
                if let Some(last_octet) = ip_part.split('.').next_back() {
                    last_octet.parse::<u8>().ok()
                } else {
                    None
                }
            } else {
                None
            }
        }

        // Valid cases
        assert_eq!(extract_last_octet("192.168.1.1"), Some(1));
        assert_eq!(extract_last_octet("10.0.0.255"), Some(255));
        assert_eq!(extract_last_octet("172.16.1.0"), Some(0));
        assert_eq!(extract_last_octet("192.168.1.42:8080"), Some(42));

        // Edge cases
        assert_eq!(extract_last_octet("192.168.1.256"), None); // > 255
        assert_eq!(extract_last_octet("192.168.1."), None); // empty last octet
        assert_eq!(extract_last_octet("192.168.1"), Some(1)); // no port, valid
        assert_eq!(extract_last_octet("not.an.ip.address"), None); // invalid format
        assert_eq!(extract_last_octet(""), None); // empty string
    }
}
