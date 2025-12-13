"""Custom exceptions for Hyperliquid SDK"""

from typing import Optional


class HyperliquidError(Exception):
    """Base exception for Hyperliquid SDK"""
    pass


class NetworkError(HyperliquidError):
    """Network-related error"""
    pass


class ApiError(HyperliquidError):
    """API error with status code and message"""

    def __init__(self, status_code: int, message: str, data: Optional[dict[str, str]] = None):
        self.status_code = status_code
        self.message = message
        self.data = data
        super().__init__(f"HTTP {status_code}: {message}")


class RateLimitError(ApiError):
    """Rate limit exceeded"""
    pass


class AuthenticationError(ApiError):
    """Authentication error"""
    pass


class ValidationError(HyperliquidError):
    """Validation error for input parameters"""
    pass


class TimeoutError(HyperliquidError):
    """Request timeout error"""
    pass
