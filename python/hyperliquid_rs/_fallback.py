"""
Fallback implementation for when the Rust module is not available.

This module provides a basic HTTP-based implementation that mimics the Rust interface
to allow the Python SDK to function even without the compiled Rust extension.
"""

from typing import Any, Optional

import httpx
from .errors import (
    ApiError,
    NetworkError,
    RateLimitError,
    TimeoutError,
)


class PyInfoClient:
    """Fallback implementation of InfoClient using pure Python."""

    def __init__(self, base_url: str, config: Optional[dict] = None):
        self._base_url = base_url
        self._config = config or {}

        # Extract configuration values
        timeout = (
            self._config.get('connect_timeout_ms', 30000) / 1000
        )  # Convert ms to seconds
        max_connections = self._config.get('max_connections_per_host', 10)

        # Create HTTP client with configuration
        limits = httpx.Limits(
            max_keepalive_connections=max_connections,
            max_connections=max_connections * 2,
        )
        self.client = httpx.Client(timeout=timeout, limits=limits)

    def _handle_response(self, response: httpx.Response) -> str:
        """Handle HTTP response with proper error mapping."""
        try:
            response.raise_for_status()
            return response.text
        except httpx.HTTPStatusError as e:
            # Map HTTP status codes to appropriate SDK errors
            if e.response.status_code == 429:
                raise RateLimitError(
                    e.response.status_code,
                    "Rate limit exceeded. Please retry later.",
                    {"retry-after": e.response.headers.get("retry-after", "unknown")}
                ) from e
            elif e.response.status_code == 401:
                raise ApiError(
                    e.response.status_code,
                    "Authentication failed. Check your credentials.",
                    {"error": "unauthorized"}
                ) from e
            elif e.response.status_code >= 500:
                raise ApiError(
                    e.response.status_code,
                    f"Server error: {e.response.reason_phrase}",
                    {"error": "server_error"}
                ) from e
            else:
                raise ApiError(
                    e.response.status_code,
                    f"HTTP error: {e.response.reason_phrase}",
                    {"error": "http_error"}
                ) from e
        except httpx.TimeoutException as e:
            raise TimeoutError(f"Request timed out: {e}") from e
        except httpx.NetworkError as e:
            raise NetworkError(f"Network error: {e}") from e

    def base_url(self) -> str:
        """Return the base URL."""
        return self._base_url

    def config(self):
        """Return client configuration."""
        class Config:
            def __init__(self, config_dict):
                self.max_connections_per_host = config_dict.get(
                    'max_connections_per_host', 10
                )
                self.connect_timeout_ms = config_dict.get(
                    'connect_timeout_ms', 30000
                )

        return Config(self._config)

    @staticmethod
    def with_default_config(
        base_url: str, config: Optional[dict] = None
    ) -> "PyInfoClient":
        """Create client with default config."""
        return PyInfoClient(base_url, config)

    def meta(self, arg: None) -> str:
        """Get market metadata."""
        response = self.client.post(f"{self._base_url}/info", json={"type": "meta"})
        return self._handle_response(response)

    def user_state(self, address: str, arg: None) -> str:
        """Get user state."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "clearinghouseState",
            "user": address
        })
        return self._handle_response(response)

    def open_orders(self, address: str, arg: None) -> str:
        """Get open orders."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "openOrders",
            "user": address
        })
        response.raise_for_status()
        return response.text

    def frontend_open_orders(self, address: str, arg: None) -> str:
        """Get frontend open orders."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "frontendOpenOrders",
            "user": address
        })
        response.raise_for_status()
        return response.text

    def l2_book(self, coin: str, arg: None) -> str:
        """Get L2 order book."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "l2Book",
            "coin": coin
        })
        response.raise_for_status()
        return response.text

    def candles_snapshot(
        self, coin: str, interval: str, dex: Optional[str]
    ) -> str:
        """Get candles snapshot."""
        req = {
            "coin": coin,
            "interval": interval
        }
        if dex:
            req["dex"] = dex
        data = {
            "type": "candleSnapshot",
            "req": req
        }
        response = self.client.post(f"{self._base_url}/info", json=data)
        response.raise_for_status()
        return response.text

    def all_mids(self, arg: None) -> str:
        """Get all mids."""
        response = self.client.post(
            f"{self._base_url}/info", json={"type": "allMids"}
        )
        response.raise_for_status()
        return response.text

    def user_staking_summary(self, address: str, arg: None) -> str:
        """Get user staking summary."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "delegatorSummary",
            "user": address
        })
        response.raise_for_status()
        return response.text

    def user_staking_delegations(self, address: str, arg: None) -> str:
        """Get user staking delegations."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "delegations",
            "user": address
        })
        response.raise_for_status()
        return response.text

    def user_staking_rewards(self, address: str, arg: None) -> str:
        """Get user staking rewards."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "delegatorRewards",
            "user": address
        })
        response.raise_for_status()
        return response.text


class PyExchangeClient:
    """Fallback implementation of ExchangeClient using pure Python."""

    def __init__(self, config: dict[str, Any]):
        self._base_url = config.get("base_url", "https://api.hyperliquid.xyz")
        self.client = httpx.Client(timeout=30.0)

    @staticmethod
    def with_default_config(config: dict[str, Any]) -> "PyExchangeClient":
        """Create client with default config."""
        return PyExchangeClient(config)

    def place_order(
        self, order_data: dict[str, Any], signing_key: Optional[str] = None
    ) -> str:
        """Place an order."""
        # Note: This is a simplified implementation
        # Real implementation would need proper signing
        response = self.client.post(f"{self._base_url}/exchange", json=order_data)
        response.raise_for_status()
        return response.text

    def cancel_order(
        self, order_data: dict[str, Any], signing_key: Optional[str] = None
    ) -> str:
        """Cancel an order."""
        response = self.client.post(f"{self._base_url}/exchange", json=order_data)
        response.raise_for_status()
        return response.text

    def get_open_orders(self, address: str) -> str:
        """Get open orders."""
        response = self.client.post(f"{self._base_url}/info", json={
            "type": "openOrders",
            "user": address
        })
        response.raise_for_status()
        return response.text

    def cancel_all_orders(
        self, data: dict[str, Any], signing_key: Optional[str] = None
    ) -> str:
        """Cancel all orders."""
        response = self.client.post(f"{self._base_url}/exchange", json=data)
        response.raise_for_status()
        return response.text


class PyExchangeClientConfig:
    """Fallback implementation of ExchangeClientConfig."""

    def __init__(self, base_url: str, max_retries: int = 3, timeout: int = 30):
        self._base_url = base_url
        self.max_retries = max_retries
        self.timeout = timeout

    @classmethod
    def testnet(cls, address: str):
        """Create testnet config."""
        return {"base_url": "https://api.hyperliquid-testnet.xyz", "address": address}


# Create module-level instances that mimic the Rust module
info = PyInfoClient
exchange = PyExchangeClient
