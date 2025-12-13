"""Hyperliquid SDK - High-performance Python client with Rust core"""

from typing import Any, Dict, Optional, List
import json

from .types import (
    MetaResponse, UserStateResponse, OrderWire, NewOrder, OrderStatus, Order,
    OrderType, TriggerCondition, PegPriceType
)
from .errors import HyperliquidError


class HyperliquidClient:
    """High-performance Hyperliquid client with Rust core"""

    def __init__(
        self,
        base_url: str = "https://api.hyperliquid.xyz",
        config: Optional[Dict[str, Any]] = None,
    ):
        """Initialize the client

        Args:
            base_url: API base URL (mainnet/testnet/local)
            config: Configuration dictionary for HTTP client
        """
        try:
            from hyperliquid_rs import PyInfoClient
        except ImportError as e:
            raise ImportError(
                "hyperliquid-rs native module not found. "
                "Please build with: maturin develop"
            ) from e

        self._info_client = PyInfoClient.with_default_config(base_url)

    def get_meta(self) -> MetaResponse:
        """Get asset metadata"""
        try:
            response = self._info_client.meta(None)
            data = json.loads(response)
            return MetaResponse(**data)
        except Exception as e:
            raise HyperliquidError(f"Failed to get meta: {e}") from e

    def get_user_state(self, address: str) -> UserStateResponse:
        """Get user state"""
        try:
            response = self._info_client.user_state(address, None)
            data = json.loads(response)
            return UserStateResponse(**data)
        except Exception as e:
            raise HyperliquidError(f"Failed to get user state: {e}") from e

    def get_open_orders(self, address: str) -> List[Dict[str, Any]]:
        """Get open orders for user"""
        try:
            response = self._info_client.open_orders(address, None)
            return json.loads(response)
        except Exception as e:
            raise HyperliquidError(f"Failed to get open orders: {e}") from e

    def place_order(self, order: OrderWire) -> Dict[str, Any]:
        """Place a new order using OrderWire format"""
        try:
            # Convert OrderWire to exchange format
            order_data = {
                "coin": order.coin,
                "is_buy": order.is_buy,
                "sz": order.sz,
                "limit_px": order.limitPx,
                "order_type": order.orderType.value,
                "reduce_only": order.reduceOnly or False,
                "cloid": order.cloid,
            }

            if order.isTrigger and order.triggerCondition:
                order_data.update({
                    "is_trigger": True,
                    "trigger_px": order.triggerPx,
                    "trigger_condition": order.triggerCondition.value,
                })

            return self.exchange_place_order(order_data)
        except Exception as e:
            raise HyperliquidError(f"Failed to place order: {e}") from e

    def place_limit_order(
        self,
        coin: str,
        is_buy: bool,
        sz: str,
        limit_px: str,
        cloid: Optional[str] = None,
        reduce_only: bool = False,
    ) -> Dict[str, Any]:
        """Place a limit order

        Args:
            coin: Trading pair (e.g., "BTC", "ETH")
            is_buy: True for buy order, False for sell order
            sz: Order size
            limit_px: Limit price
            cloid: Client order ID (optional)
            reduce_only: Whether this is a reduce-only order (default: False)

        Returns:
            Order response from API
        """
        order = OrderWire(
            coin=coin,
            is_buy=is_buy,
            sz=sz,
            limitPx=limit_px,
            orderType=OrderType.LIMIT,
            reduceOnly=reduce_only,
            cloid=cloid,
        )
        return self.place_order(order)

    def place_trigger_order(
        self,
        coin: str,
        is_buy: bool,
        sz: str,
        trigger_px: str,
        limit_px: str,
        condition: TriggerCondition,
        cloid: Optional[str] = None,
        reduce_only: bool = False,
    ) -> Dict[str, Any]:
        """Place a trigger order

        Args:
            coin: Trading pair (e.g., "BTC", "ETH")
            is_buy: True for buy order, False for sell order
            sz: Order size
            trigger_px: Trigger price
            limit_px: Limit price when triggered
            condition: Trigger condition (mark, index, last)
            cloid: Client order ID (optional)
            reduce_only: Whether this is a reduce-only order (default: False)

        Returns:
            Order response from API
        """
        order = OrderWire(
            coin=coin,
            is_buy=is_buy,
            sz=sz,
            limitPx=limit_px,
            orderType=OrderType.TRIGGER,
            isTrigger=True,
            triggerCondition=condition,
            triggerPx=trigger_px,
            reduceOnly=reduce_only,
            cloid=cloid,
        )
        return self.place_order(order)

    def place_order_high_level(self, order: Order) -> Dict[str, Any]:
        """Place an order using the high-level Order model

        Args:
            order: Order instance with float values and high-level interface

        Returns:
            Order response from API
        """
        # Convert high-level Order to OrderWire
        wire = order.to_wire()
        return self.place_order(wire)

    def cancel_order(self, coin: str, oid: int) -> Dict[str, Any]:
        """Cancel an order"""
        try:
            cancel_data = {"coin": coin, "oid": oid}
            return self.exchange_cancel_order(cancel_data)
        except Exception as e:
            raise HyperliquidError(f"Failed to cancel order: {e}") from e

    def cancel_order_by_cloid(self, coin: str, cloid: str) -> Dict[str, Any]:
        """Cancel an order by client order ID"""
        try:
            cancel_data = {"coin": coin, "cloid": cloid}
            return self.exchange_cancel_order(cancel_data)
        except Exception as e:
            raise HyperliquidError(f"Failed to cancel order by cloid: {e}") from e

    def get_l2_book(self, coin: str) -> Dict[str, Any]:
        """Get L2 order book snapshot"""
        try:
            response = self._info_client.l2_book(coin)
            return json.loads(response)
        except Exception as e:
            raise HyperliquidError(f"Failed to get L2 book: {e}") from e

    def get_candles_snapshot(self, coin: str, interval: str, dex: Optional[str] = None) -> Dict[str, Any]:
        """Get candlestick data snapshot (OHLCV)"""
        try:
            response = self._info_client.candles_snapshot(coin, interval, dex)
            return json.loads(response)
        except Exception as e:
            raise HyperliquidError(f"Failed to get candles snapshot: {e}") from e

    def get_all_mids(self) -> Dict[str, Any]:
        """Get all mid prices"""
        try:
            response = self._info_client.all_mids(None)
            return json.loads(response)
        except Exception as e:
            raise HyperliquidError(f"Failed to get all mids: {e}") from e

    # Exchange API Methods
    # ===============================================================

    def exchange_place_order(self, order_data: Dict[str, Any]) -> Dict[str, Any]:
        """Place a new order via Exchange API
        Args:
            order_data: Order request data
        Returns:
            Order response from API
        """
        try:
            from hyperliquid_rs import PyExchangeClient, PyExchangeClientConfig

            # Create exchange client for testnet
            config = PyExchangeClientConfig.testnet("0x0000000000000000000000000000000000000000")
            exchange_client = PyExchangeClient.new(config)

            order_json = json.dumps(order_data)
            response = exchange_client.place_order(order_json)
            return json.loads(response)
        except Exception as e:
            raise HyperliquidError(f"Failed to place order via Exchange API: {e}") from e

    def exchange_cancel_order(self, cancel_data: Dict[str, Any]) -> Dict[str, Any]:
        """Cancel an order via Exchange API
        Args:
            cancel_data: Cancel request data
        Returns:
            Cancel response from API
        """
        try:
            from hyperliquid_rs import PyExchangeClient, PyExchangeClientConfig

            # Create exchange client for testnet
            config = PyExchangeClientConfig.testnet("0x0000000000000000000000000000000000000000")
            exchange_client = PyExchangeClient.new(config)

            cancel_json = json.dumps(cancel_data)
            response = exchange_client.cancel_order(cancel_json)
            return json.loads(response)
        except Exception as e:
            raise HyperliquidError(f"Failed to cancel order via Exchange API: {e}") from e

    def exchange_get_open_orders(self, coin: str) -> Dict[str, Any]:
        """Get open orders via Exchange API
        Args:
            coin: Trading pair (e.g., "BTC", "ETH")
        Returns:
            Open orders response from API
        """
        try:
            from hyperliquid_rs import PyExchangeClient, PyExchangeClientConfig

            # Create exchange client for testnet
            config = PyExchangeClientConfig.testnet("0x0000000000000000000000000000000000000000")
            exchange_client = PyExchangeClient.new(config)

            response = exchange_client.get_open_orders(coin)
            return json.loads(response)
        except Exception as e:
            raise HyperliquidError(f"Failed to get open orders via Exchange API: {e}") from e

    def exchange_cancel_all_orders(self, coin: str) -> Dict[str, Any]:
        """Cancel all orders for a coin via Exchange API
        Args:
            coin: Trading pair (e.g., "BTC", "ETH")
        Returns:
            Cancel response from API
        """
        return self.exchange_cancel_order({
            "coin": coin,
            "type": "cancelAll"
        })