use hyperliquid_core::error::HyperliquidError;
use reqwest::StatusCode;

#[test]
fn test_error_display_formatting() {
    // Test Network error
    let network_err = HyperliquidError::Network(
        reqwest::Error::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .build()
            .unwrap(),
    );
    assert!(format!("{}", network_err).contains("Network error:"));

    // Test Authentication error
    let auth_err = HyperliquidError::Authentication("Invalid API key".to_string());
    assert_eq!(
        format!("{}", auth_err),
        "Authentication error: Invalid API key"
    );

    // Test Validation error
    let validation_err = HyperliquidError::Validation("Order size too large".to_string());
    assert_eq!(
        format!("{}", validation_err),
        "Validation error: Order size too large"
    );

    // Test RateLimit error
    let rate_limit_err = HyperliquidError::RateLimit("Too many requests".to_string());
    assert_eq!(
        format!("{}", rate_limit_err),
        "Rate limit exceeded: Too many requests"
    );

    // Test RateLimitWithRetry error
    let rate_limit_retry_err = HyperliquidError::RateLimitWithRetry {
        message: "Rate limited".to_string(),
        retry_after: 60,
    };
    assert!(format!("{}", rate_limit_retry_err).contains("Rate limit exceeded. Retry after 60s:"));

    // Test Client error
    let client_err = HyperliquidError::Client {
        code: 400,
        message: "Bad request".to_string(),
        data: None,
    };
    assert_eq!(
        format!("{}", client_err),
        "Client error: 400 - Bad request"
    );

    // Test Server error
    let server_err = HyperliquidError::Server {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
    };
    assert_eq!(
        format!("{}", server_err),
        "Server error: 500 Internal Server Error - Internal server error"
    );

    // Test Http error
    let http_err = HyperliquidError::Http {
        status: StatusCode::NOT_FOUND,
        message: "Not found".to_string(),
        cause: None,
    };
    assert_eq!(
        format!("{}", http_err),
        "HTTP 404 Not Found: Not found"
    );

    // Test Signing error
    let signing_err = HyperliquidError::Signing("Invalid signature".to_string());
    assert_eq!(
        format!("{}", signing_err),
        "Signing error: Invalid signature"
    );

    // Test Config error
    let config_err = HyperliquidError::Config("Missing API key".to_string());
    assert_eq!(
        format!("{}", config_err),
        "Invalid configuration: Missing API key"
    );

    // Test Unknown error
    let unknown_err = HyperliquidError::Unknown("Something went wrong".to_string());
    assert_eq!(
        format!("{}", unknown_err),
        "Unknown error: Something went wrong"
    );
}

#[test]
fn test_error_chain_functionality() {
    // Test that errors can be converted from other error types
    let json_str = "{ invalid json";
    let json_err: Result<serde_json::Value, _> = serde_json::from_str(json_str);

    // Convert serde_json::Error to HyperliquidError
    match json_err {
        Err(e) => {
            let hyperliquid_err: HyperliquidError = e.into();
            assert!(format!("{}", hyperliquid_err).contains("JSON serialization error:"));

            // Test error source chain
            let source = hyperliquid_err.source();
            assert!(source.is_some());
        }
        Ok(_) => panic!("Expected JSON parsing to fail"),
    }

    // Test reqwest::Error conversion
    let reqwest_err = reqwest::Error::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .build()
        .unwrap();
    let hyperliquid_err: HyperliquidError = reqwest_err.into();
    assert!(format!("{}", hyperliquid_err).contains("Network error:"));
}

