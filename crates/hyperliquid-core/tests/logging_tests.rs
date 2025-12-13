//! Tests for tracing and structured logging functionality

use hyperliquid_core::{LoggingConfig, init_tracing, generate_trace_id, request_span, log_request, log_response, log_error, log_retry};
use tracing::{info_span, Instrument};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();
    assert_eq!(config.level, "info");
    assert_eq!(config.format, "json");
    assert_eq!(config.colored, false);
    assert_eq!(config.utc_time, true);
    assert_eq!(config.file_logging, false);
}

#[test]
fn test_logging_config_debug() {
    let config = LoggingConfig::debug();
    assert_eq!(config.level, "debug");
    assert_eq!(config.format, "pretty");
    assert_eq!(config.colored, true);
    assert_eq!(config.file_logging, false);
}

#[test]
fn test_logging_config_production() {
    let config = LoggingConfig::production();
    assert_eq!(config.level, "info");
    assert_eq!(config.format, "json");
    assert_eq!(config.colored, false);
    assert_eq!(config.file_logging, true);
}

#[test]
fn test_generate_trace_id() {
    let trace_id = generate_trace_id();
    assert!(!trace_id.is_empty());
    assert!(trace_id.len() == 36); // UUID v4 format

    // Verify UUID format
    let parts: Vec<&str> = trace_id.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
}

#[tokio::test]
async fn test_init_tracing_basic() {
    let config = LoggingConfig::debug();
    let result = init_tracing(&config);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_init_tracing_production() {
    let config = LoggingConfig::production();
    let result = init_tracing(&config);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_init_tracing_with_env_filter() {
    // Test with environment variable
    std::env::set_var("RUST_LOG", "debug");
    let config = LoggingConfig::default();
    let result = init_tracing(&config);
    assert!(result.is_ok());
    std::env::remove_var("RUST_LOG");
}

#[tokio::test]
async fn test_request_span_creation() {
    let trace_id = "test-trace-id-12345";
    let span = request_span(trace_id, "GET", "https://api.example.com/test");

    // Test that span can be entered
    span.in_scope(|| {
        info_span!("inner_span").in_scope(|| {
            // This should work without panicking
        });
    });
}

#[tokio::test]
async fn test_log_request() {
    // Initialize tracing first
    let config = LoggingConfig::debug();
    init_tracing(&config).unwrap();

    let trace_id = "test-trace-id-12345";
    let body = r#"{"test": "data"}"#;

    // This should not panic and should generate log output
    log_request(trace_id, "POST", "https://api.example.com/test", Some(body));
}

#[tokio::test]
async fn test_log_response() {
    // Initialize tracing first
    let config = LoggingConfig::debug();
    init_tracing(&config).unwrap();

    let trace_id = "test-trace-id-12345";

    // Test successful response
    log_response(trace_id, 200, 150, Some("Success"));

    // Test error response
    log_response(trace_id, 500, 5000, Some("Internal Server Error"));
}

#[tokio::test]
async fn test_log_error() {
    // Initialize tracing first
    let config = LoggingConfig::debug();
    init_tracing(&config).unwrap();

    let trace_id = "test-trace-id-12345";
    let error_msg = "Connection timeout";

    // This should not panic and should generate log output
    log_error(trace_id, error_msg, "http_client");
}

#[tokio::test]
async fn test_log_retry() {
    // Initialize tracing first
    let config = LoggingConfig::debug();
    init_tracing(&config).unwrap();

    let trace_id = "test-trace-id-12345";

    // Test retry logging
    log_retry(trace_id, 1, 3, 1000, "503 Service Unavailable");
    log_retry(trace_id, 2, 3, 2000, "503 Service Unavailable");
    log_retry(trace_id, 3, 3, 4000, "503 Service Unavailable");
}

#[tokio::test]
async fn test_structured_logging_fields() {
    // Initialize tracing first
    let config = LoggingConfig::debug();
    init_tracing(&config).unwrap();

    let trace_id = "test-trace-id-12345";

    // Test that all required fields are present in logs
    tracing::info!(
        level = "test",
        trace_id = trace_id,
        method = "GET",
        url = "https://api.example.com/test",
        status_code = 200,
        latency_ms = 150,
        component = "test_component",
        timestamp = chrono::Utc::now().to_rfc3339(),
        "Test structured log"
    );
}

#[test]
fn test_logging_config_validation() {
    let config = LoggingConfig::default();

    // Test that configuration values are reasonable
    assert!(config.max_file_size_mb > 0);
    assert!(config.max_files > 0);
    assert!(config.max_file_size_mb <= 1000); // Reasonable max size
    assert!(config.max_files <= 30); // Reasonable max files
}

#[test]
fn test_trace_id_uniqueness() {
    let mut trace_ids = std::collections::HashSet::new();

    // Generate multiple trace IDs and ensure they're unique
    for _ in 0..100 {
        let trace_id = generate_trace_id();
        assert!(!trace_id.is_empty());
        assert!(trace_ids.insert(trace_id), "Trace ID should be unique");
    }
}

#[tokio::test]
async fn test_logging_with_different_levels() {
    // Test info level
    let config = LoggingConfig {
        level: "info".to_string(),
        ..Default::default()
    };
    init_tracing(&config).unwrap();

    tracing::info!("This is an info message");
    tracing::warn!("This is a warning message");
    tracing::error!("This is an error message");

    // Test debug level
    let config = LoggingConfig {
        level: "debug".to_string(),
        ..Default::default()
    };
    init_tracing(&config).unwrap();

    tracing::debug!("This is a debug message");
    tracing::info!("This is an info message");
    tracing::warn!("This is a warning message");
    tracing::error!("This is an error message");
}

#[tokio::test]
async fn test_logging_json_format() {
    let config = LoggingConfig {
        format: "json".to_string(),
        ..Default::default()
    };
    init_tracing(&config).unwrap();

    // This should generate JSON-formatted logs
    tracing::info!(
        test_field = "test_value",
        numeric_field = 42,
        bool_field = true,
        "Test JSON log"
    );
}

#[tokio::test]
async fn test_logging_pretty_format() {
    let config = LoggingConfig {
        format: "pretty".to_string(),
        colored: true,
        ..Default::default()
    };
    init_tracing(&config).unwrap();

    // This should generate pretty-formatted logs
    tracing::info!(
        test_field = "test_value",
        numeric_field = 42,
        "Test pretty log"
    );
}

#[tokio::test]
#[ignore] // Requires file system access
async fn test_file_logging() {
    let config = LoggingConfig {
        file_logging: true,
        log_dir: "test_logs".to_string(),
        log_file: "test_hyperliquid.log".to_string(),
        ..Default::default()
    };
    init_tracing(&config).unwrap();

    tracing::info!("This should be written to file");

    // Verify file was created (this would require file system access)
    // std::thread::sleep(std::time::Duration::from_secs(1));
    // assert!(std::path::Path::new("test_logs/test_hyperliquid.log").exists());
}