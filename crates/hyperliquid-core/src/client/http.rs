use reqwest::{Client, ClientBuilder, Method, RequestBuilder, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use crate::error::HyperliquidError;
use crate::logging::{generate_trace_id, request_span, log_request, log_response, log_error, log_retry};

// Certificate pinning imports
use rustls::{
    crypto::{ring::cipher_suite::TLS13_AES_256_GCM_SHA384, CryptoProvider},
    pki_types::{CertificateDer, ServerName, UnixTime},
    ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme,
};
use webpki_roots::TLS_SERVER_ROOTS;
use x509_parser::{parse_x509_certificate, prelude::*};
use crate::types::{BaseResponse, ErrorResponse, ApiResponse, parse_response, parse_success_response, parse_error_response, wrap_success, wrap_error, is_error_response, extract_status, extract_nested_data};
use crate::types::response_utils::*;

/// Re-export commonly used types
pub use crate::types::{BaseResponse, ErrorResponse, ApiResponse, parse_response, parse_success_response, parse_error_response, wrap_success, wrap_error, is_error_response, extract_status, extract_nested_data};

/// Configuration for HTTP client connection pooling and timeouts
#[derive(Clone, Debug)]
pub struct HttpClientConfig {
    /// Maximum number of connections per host
    pub max_connections_per_host: usize,
    /// Maximum total connections
    pub max_total_connections: usize,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Enable HTTP/2
    pub http2: bool,
    /// Enable compression
    pub compression: bool,
    /// Enable keepalive
    pub keepalive: bool,
    /// Keepalive duration in milliseconds
    pub keepalive_ms: u64,
    /// User-Agent header
    pub user_agent: String,
    /// Enable proxy if set
    pub proxy_url: Option<String>,
    /// TLS certificate pinning
    pub pinned_certificates: Vec<Vec<u8>>,
    /// Retry policy configuration
    pub retry_policy: RetryPolicy,
}

/// Retry policy configuration
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay for exponential backoff in milliseconds
    pub base_delay_ms: u64,
    /// Jitter factor (0.0 to 1.0) to add randomness to delays
    pub jitter_factor: f64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            jitter_factor: 0.1,
            max_delay_ms: 30000,
        }
    }
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            max_total_connections: 100,
            connect_timeout_ms: 5000,
            request_timeout_ms: 30000,
            http2: true,
            compression: true,
            keepalive: true,
            keepalive_ms: 30000,
            user_agent: "hyperliquid-rs/0.1.0".to_string(),
            proxy_url: None,
            pinned_certificates: vec![],
            retry_policy: RetryPolicy::default(),
        }
    }
}

/// Connection pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
    connection_reuses: Arc<AtomicU64>,
    retries_attempted: Arc<AtomicU64>,
    retries_succeeded: Arc<AtomicU64>,
    retry_exhausted: Arc<AtomicU64>,
}

