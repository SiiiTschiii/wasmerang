//! Core routing logic for the Wasmerang filter

use super::{config::Config, utils::extract_last_octet};

/// The core routing logic.
/// 
/// This struct contains the main business logic for determining how to route
/// traffic based on source IP addresses and configuration.
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
    /// 
    /// # Returns
    /// 
    /// A tuple of (cluster1, cluster2) where:
    /// - For Istio: Full Kubernetes service names with ports
    /// - For standalone Envoy: Simple cluster names
    /// 
    /// # Examples
    /// 
    /// ```
    /// use wasmerang::{Config, Router};
    /// 
    /// let istio_router = Router::new(Config { is_istio: true });
    /// let (c1, c2) = istio_router.get_cluster_names();
    /// assert_eq!(c1, "outbound|8080||egress-router1.default.svc.cluster.local");
    /// 
    /// let standalone_router = Router::new(Config { is_istio: false });
    /// let (c1, c2) = standalone_router.get_cluster_names();
    /// assert_eq!(c1, "egress-router1");
    /// ```
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
    /// 
    /// The routing decision is based on the last octet of the source IP address:
    /// - Even last octet → cluster1
    /// - Odd last octet → cluster2
    /// 
    /// # Arguments
    /// 
    /// * `source_address` - The source IP address, optionally with port (e.g., "192.168.1.10:8080")
    /// 
    /// # Returns
    /// 
    /// * `Some(cluster_name)` if the IP address can be parsed and routed
    /// * `None` if the IP address is invalid or cannot be parsed
    /// 
    /// # Examples
    /// 
    /// ```
    /// use wasmerang::{Config, Router};
    /// 
    /// let router = Router::new(Config { is_istio: false });
    /// 
    /// // Even last octet (2) → first cluster
    /// assert_eq!(
    ///     router.decide_route_cluster("10.0.0.2:12345"),
    ///     Some("egress-router1".to_string())
    /// );
    /// 
    /// // Odd last octet (1) → second cluster
    /// assert_eq!(
    ///     router.decide_route_cluster("192.168.1.1:54321"),
    ///     Some("egress-router2".to_string())
    /// );
    /// 
    /// // Invalid IP → None
    /// assert_eq!(router.decide_route_cluster("invalid"), None);
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
