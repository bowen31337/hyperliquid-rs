//! Structured logging and tracing setup for Hyperliquid SDK
//!
//! This module provides initialization of tracing subscribers with structured logging,
//! request/response logging, trace ID generation, and configurable log formats.

use std::io;
use tracing_subscriber::{
    fmt,
    fmt::{time::UtcTime, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing::{Level, Span};
use tracing_appender::{rolling, non_blocking::NonBlocking};
use uuid::Uuid;

/// Configuration for logging setup
#[derive(Clone, Debug)]
pub struct LoggingConfig {
    /// Log level filter (debug, info, warn, error)
    pub level: String,
    /// Output format: "json" or "pretty"
    pub format: String,
    /// Enable colored output for pretty format
    pub colored: bool,
    /// Enable UTC timestamps
    pub utc_time: bool,
    /// Enable file logging
    pub file_logging: bool,
    /// Log file directory
    pub log_dir: String,
    /// Log file name pattern
    pub log_file: String,
    /// Maximum log file size in MB
    pub max_file_size_mb: u64,
    /// Maximum number of log files to keep
    pub max_files: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            colored: false,
            utc_time: true,
            file_logging: false,
            log_dir: "logs".to_string(),
            log_file: "hyperliquid.log".to_string(),
            max_file_size_mb: 100,
            max_files: 5,
        }
    }
}

impl LoggingConfig {
    /// Create debug configuration for development
    pub fn debug() -> Self {
        Self {
            level: "debug".to_string(),
            format: "pretty".to_string(),
            colored: true,
            utc_time: true,
            file_logging: false,
            ..Default::default()
        }
    }

    /// Create production configuration
    pub fn production() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            colored: false,
            utc_time: true,
            file_logging: true,
            ..Default::default()
        }
    }
}

/// Initialize global tracing subscriber with structured logging
pub fn init_tracing(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Parse environment filter from config level or RUST_LOG
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Create base layer for stdout
    let stdout_layer = create_stdout_layer(config)?.with_filter(filter.clone());

    // Create file layer if file logging is enabled
    let file_layer = if config.file_logging {
        Some(create_file_layer(config)?.with_filter(filter))
    } else {
        None
    };

    // Build subscriber
    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer);

    let subscriber = if let Some(file_layer) = file_layer {
        subscriber.with(file_layer)
    } else {
        subscriber
    };

    // Initialize global subscriber
    subscriber.init();

    tracing::info!(
        level = "startup",
        component = "logging",
        config = ?config,
        "Tracing subscriber initialized"
    );

    Ok(())
}

/// Create stdout logging layer
fn create_stdout_layer(config: &LoggingConfig) -> Result<impl Layer<tracing_subscriber::registry::Registry>, io::Error> {
    let timer = if config.utc_time {
        UtcTime::rfc_3339()
    } else {
        fmt::time::LocalTime::rfc_3339()
    };

    let layer = match config.format.as_str() {
        "json" => {
            fmt::layer()
                .json()
                .with_timer(timer)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE)
                .with_span_list(true)
        }
        "pretty" => {
            fmt::layer()
                .pretty()
                .with_timer(timer)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE)
        }
        _ => {
            fmt::layer()
                .json()
                .with_timer(timer)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE)
        }
    };

    if config.colored {
        layer.with_ansi(true)
    } else {
        layer.with_ansi(false)
    }
}

/// Create file logging layer with rotation
fn create_file_layer(config: &LoggingConfig) -> Result<impl Layer<tracing_subscriber::registry::Registry>, io::Error> {
    // Create rolling file appender
    let file_appender = rolling::size(
        config.log_dir.clone(),
        config.log_file.clone(),
        config.max_file_size_mb * 1024 * 1024, // Convert MB to bytes
    );

    let (non_blocking, _guard) = NonBlocking::new(file_appender);

    let timer = if config.utc_time {
        UtcTime::rfc_3339()
    } else {
        fmt::time::LocalTime::rfc_3339()
    };

    Ok(fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_timer(timer)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE))
}

/// Create a new trace ID for request tracking
pub fn generate_trace_id() -> String {
    Uuid::new_v4().to_string()
}

/// Create a span with trace ID for request tracking
pub fn request_span(trace_id: &str, method: &str, url: &str) -> Span {
    tracing::info_span!(
        "http_request",
        trace_id = trace_id,
        method = method,
        url = url,
        user_agent = "hyperliquid-rs/1.0.0"
    )
}

/// Log request details
pub fn log_request(
    trace_id: &str,
    method: &str,
    url: &str,
    body: Option<&str>,
) {
    tracing::info!(
        level = "request",
        trace_id = trace_id,
        method = method,
        url = url,
        body = body,
        timestamp = chrono::Utc::now().to_rfc3339(),
    );
}

/// Log response details
pub fn log_response(
    trace_id: &str,
    status_code: u16,
    latency_ms: u64,
    body: Option<&str>,
) {
    tracing::info!(
        level = "response",
        trace_id = trace_id,
        status_code = status_code,
        latency_ms = latency_ms,
        body = body,
        timestamp = chrono::Utc::now().to_rfc3339(),
    );
}

/// Log error with trace ID
pub fn log_error(trace_id: &str, error: &str, component: &str) {
    tracing::error!(
        level = "error",
        trace_id = trace_id,
        error = error,
        component = component,
        timestamp = chrono::Utc::now().to_rfc3339(),
    );
}

/// Log retry attempt
pub fn log_retry(trace_id: &str, attempt: u32, max_attempts: u32, delay_ms: u64, error: &str) {
    tracing::warn!(
        level = "retry",
        trace_id = trace_id,
        attempt = attempt,
        max_attempts = max_attempts,
        delay_ms = delay_ms,
        error = error,
        timestamp = chrono::Utc::now().to_rfc3339(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[tokio::test]
    async fn test_init_tracing_basic() {
        let config = LoggingConfig::debug();
        let result = init_tracing(&config);
        assert!(result.is_ok());
    }
}