//! Unit tests for configuration module
//!
//! These tests verify the individual components of the configuration system.

use std::env;
use std::fs;
use hyperliquid_core::config::{Config, EnvironmentConfig, HttpClientConfig, WebSocketConfig, RuntimeConfig, LoggingConfig, SecurityConfig, MetricsConfig};

#[test]
fn test_environment_config_default() {
    let config = EnvironmentConfig::default();
    assert_eq!(config.env, hyperliquid_core::types::Environment::Mainnet);
    assert_eq!(config.base_url, None);
    assert_eq!(config.websocket_url, None);
}

#[test]
fn test_http_client_config_default() {
    let config = HttpClientConfig::default();
    assert_eq!(config.request_timeout_ms, 30000);
    assert_eq!(config.max_connections_per_host, 10);
    assert_eq!(config.max_total_connections, 100);
    assert_eq!(config.connect_timeout_ms, 10000);
    assert_eq!(config.enable_cert_pinning, false);
    assert_eq!(config.enable_rate_limiting, true);
}

#[test]
fn test_websocket_config_default() {
    let config = WebSocketConfig::default();
    assert_eq!(config.connect_timeout_ms, 10000);
    assert_eq!(config.reconnect_delay_ms, 1000);
    assert_eq!(config.max_reconnect_attempts, 5);
    assert_eq!(config.buffer_size, 1000);
    assert_eq!(config.enable_compression, false);
    assert_eq!(config.ping_interval_ms, 30000);
}

#[test]
fn test_runtime_config_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.worker_threads, num_cpus::get());
    assert_eq!(config.max_blocking_threads, 512);
    assert_eq!(config.thread_stack_size, 2 * 1024 * 1024);
    assert_eq!(config.enable_io, true);
    assert_eq!(config.enable_time, true);
    assert_eq!(config.global_queue_interval, 61);
    assert_eq!(config.shutdown_timeout_secs, 30);
}

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();
    assert_eq!(config.level, "info");
    assert_eq!(config.format, "json");
    assert_eq!(config.file_path, None);
    assert_eq!(config.max_log_size_mb, 100);
    assert_eq!(config.max_log_files, 5);
    assert_eq!(config.colored_output, true);
}

#[test]
fn test_security_config_default() {
    let config = SecurityConfig::default();
    assert_eq!(config.enable_cert_pinning, false);
    assert_eq!(config.key_rotation_hours, 24);
    assert_eq!(config.enable_request_signing, true);
    assert_eq!(config.strict_mode, false);
}

#[test]
fn test_certificate_pinning_config_default() {
    let config = hyperliquid_core::config::CertificatePinningConfig::default();
    assert_eq!(config.domain, "");
    assert_eq!(config.pins, Vec::<String>::new());
    assert_eq!(config.enable_backup_pins, false);
}

#[test]
fn test_metrics_config_default() {
    let config = MetricsConfig::default();
    assert_eq!(config.enabled, true);
    assert_eq!(config.endpoint, "/metrics");
    assert_eq!(config.collection_interval_secs, 10);
    assert_eq!(config.enable_prometheus, true);
    assert_eq!(config.namespace, "hyperliquid");
}

#[test]
fn test_env_override_parsing() {
    // Test valid environment values
    let mut config = Config::default();

    // Test valid environment string
    env::set_var("HYPERLIQUID_ENV", "testnet");
    config.apply_env_overrides().unwrap();
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Testnet);

    // Test valid HTTP timeout
    env::set_var("HYPERLIQUID_HTTP_TIMEOUT", "60000");
    config.apply_env_overrides().unwrap();
    assert_eq!(config.http.request_timeout_ms, 60000);

    // Test valid max connections
    env::set_var("HYPERLIQUID_MAX_CONNECTIONS", "20");
    config.apply_env_overrides().unwrap();
    assert_eq!(config.http.max_connections_per_host, 20);

    // Test valid worker threads
    env::set_var("HYPERLIQUID_WORKER_THREADS", "8");
    config.apply_env_overrides().unwrap();
    assert_eq!(config.runtime.worker_threads, 8);

    // Test valid log level
    env::set_var("HYPERLIQUID_LOG_LEVEL", "debug");
    config.apply_env_overrides().unwrap();
    assert_eq!(config.logging.level, "debug");

    // Clean up
    env::remove_var("HYPERLIQUID_ENV");
    env::remove_var("HYPERLIQUID_HTTP_TIMEOUT");
    env::remove_var("HYPERLIQUID_MAX_CONNECTIONS");
    env::remove_var("HYPERLIQUID_WORKER_THREADS");
    env::remove_var("HYPERLIQUID_LOG_LEVEL");
}

