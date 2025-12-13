//! Integration tests for configuration loading
//!
//! These tests verify that the configuration system works correctly
//! with TOML files, environment variables, and validation.

use std::fs;
use std::env;
use hyperliquid_core::{Config, EnvironmentConfig, HttpClientConfig, WebSocketConfig, RuntimeConfig, LoggingConfig, SecurityConfig, MetricsConfig};

#[test]
fn test_config_default() {
    let config = Config::default();

    // Test default environment
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Mainnet);

    // Test default HTTP settings
    assert_eq!(config.http.request_timeout_ms, 30000);
    assert_eq!(config.http.max_connections_per_host, 10);

    // Test default runtime settings
    assert_eq!(config.runtime.worker_threads, num_cpus::get());

    // Test default logging
    assert_eq!(config.logging.level, "info");
    assert_eq!(config.logging.format, "json");
}

#[test]
fn test_config_mainnet() {
    let config = Config::mainnet();
    assert_eq!(config.get_base_url(), "https://api.hyperliquid.xyz");
    assert_eq!(config.get_websocket_url(), "wss://api.hyperliquid.xyz/ws");
    assert_eq!(config.get_environment(), hyperliquid_core::types::Environment::Mainnet);
}

#[test]
fn test_config_testnet() {
    let config = Config::testnet();
    assert_eq!(config.get_base_url(), "https://api.hyperliquid-testnet.xyz");
    assert_eq!(config.get_websocket_url(), "wss://api.hyperliquid-testnet.xyz/ws");
    assert_eq!(config.get_environment(), hyperliquid_core::types::Environment::Testnet);
}

#[test]
fn test_config_local() {
    let config = Config::local();
    assert_eq!(config.get_base_url(), "http://localhost:3001");
    assert_eq!(config.get_websocket_url(), "ws://localhost:3001/ws");
    assert_eq!(config.get_environment(), hyperliquid_core::types::Environment::Local);
}

#[test]
fn test_config_file_load() {
    // Create a test config file
    let config_content = r#"
        [environment]
        env = "testnet"
        base_url = "https://api.test.hyperliquid.xyz"
        websocket_url = "wss://api.test.hyperliquid.xyz/ws"

        [http]
        request_timeout_ms = 60000
        max_connections_per_host = 20
        connect_timeout_ms = 5000

        [runtime]
        worker_threads = 4
        max_blocking_threads = 256

        [logging]
        level = "debug"
        format = "pretty"
        colored_output = false

        [security]
        enable_cert_pinning = true
        key_rotation_hours = 12
        strict_mode = true

        [metrics]
        enabled = false
        endpoint = "/debug/metrics"
        collection_interval_secs = 5
    "#;

    fs::write("test_config.toml", config_content).unwrap();

    let config = Config::load("test_config.toml").unwrap();

    // Test environment settings
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Testnet);
    assert_eq!(config.environment.base_url, Some("https://api.test.hyperliquid.xyz".to_string()));
    assert_eq!(config.environment.websocket_url, Some("wss://api.test.hyperliquid.xyz/ws".to_string()));

    // Test HTTP settings
    assert_eq!(config.http.request_timeout_ms, 60000);
    assert_eq!(config.http.max_connections_per_host, 20);
    assert_eq!(config.http.connect_timeout_ms, 5000);

    // Test runtime settings
    assert_eq!(config.runtime.worker_threads, 4);
    assert_eq!(config.runtime.max_blocking_threads, 256);

    // Test logging settings
    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.logging.format, "pretty");
    assert_eq!(config.logging.colored_output, false);

    // Test security settings
    assert_eq!(config.security.enable_cert_pinning, true);
    assert_eq!(config.security.key_rotation_hours, 12);
    assert_eq!(config.security.strict_mode, true);

    // Test metrics settings
    assert_eq!(config.metrics.enabled, false);
    assert_eq!(config.metrics.endpoint, "/debug/metrics");
    assert_eq!(config.metrics.collection_interval_secs, 5);

    // Test URL getters
    assert_eq!(config.get_base_url(), "https://api.test.hyperliquid.xyz");
    assert_eq!(config.get_websocket_url(), "wss://api.test.hyperliquid.xyz/ws");

    // Clean up
    fs::remove_file("test_config.toml").unwrap();
}

