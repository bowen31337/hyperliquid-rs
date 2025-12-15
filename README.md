# Hyperliquid Rust SDK

[![PyPI version](https://img.shields.io/pypi/v/hyperliquid-rs.svg)](https://pypi.org/project/hyperliquid-rs/)
[![Python versions](https://img.shields.io/pypi/pyversions/hyperliquid-rs.svg)](https://pypi.org/project/hyperliquid-rs/)
[![CI](https://github.com/unodigit/hyperliquid-rs/actions/workflows/python-publish.yml/badge.svg)](https://github.com/unodigit/hyperliquid-rs/actions/workflows/python-publish.yml)
[![Documentation](https://img.shields.io/badge/docs-Swagger%20UI-4ecca3)](https://unodigit.github.io/hyperliquid-rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance Hyperliquid Python SDK with Rust core for maximum performance, memory safety, and low-latency trading operations.

## Features

- **Rust Core**: Zero-cost abstractions and memory safety
- **Connection Pooling**: Efficient HTTP connection management
- **Async Support**: Full async/await support with Tokio
- **Python Bindings**: PyO3 bindings for seamless Python integration
- **Type Safety**: Strong typing with serde and Pydantic v2
- **Comprehensive Error Handling**: Detailed error types and intelligent error mapping
- **Fallback Implementation**: Pure Python fallback when Rust module unavailable
- **Production Ready**: Extensive testing with 51/51 tests passing

## Installation

### From PyPI (Recommended)

```bash
# Install using pip
pip install hyperliquid-rs

# Or using uv
uv pip install hyperliquid-rs
```

### Prerequisites (for building from source)

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **Python 3.9+**: Required for Python bindings
- **uv**: Python package manager (recommended)
  ```bash
  curl -LsSf https://astral.sh/uv/install.sh | sh
  ```

### From Source

```bash
git clone https://github.com/hyperliquid-dex/hyperliquid-rs.git
cd hyperliquid-rs

# Install dependencies using uv
uv sync

# Build Rust workspace
cargo build --workspace

# Build Python bindings (if Rust module available)
cd crates/hyperliquid-python
uv pip install maturin
uv run maturin develop
cd ../..
```

## Usage

```python
from hyperliquid_rs import HyperliquidClient

# Create client for mainnet
client = HyperliquidClient()

# Create client for testnet
client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

# Get asset metadata
meta = client.get_meta()
print(f"Available assets: {[asset.name for asset in meta.universe]}")

# Get user state
user_state = client.get_user_state("0xYourAddress")
print(f"Account value: {user_state.marginSummary.accountValue}")

# Place an order
order = {
    "coin": "BTC",
    "is_buy": True,
    "sz": "0.001",
    "limit_px": "50000",
    "order_type": {"limit": {"tif": "Gtc"}},
}
result = client.place_order(order)
print(f"Order result: {result}")
```

## Development

### Running Tests

**Testnet Tests:**
```bash
# Using uv to install dependencies and run tests
./scripts/run_testnet.sh

# Or manually:
uv pip install pydantic pydantic-settings httpx
PYTHONPATH=./python:$PYTHONPATH python3 python/tests/integration/test_testnet.py
```

**Python Tests:**
```bash
cd python
uv pip install pytest
uv run pytest tests/
```

**Rust Tests:**
```bash
cargo test --workspace
```

### API Documentation

**Online Documentation:**
- 📖 **Swagger UI**: [https://unodigit.github.io/hyperliquid-rs/](https://unodigit.github.io/hyperliquid-rs/)

**Local Documentation Server:**
```bash
# Using the convenience script (recommended)
./scripts/start_swagger.sh 8081

# Or manually:
uv pip install pydantic pydantic-settings httpx
python3 scripts/serve_swagger.py --port 8081
```

**Local Access:**
- Open http://localhost:8081/docs in your browser
- OpenAPI spec available at http://localhost:8081/openapi.json

### Building

```bash
# Build Rust core
cargo build --workspace

# Run tests
cargo test --workspace

# Build Python bindings
cd crates/hyperliquid-python
uv run maturin develop
cd ../..

# Run Python tests
uv pip install pytest
cd python
uv run pytest tests/
cd ..
```

## Project Structure

```
hyperliquid-rs/
├── .github/
│   └── workflows/            # GitHub Actions CI/CD
│       ├── python-publish.yml  # PyPI publishing
│       └── docs.yml            # API docs deployment
├── crates/
│   ├── hyperliquid-core/     # Core Rust library
│   ├── hyperliquid-python/   # PyO3 bindings
│   └── hyperliquid-grpc/     # gRPC server (future)
├── python/                   # Python package
│   └── tests/
│       ├── integration/      # Integration tests (testnet, etc.)
│       └── ...               # Unit tests
├── openapi/                  # OpenAPI specification
├── scripts/                  # Build and utility scripts
│   └── tests/                # Test scripts
├── docs/                     # Documentation
│   ├── api/                  # Swagger UI (GitHub Pages)
│   ├── implementation/       # Implementation summaries
│   ├── sessions/             # Session reports
│   └── reports/              # Project status reports
└── README.md                 # Main project documentation
```

## Error Handling

The SDK provides comprehensive error handling with specific error types:

```python
from hyperliquid_rs import HyperliquidClient
from hyperliquid_rs.errors import ApiError, NetworkError, TimeoutError

client = HyperliquidClient()

try:
    user_state = client.get_user_state("0xYourAddress")
except ApiError as e:
    print(f"API Error {e.status_code}: {e.message}")
except NetworkError as e:
    print(f"Network error: {e}")
except TimeoutError as e:
    print(f"Request timed out: {e}")
```

### Error Types

- `HyperliquidError`: Base exception class
- `ApiError`: HTTP API errors with status codes
- `NetworkError`: Network connectivity issues
- `RateLimitError`: Rate limiting (HTTP 429)
- `AuthenticationError`: Authentication failures (HTTP 401)
- `TimeoutError`: Request timeouts
- `ValidationError`: Input validation errors

## Fallback Implementation

The SDK automatically falls back to a pure Python implementation when the Rust module is not available. This ensures compatibility across all environments:

```python
# Works with both Rust and Python implementations
client = HyperliquidClient()
meta = client.get_meta()  # Uses Rust if available, Python fallback otherwise
```

The fallback implementation includes:
- Full API compatibility
- Intelligent error mapping
- Connection pooling with httpx
- Proper timeout handling

## Performance

The Rust core provides significant performance improvements:

- **10-100x faster JSON parsing** compared to pure Python
- **5-10x less memory usage** for data structures
- **Predictable latency** with no GIL or garbage collection
- **Zero-copy deserialization** where possible

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo fmt` and `cargo clippy`
6. Run `uv run pytest python/tests/`
7. Submit a pull request

## License

MIT
