use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;

use hyperliquid_core::{HttpClient, HttpClientConfig, info::InfoClient, Result as HyperliquidResult, exchange::{ExchangeClient, ExchangeClientConfig}};
use ethers_core::types::Address;
use serde_json;

/// Python bindings for HttpClientConfig
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyHttpClientConfig {
    inner: HttpClientConfig,
}

#[pymethods]
impl PyHttpClientConfig {
    #[new]
    fn new() -> Self {
        Self {
            inner: HttpClientConfig::default(),
        }
    }

    #[getter]
    fn max_connections_per_host(&self) -> usize {
        self.inner.max_connections_per_host
    }

    #[setter]
    fn set_max_connections_per_host(&mut self, value: usize) {
        self.inner.max_connections_per_host = value;
    }

    #[getter]
    fn max_total_connections(&self) -> usize {
        self.inner.max_total_connections
    }

    #[setter]
    fn set_max_total_connections(&mut self, value: usize) {
        self.inner.max_total_connections = value;
    }

    #[getter]
    fn connect_timeout_ms(&self) -> u64 {
        self.inner.connect_timeout_ms
    }

    #[setter]
    fn set_connect_timeout_ms(&mut self, value: u64) {
        self.inner.connect_timeout_ms = value;
    }

    #[getter]
    fn request_timeout_ms(&self) -> u64 {
        self.inner.request_timeout_ms
    }

    #[setter]
    fn set_request_timeout_ms(&mut self, value: u64) {
        self.inner.request_timeout_ms = value;
    }

    #[getter]
    fn http2(&self) -> bool {
        self.inner.http2
    }

    #[setter]
    fn set_http2(&mut self, value: bool) {
        self.inner.http2 = value;
    }

    #[getter]
    fn compression(&self) -> bool {
        self.inner.compression
    }

    #[setter]
    fn set_compression(&mut self, value: bool) {
        self.inner.compression = value;
    }

    #[getter]
    fn keepalive(&self) -> bool {
        self.inner.keepalive
    }

    #[setter]
    fn set_keepalive(&mut self, value: bool) {
        self.inner.keepalive = value;
    }

    #[getter]
    fn keepalive_ms(&self) -> u64 {
        self.inner.keepalive_ms
    }

    #[setter]
    fn set_keepalive_ms(&mut self, value: u64) {
        self.inner.keepalive_ms = value;
    }

    #[getter]
    fn user_agent(&self) -> String {
        self.inner.user_agent.clone()
    }

    #[setter]
    fn set_user_agent(&mut self, value: String) {
        self.inner.user_agent = value;
    }

    #[getter]
    fn proxy_url(&self) -> Option<String> {
        self.inner.proxy_url.clone()
    }

    #[setter]
    fn set_proxy_url(&mut self, value: Option<String>) {
        self.inner.proxy_url = value;
    }
}

/// Python bindings for HttpClient
#[pyclass]
pub struct PyHttpClient {
    inner: HttpClient,
}

#[pymethods]
impl PyHttpClient {
    #[new]
    fn new(base_url: String, config: Option<PyHttpClientConfig>) -> PyResult<Self> {
        let config = config
            .map(|c| c.inner)
            .unwrap_or_else(|| HttpClientConfig::default());

        let inner = HttpClient::new(base_url, config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { inner })
    }

    #[staticmethod]
    fn with_default_config(base_url: String) -> PyResult<Self> {
        let inner = HttpClient::with_default_config(base_url)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { inner })
    }

    #[getter]
    fn base_url(&self) -> String {
        self.inner.base_url().to_string()
    }

    #[getter]
    fn config(&self) -> PyHttpClientConfig {
        PyHttpClientConfig {
            inner: self.inner.config().clone(),
        }
    }

    /// Make a POST request with JSON body
    #[pyo3(signature = (path, body))]
    fn post(&self, path: String, body: String) -> PyResult<String> {
        // This needs to be async, but PyO3 async support is complex
        // For now, we'll implement a sync wrapper
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let body_json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid JSON body: {}", e)))?;

            let result: serde_json::Value = self.inner.post(&path, &body_json).await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP request failed: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize response: {}", e)))
        })
    }

    /// Make a GET request
    fn get(&self, path: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let result: serde_json::Value = self.inner.get(&path).await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP request failed: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize response: {}", e)))
        })
    }

    /// Make a PUT request with JSON body
    #[pyo3(signature = (path, body))]
    fn put(&self, path: String, body: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let body_json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid JSON body: {}", e)))?;

            let result: serde_json::Value = self.inner.put(&path, &body_json).await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP request failed: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize response: {}", e)))
        })
    }

    /// Make a DELETE request
    fn delete(&self, path: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let result: serde_json::Value = self.inner.delete(&path).await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP request failed: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize response: {}", e)))
        })
    }
}

/// Python bindings for InfoClient
#[pyclass]
pub struct PyInfoClient {
    inner: InfoClient,
}

#[pymethods]
impl PyInfoClient {
    #[new]
    fn new(http_client: PyHttpClient) -> Self {
        Self {
            inner: InfoClient::new(http_client.inner),
        }
    }

