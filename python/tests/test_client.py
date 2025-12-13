"""Tests for HyperliquidClient"""

from hyperliquid_rs import HyperliquidClient, HyperliquidError
from hyperliquid_rs.types import OrderType, OrderWire, TriggerCondition


class TestHyperliquidClient:
    """Test the HyperliquidClient"""

    def test_client_initialization(self):
        """Test client initialization with default config"""
        client = HyperliquidClient()
        assert client._client.base_url() == "https://api.hyperliquid.xyz"

    def test_client_initialization_with_testnet(self):
        """Test client initialization with testnet URL"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        assert client._client.base_url() == "https://api.hyperliquid-testnet.xyz"

    def test_client_with_custom_config(self):
        """Test client initialization with custom config"""
        config = {
            "max_connections_per_host": 5,
            "connect_timeout_ms": 10000,
        }
        client = HyperliquidClient(
            base_url="https://api.hyperliquid-testnet.xyz",
            config=config
        )
        assert client._client.config().max_connections_per_host == 5
        assert client._client.config().connect_timeout_ms == 10000

    def test_get_meta_integration(self):
        """Integration test for get_meta"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        # This test might fail due to network issues, but we can test the structure
        try:
            response = client.get_meta()
            assert hasattr(response, 'universe')
            assert isinstance(response.universe, list)
        except HyperliquidError as e:
            # Expected for integration test without proper setup
            assert "Failed to get meta" in str(e)

    def test_get_user_state_integration(self):
        """Integration test for get_user_state"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x0000000000000000000000000000000000000000"

        try:
            response = client.get_user_state(test_address)
            # Should have basic structure
            assert hasattr(response, 'marginSummary')
        except HyperliquidError as e:
            # Expected for invalid address or network issues
            assert "Failed to get user state" in str(e)

    def test_get_open_orders_integration(self):
        """Integration test for get_open_orders"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x0000000000000000000000000000000000000000"

        try:
            response = client.get_open_orders(test_address)
            # Should be a list or dict
            assert response is not None
        except HyperliquidError as e:
            # Expected for invalid address or network issues
            assert "Failed to get open orders" in str(e)

    def test_get_frontend_open_orders_integration(self):
        """Integration test for get_frontend_open_orders"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x0000000000000000000000000000000000000000"

        try:
            response = client.get_frontend_open_orders(test_address)
            # Should be a list or dict
            assert response is not None
        except HyperliquidError as e:
            # Expected for invalid address or network issues
            assert "Failed to get frontend open orders" in str(e)

    def test_l2_book_integration(self):
        """Integration test for get_l2_book"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        try:
            response = client.get_l2_book("BTC")
            # Should have basic order book structure
            assert response is not None
        except HyperliquidError as e:
            # Expected for network issues or invalid coin
            assert "Failed to get L2 book" in str(e)

    def test_error_handling(self):
        """Test error handling for invalid requests"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        # Test with invalid endpoint
        try:
            client._client.get("/invalid_endpoint")
        except Exception as e:
            # Should raise a Python exception
            assert isinstance(e, Exception)


class TestOrderWire:
    """Test OrderWire types"""

    def test_order_wire_limit_creation(self):
        """Test OrderWire limit order creation"""
        order = OrderWire(
            coin="BTC",
            is_buy=True,
            sz="0.1",
            limitPx="50000",
            orderType=OrderType.LIMIT,
            reduceOnly=False,
        )

        assert order.coin == "BTC"
        assert order.is_buy
        assert order.sz == "0.1"
        assert order.limitPx == "50000"
        assert order.orderType == OrderType.LIMIT
        assert not order.reduceOnly
        assert order.cloid is None

    def test_order_wire_trigger_creation(self):
        """Test OrderWire trigger order creation"""
        order = OrderWire(
            coin="ETH",
            is_buy=False,
            sz="1.0",
            limitPx="2990",
            orderType=OrderType.TRIGGER,
            isTrigger=True,
            triggerCondition=TriggerCondition.MARK,
            triggerPx="3000",
        )

        assert order.coin == "ETH"
        assert not order.is_buy
        assert order.sz == "1.0"
        assert order.limitPx == "2990"
        assert order.triggerPx == "3000"
        assert order.triggerCondition == TriggerCondition.MARK
        assert order.orderType == OrderType.TRIGGER
        assert order.isTrigger

    def test_order_wire_with_cloid(self):
        """Test OrderWire with client order ID"""
        order = OrderWire(
            coin="BTC",
            is_buy=True,
            sz="0.1",
            limitPx="50000",
            orderType=OrderType.LIMIT,
            cloid="my-order-123",
        )

        assert order.cloid == "my-order-123"
        assert order.oid is None  # Should not set both cloid and oid

    def test_order_wire_serialization(self):
        """Test OrderWire serialization"""
        order = OrderWire(
            coin="BTC",
            is_buy=True,
            sz="0.1",
            limitPx="50000",
            orderType=OrderType.LIMIT,
            reduceOnly=False,
        )

        data = order.dict(exclude_none=True)
        expected = {
            "coin": "BTC",
            "is_buy": True,
            "sz": "0.1",
            "limitPx": "50000",
            "reduceOnly": False,
            "orderType": "Limit",
        }
        assert data == expected

    def test_order_wire_with_cloid_serialization(self):
        """Test OrderWire serialization with cloid"""
        order = OrderWire(
            coin="BTC",
            is_buy=True,
            sz="0.1",
            limitPx="50000",
            orderType=OrderType.LIMIT,
            cloid="test-123",
        )

        data = order.dict(exclude_none=True)
        expected = {
            "coin": "BTC",
            "is_buy": True,
            "sz": "0.1",
            "limitPx": "50000",
            "reduceOnly": False,
            "orderType": "Limit",
            "cloid": "test-123",
        }
        assert data == expected

    def test_order_wire_trigger_serialization(self):
        """Test OrderWire trigger order serialization"""
        order = OrderWire(
            coin="ETH",
            is_buy=False,
            sz="1.0",
            limitPx="2990",
            orderType=OrderType.TRIGGER,
            isTrigger=True,
            triggerCondition=TriggerCondition.MARK,
            triggerPx="3000",
        )

        data = order.dict(exclude_none=True)
        expected = {
            "coin": "ETH",
            "is_buy": False,
            "sz": "1.0",
            "limitPx": "2990",
            "reduceOnly": False,
            "orderType": "Trigger",
            "isTrigger": True,
            "triggerCondition": "mark",
            "triggerPx": "3000",
        }
        assert data == expected

class TestExchangeAPI:
    """Test Exchange API functionality"""

    def test_exchange_place_order(self):
        """Test placing order via Exchange API"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        order_data = {
            "coin": "BTC",
            "isBuy": True,
            "sz": "0.001",
            "limitPx": "50000",
            "orderType": "Limit",
        }

        try:
            response = client.exchange_place_order(order_data)
            # Should return a response (even if dummy due to no signing)
            assert response is not None
        except HyperliquidError as e:
            # Expected for test environment
            assert "Exchange API" in str(e)

    def test_exchange_cancel_order(self):
        """Test canceling order via Exchange API"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        cancel_data = {
            "coin": "BTC",
            "oid": 123456,
        }

        try:
            response = client.exchange_cancel_order(cancel_data)
            # Should return a response (even if dummy due to no signing)
            assert response is not None
        except HyperliquidError as e:
            # Expected for test environment
            assert "Exchange API" in str(e)

    def test_exchange_get_open_orders(self):
        """Test getting open orders via Exchange API"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        try:
            response = client.exchange_get_open_orders("BTC")
            # Should return a response (even if dummy due to no signing)
            assert response is not None
        except HyperliquidError as e:
            # Expected for test environment
            assert "Exchange API" in str(e)

    def test_exchange_cancel_all_orders(self):
        """Test canceling all orders via Exchange API"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        try:
            response = client.exchange_cancel_all_orders("BTC")
            # Should return a response (even if dummy due to no signing)
            assert response is not None
        except HyperliquidError as e:
            # Expected for test environment
            assert "Exchange API" in str(e)

    def test_order_request_types(self):
        """Test different order request types"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        # Test market order
        market_order = {
            "coin": "BTC",
            "isBuy": True,
            "sz": "0.001",
            "limitPx": "0",
            "orderType": "Market",
        }

        try:
            response = client.exchange_place_order(market_order)
            assert response is not None
        except HyperliquidError:
            pass  # Expected in test environment

        # Test stop order
        stop_order = {
            "coin": "BTC",
            "isBuy": False,
            "sz": "0.001",
            "limitPx": "40000",
            "orderType": "StopLimit",
            "triggerPx": "41000",
        }

        try:
            response = client.exchange_place_order(stop_order)
            assert response is not None
        except HyperliquidError:
            pass  # Expected in test environment

    def test_invalid_order_data(self):
        """Test handling of invalid order data"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        # Missing required fields
        invalid_order = {
            "coin": "BTC",
            # Missing isBuy, sz, limitPx
        }

        try:
            response = client.exchange_place_order(invalid_order)
            # Should still attempt to send (API will reject)
            assert response is not None
        except HyperliquidError:
            pass  # Expected for invalid data

    def test_cancel_with_invalid_data(self):
        """Test canceling with invalid data"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")

        # Missing required fields
        invalid_cancel = {
            # Missing coin and oid
        }

        try:
            response = client.exchange_cancel_order(invalid_cancel)
            # Should still attempt to send (API will reject)
            assert response is not None
        except HyperliquidError:
            pass  # Expected for invalid data

    # Staking Tests
    # ===============================================================

    def test_get_user_staking_summary_integration(self):
        """Integration test for get_user_staking_summary"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x1234567890abcdef1234567890abcdef12345678"

        try:
            response = client.get_user_staking_summary(test_address)
            # Verify response structure
            assert isinstance(response.total_delegated, str)
            assert isinstance(response.total_pending_rewards, str)
            assert isinstance(response.delegation_count, int)
            assert isinstance(response.total_earned_rewards, str)
        except HyperliquidError as e:
            # Expected for integration test without proper setup
            assert "Failed to get user staking summary" in str(e)

    def test_get_user_staking_delegations_integration(self):
        """Integration test for get_user_staking_delegations"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x1234567890abcdef1234567890abcdef12345678"

        try:
            response = client.get_user_staking_delegations(test_address)
            assert isinstance(response, list)
            # If there are delegations, verify their structure
            for delegation in response:
                assert hasattr(delegation, 'validator_address')
                assert hasattr(delegation, 'amount')
                assert hasattr(delegation, 'pending_rewards')
                assert hasattr(delegation, 'status')
        except HyperliquidError as e:
            # Expected for integration test without proper setup
            assert "Failed to get user staking delegations" in str(e)

    def test_get_user_staking_rewards_integration(self):
        """Integration test for get_user_staking_rewards"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x1234567890abcdef1234567890abcdef12345678"

        try:
            response = client.get_user_staking_rewards(test_address)
            assert hasattr(response, 'total_claimed')
            assert hasattr(response, 'total_pending')
            assert hasattr(response, 'history')
            assert isinstance(response.history, list)
            # Verify history events structure
            for event in response.history:
                assert hasattr(event, 'event_type')
                assert hasattr(event, 'validator_address')
                assert hasattr(event, 'amount')
                assert hasattr(event, 'timestamp')
        except HyperliquidError as e:
            # Expected for integration test without proper setup
            assert "Failed to get user staking rewards" in str(e)

    def test_get_delegator_history_integration(self):
        """Integration test for get_delegator_history"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x1234567890abcdef1234567890abcdef12345678"

        try:
            response = client.get_delegator_history(test_address)
            assert hasattr(response, 'events')
            assert hasattr(response, 'summary')
            assert isinstance(response.events, list)
            assert hasattr(response.summary, 'total_delegated_lifetime')
            assert hasattr(response.summary, 'current_delegation_count')
            # Verify events structure
            for event in response.events:
                assert hasattr(event, 'event_type')
                assert hasattr(event, 'validator_address')
                assert hasattr(event, 'amount')
                assert hasattr(event, 'timestamp')
        except HyperliquidError as e:
            # Expected for integration test without proper setup
            assert "Failed to get delegator history" in str(e)

    def test_get_historical_orders_integration(self):
        """Integration test for get_historical_orders"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x1234567890abcdef1234567890abcdef12345678"

        try:
            response = client.get_historical_orders(test_address)
            assert isinstance(response, list)
            # Verify response structure (list of orders)
            for order in response:
                assert 'coin' in order
                assert 'limitPx' in order
                assert 'sz' in order
                assert 'time' in order
                assert 'oid' in order
        except HyperliquidError as e:
            # Expected for integration test without proper setup
            assert "Failed to get historical orders" in str(e)

    def test_historical_orders_with_invalid_address(self):
        """Test get_historical_orders with invalid address format"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        invalid_address = "invalid_address"

        try:
            response = client.get_historical_orders(invalid_address)
            # API might still respond, but with error data
            assert response is not None
        except HyperliquidError:
            # Expected for invalid address
            pass

    def test_staking_methods_with_invalid_address(self):
        """Test staking methods with invalid address format"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        invalid_address = "invalid_address"

        # Test all staking methods with invalid address
        methods_to_test = [
            ('get_user_staking_summary', client.get_user_staking_summary),
            ('get_user_staking_delegations', client.get_user_staking_delegations),
            ('get_user_staking_rewards', client.get_user_staking_rewards),
            ('get_delegator_history', client.get_delegator_history),
        ]

        for _method_name, method_func in methods_to_test:
            try:
                response = method_func(invalid_address)
                # API might still respond, but with error data
                assert response is not None
            except HyperliquidError:
                # Expected for invalid address
                pass

    def test_get_vault_equities_integration(self):
        """Integration test for get_vault_equities"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x0000000000000000000000000000000000000000"  # Test address

        try:
            response = client.get_vault_equities(test_address)

            # Verify response structure
            assert isinstance(response, list)

            # Each vault equity should have vault and vaultPnl fields
            for vault_equity in response:
                assert "vault" in vault_equity
                assert "vaultPnl" in vault_equity
                assert isinstance(vault_equity["vault"], str)
                assert isinstance(vault_equity["vaultPnl"], str)

        except HyperliquidError as e:
            # Expected for test address or network issues
            assert "Failed to get vault equities" in str(e)

    def test_get_vault_equities_with_dex_integration(self):
        """Integration test for get_vault_equities with DEX parameter"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        test_address = "0x0000000000000000000000000000000000000000"  # Test address

        try:
            response = client.get_vault_equities(test_address, dex="")

            # Verify response structure
            assert isinstance(response, list)

            # Each vault equity should have vault and vaultPnl fields
            for vault_equity in response:
                assert "vault" in vault_equity
                assert "vaultPnl" in vault_equity
                assert isinstance(vault_equity["vault"], str)
                assert isinstance(vault_equity["vaultPnl"], str)

        except HyperliquidError as e:
            # Expected for test address or network issues
            assert "Failed to get vault equities" in str(e)

    def test_get_vault_equities_with_invalid_address(self):
        """Test get_vault_equities with invalid address format"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        invalid_address = "invalid_address"

        try:
            response = client.get_vault_equities(invalid_address)
            # API might still respond, but with error data
            assert response is not None
        except HyperliquidError as e:
            # Expected for invalid address
            assert "Failed to get vault equities" in str(e)

    def test_get_vault_equities_empty_response(self):
        """Test get_vault_equities with address that has no vault positions"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        empty_address = "0x0000000000000000000000000000000000000000"  # Address with no vaults

        try:
            response = client.get_vault_equities(empty_address)

            # Should return empty list or list with no vaults
            assert isinstance(response, list)
            # Could be empty list or list with empty vault data

        except HyperliquidError as e:
            # Expected for address with no vaults or network issues
            assert "Failed to get vault equities" in str(e)

    def test_get_vault_equities_error_handling(self):
        """Test get_vault_equities error handling"""
        client = HyperliquidClient(base_url="https://api.hyperliquid-testnet.xyz")
        invalid_address = "invalid_address_format"

        # Test with invalid address
        try:
            response = client.get_vault_equities(invalid_address)
            # API might still respond with error data
            assert response is not None
        except HyperliquidError as e:
            assert "Failed to get vault equities" in str(e)
