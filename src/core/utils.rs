//! Utility functions for IP address parsing

/// Extracts the last octet from an IP address string (v4 or v6).
///
/// This function handles various IP address formats:
/// - IPv4 with port: "192.168.1.10:8080" → Some(10)
/// - IPv4 without port: "192.168.1.10" → Some(10)
/// - IPv6 with port: "[::ffff:192.168.1.100]:8080" → Some(100)
/// - IPv4-mapped IPv6: "::ffff:192.168.1.100" → Some(100)
///
/// # Arguments
///
/// * `ip_address` - The IP address string to parse
///
/// # Returns
///
/// * `Some(octet)` if the last octet can be successfully parsed as a u8
/// * `None` if parsing fails for any reason
///
/// # Examples
///
/// ```
/// use wasmstreamcontext::core::extract_last_octet;
///
/// assert_eq!(extract_last_octet("1.2.3.4:5678"), Some(4));
/// assert_eq!(extract_last_octet("10.244.0.94:33198"), Some(94));
/// assert_eq!(extract_last_octet("127.0.0.1"), Some(1));
/// assert_eq!(extract_last_octet("[::ffff:192.168.1.100]:8080"), Some(100));
/// assert_eq!(extract_last_octet("not-an-ip"), None);
/// assert_eq!(extract_last_octet("1.2.3.256"), None);
/// assert_eq!(extract_last_octet(""), None);
/// ```
pub fn extract_last_octet(ip_address: &str) -> Option<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