#[test]
fn test_config_file_load_auto() {
    // Test loading with environment variable
    env::set_var("HYPERLIQUID_CONFIG", "config/default.toml");

    let config = Config::load_auto().unwrap();
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Mainnet);

    // Clean up
    env::remove_var("HYPERLIQUID_CONFIG");
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();

    // Test valid config
    assert!(config.validate().is_ok());

    // Test invalid HTTP timeout
    config.http.request_timeout_ms = 500;
    assert!(config.validate().is_err());

    // Fix timeout and test invalid connection pool
    config.http.request_timeout_ms = 5000;
    config.http.max_connections_per_host = 0;
    assert!(config.validate().is_err());

    // Fix connection pool and test invalid worker threads
    config.http.max_connections_per_host = 10;
    config.runtime.worker_threads = 0;
    assert!(config.validate().is_err());

    // Fix worker threads and test invalid log level
    config.runtime.worker_threads = 1;
    config.logging.level = "invalid".to_string();
    assert!(config.validate().is_err());

    // Fix log level and test valid again
    config.logging.level = "info".to_string();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_env_overrides() {
    // Set environment variables
    env::set_var("HYPERLIQUID_ENV", "testnet");
    env::set_var("HYPERLIQUID_HTTP_TIMEOUT", "60000");
    env::set_var("HYPERLIQUID_MAX_CONNECTIONS", "20");
    env::set_var("HYPERLIQUID_WORKER_THREADS", "8");
    env::set_var("HYPERLIQUID_LOG_LEVEL", "debug");
    env::set_var("HYPERLIQUID_BASE_URL", "https://custom.hyperliquid.xyz");
    env::set_var("HYPERLIQUID_WS_URL", "wss://custom.hyperliquid.xyz/ws");

    let mut config = Config::default();
    config.apply_env_overrides().unwrap();

    // Test environment override
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Testnet);
    assert_eq!(config.environment.base_url, Some("https://custom.hyperliquid.xyz".to_string()));
    assert_eq!(config.environment.websocket_url, Some("wss://custom.hyperliquid.xyz/ws".to_string()));

    // Test HTTP overrides
    assert_eq!(config.http.request_timeout_ms, 60000);
    assert_eq!(config.http.max_connections_per_host, 20);

    // Test runtime overrides
    assert_eq!(config.runtime.worker_threads, 8);

    // Test logging overrides
    assert_eq!(config.logging.level, "debug");

    // Clean up
    env::remove_var("HYPERLIQUID_ENV");
    env::remove_var("HYPERLIQUID_HTTP_TIMEOUT");
    env::remove_var("HYPERLIQUID_MAX_CONNECTIONS");
    env::remove_var("HYPERLIQUID_WORKER_THREADS");
    env::remove_var("HYPERLIQUID_LOG_LEVEL");
    env::remove_var("HYPERLIQUID_BASE_URL");
    env::remove_var("HYPERLIQUID_WS_URL");
}

#[test]
fn test_config_url_override_precedence() {
    // Test that environment URL overrides take precedence over env enum
    let config_content = r#"
        [environment]
        env = "mainnet"
        base_url = "https://api.hyperliquid.xyz"
        websocket_url = "wss://api.hyperliquid.xyz/ws"
    "#;

    fs::write("test_config.toml", config_content).unwrap();

    // Set environment override
    env::set_var("HYPERLIQUID_BASE_URL", "https://override.hyperliquid.xyz");
    env::set_var("HYPERLIQUID_WS_URL", "wss://override.hyperliquid.xyz/ws");

    let config = Config::load("test_config.toml").unwrap();

    // Environment should be mainnet but URLs should be overridden
    assert_eq!(config.get_environment(), hyperliquid_core::types::Environment::Mainnet);
    assert_eq!(config.get_base_url(), "https://override.hyperliquid.xyz");
    assert_eq!(config.get_websocket_url(), "wss://override.hyperliquid.xyz/ws");

    // Clean up
    env::remove_var("HYPERLIQUID_BASE_URL");
    env::remove_var("HYPERLIQUID_WS_URL");
    fs::remove_file("test_config.toml").unwrap();
}

#[test]
fn test_config_missing_file() {
    let result = Config::load("nonexistent.toml");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read config file"));
}

#[test]
fn test_config_invalid_toml() {
    let config_content = r#"
        [environment]
        env = "invalid_env"
    "#;

    fs::write("invalid_config.toml", config_content).unwrap();

    let result = Config::load("invalid_config.toml");
    assert!(result.is_err());

    fs::remove_file("invalid_config.toml").unwrap();
}

#[test]
fn test_config_empty_file() {
    fs::write("empty_config.toml", "").unwrap();

    let config = Config::load("empty_config.toml").unwrap();

    // Should get defaults
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Mainnet);
    assert_eq!(config.http.request_timeout_ms, 30000);

    fs::remove_file("empty_config.toml").unwrap();
}