    #[staticmethod]
    fn with_default_config(base_url: String) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let inner = InfoClient::with_default_config(&base_url).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to create Info client: {}", e)))?;
            Ok(Self { inner })
        })
    }

    /// Get asset metadata
    fn meta(&self, dex: Option<String>) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let dex_str = dex.unwrap_or_default();
            let result = self.inner.meta(&dex_str).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get meta: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize meta: {}", e)))
        })
    }

    /// Get user state
    fn user_state(&self, address: String, dex: Option<String>) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let dex_str = dex.unwrap_or_default();
            let result = self.inner.user_state(&address, &dex_str).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get user state: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize user state: {}", e)))
        })
    }

    /// Get open orders for user
    fn open_orders(&self, address: String, dex: Option<String>) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let dex_str = dex.unwrap_or_default();
            let result = self.inner.open_orders(&address, &dex_str).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get open orders: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize open orders: {}", e)))
        })
    }

    /// Get L2 orderbook snapshot
    fn l2_book(&self, coin: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let result = self.inner.l2_book(&coin).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get L2 book: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize L2 book: {}", e)))
        })
    }

    /// Get candlestick data snapshot (OHLCV)
    #[pyo3(signature = (coin, interval, dex=None))]
    fn candles_snapshot(&self, coin: String, interval: String, dex: Option<String>) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let dex_str = dex.unwrap_or_default();
            let result = self.inner.candles_snapshot(&coin, &interval, &dex_str).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get candles snapshot: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize candles: {}", e)))
        })
    }

    /// Get all mid prices
    fn all_mids(&self, dex: Option<String>) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let dex_str = dex.unwrap_or_default();
            let result = self.inner.all_mids(&dex_str).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get all mids: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize all mids: {}", e)))
        })
    }

    /// Get funding history for a coin (general funding rates, not user-specific)
    #[pyo3(signature = (coin, start_time=None, end_time=None, dex=None))]
    fn funding_history(&self, coin: String, start_time: Option<i64>, end_time: Option<i64>, dex: Option<String>) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let dex_str = dex.unwrap_or_default();
            let result = self.inner.funding_history(&coin, start_time, end_time, &dex_str).await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get funding history: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize funding history: {}", e)))
        })
    }
}

/// Convert HyperliquidError to Python exceptions
pub fn to_py_error(err: hyperliquid_core::HyperliquidError) -> PyErr {
    use hyperliquid_core::HyperliquidError;

    match err {
        HyperliquidError::Network(_) => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::Http { .. } => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::RateLimit(_) => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::Server { .. } => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::Client { .. } => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::InvalidUrl(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
        HyperliquidError::Json(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
        HyperliquidError::Timeout(_) => pyo3::exceptions::PyTimeoutError::new_err(err.to_string()),
        HyperliquidError::RetryExhausted { .. } => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::WebSocket(_) => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::Signing(_) => PyRuntimeError::new_err(err.to_string()),
        HyperliquidError::Config(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
        HyperliquidError::Unknown(_) => PyRuntimeError::new_err(err.to_string()),
    }
}

/// Python bindings for ExchangeClientConfig
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyExchangeClientConfig {
    inner: ExchangeClientConfig,
}

#[pymethods]
impl PyExchangeClientConfig {
    #[new]
    fn new(account: String) -> PyResult<Self> {
        let address = Address::from_str(&account)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid address: {}", e)))?;

        Ok(Self {
            inner: ExchangeClientConfig::testnet(address),
        })
    }

    #[staticmethod]
    fn mainnet(account: String) -> PyResult<Self> {
        let address = Address::from_str(&account)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid address: {}", e)))?;

        Ok(Self {
            inner: ExchangeClientConfig::mainnet(address),
        })
    }

    #[staticmethod]
    fn testnet(account: String) -> PyResult<Self> {
        let address = Address::from_str(&account)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid address: {}", e)))?;

        Ok(Self {
            inner: ExchangeClientConfig::testnet(address),
        })
    }

    #[getter]
    fn base_url(&self) -> String {
        self.inner.base_url.clone()
    }

    #[getter]
    fn account(&self) -> String {
        format!("{:?}", self.inner.account)
    }
}

/// Python bindings for ExchangeClient
#[pyclass]
pub struct PyExchangeClient {
    inner: ExchangeClient,
}

#[pymethods]
impl PyExchangeClient {
    #[new]
    fn new(config: PyExchangeClientConfig) -> Self {
        Self {
            inner: ExchangeClient::new(config.inner),
        }
    }

    /// Place a new order
    #[pyo3(signature = (order_json))]
    fn place_order(&self, order_json: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let order: hyperliquid_core::types::exchange::OrderRequest = serde_json::from_str(&order_json)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid order JSON: {}", e)))?;

            // Use dummy private key for now (signing not implemented)
            let dummy_key = [0u8; 32];

            let result = self.inner.place_order(order, &dummy_key).await
                .map_err(|e| PyRuntimeError::new_err(format!("Order placement failed: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize response: {}", e)))
        })
    }

    /// Cancel an order
    #[pyo3(signature = (cancel_json))]
    fn cancel_order(&self, cancel_json: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let cancel: hyperliquid_core::types::exchange::CancelRequest = serde_json::from_str(&cancel_json)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid cancel JSON: {}", e)))?;

            // Use dummy private key for now (signing not implemented)
            let dummy_key = [0u8; 32];

            let result = self.inner.cancel_order(cancel, &dummy_key).await
                .map_err(|e| PyRuntimeError::new_err(format!("Cancel failed: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize response: {}", e)))
        })
    }

    /// Get open orders
    #[pyo3(signature = (coin))]
    fn get_open_orders(&self, coin: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let request = hyperliquid_core::types::exchange::OpenOrdersRequest {
                coin,
            };

            let result = self.inner.get_open_orders(request).await
                .map_err(|e| PyRuntimeError::new_err(format!("Get open orders failed: {}", e)))?;

            serde_json::to_string(&result)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize response: {}", e)))
        })
    }
}

#[pymodule]
fn hyperliquid_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyHttpClientConfig>()?;
    m.add_class::<PyHttpClient>()?;
    m.add_class::<PyInfoClient>()?;
    m.add_class::<PyExchangeClientConfig>()?;
    m.add_class::<PyExchangeClient>()?;
    Ok(())
}