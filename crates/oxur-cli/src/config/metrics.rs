//! Metrics configuration for REPL observability
//!
//! Controls whether metrics are collected and where they are exported.

use serde::{Deserialize, Serialize};

/// Default TCP exporter address
pub const DEFAULT_METRICS_ADDR: &str = "127.0.0.1:9100";

/// Metrics configuration
///
/// Controls metrics collection and export settings for the REPL.
///
/// # Example TOML
///
/// ```toml
/// [metrics]
/// enabled = true
/// exporter_address = "127.0.0.1:9100"
/// ```
///
/// # Environment Variables
///
/// - `OXUR_METRICS_ENABLED` - Enable metrics (default: false)
/// - `OXUR_METRICS_ADDRESS` - TCP exporter address
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetricsConfig {
    /// Whether metrics collection is enabled
    ///
    /// When false, metrics calls are no-ops (zero overhead).
    /// Default: false
    pub enabled: bool,

    /// TCP exporter address for metrics streaming
    ///
    /// Format: "host:port"
    /// Default: "127.0.0.1:9100"
    pub exporter_address: String,

    /// Enable client-side metrics (request counts, latency)
    ///
    /// Only applies when `enabled` is true.
    /// Default: true
    pub client_metrics: bool,

    /// Enable server-side metrics (connections, sessions)
    ///
    /// Only applies when `enabled` is true.
    /// Default: true
    pub server_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            exporter_address: DEFAULT_METRICS_ADDR.to_string(),
            client_metrics: true,
            server_metrics: true,
        }
    }
}

impl MetricsConfig {
    /// Create a new builder
    pub fn builder() -> MetricsConfigBuilder {
        MetricsConfigBuilder::new()
    }

    /// Merge another config into this one
    ///
    /// Only merges explicitly set values from `other`.
    pub fn merge(&mut self, other: MetricsConfig) {
        // For metrics, we take the other's values if they differ from defaults
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.exporter_address != DEFAULT_METRICS_ADDR {
            self.exporter_address = other.exporter_address;
        }
        // Always take explicit settings for granular control
        self.client_metrics = other.client_metrics;
        self.server_metrics = other.server_metrics;
    }
}

/// Builder for MetricsConfig
#[derive(Debug, Clone, Default)]
pub struct MetricsConfigBuilder {
    config: MetricsConfig,
}

impl MetricsConfigBuilder {
    /// Create a new builder with defaults
    pub fn new() -> Self {
        Self { config: MetricsConfig::default() }
    }

    /// Enable or disable metrics
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set the TCP exporter address
    pub fn exporter_address(mut self, addr: impl Into<String>) -> Self {
        self.config.exporter_address = addr.into();
        self
    }

    /// Enable or disable client-side metrics
    pub fn client_metrics(mut self, enabled: bool) -> Self {
        self.config.client_metrics = enabled;
        self
    }

    /// Enable or disable server-side metrics
    pub fn server_metrics(mut self, enabled: bool) -> Self {
        self.config.server_metrics = enabled;
        self
    }

    /// Build the MetricsConfig
    pub fn build(self) -> MetricsConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.exporter_address, "127.0.0.1:9100");
        assert!(config.client_metrics);
        assert!(config.server_metrics);
    }

    #[test]
    fn test_metrics_config_builder() {
        let config = MetricsConfig::builder()
            .enabled(true)
            .exporter_address("0.0.0.0:9200")
            .client_metrics(true)
            .server_metrics(false)
            .build();

        assert!(config.enabled);
        assert_eq!(config.exporter_address, "0.0.0.0:9200");
        assert!(config.client_metrics);
        assert!(!config.server_metrics);
    }

    #[test]
    fn test_serde_roundtrip() {
        let config =
            MetricsConfig::builder().enabled(true).exporter_address("localhost:9100").build();

        let toml = toml::to_string(&config).unwrap();
        let parsed: MetricsConfig = toml::from_str(&toml).unwrap();

        assert_eq!(config.enabled, parsed.enabled);
        assert_eq!(config.exporter_address, parsed.exporter_address);
    }
}
