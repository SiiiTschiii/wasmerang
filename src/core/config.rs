//! Configuration parsing for the Wasmerang filter

/// Represents the parsed configuration for the filter.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Config {
    pub is_istio: bool,
}

impl Config {
    /// Parses the configuration from a byte slice.
    /// 
    /// The configuration is expected to be a UTF-8 string.
    /// If the string contains "istio", the filter will use Istio-style cluster names.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use wasmerang::Config;
    /// 
    /// let config = Config::from_bytes(b"istio");
    /// assert!(config.is_istio);
    /// 
    /// let config = Config::from_bytes(b"standalone");
    /// assert!(!config.is_istio);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let config_str = std::str::from_utf8(bytes).unwrap_or("");
        Config {
            is_istio: config_str.contains("istio"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config = Config::from_bytes(b"istio");
        assert!(config.is_istio);

        let config = Config::from_bytes(b"standalone");
        assert!(!config.is_istio);

        let config = Config::from_bytes(b"");
        assert!(!config.is_istio);
    }
}