#[test]
fn test_is_retryable() {
    // Network errors should be retryable
    let network_err = HyperliquidError::Network(
        reqwest::Error::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .build()
            .unwrap(),
    );
    assert!(network_err.is_retryable());

    // Timeout errors should be retryable
    let timeout_err = HyperliquidError::Timeout("Request timed out".to_string());
    assert!(timeout_err.is_retryable());

    // 5xx HTTP errors should be retryable
    let server_err = HyperliquidError::Http {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        cause: None,
    };
    assert!(server_err.is_retryable());

    // 429 HTTP errors should be retryable
    let rate_limit_err = HyperliquidError::Http {
        status: StatusCode::TOO_MANY_REQUESTS,
        message: "Too many requests".to_string(),
        cause: None,
    };
    assert!(rate_limit_err.is_retryable());

    // RateLimit errors should be retryable
    let rate_limit_err = HyperliquidError::RateLimit("Too many requests".to_string());
    assert!(rate_limit_err.is_retryable());

    // RateLimitWithRetry errors should be retryable
    let rate_limit_retry_err = HyperliquidError::RateLimitWithRetry {
        message: "Rate limited".to_string(),
        retry_after: 60,
    };
    assert!(rate_limit_retry_err.is_retryable());

    // Server errors should be retryable
    let server_err = HyperliquidError::Server {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
    };
    assert!(server_err.is_retryable());

    // Authentication errors should NOT be retryable
    let auth_err = HyperliquidError::Authentication("Invalid API key".to_string());
    assert!(!auth_err.is_retryable());

    // Validation errors should NOT be retryable
    let validation_err = HyperliquidError::Validation("Invalid order".to_string());
    assert!(!validation_err.is_retryable());

    // Client errors should NOT be retryable
    let client_err = HyperliquidError::Client {
        code: 400,
        message: "Bad request".to_string(),
        data: None,
    };
    assert!(!client_err.is_retryable());

    // 4xx HTTP errors (except 429) should NOT be retryable
    let bad_request_err = HyperliquidError::Http {
        status: StatusCode::BAD_REQUEST,
        message: "Bad request".to_string(),
        cause: None,
    };
    assert!(!bad_request_err.is_retryable());
}

#[test]
fn test_should_retry_immediately() {
    // Network errors should retry immediately
    let network_err = HyperliquidError::Network(
        reqwest::Error::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .build()
            .unwrap(),
    );
    assert!(network_err.should_retry_immediately());

    // Timeout errors should retry immediately
    let timeout_err = HyperliquidError::Timeout("Request timed out".to_string());
    assert!(timeout_err.should_retry_immediately());

    // 502/503/504 HTTP errors should retry immediately
    for status in [StatusCode::BAD_GATEWAY, StatusCode::SERVICE_UNAVAILABLE, StatusCode::GATEWAY_TIMEOUT] {
        let http_err = HyperliquidError::Http {
            status,
            message: "Gateway error".to_string(),
            cause: None,
        };
        assert!(http_err.should_retry_immediately());
    }

    // Other errors should NOT retry immediately
    let auth_err = HyperliquidError::Authentication("Invalid API key".to_string());
    assert!(!auth_err.should_retry_immediately());

    let validation_err = HyperliquidError::Validation("Invalid order".to_string());
    assert!(!validation_err.should_retry_immediately());

    let client_err = HyperliquidError::Client {
        code: 400,
        message: "Bad request".to_string(),
        data: None,
    };
    assert!(!client_err.should_retry_immediately());

    let server_err = HyperliquidError::Server {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
    };
    assert!(!server_err.should_retry_immediately());
}

#[test]
fn test_error_convenience_constructors() {
    // Test creating Authentication error
    let auth_error = HyperliquidError::Authentication("Invalid credentials".to_string());
    assert!(matches!(auth_error, HyperliquidError::Authentication(_)));

    // Test creating Validation error
    let validation_error = HyperliquidError::Validation("Field required".to_string());
    assert!(matches!(validation_error, HyperliquidError::Validation(_)));

    // Test creating RateLimit error
    let rate_limit_error = HyperliquidError::RateLimit("Too many requests".to_string());
    assert!(matches!(rate_limit_error, HyperliquidError::RateLimit(_)));

    // Test creating RateLimitWithRetry error
    let rate_limit_retry_error = HyperliquidError::RateLimitWithRetry {
        message: "Rate limited".to_string(),
        retry_after: 30,
    };
    assert!(matches!(rate_limit_retry_error, HyperliquidError::RateLimitWithRetry { .. }));
}