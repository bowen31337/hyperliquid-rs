"""Tests for Order Pydantic model"""


import pytest
from pydantic import ValidationError

from hyperliquid_rs.types import (
    Order,
    OrderType,
    OrderWire,
    PegPriceType,
    TriggerCondition,
)


class TestOrder:
    """Test the Order Pydantic model"""

    def test_order_creation_basic(self):
        """Test basic Order creation with valid data"""
        order = Order(
            coin="BTC",
            is_buy=True,
            size=0.1,
            limit_price=50000.0
        )

        assert order.coin == "BTC"
        assert order.is_buy is True
        assert order.size == 0.1
        assert order.limit_price == 50000.0
        assert order.order_type == OrderType.LIMIT
        assert order.reduce_only is False

    def test_order_creation_with_all_fields(self):
        """Test Order creation with all optional fields"""
        order = Order(
            coin="ETH",
            is_buy=False,
            size=1.5,
            limit_price=3000.0,
            order_type=OrderType.TRIGGER,
            reduce_only=True,
            client_order_id="test-order-123",
            peg_offset_value=0.01,
            peg_price_type=PegPriceType.MID,
            is_trigger=True,
            trigger_condition=TriggerCondition.MARK,
            trigger_price=3100.0
        )

        assert order.coin == "ETH"
        assert order.is_buy is False
        assert order.size == 1.5
        assert order.limit_price == 3000.0
        assert order.order_type == OrderType.TRIGGER
        assert order.reduce_only is True
        assert order.client_order_id == "test-order-123"
        assert order.peg_offset_value == 0.01
        assert order.peg_price_type == PegPriceType.MID
        assert order.is_trigger is True
        assert order.trigger_condition == TriggerCondition.MARK
        assert order.trigger_price == 3100.0

    def test_order_validation_positive_size(self):
        """Test that size must be positive"""
        with pytest.raises(ValidationError, match="Input should be greater than 0"):
            Order(
                coin="BTC",
                is_buy=True,
                size=0.0,  # Invalid: zero size
                limit_price=50000.0
            )

        with pytest.raises(ValidationError, match="Input should be greater than 0"):
            Order(
                coin="BTC",
                is_buy=True,
                size=-0.1,  # Invalid: negative size
                limit_price=50000.0
            )

    def test_order_validation_positive_limit_price(self):
        """Test that limit_price must be positive"""
        with pytest.raises(ValidationError, match="Input should be greater than 0"):
            Order(
                coin="BTC",
                is_buy=True,
                size=0.1,
                limit_price=0.0  # Invalid: zero price
            )

        with pytest.raises(ValidationError, match="Input should be greater than 0"):
            Order(
                coin="BTC",
                is_buy=True,
                size=0.1,
                limit_price=-1000.0  # Invalid: negative price
            )

    def test_order_default_values(self):
        """Test default values for optional fields"""
        order = Order(
            coin="BTC",
            is_buy=True,
            size=0.1,
            limit_price=50000.0
        )

        assert order.order_type == OrderType.LIMIT
        assert order.reduce_only is False
        assert order.client_order_id is None
        assert order.peg_offset_value is None
        assert order.peg_price_type is None
        assert order.is_trigger is False
        assert order.trigger_condition is None
        assert order.trigger_price is None

    def test_order_to_wire_basic(self):
        """Test conversion from Order to OrderWire"""
        order = Order(
            coin="BTC",
            is_buy=True,
            size=0.1,
            limit_price=50000.0,
            client_order_id="test-123"
        )

        wire = order.to_wire()

        assert isinstance(wire, OrderWire)
        assert wire.coin == "BTC"
        assert wire.is_buy is True
        assert wire.sz == "0.1"
        assert wire.limitPx == "50000"
        assert wire.cloid == "test-123"
        assert wire.reduceOnly is False
        assert wire.orderType == OrderType.LIMIT

    def test_order_to_wire_precision(self):
        """Test that float to string conversion maintains precision"""
        order = Order(
            coin="BTC",
            is_buy=True,
            size=0.123456789,  # High precision
            limit_price=50000.123456789
        )

        wire = order.to_wire()

        # Should convert to appropriate string representation
        assert isinstance(wire.sz, str)
        assert isinstance(wire.limitPx, str)

        # Should be convertible back to float
        assert float(wire.sz) == 0.123456789
        assert float(wire.limitPx) == 50000.123456789

    def test_order_to_wire_optional_fields(self):
        """Test conversion with optional trigger fields"""
        order = Order(
            coin="ETH",
            is_buy=False,
            size=1.0,
            limit_price=3000.0,
            is_trigger=True,
            trigger_condition=TriggerCondition.LAST,
            trigger_price=3100.0,
            peg_offset_value=0.005,
            peg_price_type=PegPriceType.ORACLE
        )

        wire = order.to_wire()

        assert wire.isTrigger is True
        assert wire.triggerCondition == TriggerCondition.LAST
        assert wire.triggerPx == "3100"
        assert wire.pegOffsetValue == "0.005"
        assert wire.pegPriceType == PegPriceType.ORACLE

    def test_order_from_wire_basic(self):
        """Test creation of Order from OrderWire"""
        wire = OrderWire(
            coin="BTC",
            is_buy=True,
            sz="0.1",
            limitPx="50000",
            cloid="test-123"
        )

        order = Order.from_wire(wire)

        assert isinstance(order, Order)
        assert order.coin == "BTC"
        assert order.is_buy is True
        assert order.size == 0.1
        assert order.limit_price == 50000.0
        assert order.client_order_id == "test-123"

    def test_order_from_wire_optional_fields(self):
        """Test creation of Order from OrderWire with optional fields"""
        wire = OrderWire(
            coin="ETH",
            is_buy=False,
            sz="1.5",
            limitPx="3000",
            orderType=OrderType.TRIGGER,
            reduceOnly=True,
            isTrigger=True,
            triggerCondition=TriggerCondition.MARK,
            triggerPx="3100",
            pegOffsetValue="0.01",
            pegPriceType=PegPriceType.MID
        )

        order = Order.from_wire(wire)

        assert order.coin == "ETH"
        assert order.is_buy is False
        assert order.size == 1.5
        assert order.limit_price == 3000.0
        assert order.order_type == OrderType.TRIGGER
        assert order.reduce_only is True
        assert order.is_trigger is True
        assert order.trigger_condition == TriggerCondition.MARK
        assert order.trigger_price == 3100.0
        assert order.peg_offset_value == 0.01
        assert order.peg_price_type == PegPriceType.MID

    def test_order_round_trip_conversion(self):
        """Test that Order -> OrderWire -> Order preserves data"""
        original_order = Order(
            coin="BTC",
            is_buy=True,
            size=0.5,
            limit_price=45000.0,
            order_type=OrderType.LIMIT,
            reduce_only=False,
            client_order_id="round-trip-test",
            peg_offset_value=0.001,
            peg_price_type=PegPriceType.OPPOSITE,
            is_trigger=False
        )

        # Convert to wire and back
        wire = original_order.to_wire()
        restored_order = Order.from_wire(wire)

        # Compare all fields
        assert restored_order.coin == original_order.coin
        assert restored_order.is_buy == original_order.is_buy
        assert restored_order.size == original_order.size
        assert restored_order.limit_price == original_order.limit_price
        assert restored_order.order_type == original_order.order_type
        assert restored_order.reduce_only == original_order.reduce_only
        assert restored_order.client_order_id == original_order.client_order_id
        assert restored_order.peg_offset_value == original_order.peg_offset_value
        assert restored_order.peg_price_type == original_order.peg_price_type
        assert restored_order.is_trigger == original_order.is_trigger

    def test_order_with_trigger_condition(self):
        """Test Order with different trigger conditions"""
        for condition in TriggerCondition:
            order = Order(
                coin="BTC",
                is_buy=True,
                size=0.1,
                limit_price=50000.0,
                is_trigger=True,
                trigger_condition=condition,
                trigger_price=51000.0
            )

            wire = order.to_wire()
            assert wire.triggerCondition == condition

            restored = Order.from_wire(wire)
            assert restored.trigger_condition == condition

    def test_order_with_peg_price_types(self):
        """Test Order with different peg price types"""
        for peg_type in PegPriceType:
            order = Order(
                coin="ETH",
                is_buy=False,
                size=1.0,
                limit_price=3000.0,
                peg_offset_value=0.01,
                peg_price_type=peg_type
            )

            wire = order.to_wire()
            assert wire.pegPriceType == peg_type

            restored = Order.from_wire(wire)
            assert restored.peg_price_type == peg_type

    def test_order_json_serialization(self):
        """Test JSON serialization of Order"""
        order = Order(
            coin="BTC",
            is_buy=True,
            size=0.1,
            limit_price=50000.0,
            client_order_id="json-test"
        )

        # Should be JSON serializable
        import json
        json_str = json.dumps(order.dict())
        assert "BTC" in json_str
        assert "0.1" in json_str
        assert "50000" in json_str
        assert "json-test" in json_str

    def test_order_string_precision_handling(self):
        """Test that string precision is handled correctly"""
        # Test various float values and their string representations
        test_cases = [
            (0.1, "0.1"),
            (1.0, "1"),
            (0.123456789, "0.123456789"),
            (50000.0, "50000"),
            (50000.123456789, "50000.123456789"),
            (1e-8, "1e-08"),  # Very small number
            (1e8, "100000000"),  # Very large number
        ]

        for float_val, expected_str in test_cases:
            order = Order(
                coin="TEST",
                is_buy=True,
                size=float_val,
                limit_price=float_val
            )

            wire = order.to_wire()

            # Check that string representation is correct
            assert wire.sz == expected_str or wire.sz == f"{float_val:g}"
            assert wire.limitPx == expected_str or wire.limitPx == f"{float_val:g}"

            # Check round-trip conversion
            restored = Order.from_wire(wire)
            assert restored.size == float_val
            assert restored.limit_price == float_val

    def test_order_empty_peg_fields_handling(self):
        """Test handling of empty/None peg fields"""
        order = Order(
            coin="BTC",
            is_buy=True,
            size=0.1,
            limit_price=50000.0
            # peg_offset_value and peg_price_type are None
        )

        wire = order.to_wire()
        assert wire.pegOffsetValue is None
        assert wire.pegPriceType is None

        restored = Order.from_wire(wire)
        assert restored.peg_offset_value is None
        assert restored.peg_price_type is None

    def test_order_empty_trigger_fields_handling(self):
        """Test handling of empty/None trigger fields"""
        order = Order(
            coin="ETH",
            is_buy=False,
            size=1.0,
            limit_price=3000.0,
            is_trigger=False
            # trigger_condition and trigger_price are None
        )

        wire = order.to_wire()
        assert wire.isTrigger is False
        assert wire.triggerCondition is None
        assert wire.triggerPx is None

        restored = Order.from_wire(wire)
        assert restored.is_trigger is False
        assert restored.trigger_condition is None
        assert restored.trigger_price is None
