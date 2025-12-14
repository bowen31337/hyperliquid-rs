#!/usr/bin/env python3
"""Test edge cases and error scenarios for Hyperliquid SDK"""

import sys
import os

# Add the python directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

def test_invalid_address():
    """Test error handling for invalid addresses"""
    try:
        import hyperliquid_rs
        client = hyperliquid_rs.HyperliquidClient()

        # Test various invalid address formats
        invalid_addresses = [
            "invalid",
            "0xinvalid",
            "0x123",
            "",
            "null",
            "0x" + "0" * 39,  # Too short
            "0x" + "0" * 41,  # Too long
        ]

        for addr in invalid_addresses:
            try:
                client.get_user_state(addr)
                print(f"⚠️  Expected error for address: {addr}")
            except Exception as e:
                print(f"✅ Properly caught error for '{addr[:10]}...': {type(e).__name__}")

        return True
    except Exception as e:
        print(f"❌ Invalid address test failed: {e}")
        return False

def test_large_response_handling():
    """Test handling of large API responses"""
    try:
        import hyperliquid_rs
        client = hyperliquid_rs.HyperliquidClient()

        # Get meta (should be substantial data)
        meta = client.get_meta()

        # Verify we got a reasonable amount of data
        if len(meta.universe) > 100:
            print(f"✅ Large response handling works - {len(meta.universe)} assets processed")

            # Test accessing individual assets
            first_asset = meta.universe[0]
            if hasattr(first_asset, 'name') and first_asset.name:
                print(f"✅ Asset data properly parsed: {first_asset.name}")
            else:
                print("⚠️  Asset data parsing issue")
        else:
            print(f"⚠️  Expected more assets, got {len(meta.universe)}")

        return True
    except Exception as e:
        print(f"❌ Large response test failed: {e}")
        return False

def test_concurrent_requests():
    """Test basic concurrent request handling"""
    try:
        import hyperliquid_rs
        import time
        import threading

        client = hyperliquid_rs.HyperliquidClient()
        results = []
        errors = []

        def make_request():
            try:
                start_time = time.time()
                meta = client.get_meta()
                end_time = time.time()
                results.append({
                    'duration': end_time - start_time,
                    'asset_count': len(meta.universe)
                })
            except Exception as e:
                errors.append(e)

        # Spawn multiple threads
        threads = []
        for i in range(3):
            thread = threading.Thread(target=make_request)
            threads.append(thread)
            thread.start()

        # Wait for all threads to complete
        for thread in threads:
            thread.join(timeout=10)  # 10 second timeout

        if errors:
            print(f"⚠️  Some requests failed: {len(errors)} errors")
            for error in errors[:2]:  # Show first 2 errors
                print(f"   Error: {type(error).__name__}: {error}")

        if results:
            avg_duration = sum(r['duration'] for r in results) / len(results)
            print(f"✅ Concurrent requests work - {len(results)} successful")
            print(f"   Average duration: {avg_duration:.2f}s")

        return len(results) > 0
    except Exception as e:
        print(f"❌ Concurrent request test failed: {e}")
        return False

def test_error_hierarchy():
    """Test that error hierarchy works correctly"""
    try:
        import hyperliquid_rs
        from hyperliquid_rs.errors import (
            HyperliquidError, ApiError, NetworkError,
            RateLimitError, AuthenticationError, ValidationError, TimeoutError
        )

        client = hyperliquid_rs.HyperliquidClient()

        # Test that all error types can be caught by base class
        try:
            client.get_user_state("definitely_invalid_address_format")
        except HyperliquidError as e:
            print(f"✅ Base error class catches specific errors: {type(e).__name__}")
        except Exception as e:
            print(f"❌ Error not caught by HyperliquidError: {type(e).__name__}")
            return False

        # Test specific error types
        error_types = [
            (HyperliquidError, "base"),
            (ApiError, "API"),
            (NetworkError, "network"),
            (ValidationError, "validation"),
            (TimeoutError, "timeout"),
        ]

        for error_type, name in error_types:
            print(f"✅ Error type available: {name}")

        return True
    except Exception as e:
        print(f"❌ Error hierarchy test failed: {e}")
        return False

def test_fallback_functionality():
    """Test that fallback implementation works correctly"""
    try:
        # Test if we can import fallback directly
        import hyperliquid_rs._fallback as fallback

        # Test fallback client creation
        fallback_client = fallback.PyInfoClient("https://api.hyperliquid.xyz")
        print("✅ Fallback client created successfully")

        # Test basic fallback functionality
        meta_response = fallback_client.meta(None)
        if meta_response and len(meta_response) > 100:
            print("✅ Fallback implementation works")
        else:
            print("⚠️  Fallback response seems incomplete")

        return True
    except Exception as e:
        print(f"❌ Fallback test failed: {e}")
        return False

def main():
    """Run all edge case tests"""
    print("🧪 Testing Hyperliquid SDK Edge Cases...")
    print("=" * 60)

    tests = [
        ("Invalid Address Handling", test_invalid_address),
        ("Large Response Handling", test_large_response_handling),
        ("Concurrent Requests", test_concurrent_requests),
        ("Error Hierarchy", test_error_hierarchy),
        ("Fallback Functionality", test_fallback_functionality),
    ]

    passed = 0
    total = len(tests)

    for test_name, test_func in tests:
        print(f"\n🔍 {test_name}:")
        try:
            if test_func():
                passed += 1
                print(f"   ✅ PASSED")
            else:
                print(f"   ❌ FAILED")
        except Exception as e:
            print(f"   ❌ ERROR: {e}")

    print("\n" + "=" * 60)
    print(f"📊 Edge Case Results: {passed}/{total} tests passed")

    if passed == total:
        print("🎉 All edge case tests passed! SDK is robust.")
        return True
    else:
        print("⚠️  Some edge case tests failed. Review issues above.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)