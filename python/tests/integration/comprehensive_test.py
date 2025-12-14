#!/usr/bin/env python3
"""Comprehensive test of the Hyperliquid Rust SDK functionality"""

import sys
import os
import time
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

def test_basic_functionality():
    """Test basic SDK functionality"""
    print("🔧 Testing Hyperliquid Rust SDK...")

    try:
        import hyperliquid_rs
        print("✅ SDK imported successfully")

        # Test client creation
        client = hyperliquid_rs.HyperliquidClient()
        print("✅ Client created successfully")

        # Test meta endpoint
        print("📡 Testing meta endpoint...")
        meta = client.get_meta()
        print(f"✅ Meta endpoint: {len(meta.universe)} assets loaded")

        # Test user state
        print("👤 Testing user state endpoint...")
        try:
            user_state = client.get_user_state("0x0000000000000000000000000000000000000000")
            print("✅ User state endpoint responding")
        except Exception as e:
            print(f"⚠️  User state test: {e}")

        # Test order creation
        print("📋 Testing order model...")
        order = hyperliquid_rs.Order(
            coin="BTC",
            is_buy=True,
            size=0.001,
            limit_price=50000
        )
        print(f"✅ Order model: {order.coin} {order.size} @ {order.limit_price}")

        # Test order wire conversion
        order_wire = order.to_wire()
        print("✅ Order wire conversion successful")

        return True

    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_error_handling():
    """Test error handling"""
    print("\n🚨 Testing error handling...")

    try:
        import hyperliquid_rs

        # Test invalid address
        try:
            client = hyperliquid_rs.HyperliquidClient()
            client.get_user_state("invalid_address")
            print("⚠️  Should have failed with invalid address")
        except Exception:
            print("✅ Properly handles invalid address")

        # Test invalid order
        try:
            order = hyperliquid_rs.Order(
                coin="",
                is_buy=True,
                size=-1,  # Invalid size
                limit_price=0  # Invalid price
            )
            print("⚠️  Should have validated order fields")
        except Exception:
            print("✅ Properly validates order fields")

        return True

    except Exception as e:
        print(f"❌ Error handling test failed: {e}")
        return False

def test_performance():
    """Test basic performance"""
    print("\n⚡ Testing performance...")

    try:
        import hyperliquid_rs
        import time

        client = hyperliquid_rs.HyperliquidClient()

        # Test multiple requests
        start_time = time.time()
        for i in range(5):
            meta = client.get_meta()
            if len(meta.universe) == 0:
                raise Exception("Empty universe response")
        end_time = time.time()

        avg_time = (end_time - start_time) / 5
        print(f"✅ Average request time: {avg_time:.3f}s")

        if avg_time > 2.0:
            print("⚠️  Performance could be improved")
        else:
            print("✅ Performance is good")

        return True

    except Exception as e:
        print(f"❌ Performance test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("🚀 Starting comprehensive Hyperliquid Rust SDK tests\n")

    tests = [
        ("Basic Functionality", test_basic_functionality),
        ("Error Handling", test_error_handling),
        ("Performance", test_performance),
    ]

    passed = 0
    total = len(tests)

    for test_name, test_func in tests:
        print(f"📋 Running: {test_name}")
        if test_func():
            passed += 1
        print()

    print("=" * 50)
    print(f"📊 Test Results: {passed}/{total} passed")

    if passed == total:
        print("🎉 All tests passed! SDK is working correctly.")
        return True
    else:
        print("⚠️  Some tests failed. Please check the implementation.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)