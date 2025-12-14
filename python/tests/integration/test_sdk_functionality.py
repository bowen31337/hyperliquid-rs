#!/usr/bin/env python3
"""Test script to verify Hyperliquid SDK functionality"""

import sys
import os

# Add the python directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

def test_import():
    """Test if the SDK can be imported"""
    try:
        import hyperliquid_rs
        print("✅ Hyperliquid SDK imported successfully")
        print(f"   Version: {hyperliquid_rs.__version__}")
        print(f"   Available: {len(hyperliquid_rs.__all__)} exports")
        return True
    except Exception as e:
        print(f"❌ Import failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_client_init():
    """Test client initialization"""
    try:
        import hyperliquid_rs
        client = hyperliquid_rs.HyperliquidClient()
        print("✅ Client initialized successfully")
        return True
    except Exception as e:
        print(f"❌ Client initialization failed: {e}")
        return False

def test_basic_functionality():
    """Test basic API connectivity"""
    try:
        import hyperliquid_rs
        client = hyperliquid_rs.HyperliquidClient()

        # Test meta endpoint
        meta = client.get_meta()
        print(f"✅ Meta API working - {len(meta.universe)} assets found")

        # Test error handling with invalid address
        try:
            client.get_user_state("invalid_address")
        except Exception as e:
            print(f"✅ Error handling working - {type(e).__name__}")

        return True
    except Exception as e:
        print(f"❌ Basic functionality test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("🚀 Testing Hyperliquid SDK...")
    print("=" * 50)

    tests = [
        ("Import Test", test_import),
        ("Client Initialization", test_client_init),
        ("Basic Functionality", test_basic_functionality),
    ]

    passed = 0
    total = len(tests)

    for test_name, test_func in tests:
        print(f"\n📋 {test_name}:")
        if test_func():
            passed += 1
        else:
            print(f"   ⚠️  {test_name} failed")

    print("\n" + "=" * 50)
    print(f"📊 Results: {passed}/{total} tests passed")

    if passed == total:
        print("🎉 All tests passed! SDK is working correctly.")
        return True
    else:
        print("❌ Some tests failed. Check the errors above.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)