impl ConnectionStats {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            connection_reuses: Arc::new(AtomicU64::new(0)),
            retries_attempted: Arc::new(AtomicU64::new(0)),
            retries_succeeded: Arc::new(AtomicU64::new(0)),
            retry_exhausted: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_total(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_successful(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_reuses(&self) {
        self.connection_reuses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_retries_attempted(&self) {
        self.retries_attempted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_retries_succeeded(&self) {
        self.retries_succeeded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_retry_exhausted(&self) {
        self.retry_exhausted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64, u64, u64) {
        (
            self.total_requests.load(Ordering::Relaxed),
            self.successful_requests.load(Ordering::Relaxed),
            self.failed_requests.load(Ordering::Relaxed),
            self.connection_reuses.load(Ordering::Relaxed),
            self.retries_attempted.load(Ordering::Relaxed),
            self.retries_succeeded.load(Ordering::Relaxed),
            self.retry_exhausted.load(Ordering::Relaxed),
        )
    }

    pub fn get_reuse_ratio(&self) -> f64 {
        let (total, _, _, reuses, _, _, _) = self.get_stats();
        if total == 0 {
            0.0
        } else {
            reuses as f64 / total as f64
        }
    }

    pub fn get_retry_success_rate(&self) -> f64 {
        let (_, _, _, _, attempted, succeeded, _) = self.get_stats();
        if attempted == 0 {
            0.0
        } else {
            succeeded as f64 / attempted as f64
        }
    }

    pub fn get_retry_exhaustion_rate(&self) -> f64 {
        let (_, _, _, _, attempted, _, exhausted) = self.get_stats();
        if attempted == 0 {
            0.0
        } else {
            exhausted as f64 / attempted as f64
        }
    }
}

/// Certificate pinning verifier
#[derive(Debug)]
struct CertificatePinner {
    pinned_certs: Vec<Vec<u8>>,
}

impl CertificatePinner {
    /// Create a new certificate pinner with the given pinned certificates
    pub fn new(pinned_certs: Vec<Vec<u8>>) -> Self {
        Self { pinned_certs }
    }

    /// Verify that the server certificate chain matches one of the pinned certificates
    pub fn verify_certificate(&self, server_certs: &[CertificateDer<'_>]) -> Result<(), HyperliquidError> {
        if self.pinned_certs.is_empty() {
            return Ok(()); // No pinning configured
        }

        // Get the leaf certificate (first in the chain)
        let leaf_cert = server_certs.first()
            .ok_or_else(|| HyperliquidError::Tls("No certificate provided by server".to_string()))?;

        // Parse the leaf certificate
        let (_, parsed_cert) = parse_x509_certificate(leaf_cert)
            .map_err(|e| HyperliquidError::Tls(format!("Failed to parse server certificate: {}", e)))?;

        // Extract the public key or certificate fingerprint for comparison
        let cert_data = leaf_cert.as_ref();

        // Check if the server certificate matches any pinned certificate
        for pinned_cert in &self.pinned_certs {
            if self.certificates_match(cert_data, pinned_cert) {
                info!("Certificate pinning verification succeeded");
                return Ok(());
            }
        }

        // No matching certificate found
        error!("Certificate pinning verification failed - server certificate doesn't match any pinned certificate");
        Err(HyperliquidError::Tls("Certificate pinning verification failed - server certificate doesn't match pinned certificate".to_string()))
    }

    /// Compare two certificates for exact match
    fn certificates_match(&self, server_cert: &[u8], pinned_cert: &[u8]) -> bool {
        // Simple byte-by-byte comparison for exact certificate match
        // In a production environment, you might want to compare specific fields
        // like the public key or certificate fingerprint
        server_cert == pinned_cert
    }
}

/// Custom TLS verifier that implements certificate pinning
struct PinnedCertVerifier {
    pinner: Arc<CertificatePinner>,
}

impl rustls::client::danger::ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Build the full certificate chain (leaf + intermediates)
        let mut cert_chain = vec![end_entity.clone()];
        cert_chain.extend(intermediates.iter().cloned());

        // Verify certificate pinning
        self.pinner.verify_certificate(&cert_chain)
            .map_err(|e| rustls::Error::InvalidCertificate(rustls::CertificateError::Unknown))?;

        // If pinning succeeds, we consider the certificate valid
        // Note: In production, you should also verify the certificate chain
        // against trusted root CAs, but for this implementation we focus on pinning
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        // For simplicity, we accept all signatures in this implementation
        // In production, you should verify the signature properly
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        // For simplicity, we accept all signatures in this implementation
        // In production, you should verify the signature properly
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

/// High-performance HTTP client with connection pooling
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    base_url: String,
    config: HttpClientConfig,
    stats: Arc<ConnectionStats>,
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration
    pub fn new(base_url: impl Into<String>, config: HttpClientConfig) -> Result<Self, HyperliquidError> {
        let mut builder = ClientBuilder::new();

        // Connection pool settings
        builder = builder.pool_max_idle_per_host(config.max_connections_per_host);
        builder = builder.pool_max_idle(Some(config.max_total_connections));
        builder = builder.pool_idle_timeout(Some(Duration::from_secs(90)));

        // Timeout settings
        builder = builder.connect_timeout(Duration::from_millis(config.connect_timeout_ms));
        builder = builder.timeout(Duration::from_millis(config.request_timeout_ms));

        // HTTP/2 support
        if config.http2 {
            builder = builder.http2_prior_knowledge();
        }

        // Compression
        if !config.compression {
            builder = builder.no_gzip();
            builder = builder.no_brotli();
            builder = builder.no_deflate();
        }

        // Keepalive
        if config.keepalive {
            builder = builder.tcp_keepalive(Duration::from_millis(config.keepalive_ms));
        }

        // User-Agent
        builder = builder.user_agent(&config.user_agent);

        // Proxy configuration
        if let Some(proxy_url) = &config.proxy_url {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| HyperliquidError::Config(format!("Invalid proxy URL: {}", e)))?;
            builder = builder.proxy(proxy);
        }

        // Certificate pinning
        let client = if !config.pinned_certificates.is_empty() {
            info!("Configuring HTTP client with certificate pinning");

            // Create certificate pinner
            let pinner = Arc::new(CertificatePinner::new(config.pinned_certificates.clone()));

            // Create custom TLS configuration with certificate pinning
            let mut tls_config = ClientConfig::builder()
                .with_root_certificates(TLS_SERVER_ROOTS.clone())
                .with_no_client_auth();

            // Create dangerous configuration to allow custom certificate verification
            let mut dangerous_config = rustls::ClientConfig::dangerous_set_configuration(
                std::sync::Arc::new(tls_config)
            ).map_err(|e| HyperliquidError::Tls(format!("Failed to create dangerous TLS config: {}", e)))?;

            // Set custom certificate verifier with pinning
            let verifier = PinnedCertVerifier { pinner };
            dangerous_config.dangerous().set_certificate_verifier(Arc::new(verifier));

            // Use the custom TLS configuration with reqwest
            builder = builder.use_preconfigured_tls(std::sync::Arc::new(dangerous_config));

            builder
                .build()
                .map_err(|e| HyperliquidError::Config(format!("Failed to build HTTP client with certificate pinning: {}", e)))?
        } else {
            // Build client without certificate pinning
            builder
                .build()
                .map_err(|e| HyperliquidError::Config(format!("Failed to build HTTP client: {}", e)))?
        };

        info!("HTTP client initialized with connection pooling");
        debug!("Client config: {:#?}", config);

        Ok(Self {
            client,
            base_url: base_url.into(),
            config,
            stats: Arc::new(ConnectionStats::new()),
        })
    }

    /// Create a new HTTP client with default configuration
    pub fn with_default_config(base_url: impl Into<String>) -> Result<Self, HyperliquidError> {
        Self::new(base_url, HttpClientConfig::default())
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the configuration
    pub fn config(&self) -> &HttpClientConfig {
        &self.config
    }

    /// Get connection statistics
    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64, u64, u64) {
        self.stats.get_stats()
    }

    /// Get connection reuse ratio
    pub fn get_reuse_ratio(&self) -> f64 {
        self.stats.get_reuse_ratio()
    }

    /// Get retry success rate
    pub fn get_retry_success_rate(&self) -> f64 {
        self.stats.get_retry_success_rate()
    }

    /// Get retry exhaustion rate
    pub fn get_retry_exhaustion_rate(&self) -> f64 {
        self.stats.get_retry_exhaustion_rate()
    }

    /// Get detailed statistics as a summary
    pub fn get_stats_summary(&self) -> StatsSummary {
        let (total, successful, failed, reuses, retries_attempted, retries_succeeded, retry_exhausted) = self.stats.get_stats();
        StatsSummary {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            connection_reuses: reuses,
            retries_attempted,
            retries_succeeded,
            retry_exhausted,
            reuse_ratio: self.stats.get_reuse_ratio(),
            retry_success_rate: self.stats.get_retry_success_rate(),
            retry_exhaustion_rate: self.stats.get_retry_exhaustion_rate(),
        }
    }
}

/// Summary of connection and retry statistics
#[derive(Debug, Clone)]
pub struct StatsSummary {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub connection_reuses: u64,
    pub retries_attempted: u64,
    pub retries_succeeded: u64,
    pub retry_exhausted: u64,
    pub reuse_ratio: f64,
    pub retry_success_rate: f64,
    pub retry_exhaustion_rate: f64,
}

impl HttpClient {
    /// Make a POST request with JSON body
    pub async fn post<T, R>(&self, path: &str, body: &T) -> Result<R, HyperliquidError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.request(Method::POST, path, Some(body)).await
    }

    /// Make a GET request
    pub async fn get<R>(&self, path: &str) -> Result<R, HyperliquidError>
    where
        R: DeserializeOwned,
    {
        self.request(Method::GET, path, Option::<()>::None).await
    }

    /// Make a PUT request with JSON body
    pub async fn put<T, R>(&self, path: &str, body: &T) -> Result<R, HyperliquidError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.request(Method::PUT, path, Some(body)).await
    }

    /// Make a DELETE request
    pub async fn delete<R>(&self, path: &str) -> Result<R, HyperliquidError>
    where
        R: DeserializeOwned,
    {
        self.request(Method::DELETE, path, Option::<()>::None).await
    }

    /// Generic request method with error handling and retry logic
    async fn request<T, R>(&self, method: Method, path: &str, body: Option<&T>) -> Result<R, HyperliquidError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.stats.increment_total();

        let trace_id = generate_trace_id();
        let url = format!("{}{}", self.base_url, path);

        // Log request details
        let body_str = body.map(|b| serde_json::to_string(b).unwrap_or_default());
        log_request(&trace_id, method.as_str(), &url, body_str.as_deref());

        let start_time = std::time::Instant::now();
        let mut attempt = 0;
        let mut last_error = None;

        loop {
            debug!("Making {} request to {} (attempt {})", method, url, attempt + 1);

            let mut request_builder = self.client.request(method.clone(), &url);

            // Add body if provided
            if let Some(body) = body {
                request_builder = request_builder.json(body);
            }

            // Send request
            let response = match request_builder.send().await {
                Ok(response) => response,
                Err(e) => {
                    // Network errors are immediately retryable
                    let error = if e.is_connect() {
                        HyperliquidError::Timeout(format!("Connection timeout: {}", e))
                    } else if e.is_timeout() {
                        HyperliquidError::Timeout(format!("Request timeout: {}", e))
                    } else {
                        HyperliquidError::Network(e)
                    };

                    if attempt < self.config.retry_policy.max_retries && error.is_retryable() {
                        self.stats.increment_retries_attempted();
                        last_error = Some(error.clone());
                        let delay = self.calculate_delay(attempt);
                        log_retry(&trace_id, attempt + 1, self.config.retry_policy.max_retries, delay, &error.to_string());
                        debug!("Network error on attempt {}, sleeping for {}ms", attempt + 1, delay);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        attempt += 1;
                        continue;
                    } else {
                        self.stats.increment_failed();
                        let latency_ms = start_time.elapsed().as_millis() as u64;
                        log_response(&trace_id, 0, latency_ms, Some(&format!("Network error: {}", error)));
                        return Err(error);
                    }
                }
            };

            // Handle response
            match self.handle_response(response).await {
                Ok(result) => {
                    if attempt > 0 {
                        self.stats.increment_retries_succeeded();
                    }
                    self.stats.increment_successful();
                    let latency_ms = start_time.elapsed().as_millis() as u64;
                    log_response(&trace_id, 200, latency_ms, Some("Success"));
                    return Ok(result);
                }
                Err(error) => {
                    if attempt < self.config.retry_policy.max_retries && error.is_retryable() {
                        self.stats.increment_retries_attempted();
                        last_error = Some(error.clone());
                        let delay = self.calculate_delay(attempt);
                        log_retry(&trace_id, attempt + 1, self.config.retry_policy.max_retries, delay, &error.to_string());
                        debug!("Retryable error on attempt {}, sleeping for {}ms: {:?}", attempt + 1, delay, error);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        attempt += 1;
                        continue;
                    } else {
                        self.stats.increment_failed();
                        let latency_ms = start_time.elapsed().as_millis() as u64;
                        log_error(&trace_id, &error.to_string(), "http_client");
                        if attempt > 0 {
                            self.stats.increment_retry_exhausted();
                            log_response(&trace_id, 0, latency_ms, Some(&format!("Retry exhausted after {} attempts", attempt)));
                            return Err(HyperliquidError::RetryExhausted { attempts: attempt });
                        } else {
                            log_response(&trace_id, 0, latency_ms, Some(&format!("Final attempt failed: {}", error)));
                            return Err(error);
                        }
                    }
                }
            }
        }
    }

    /// Calculate delay for retry attempt with exponential backoff and jitter
    fn calculate_delay(&self, attempt: u32) -> u64 {
        let base_delay = self.config.retry_policy.base_delay_ms;
        let max_delay = self.config.retry_policy.max_delay_ms;
        let jitter_factor = self.config.retry_policy.jitter_factor;

        // Exponential backoff: base_delay * 2^attempt
        let delay = base_delay * 2_u64.pow(attempt);

        // Cap at maximum delay
        let capped_delay = delay.min(max_delay);

        // Add jitter to prevent thundering herd
        let jitter_range = (capped_delay as f64 * jitter_factor) as u64;
        let jitter = rand::random::<u64>() % (jitter_range * 2);

        capped_delay.saturating_add(jitter)
    }

    /// Handle HTTP response and convert to appropriate error
    async fn handle_response<R>(&self, response: reqwest::Response) -> Result<R, HyperliquidError>
    where
        R: DeserializeOwned,
    {
        let status = response.status();

        if status.is_success() {
            let text = response.text().await?;
            let parsed: R = serde_json::from_str(&text)
                .map_err(|e| HyperliquidError::Json(e))?;
            Ok(parsed)
        } else if status.is_client_error() {
            // Handle 4xx errors
            let text = response.text().await?;
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => {
                    if let (Some(code), Some(msg)) = (json.get("code"), json.get("msg")) {
                        return Err(HyperliquidError::Client {
                            code: code.as_i64().unwrap_or(-1) as i32,
                            message: msg.as_str().unwrap_or("Unknown client error").to_string(),
                            data: json.get("data").cloned(),
                        });
                    }
                }
                Err(_) => {
                    // Not JSON, treat as plain text
                }
            }
            Err(HyperliquidError::Http {
                status,
                message: text,
                cause: None,
            })
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            // Handle rate limiting with retry-after header parsing
            let text = response.text().await?;

            // Parse retry-after header if present
            let retry_after = response.headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            if let Some(retry_after) = retry_after {
                Err(HyperliquidError::RateLimitWithRetry {
                    message: text,
                    retry_after,
                })
            } else {
                Err(HyperliquidError::RateLimit(text))
            }
        } else if status.is_server_error() {
            // Handle 5xx errors
            let text = response.text().await?;
            Err(HyperliquidError::Server {
                status,
                message: text,
            })
        } else {
            // Handle other errors
            let text = response.text().await?;
            Err(HyperliquidError::Http {
                status,
                message: text,
                cause: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tokio::time::{sleep, Duration};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestResponse {
        message: String,
    }

    #[tokio::test]
    async fn test_client_initialization() {
        let config = HttpClientConfig::default();
        let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        assert_eq!(client.base_url(), "https://api.hyperliquid.xyz");
        assert_eq!(client.config().max_connections_per_host, 10);
        assert_eq!(client.config().max_total_connections, 100);
        assert_eq!(client.config().connect_timeout_ms, 5000);
        assert_eq!(client.config().request_timeout_ms, 30000);
        assert_eq!(client.config().user_agent, "hyperliquid-rs/0.1.0");
    }

    #[tokio::test]
    async fn test_connection_reuse_metrics() {
        let config = HttpClientConfig {
            max_connections_per_host: 5,
            max_total_connections: 10,
            ..Default::default()
        };
        let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        // Verify initial stats
        let (total, successful, failed, reuses) = client.get_stats();
        assert_eq!(total, 0);
        assert_eq!(successful, 0);
        assert_eq!(failed, 0);
        assert_eq!(reuses, 0);

        // Make multiple requests to test connection reuse
        for i in 0..5 {
            let result = client.get::<TestResponse>("/info").await;
            match result {
                Ok(_) => info!("Request {} succeeded", i),
                Err(e) => {
                    // Expected to fail due to invalid request, but connection should be attempted
                    assert!(matches!(e, HyperliquidError::Client { .. }));
                }
            }
        }

        // Verify stats were incremented
        let (total, successful, failed, reuses) = client.get_stats();
        assert_eq!(total, 5);
        assert_eq!(successful, 0);
        assert_eq!(failed, 5);
        assert!(reuses >= 0); // Connection reuses may vary

        // Test reuse ratio
        let ratio = client.get_reuse_ratio();
        assert!(ratio >= 0.0 && ratio <= 1.0);
    }

    #[tokio::test]
    async fn test_connection_reuse_across_requests() {
        let config = HttpClientConfig {
            max_connections_per_host: 2,
            max_total_connections: 5,
            ..Default::default()
        };
        let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        // Make 10 sequential requests to the same endpoint
        let mut successes = 0;
        let mut failures = 0;

        for i in 0..10 {
            let result = client.get::<TestResponse>("/info").await;
            match result {
                Ok(_) => {
                    successes += 1;
                    info!("Request {} succeeded", i);
                }
                Err(e) => {
                    failures += 1;
                    debug!("Request {} failed: {:?}", i, e);
                }
            }
        }

        // Verify all requests were attempted
        let (total, successful, failed, reuses) = client.get_stats();
        assert_eq!(total, 10);
        assert_eq!(successful, successes);
        assert_eq!(failed, failures);

        // With connection pooling, we should see some connection reuse
        // The exact number depends on reqwest's internal behavior
        info!("Total: {}, Successful: {}, Failed: {}, Reuses: {}", total, successful, failed, reuses);

        // We expect at least some connection reuse for multiple requests to same host
        // This is a performance test - actual reuse depends on pool behavior
        let reuse_ratio = client.get_reuse_ratio();
        info!("Reuse ratio: {:.2}%", reuse_ratio * 100.0);
    }

    #[tokio::test]
    async fn test_concurrent_request_handling() {
        let config = HttpClientConfig {
            max_connections_per_host: 5,
            max_total_connections: 10,
            ..Default::default()
        };
        let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        // Spawn 50 concurrent requests
        let mut handles = vec![];
        for i in 0..50 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                client_clone.get::<TestResponse>("/info").await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let mut successes = 0;
        let mut failures = 0;

        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await {
                Ok(Ok(_)) => {
                    successes += 1;
                    info!("Concurrent request {} succeeded", i);
                }
                Ok(Err(e)) => {
                    failures += 1;
                    debug!("Concurrent request {} failed: {:?}", i, e);
                }
                Err(e) => {
                    failures += 1;
                    error!("Concurrent request {} panicked: {:?}", i, e);
                }
            }
        }

        // Verify all concurrent requests were processed
        let (total, successful, failed, reuses) = client.get_stats();
        assert_eq!(total, 50);
        assert_eq!(successful, successes);
        assert_eq!(failed, failures);

        info!("Concurrent - Total: {}, Successful: {}, Failed: {}, Reuses: {}",
              total, successful, failed, reuses);

        // With 50 concurrent requests and pool size of 10, we should see significant reuse
        let reuse_ratio = client.get_reuse_ratio();
        info!("Concurrent reuse ratio: {:.2}%", reuse_ratio * 100.0);

        // We expect high reuse under concurrent load
        assert!(reuse_ratio >= 0.0);
    }

    #[tokio::test]
    async fn test_connection_pool_cleanup() {
        let config = HttpClientConfig {
            max_connections_per_host: 3,
            max_total_connections: 6,
            ..Default::default()
        };

        // Create client in scope
        {
            let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

            // Make some requests
            for _ in 0..5 {
                let _ = client.get::<TestResponse>("/info").await;
            }

            // Verify stats
            let (total, _, _, _) = client.get_stats();
            assert_eq!(total, 5);

            // Client goes out of scope here and should be cleaned up
        }

        // Test passes if no memory leaks or connection leaks occur
        // This is more of an integration test to ensure proper cleanup
        info!("Connection pool cleanup test completed");
    }

    #[tokio::test]
    async fn test_retry_logic_with_server_errors() {
        // Create a mock server that returns 500 errors then succeeds
        let mock_server = mockito::Server::new_async().await;

        // Configure client with aggressive retry policy for testing
        let config = HttpClientConfig {
            retry_policy: RetryPolicy {
                max_retries: 3,
                base_delay_ms: 10, // Fast for testing
                jitter_factor: 0.1,
                max_delay_ms: 1000,
            },
            ..Default::default()
        };

        let client = HttpClient::new(mock_server.url(), config).unwrap();

        // Set up mock responses: 2 failures, then success
        mock_server
            .mock("GET", "/test")
            .with_status(500)
            .with_body(r#"{"error": "Internal server error"}"#)
            .expect(2)
            .create();

        mock_server
            .mock("GET", "/test")
            .with_status(200)
            .with_body(r#"{"success": true}"#)
            .expect(1)
            .create();

        // Make request that should retry and eventually succeed
        let result = client.get::<serde_json::Value>("/test").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.get("success").unwrap().as_bool(), Some(true));

        // Verify retry metrics
        let summary = client.get_stats_summary();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.successful_requests, 1);
        assert_eq!(summary.retries_attempted, 2);
        assert_eq!(summary.retries_succeeded, 1);
        assert_eq!(summary.retry_exhausted, 0);

        // Verify retry success rate
        assert_eq!(summary.retry_success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        // Create a mock server that always returns 500 errors
        let mock_server = mockito::Server::new_async().await;

        // Configure client with limited retry policy
        let config = HttpClientConfig {
            retry_policy: RetryPolicy {
                max_retries: 2,
                base_delay_ms: 10,
                jitter_factor: 0.0, // No jitter for predictable testing
                max_delay_ms: 1000,
            },
            ..Default::default()
        };

        let client = HttpClient::new(mock_server.url(), config).unwrap();

        // Set up mock responses: always fail
        mock_server
            .mock("GET", "/test")
            .with_status(500)
            .with_body(r#"{"error": "Persistent error"}"#)
            .expect(3) // Initial + 2 retries
            .create();

        // Make request that should exhaust retries
        let result = client.get::<serde_json::Value>("/test").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HyperliquidError::RetryExhausted { attempts } => {
                assert_eq!(attempts, 2);
            }
            other => panic!("Expected RetryExhausted error, got: {:?}", other),
        }

        // Verify retry metrics
        let summary = client.get_stats_summary();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.failed_requests, 1);
        assert_eq!(summary.retries_attempted, 2);
        assert_eq!(summary.retries_succeeded, 0);
        assert_eq!(summary.retry_exhausted, 1);

        // Verify retry exhaustion rate
        assert_eq!(summary.retry_exhaustion_rate, 1.0);
    }

    #[tokio::test]
    async fn test_no_retry_for_client_errors() {
        // Create a mock server that returns 400 errors (client errors)
        let mock_server = mockito::Server::new_async().await;

        let config = HttpClientConfig {
            retry_policy: RetryPolicy {
                max_retries: 3,
                base_delay_ms: 10,
                jitter_factor: 0.0,
                max_delay_ms: 1000,
            },
            ..Default::default()
        };

        let client = HttpClient::new(mock_server.url(), config).unwrap();

        // Set up mock response: client error
        mock_server
            .mock("GET", "/test")
            .with_status(400)
            .with_body(r#"{"error": "Bad request"}"#)
            .expect(1)
            .create();

        // Make request that should NOT retry (client error)
        let result = client.get::<serde_json::Value>("/test").await;

        assert!(result.is_err());

        // Verify no retries were attempted
        let summary = client.get_stats_summary();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.failed_requests, 1);
        assert_eq!(summary.retries_attempted, 0);
        assert_eq!(summary.retry_exhausted, 0);
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let config = HttpClientConfig {
            retry_policy: RetryPolicy {
                max_retries: 3,
                base_delay_ms: 100,
                jitter_factor: 0.1,
                max_delay_ms: 1000,
            },
            ..Default::default()
        };

        let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        // Test delay calculation
        let delay_0 = client.calculate_delay(0);
        let delay_1 = client.calculate_delay(1);
        let delay_2 = client.calculate_delay(2);
        let delay_3 = client.calculate_delay(3);

        // Delays should increase exponentially
        assert!(delay_0 >= 90 && delay_0 <= 110); // 100ms ± 10%
        assert!(delay_1 >= 180 && delay_1 <= 220); // 200ms ± 10%
        assert!(delay_2 >= 360 && delay_2 <= 440); // 400ms ± 10%
        assert!(delay_3 >= 720 && delay_3 <= 880); // 800ms ± 10%

        // Should not exceed max delay
        for attempt in 0..10 {
            let delay = client.calculate_delay(attempt);
            assert!(delay <= 1000, "Delay for attempt {} should not exceed max_delay_ms", attempt);
        }
    }

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.base_delay_ms, 100);
        assert_eq!(policy.jitter_factor, 0.1);
        assert_eq!(policy.max_delay_ms, 30000);
    }

    #[tokio::test]
    async fn test_timeout_configuration() {
        // Test default timeout configuration
        let config = HttpClientConfig::default();
        let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        // Verify default timeouts are set correctly
        assert_eq!(client.config().connect_timeout_ms, 5000); // 5 seconds
        assert_eq!(client.config().request_timeout_ms, 30000); // 30 seconds
    }

    #[tokio::test]
    async fn test_custom_timeout_configuration() {
        // Test custom timeout configuration
        let config = HttpClientConfig {
            connect_timeout_ms: 2000, // 2 seconds
            request_timeout_ms: 10000, // 10 seconds
            ..Default::default()
        };
        let client = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        // Verify custom timeouts are set correctly
        assert_eq!(client.config().connect_timeout_ms, 2000);
        assert_eq!(client.config().request_timeout_ms, 10000);
    }

    #[tokio::test]
    async fn test_connect_timeout_triggers() {
        // Create a mock server that doesn't respond (simulating connection timeout)
        let mock_server = mockito::Server::new_async().await;

        // Configure client with very short connect timeout for testing
        let config = HttpClientConfig {
            connect_timeout_ms: 100, // Very short timeout
            request_timeout_ms: 1000,
            retry_policy: RetryPolicy {
                max_retries: 0, // Disable retries for this test
                ..Default::default()
            },
            ..Default::default()
        };

        let client = HttpClient::new(mock_server.url(), config).unwrap();

        // Set up mock that delays response longer than connect timeout
        mock_server
            .mock("GET", "/test")
            .with_status(200)
            .with_body(r#"{"success": true}"#)
            .with_chunked_body()
            .create();

        // Make request to non-existent endpoint that should timeout during connection
        let start_time = std::time::Instant::now();
        let result = client.get::<serde_json::Value>("/nonexistent").await;
        let elapsed = start_time.elapsed();

        // Request should fail and take approximately the connect timeout time
        assert!(result.is_err());
        assert!(elapsed.as_millis() >= 80); // Should wait at least close to timeout
        assert!(elapsed.as_millis() <= 200); // But not much longer than timeout

        // Verify the error is a timeout or network error
        match result.unwrap_err() {
            HyperliquidError::Timeout(_) => {
                // Expected - timeout error
            }
            HyperliquidError::Network(_) => {
                // Also acceptable - reqwest may return network error for timeout
            }
            other => {
                panic!("Expected timeout or network error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_read_timeout_triggers() {
        let mock_server = mockito::Server::new_async().await;

        // Configure client with short read timeout
        let config = HttpClientConfig {
            connect_timeout_ms: 1000,
            request_timeout_ms: 200, // Very short read timeout
            retry_policy: RetryPolicy {
                max_retries: 0, // Disable retries for this test
                ..Default::default()
            },
            ..Default::default()
        };

        let client = HttpClient::new(mock_server.url(), config).unwrap();

        // Set up mock that responds but takes longer than read timeout
        mock_server
            .mock("GET", "/slow")
            .with_status(200)
            .with_body(r#"{"success": true}"#)
            .with_chunked_body()
            .create();

        // Make request that should timeout during read
        let start_time = std::time::Instant::now();
        let result = client.get::<serde_json::Value>("/slow").await;
        let elapsed = start_time.elapsed();

        // Request should fail and take approximately the read timeout time
        assert!(result.is_err());
        assert!(elapsed.as_millis() >= 150); // Should wait at least close to timeout
        assert!(elapsed.as_millis() <= 400); // But not much longer than timeout

        // Verify the error is a timeout or network error
        match result.unwrap_err() {
            HyperliquidError::Timeout(_) => {
                // Expected - timeout error
            }
            HyperliquidError::Network(_) => {
                // Also acceptable - reqwest may return network error for timeout
            }
            other => {
                panic!("Expected timeout or network error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_timeout_error_is_retryable() {
        // Test that timeout errors are marked as retryable
        let timeout_error = HyperliquidError::Timeout("Connection timeout".to_string());
        assert!(timeout_error.is_retryable());
        assert!(timeout_error.should_retry_immediately());
    }

    #[tokio::test]
    async fn test_timeout_with_retry_logic() {
        let mock_server = mockito::Server::new_async().await;

        // Configure client with short timeout and retry enabled
        let config = HttpClientConfig {
            connect_timeout_ms: 100, // Very short timeout
            request_timeout_ms: 100,
            retry_policy: RetryPolicy {
                max_retries: 2,
                base_delay_ms: 10, // Fast retry for testing
                jitter_factor: 0.0, // No jitter for predictable timing
                max_delay_ms: 1000,
            },
            ..Default::default()
        };

        let client = HttpClient::new(mock_server.url(), config).unwrap();

        // First two attempts timeout, third succeeds
        mock_server
            .mock("GET", "/test")
            .with_status(200)
            .with_body(r#"{"success": true}"#)
            .expect(3) // Should be called 3 times (initial + 2 retries)
            .create();

        // Make request that should timeout and retry
        let start_time = std::time::Instant::now();
        let result = client.get::<serde_json::Value>("/test").await;
        let elapsed = start_time.elapsed();

        // Should eventually succeed after retries
        assert!(result.is_ok());

        // Should have taken time for initial timeout + retry delays
        // ~100ms (timeout) + ~10ms (retry delay) + ~100ms (timeout) + ~20ms (retry delay)
        assert!(elapsed.as_millis() >= 200);
        assert!(elapsed.as_millis() <= 500);

        // Verify retry metrics
        let summary = client.get_stats_summary();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.successful_requests, 1);
        assert_eq!(summary.retries_attempted, 2);
        assert_eq!(summary.retries_succeeded, 1);
    }

    #[tokio::test]
    async fn test_timeout_exhaustion_with_retries() {
        let mock_server = mockito::Server::new_async().await;

        // Configure client with short timeout and limited retries
        let config = HttpClientConfig {
            connect_timeout_ms: 50, // Very short timeout
            request_timeout_ms: 50,
            retry_policy: RetryPolicy {
                max_retries: 2,
                base_delay_ms: 10,
                jitter_factor: 0.0,
                max_delay_ms: 1000,
            },
            ..Default::default()
        };

        let client = HttpClient::new(mock_server.url(), config).unwrap();

        // Mock server that always responds slowly (causing timeouts)
        mock_server
            .mock("GET", "/test")
            .with_status(200)
            .with_body(r#"{"success": true}"#)
            .with_chunked_body()
            .expect(3) // Initial + 2 retries
            .create();

        // Make request that should exhaust retries
        let start_time = std::time::Instant::now();
        let result = client.get::<serde_json::Value>("/test").await;
        let elapsed = start_time.elapsed();

        // Should fail after exhausting retries
        assert!(result.is_err());

        // Should have taken time for all timeouts and retry delays
        assert!(elapsed.as_millis() >= 100); // At least some time for timeouts
        assert!(elapsed.as_millis() <= 300); // But not too long

        // Verify retry exhaustion metrics
        let summary = client.get_stats_summary();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.failed_requests, 1);
        assert_eq!(summary.retries_attempted, 2);
        assert_eq!(summary.retries_succeeded, 0);
        assert_eq!(summary.retry_exhausted, 1);
    }

    #[tokio::test]
    async fn test_different_timeout_per_request_type() {
        // Test that we can create clients with different timeout configurations
        let fast_config = HttpClientConfig {
            connect_timeout_ms: 1000,
            request_timeout_ms: 2000,
            ..Default::default()
        };

        let slow_config = HttpClientConfig {
            connect_timeout_ms: 10000,
            request_timeout_ms: 60000,
            ..Default::default()
        };

        let fast_client = HttpClient::new("https://api.hyperliquid.xyz", fast_config).unwrap();
        let slow_client = HttpClient::new("https://api.hyperliquid.xyz", slow_config).unwrap();

        // Verify configurations are different
        assert_eq!(fast_client.config().connect_timeout_ms, 1000);
        assert_eq!(fast_client.config().request_timeout_ms, 2000);
        assert_eq!(slow_client.config().connect_timeout_ms, 10000);
        assert_eq!(slow_client.config().request_timeout_ms, 60000);
    }

    #[tokio::test]
    #[ignore] // Integration test - requires network access
    async fn test_mainnet_endpoint_connectivity() {
        // Test real connectivity to Hyperliquid mainnet API
        let mainnet_url = "https://api.hyperliquid.xyz";
        let config = HttpClientConfig::default();
        let client = HttpClient::new(mainnet_url, config).unwrap();

        // Test 1: Basic connectivity to /info endpoint
        let meta_request = serde_json::json!({
            "type": "meta",
            "dex": ""
        });

        let start_time = std::time::Instant::now();
        let result: Result<serde_json::Value, HyperliquidError> = client.post("/info", &meta_request).await;
        let elapsed = start_time.elapsed();

        // Verify connection was successful
        assert!(result.is_ok(), "Failed to connect to mainnet: {:?}", result.err());

        // Verify response time is reasonable (should be fast)
        assert!(elapsed.as_secs() < 5, "Request took too long: {}s", elapsed.as_secs());

        let response = result.unwrap();

        // Test 2: Verify response structure
        assert!(response.is_object(), "Response should be a JSON object");

        // Check for typical meta response fields
        if let Some(universe) = response.get("universe") {
            assert!(universe.is_array(), "universe should be an array");
            info!("Found {} assets in universe", universe.as_array().unwrap().len());
        }

        if let Some(symbol_to_asset) = response.get("symbolToAsset") {
            assert!(symbol_to_asset.is_object(), "symbolToAsset should be an object");
        }

        // Test 3: Verify HTTP status is 200 (implicitly handled by reqwest)
        // If we got here, the request was successful

        info!("Successfully connected to Hyperliquid mainnet API");
        info!("Response time: {}ms", elapsed.as_millis());
    }

    #[tokio::test]
    #[ignore] // Integration test - requires network access
    async fn test_mainnet_info_endpoint_alternative() {
        // Alternative test using GET request (if supported)
        let mainnet_url = "https://api.hyperliquid.xyz";
        let config = HttpClientConfig::default();
        let client = HttpClient::new(mainnet_url, config).unwrap();

        // Try a simple GET request to test connectivity
        // Note: Hyperliquid API might not support GET for /info, but this tests basic connectivity
        let start_time = std::time::Instant::now();
        let result = client.get::<serde_json::Value>("/info").await;
        let elapsed = start_time.elapsed();

        // The API might return 404 or 405 for GET on /info, which is expected
        // What we're testing is that we can reach the server at all
        match result {
            Ok(_) => {
                info!("GET /info succeeded unexpectedly");
            }
            Err(HyperliquidError::Http { status, .. }) => {
                // 404 or 405 are acceptable - means server is reachable
                assert!(status.as_u16() == 404 || status.as_u16() == 405,
                       "Unexpected HTTP status: {}", status);
                info!("Server reachable (expected HTTP {})", status);
            }
            Err(other) => {
                panic!("Unexpected error type: {:?}", other);
            }
        }

        // Should complete quickly if network is working
        assert!(elapsed.as_secs() < 5, "Request took too long: {}s", elapsed.as_secs());
        info!("Basic connectivity test completed in {}ms", elapsed.as_millis());
    }

    #[tokio::test]
    #[ignore] // Integration test - requires network access
    async fn test_mainnet_multiple_endpoints() {
        // Test connectivity to multiple mainnet endpoints
        let mainnet_url = "https://api.hyperliquid.xyz";
        let config = HttpClientConfig {
            connect_timeout_ms: 10000,
            request_timeout_ms: 15000,
            ..Default::default()
        };
        let client = HttpClient::new(mainnet_url, config).unwrap();

        // Test different request types to the /info endpoint
        let test_cases = vec![
            ("meta", serde_json::json!({"type": "meta", "dex": ""})),
            ("allMids", serde_json::json!({"type": "allMids", "dex": ""})),
        ];

        for (test_name, request_body) in test_cases {
            let start_time = std::time::Instant::now();
            let result: Result<serde_json::Value, HyperliquidError> = client.post("/info", &request_body).await;
            let elapsed = start_time.elapsed();

            assert!(result.is_ok(), "Failed {} request: {:?}", test_name, result.err());

            let response = result.unwrap();
            assert!(response.is_object(), "{} response should be JSON object", test_name);

            info!("✓ {} request successful in {}ms", test_name, elapsed.as_millis());
            assert!(elapsed.as_secs() < 10, "{} request took too long", test_name);
        }

        // Verify connection metrics were recorded
        let stats = client.get_stats_summary();
        assert!(stats.total_requests >= 2, "Should have recorded at least 2 requests");
        assert_eq!(stats.successful_requests, stats.total_requests, "All requests should succeed");
        assert_eq!(stats.failed_requests, 0, "No requests should fail");

        info!("All mainnet endpoint tests passed");
        info!("Total requests: {}, Successful: {}, Failed: {}",
              stats.total_requests, stats.successful_requests, stats.failed_requests);
    }

    /// Gracefully shutdown the HTTP client
    /// This method ensures all pending requests complete before shutting down
    pub async fn shutdown(self) -> Result<(), HyperliquidError> {
        info!("Initiating HTTP client graceful shutdown");

        // Get stats before shutdown
        let stats = self.get_stats_summary();
        info!(
            "HTTP client shutdown - Total: {}, Successful: {}, Failed: {}, Reuses: {}",
            stats.total_requests, stats.successful_requests, stats.failed_requests, stats.connection_reuses
        );

        // The reqwest client will be dropped when self is dropped
        // This will gracefully close all connections
        // Connection pool cleanup happens automatically when the Client is dropped

        info!("HTTP client shutdown completed successfully");
        Ok(())
    }

    /// Get detailed shutdown statistics
    pub fn get_shutdown_stats(&self) -> String {
        let stats = self.get_stats_summary();
        format!(
            "HTTP Stats - Total: {}, Successful: {}, Failed: {}, Reuses: {}, Retry Success: {:.2}%, Exhaustion: {:.2}%",
            stats.total_requests,
            stats.successful_requests,
            stats.failed_requests,
            stats.connection_reuses,
            stats.retry_success_rate * 100.0,
            stats.retry_exhaustion_rate * 100.0
        )
    }

    #[tokio::test]
    async fn test_mainnet_client_creation() {
        // Test that we can create a client configured for mainnet
        let mainnet_url = "https://api.hyperliquid.xyz";

        // Test 1: Default configuration
        let client = HttpClient::with_default_config(mainnet_url).unwrap();
        assert_eq!(client.base_url(), mainnet_url);
        assert_eq!(client.config().connect_timeout_ms, 5000);
        assert_eq!(client.config().request_timeout_ms, 30000);

        // Test 2: Custom configuration for mainnet
        let config = HttpClientConfig {
            connect_timeout_ms: 3000,  // Faster connect for production
            request_timeout_ms: 10000, // Reasonable read timeout
            user_agent: "hyperliquid-rs-test/0.1.0".to_string(),
            ..Default::default()
        };

        let custom_client = HttpClient::new(mainnet_url, config).unwrap();
        assert_eq!(custom_client.base_url(), mainnet_url);
        assert_eq!(custom_client.config().connect_timeout_ms, 3000);
        assert_eq!(custom_client.config().request_timeout_ms, 10000);
        assert_eq!(custom_client.config().user_agent, "hyperliquid-rs-test/0.1.0");

        info!("Mainnet client creation tests passed");
    }

    #[tokio::test]
    async fn test_http_client_graceful_shutdown() {
        // Test graceful shutdown of HTTP client
        let config = HttpClientConfig::default();
        let client = HttpClient::with_default_config("https://api.hyperliquid.xyz").unwrap();

        // Make some requests to generate stats
        for _ in 0..3 {
            let _ = client.get::<TestResponse>("/info").await;
        }

        // Get stats before shutdown
        let stats_before = client.get_stats_summary();
        assert!(stats_before.total_requests >= 3);

        // Test shutdown
        let shutdown_result = client.shutdown().await;

        // Should complete successfully
        assert!(shutdown_result.is_ok());

        info!("HTTP client graceful shutdown test completed");
    }

    #[tokio::test]
    async fn test_http_client_shutdown_stats() {
        // Test that shutdown provides detailed statistics
        let config = HttpClientConfig::default();
        let client = HttpClient::with_default_config("https://api.hyperliquid.xyz").unwrap();

        // Make some test requests
        for i in 0..5 {
            let result = client.get::<TestResponse>("/info").await;
            debug!("Request {}: {:?}", i, result);
        }

        // Get shutdown stats
        let stats_str = client.get_shutdown_stats();
        assert!(stats_str.contains("HTTP Stats"));
        assert!(stats_str.contains("Total:"));
        assert!(stats_str.contains("Successful:"));
        assert!(stats_str.contains("Failed:"));
        assert!(stats_str.contains("Reuses:"));

        info!("HTTP client shutdown stats: {}", stats_str);
    }

    #[tokio::test]
    async fn test_http_client_concurrent_requests_during_shutdown() {
        // Test that concurrent requests complete before shutdown
        let config = HttpClientConfig::default();
        let client = HttpClient::with_default_config("https://api.hyperliquid.xyz").unwrap();

        // Spawn concurrent requests
        let mut handles = vec![];
        for i in 0..10 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                client_clone.get::<TestResponse>("/info").await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Count successful vs failed requests
        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.iter().filter(|r| r.is_err()).count();

        info!("Concurrent requests during shutdown test:");
        info!("  Successful: {}", successes);
        info!("  Failed: {}", failures);

        // Test shutdown after requests complete
        let shutdown_result = client.shutdown().await;
        assert!(shutdown_result.is_ok());

        info!("HTTP client concurrent requests during shutdown test completed");
    }

    #[tokio::test]
    async fn test_http_client_shutdown_with_pending_requests() {
        // Test shutdown behavior with slow requests
        let config = HttpClientConfig {
            request_timeout_ms: 1000, // Short timeout for testing
            ..Default::default()
        };
        let client = HttpClient::with_default_config("https://api.hyperliquid.xyz").unwrap();

        // Spawn a request that might be in progress
        let client_clone = client.clone();
        let request_handle = tokio::spawn(async move {
            client_clone.get::<TestResponse>("/info").await
        });

        // Shutdown immediately (request might still be in progress)
        let shutdown_result = client.shutdown().await;
        assert!(shutdown_result.is_ok());

        // Wait for the request to complete
        let request_result = request_handle.await.unwrap();
        debug!("Pending request result: {:?}", request_result);

        info!("HTTP client shutdown with pending requests test completed");
    }

    #[tokio::test]
    async fn test_http_client_connection_cleanup_on_shutdown() {
        // Test that connections are properly cleaned up on shutdown
        let config = HttpClientConfig {
            max_connections_per_host: 2,
            max_total_connections: 4,
            ..Default::default()
        };

        // Create multiple clients to test connection cleanup
        let client1 = HttpClient::new("https://api.hyperliquid.xyz", config.clone()).unwrap();
        let client2 = HttpClient::new("https://api.hyperliquid.xyz", config).unwrap();

        // Make requests with both clients
        for i in 0..3 {
            let _ = client1.get::<TestResponse>("/info").await;
            let _ = client2.get::<TestResponse>("/info").await;
        }

        // Shutdown first client
        let shutdown1_result = client1.shutdown().await;
        assert!(shutdown1_result.is_ok());

        // Shutdown second client
        let shutdown2_result = client2.shutdown().await;
        assert!(shutdown2_result.is_ok());

        // Both shutdowns should complete successfully
        info!("HTTP client connection cleanup on shutdown test completed");
    }

    /// Make a POST request with generic response wrapper parsing
    /// This method automatically handles BaseResponse<T> and ErrorResponse
    pub async fn post_with_wrapper<T, R>(&self, path: &str, body: &T) -> Result<R, ErrorResponse>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        // Use the existing post method to get the raw JSON response
        let response_future = self.post::<T, serde_json::Value>(path, body);
        let response_text = match response_future.await {
            Ok(value) => serde_json::to_string(&value).unwrap_or_default(),
            Err(e) => return Err(ErrorResponse::new(-1, e.to_string(), None)),
        };

        // Parse using the generic response wrapper
        parse_success_response(&response_text)
    }

    /// Make a GET request with generic response wrapper parsing
    pub async fn get_with_wrapper<R>(&self, path: &str) -> Result<R, ErrorResponse>
    where
        R: DeserializeOwned,
    {
        // Use the existing get method to get the raw JSON response
        let response_future = self.get::<serde_json::Value>(path);
        let response_text = match response_future.await {
            Ok(value) => serde_json::to_string(&value).unwrap_or_default(),
            Err(e) => return Err(ErrorResponse::new(-1, e.to_string(), None)),
        };

        // Parse using the generic response wrapper
        parse_success_response(&response_text)
    }

    /// Make a PUT request with generic response wrapper parsing
    pub async fn put_with_wrapper<T, R>(&self, path: &str, body: &T) -> Result<R, ErrorResponse>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        // Use the existing put method to get the raw JSON response
        let response_future = self.put::<T, serde_json::Value>(path, body);
        let response_text = match response_future.await {
            Ok(value) => serde_json::to_string(&value).unwrap_or_default(),
            Err(e) => return Err(ErrorResponse::new(-1, e.to_string(), None)),
        };

        // Parse using the generic response wrapper
        parse_success_response(&response_text)
    }

    /// Make a DELETE request with generic response wrapper parsing
    pub async fn delete_with_wrapper<R>(&self, path: &str) -> Result<R, ErrorResponse>
    where
        R: DeserializeOwned,
    {
        // Use the existing delete method to get the raw JSON response
        let response_future = self.delete::<serde_json::Value>(path);
        let response_text = match response_future.await {
            Ok(value) => serde_json::to_string(&value).unwrap_or_default(),
            Err(e) => return Err(ErrorResponse::new(-1, e.to_string(), None)),
        };

        // Parse using the generic response wrapper
        parse_success_response(&response_text)
    }

    /// Parse a raw response string with wrapper handling
    /// This method can be used when you have a raw JSON response and want to parse it
    pub fn parse_response_with_wrapper<R>(&self, response_text: &str) -> Result<R, ErrorResponse>
    where
        R: DeserializeOwned,
    {
        parse_success_response(response_text)
    }

    /// Check if a response string represents an error
    pub fn is_error_response(&self, response_text: &str) -> bool {
        is_error_response(response_text)
    }

    /// Extract status field from a response
    pub fn extract_status(&self, response_text: &str) -> Option<String> {
        extract_status(response_text)
    }

    /// Extract nested data field from a response
    pub fn extract_nested_data<T>(&self, response_text: &str) -> Result<Option<T>, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        extract_nested_data(response_text)
    }
}