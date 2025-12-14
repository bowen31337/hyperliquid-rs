#!/usr/bin/env python3
"""Test script to verify PyO3 bindings work correctly"""

import sys
import os

# Add the Python package to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

def test_imports():
    """Test that we can import the Python bindings"""
    print("Testing imports...")

    try:
        # Test direct Rust module import
        from hyperliquid_rs import PyHttpClient, PyHttpClientConfig, PyInfoClient, PyExchangeClient, PyExchangeClientConfig
        print("✓ Successfully imported PyO3 bindings")
    except ImportError as e:
        print(f"✗ Failed to import PyO3 bindings: {e}")
        return False

    try:
        # Test Python wrapper import
        from hyperliquid_rs import HyperliquidClient, MetaResponse, UserStateResponse
        print("✓ Successfully imported Python wrapper")
    except ImportError as e:
        print(f"✗ Failed to import Python wrapper: {e}")
        return False

    return True

def test_info_client():
    """Test InfoClient functionality"""
    print("\nTesting InfoClient...")

    try:
        from hyperliquid_rs import HyperliquidClient

        # Create client
        client = HyperliquidClient()
        print("✓ Successfully created HyperliquidClient")

        # Test meta endpoint (should hit the API)
        try:
            meta = client.get_meta()
            print("✓ Successfully called get_meta()")
            print(f"  Meta response keys: {list(meta.__dict__.keys())}")
        except Exception as e:
            print(f"✗ get_meta() failed: {e}")

        return True

    except Exception as e:
        print(f"✗ InfoClient test failed: {e}")
        return False

def test_types():
    """Test type definitions"""
    print("\nTesting type definitions...")

    try:
        from hyperliquid_rs import OrderWire, OrderType, TriggerCondition

        # Test OrderWire creation
        order = OrderWire(
            coin="BTC",
            is_buy=True,
            sz="0.001",
            limitPx="50000",
            orderType=OrderType.LIMIT,
        )
        print("✓ Successfully created OrderWire")
        print(f"  Order: {order}")

        # Test TriggerCondition
        trigger = TriggerCondition.MARK
        print(f"✓ Successfully created TriggerCondition: {trigger}")

        return True

    except Exception as e:
        print(f"✗ Type test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("Hyperliquid Rust SDK - PyO3 Bindings Test")
    print("=" * 50)

    success = True
    success &= test_imports()
    success &= test_info_client()
    success &= test_types()

    print("\n" + "=" * 50)
    if success:
        print("✓ All tests passed!")
        return 0
    else:
        print("✗ Some tests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())