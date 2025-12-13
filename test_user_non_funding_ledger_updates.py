#!/usr/bin/env python3
"""
Test script for user_non_funding_ledger_updates feature (Feature #84)

This script tests the implementation of the user_non_funding_ledger_updates
method across all layers of the Hyperliquid Rust SDK.
"""

import json
import sys
from typing import Any, Dict, List, Optional

def test_rust_core():
    """Test the Rust core implementation"""
    print("Testing Rust core implementation...")

    try:
        # Import the Rust core module
        from hyperliquid_rs import PyInfoClient, PyHttpClient, PyHttpClientConfig

        # Test 1: Create InfoClient
        config = PyHttpClientConfig()
        http_client = PyHttpClient(config)
        info_client = PyInfoClient(http_client)
        print("✓ InfoClient created successfully")

        # Test 2: Test method exists
        assert hasattr(info_client, 'user_non_funding_ledger_updates'), "Method not found"
        assert hasattr(info_client, 'user_non_funding_ledger_updates_mainnet'), "Mainnet method not found"
        print("✓ Methods exist")

        # Test 3: Test method signature (should not crash)
        test_address = "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c"
        start_time = 1681923833000
        end_time = 1682010233000

        # This will fail due to network but should not crash due to method signature
        try:
            result = info_client.user_non_funding_ledger_updates(test_address, start_time, end_time)
            print("✓ Method executed (unexpected success)")
        except Exception as e:
            if "Failed to get user non-funding ledger updates" in str(e):
                print("✓ Method executed and failed as expected (network error)")
            else:
                print(f"✗ Unexpected error: {e}")
                return False

        # Test 4: Test without end_time
        try:
            result = info_client.user_non_funding_ledger_updates(test_address, start_time, None)
            print("✓ Method with None end_time executed (unexpected success)")
        except Exception as e:
            if "Failed to get user non-funding ledger updates" in str(e):
                print("✓ Method with None end_time executed and failed as expected")
            else:
                print(f"✗ Unexpected error: {e}")
                return False

        return True

    except ImportError as e:
        print(f"✗ Failed to import Rust module: {e}")
        return False
    except Exception as e:
        print(f"✗ Unexpected error in Rust test: {e}")
        return False

def test_python_client():
    """Test the Python client wrapper"""
    print("\nTesting Python client wrapper...")

    try:
        from hyperliquid_rs.client import HyperliquidClient

        # Test 1: Create client
        client = HyperliquidClient("https://api.hyperliquid.xyz")
        print("✓ Python client created successfully")

        # Test 2: Test method exists
        assert hasattr(client, 'get_user_non_funding_ledger_updates'), "Method not found"
        print("✓ Method exists in Python client")

        # Test 3: Test method signature
        test_address = "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c"
        start_time = 1681923833000
        end_time = 1682010233000

        try:
            result = client.get_user_non_funding_ledger_updates(test_address, start_time, end_time)
            print("✓ Method executed (unexpected success)")
        except Exception as e:
            if "Failed to get user non-funding ledger updates" in str(e):
                print("✓ Method executed and failed as expected (network error)")
            else:
                print(f"✗ Unexpected error: {e}")
                return False

        # Test 4: Test without end_time
        try:
            result = client.get_user_non_funding_ledger_updates(test_address, start_time, None)
            print("✓ Method with None end_time executed (unexpected success)")
        except Exception as e:
            if "Failed to get user non-funding ledger updates" in str(e):
                print("✓ Method with None end_time executed and failed as expected")
            else:
                print(f"✗ Unexpected error: {e}")
                return False

        return True

    except ImportError as e:
        print(f"✗ Failed to import Python client: {e}")
        return False
    except Exception as e:
        print(f"✗ Unexpected error in Python test: {e}")
        return False

def test_method_signature():
    """Test that method signature matches expected format"""
    print("\nTesting method signature format...")

    try:
        from hyperliquid_rs import PyInfoClient, PyHttpClient, PyHttpClientConfig

        config = PyHttpClientConfig()
        http_client = PyHttpClient(config)
        info_client = PyInfoClient(http_client)

        # Test the request format by examining the method
        test_address = "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c"
        start_time = 1681923833000
        end_time = 1682010233000

        # We can't directly test the request format without network access,
        # but we can verify the method accepts the correct parameters
        import inspect

        # Check method signature
        sig = inspect.signature(info_client.user_non_funding_ledger_updates)
        params = list(sig.parameters.keys())

        expected_params = ['self', 'user', 'start_time', 'end_time']
        if params == expected_params:
            print("✓ Method signature matches expected format")
            return True
        else:
            print(f"✗ Method signature mismatch. Expected {expected_params}, got {params}")
            return False

    except Exception as e:
        print(f"✗ Error testing method signature: {e}")
        return False

def main():
    """Run all tests"""
    print("=== Testing Feature #84: user_non_funding_ledger_updates ===\n")

    tests = [
        test_rust_core,
        test_python_client,
        test_method_signature,
    ]

    results = []
    for test in tests:
        try:
            result = test()
            results.append(result)
        except Exception as e:
            print(f"✗ Test {test.__name__} failed with exception: {e}")
            results.append(False)

    print(f"\n=== Test Results ===")
    print(f"Tests passed: {sum(results)}/{len(results)}")

    if all(results):
        print("🎉 All tests passed! Feature #84 implementation is complete.")
        return 0
    else:
        print("❌ Some tests failed. Please check the implementation.")
        return 1

if __name__ == "__main__":
    sys.exit(main())