#[test]
fn test_config_partial_override() {
    let config_content = r#"
        [environment]
        env = "testnet"

        [http]
        request_timeout_ms = 60000
    "#;

    fs::write("partial_config.toml", config_content).unwrap();

    let config = Config::load("partial_config.toml").unwrap();

    // Environment should be overridden
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Testnet);

    // HTTP timeout should be overridden
    assert_eq!(config.http.request_timeout_ms, 60000);

    // Other HTTP settings should be default
    assert_eq!(config.http.max_connections_per_host, 10);

    // Other sections should be default
    assert_eq!(config.runtime.worker_threads, num_cpus::get());
    assert_eq!(config.logging.level, "info");

    fs::remove_file("partial_config.toml").unwrap();
}

#[test]
fn test_config_toml_types() {
    let config_content = r#"
        [environment]
        env = "local"
        base_url = "http://localhost:3001"
        websocket_url = "ws://localhost:3001/ws"

        [http]
        request_timeout_ms = 30000
        max_connections_per_host = 10
        max_total_connections = 100
        connect_timeout_ms = 10000
        enable_cert_pinning = false
        enable_rate_limiting = true

        [websocket]
        connect_timeout_ms = 10000
        reconnect_delay_ms = 1000
        max_reconnect_attempts = 5
        buffer_size = 1000
        enable_compression = false
        ping_interval_ms = 30000

        [runtime]
        worker_threads = 4
        max_blocking_threads = 512
        thread_stack_size = 2097152
        enable_io = true
        enable_time = true
        global_queue_interval = 61
        shutdown_timeout_secs = 30

        [logging]
        level = "debug"
        format = "pretty"
        file_path = "/tmp/test.log"
        max_log_size_mb = 100
        max_log_files = 5
        colored_output = true

        [security]
        enable_cert_pinning = true
        enable_request_signing = false
        strict_mode = false
        key_rotation_hours = 24

        [metrics]
        enabled = true
        endpoint = "/metrics"
        collection_interval_secs = 10
        enable_prometheus = true
        namespace = "test"
    "#;

    fs::write("types_config.toml", config_content).unwrap();

    let config = Config::load("types_config.toml").unwrap();

    // Test all types are correctly parsed
    assert_eq!(config.environment.env, hyperliquid_core::types::Environment::Local);
    assert_eq!(config.environment.base_url, Some("http://localhost:3001".to_string()));
    assert_eq!(config.environment.websocket_url, Some("ws://localhost:3001/ws".to_string()));

    assert_eq!(config.http.request_timeout_ms, 30000);
    assert_eq!(config.http.max_connections_per_host, 10);
    assert_eq!(config.http.max_total_connections, 100);
    assert_eq!(config.http.connect_timeout_ms, 10000);
    assert_eq!(config.http.enable_cert_pinning, false);
    assert_eq!(config.http.enable_rate_limiting, true);

    assert_eq!(config.websocket.connect_timeout_ms, 10000);
    assert_eq!(config.websocket.reconnect_delay_ms, 1000);
    assert_eq!(config.websocket.max_reconnect_attempts, 5);
    assert_eq!(config.websocket.buffer_size, 1000);
    assert_eq!(config.websocket.enable_compression, false);
    assert_eq!(config.websocket.ping_interval_ms, 30000);

    assert_eq!(config.runtime.worker_threads, 4);
    assert_eq!(config.runtime.max_blocking_threads, 512);
    assert_eq!(config.runtime.thread_stack_size, 2097152);
    assert_eq!(config.runtime.enable_io, true);
    assert_eq!(config.runtime.enable_time, true);
    assert_eq!(config.runtime.global_queue_interval, 61);
    assert_eq!(config.runtime.shutdown_timeout_secs, 30);

    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.logging.format, "pretty");
    assert_eq!(config.logging.file_path, Some("/tmp/test.log".to_string()));
    assert_eq!(config.logging.max_log_size_mb, 100);
    assert_eq!(config.logging.max_log_files, 5);
    assert_eq!(config.logging.colored_output, true);

    assert_eq!(config.security.enable_cert_pinning, true);
    assert_eq!(config.security.enable_request_signing, false);
    assert_eq!(config.security.strict_mode, false);
    assert_eq!(config.security.key_rotation_hours, 24);

    assert_eq!(config.metrics.enabled, true);
    assert_eq!(config.metrics.endpoint, "/metrics");
    assert_eq!(config.metrics.collection_interval_secs, 10);
    assert_eq!(config.metrics.enable_prometheus, true);
    assert_eq!(config.metrics.namespace, "test");

    fs::remove_file("types_config.toml").unwrap();
}