#[test]
fn test_env_override_invalid_values() {
    let mut config = Config::default();

    // Test invalid HTTP timeout
    env::set_var("HYPERLIQUID_HTTP_TIMEOUT", "invalid");
    let result = config.apply_env_overrides();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid HYPERLIQUID_HTTP_TIMEOUT"));

    // Test invalid max connections
    env::set_var("HYPERLIQUID_MAX_CONNECTIONS", "invalid");
    config.apply_env_overrides().unwrap(); // Reset
    let result = config.apply_env_overrides();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid HYPERLIQUID_MAX_CONNECTIONS"));

    // Test invalid worker threads
    env::set_var("HYPERLIQUID_WORKER_THREADS", "invalid");
    config.apply_env_overrides().unwrap(); // Reset
    let result = config.apply_env_overrides();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid HYPERLIQUID_WORKER_THREADS"));

    // Clean up
    env::remove_var("HYPERLIQUID_HTTP_TIMEOUT");
    env::remove_var("HYPERLIQUID_MAX_CONNECTIONS");
    env::remove_var("HYPERLIQUID_WORKER_THREADS");
}

#[test]
fn test_config_getters() {
    let config = Config::default();

    // Test default URLs
    assert_eq!(config.get_base_url(), "https://api.hyperliquid.xyz");
    assert_eq!(config.get_websocket_url(), "wss://api.hyperliquid.xyz/ws");
    assert_eq!(config.get_environment(), hyperliquid_core::types::Environment::Mainnet);

    // Test with overridden URLs
    let mut config = Config::default();
    config.environment.base_url = Some("https://custom.hyperliquid.xyz".to_string());
    config.environment.websocket_url = Some("wss://custom.hyperliquid.xyz/ws".to_string());

    assert_eq!(config.get_base_url(), "https://custom.hyperliquid.xyz");
    assert_eq!(config.get_websocket_url(), "wss://custom.hyperliquid.xyz/ws");
}

#[test]
fn test_config_validation_edge_cases() {
    let mut config = Config::default();

    // Test minimum valid values
    config.http.request_timeout_ms = 1000; // Minimum
    config.http.max_connections_per_host = 1; // Minimum
    config.runtime.worker_threads = 1; // Minimum
    config.logging.level = "error".to_string(); // Valid level

    assert!(config.validate().is_ok());

    // Test values below minimum
    config.http.request_timeout_ms = 999;
    assert!(config.validate().is_err());

    config.http.request_timeout_ms = 1000;
    config.http.max_connections_per_host = 0;
    assert!(config.validate().is_err());

    config.http.max_connections_per_host = 1;
    config.runtime.worker_threads = 0;
    assert!(config.validate().is_err());

    config.runtime.worker_threads = 1;
    config.logging.level = "invalid".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn test_config_file_not_found() {
    let result = Config::load("nonexistent.toml");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read config file"));
}

#[test]
fn test_config_parsing_errors() {
    // Test invalid TOML syntax
    let config_content = r#"
        [environment]
        env = "mainnet"
        invalid_syntax
    "#;

    fs::write("invalid_syntax.toml", config_content).unwrap();

    let result = Config::load("invalid_syntax.toml");
    assert!(result.is_err());

    fs::remove_file("invalid_syntax.toml").unwrap();
}

#[test]
fn test_config_auto_load_fallback() {
    // Clear any existing config files
    let _ = fs::remove_file("config/default.toml");
    let _ = fs::remove_file("config/testnet.toml");
    let _ = fs::remove_file("config/local.toml");

    // Should return default config when no files exist
    let config = Config::load_auto().unwrap();
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Mainnet);
}