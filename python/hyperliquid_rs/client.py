"""Hyperliquid SDK - High-performance Python client with Rust core"""

import json
from typing import TYPE_CHECKING, Any, Optional, cast

if TYPE_CHECKING:
    # Type stubs for the Rust module (these won't exist at runtime but help mypy)
    class PyInfoClient:
        @staticmethod
        def with_default_config(base_url: str) -> "PyInfoClient": ...
        def meta(self, arg: None) -> str: ...
        def user_state(self, address: str, arg: None) -> str: ...
        def open_orders(self, address: str, arg: None) -> str: ...
        def frontend_open_orders(self, address: str, arg: None) -> str: ...
        def l2_book(self, coin: str, arg: None) -> str: ...
        def candles_snapshot(
            self, coin: str, interval: str, dex: Optional[str]
        ) -> str: ...
        def all_mids(self, arg: None) -> str: ...
        def user_staking_summary(self, address: str, arg: None) -> str: ...
        def user_staking_delegations(self, address: str, arg: None) -> str: ...
        def user_staking_rewards(self, address: str, arg: None) -> str: ...

    class PyExchangeClient:
        @staticmethod
        def with_default_config(
            config: "PyExchangeClientConfig"
        ) -> "PyExchangeClient": ...
        def place_order(
            self, order_data: dict[str, Any], signing_key: Optional[str]
        ) -> str: ...
        def cancel_order(
            self, cancel_data: dict[str, Any], signing_key: Optional[str]
        ) -> str: ...
        def get_open_orders(self, coin: str) -> str: ...

    class PyExchangeClientConfig:
        def __init__(self, base_url: str, max_retries: int, timeout: int): ...

    # Mock module for type checking
    _hyperliquid_rs = type('_hyperliquid_rs', (), {
        'PyInfoClient': PyInfoClient,
        'PyExchangeClient': PyExchangeClient,
        'PyExchangeClientConfig': PyExchangeClientConfig,
    })()

from .errors import HyperliquidError
from .types import (
    Delegation,
    DelegatorEvent,
    DelegatorHistory,
    DelegatorSummary,
    MetaResponse,
    Order,
    OrderType,
    OrderWire,
    RewardEvent,
    StakingRewards,
    StakingSummary,
    TriggerCondition,
    UserStateResponse,
)


def _get_exchange_clients():
    """Helper function to get exchange client classes (Rust or fallback)"""
    try:
        import _hyperliquid_rs as _rust_module  # type: ignore
        PyExchangeClient = _rust_module.PyExchangeClient
        PyExchangeClientConfig = _rust_module.PyExchangeClientConfig
    except ImportError:
        from ._fallback import PyExchangeClient, PyExchangeClientConfig
        # Add the testnet method to fallback config
        if not hasattr(PyExchangeClientConfig, 'testnet'):
            def testnet_method(cls, address: str):
                config_dict = {"base_url": "https://api.hyperliquid-testnet.xyz"}
                return config_dict
            PyExchangeClientConfig.testnet = classmethod(testnet_method)

    return PyExchangeClient, PyExchangeClientConfig


