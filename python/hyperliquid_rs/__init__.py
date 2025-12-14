"""Hyperliquid SDK - High-performance Python client with Rust core"""

from .client import HyperliquidClient
from .errors import (
    ApiError,
    AuthenticationError,
    HyperliquidError,
    NetworkError,
    RateLimitError,
    TimeoutError,
    ValidationError,
)
from .types import (
    AssetMeta,
    AssetPosition,
    MarginSummary,
    MetaResponse,
    Order,
    Position,
    PositionDetails,
    UserStateResponse,
)

__version__ = "0.1.0"
__all__ = [
    "HyperliquidClient",
    "MetaResponse",
    "AssetMeta",
    "MarginSummary",
    "PositionDetails",
    "Position",
    "UserStateResponse",
    "AssetPosition",
    "Order",
    "HyperliquidError",
    "NetworkError",
    "ApiError",
    "RateLimitError",
    "AuthenticationError",
    "ValidationError",
    "TimeoutError",
]