class HyperliquidClient:
    """High-performance Hyperliquid client with Rust core"""

    def __init__(
        self,
        base_url: str = "https://api.hyperliquid.xyz",
        config: Optional[dict[str, Any]] = None,
    ):
        """Initialize the client

        Args:
            base_url: API base URL (mainnet/testnet/local)
            config: Configuration dictionary for HTTP client
        """
        try:
            # Try to import compiled Rust module (available after maturin build)
            import _hyperliquid_rs as _rust_module  # type: ignore
            PyInfoClient = _rust_module.PyInfoClient
            PyExchangeClient = _rust_module.PyExchangeClient
            PyExchangeClientConfig = _rust_module.PyExchangeClientConfig
        except ImportError:
            # Fall back to pure Python implementation if Rust module is not available
            try:
                from ._fallback import (
                    PyExchangeClient,
                    PyExchangeClientConfig,
                    PyInfoClient,
                )
                # Use a fallback config class
                class PyExchangeClientConfig:
                    def __init__(self, base_url: str, max_retries: int = 3, timeout: int = 30):
                        self.base_url = base_url
                        self.max_retries = max_retries
                        self.timeout = timeout
            except ImportError as e:
                raise ImportError(
                    "Neither Rust module nor fallback implementation available. "
                    "Please build with: maturin develop or ensure fallback is available"
                ) from e

        self._info_client = PyInfoClient.with_default_config(base_url, config)
        self._client = self._info_client  # For backward compatibility with tests

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

    def get_open_orders(self, address: str) -> list[dict[str, Any]]:
        """Get open orders for user"""
        try:
            response = self._info_client.open_orders(address, None)
            return cast(list[dict[str, Any]], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get open orders: {e}") from e

    def get_frontend_open_orders(self, address: str) -> list[dict[str, Any]]:
        """Get frontend open orders for user with enhanced UI details"""
        try:
            response = self._info_client.frontend_open_orders(address, None)
            return cast(list[dict[str, Any]], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get frontend open orders: {e}") from e

    def place_order(self, order: OrderWire) -> dict[str, Any]:
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
    ) -> dict[str, Any]:
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
    ) -> dict[str, Any]:
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

    def place_order_high_level(self, order: Order) -> dict[str, Any]:
        """Place an order using the high-level Order model

        Args:
            order: Order instance with float values and high-level interface

        Returns:
            Order response from API
        """
        # Convert high-level Order to OrderWire
        wire = order.to_wire()
        return self.place_order(wire)

    def cancel_order(self, coin: str, oid: int) -> dict[str, Any]:
        """Cancel an order"""
        try:
            cancel_data = {"coin": coin, "oid": oid}
            return self.exchange_cancel_order(cancel_data)
        except Exception as e:
            raise HyperliquidError(f"Failed to cancel order: {e}") from e

    def cancel_order_by_cloid(self, coin: str, cloid: str) -> dict[str, Any]:
        """Cancel an order by client order ID"""
        try:
            cancel_data = {"coin": coin, "cloid": cloid}
            return self.exchange_cancel_order(cancel_data)
        except Exception as e:
            raise HyperliquidError(f"Failed to cancel order by cloid: {e}") from e

    def get_l2_book(self, coin: str) -> dict[str, Any]:
        """Get L2 order book snapshot"""
        try:
            response = self._info_client.l2_book(coin, None)
            return cast(dict[str, Any], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get L2 book: {e}") from e

    def get_candles_snapshot(
        self, coin: str, interval: str, dex: Optional[str] = None
    ) -> dict[str, Any]:
        """Get candlestick data snapshot (OHLCV)"""
        try:
            response = self._info_client.candles_snapshot(coin, interval, dex)
            return cast(dict[str, Any], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get candles snapshot: {e}") from e

    def get_all_mids(self) -> dict[str, Any]:
        """Get all mid prices"""
        try:
            response = self._info_client.all_mids(None)
            return cast(dict[str, Any], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get all mids: {e}") from e

    # Exchange API Methods
    # ===============================================================

    def exchange_place_order(self, order_data: dict[str, Any]) -> dict[str, Any]:
        """Place a new order via Exchange API
        Args:
            order_data: Order request data
        Returns:
            Order response from API
        """
        try:
            PyExchangeClient, PyExchangeClientConfig = _get_exchange_clients()

            # Create exchange client for testnet
            config = PyExchangeClientConfig.testnet(
                "0x0000000000000000000000000000000000000000"
            )
            exchange_client = PyExchangeClient.with_default_config(config)

            order_json = json.dumps(order_data)
            response = exchange_client.place_order(order_json)
            return cast(dict[str, Any], json.loads(response))
        except Exception as e:
            raise HyperliquidError(
                f"Failed to place order via Exchange API: {e}"
            ) from e

    def exchange_cancel_order(self, cancel_data: dict[str, Any]) -> dict[str, Any]:
        """Cancel an order via Exchange API
        Args:
            cancel_data: Cancel request data
        Returns:
            Cancel response from API
        """
        try:
            PyExchangeClient, PyExchangeClientConfig = _get_exchange_clients()

            # Create exchange client for testnet
            config = PyExchangeClientConfig.testnet(
                "0x0000000000000000000000000000000000000000"
            )
            exchange_client = PyExchangeClient.with_default_config(config)

            cancel_json = json.dumps(cancel_data)
            response = exchange_client.cancel_order(cancel_json)
            return cast(dict[str, Any], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to cancel order via Exchange API: {e}") from e

    def exchange_get_open_orders(self, coin: str) -> dict[str, Any]:
        """Get open orders via Exchange API
        Args:
            coin: Trading pair (e.g., "BTC", "ETH")
        Returns:
            Open orders response from API
        """
        try:
            PyExchangeClient, PyExchangeClientConfig = _get_exchange_clients()

            # Create exchange client for testnet
            config = PyExchangeClientConfig.testnet(
                "0x0000000000000000000000000000000000000000"
            )
            exchange_client = PyExchangeClient.with_default_config(config)

            response = exchange_client.get_open_orders(coin)
            return cast(dict[str, Any], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get open orders via Exchange API: {e}") from e

    def exchange_cancel_all_orders(self, coin: str) -> dict[str, Any]:
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

    # Staking Methods
    # ===============================================================

    def get_user_staking_summary(self, address: str) -> StakingSummary:
        """Get user's staking summary including total delegated and rewards
        Args:
            address: User wallet address
        Returns:
            StakingSummary with delegation and rewards information
        """
        try:
            response = self._info_client.user_staking_summary(address, None)
            data = json.loads(response)
            return StakingSummary(**data)
        except Exception as e:
            raise HyperliquidError(f"Failed to get user staking summary: {e}") from e

    def get_user_staking_delegations(self, address: str) -> list[Delegation]:
        """Get user's staking delegations
        Args:
            address: User wallet address
        Returns:
            List of Delegation objects
        """
        try:
            response = self._info_client.user_staking_delegations(address, None)
            data = json.loads(response)
            return [Delegation(**delegation) for delegation in data]
        except Exception as e:
            raise HyperliquidError(f"Failed to get user staking delegations: {e}") from e

    def get_user_staking_rewards(self, address: str) -> StakingRewards:
        """Get user's staking rewards
        Args:
            address: User wallet address
        Returns:
            StakingRewards with claimed and pending rewards
        """
        try:
            response = self._info_client.user_staking_rewards(address, None)
            data = json.loads(response)
            rewards_data = data.copy()
            rewards_data['history'] = [RewardEvent(**event) for event in data.get('history', [])]
            return StakingRewards(**rewards_data)
        except Exception as e:
            raise HyperliquidError(f"Failed to get user staking rewards: {e}") from e

    def get_delegator_history(self, address: str) -> DelegatorHistory:
        """Get comprehensive delegator history
        Args:
            address: User wallet address
        Returns:
            DelegatorHistory with all events and summary
        """
        try:
            response = self._info_client.delegator_history(address, None)
            data = json.loads(response)
            history_data = data.copy()
            history_data['events'] = [DelegatorEvent(**event) for event in data.get('events', [])]
            history_data['summary'] = DelegatorSummary(**data.get('summary', {}))
            return DelegatorHistory(**history_data)
        except Exception as e:
            raise HyperliquidError(f"Failed to get delegator history: {e}") from e

    def get_historical_orders(self, address: str) -> list[dict[str, Any]]:
        """Get historical orders for a user (up to 2000 orders)
        Args:
            address: User wallet address
        Returns:
            List of historical orders
        """
        try:
            response = self._info_client.historical_orders(address, None)
            return cast(list[dict[str, Any]], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get historical orders: {e}") from e

    def get_user_non_funding_ledger_updates(
        self, address: str, start_time: int, end_time: Optional[int] = None
    ) -> list[dict[str, Any]]:
        """Get non-funding ledger updates for a user
        Args:
            address: User wallet address
            start_time: Start time in milliseconds (epoch timestamp)
            end_time: Optional end time in milliseconds (epoch timestamp)
        Returns:
            List of ledger updates excluding funding payments
        """
        try:
            response = self._info_client.user_non_funding_ledger_updates(
                address, start_time, end_time
            )
            return cast(list[dict[str, Any]], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get user non-funding ledger updates: {e}") from e

    def get_portfolio(self, user: str) -> dict[str, Any]:
        """Get user's portfolio performance data

        Args:
            user: Onchain address in 42-character hexadecimal format

        Returns:
            Portfolio performance data including account value history,
            PnL history, trading volume metrics, and asset allocation
        """
        try:
            response = self._info_client.portfolio(user)
            return cast(dict[str, Any], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get portfolio: {e}") from e

    def get_vault_equities(self, user: str, dex: Optional[str] = None) -> list[dict[str, Any]]:
        """Get user's vault equity positions

        Args:
            user: Onchain address in 42-character hexadecimal format
            dex: Optional DEX parameter for cross-DEX queries

        Returns:
            List of vault equity positions with vault addresses and PnL values
        """
        try:
            response = self._info_client.user_vault_equities(user, dex)
            return cast(list[dict[str, Any]], json.loads(response))
        except Exception as e:
            raise HyperliquidError(f"Failed to get vault equities: {e}") from e
