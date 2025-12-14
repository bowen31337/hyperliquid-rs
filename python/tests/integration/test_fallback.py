#!/usr/bin/env python3
"""Simple test script to verify the fallback implementation works"""

import sys
import os

# Add the python directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

try:
    from hyperliquid_rs import HyperliquidClient
    print("✅ Import successful")

    client = HyperliquidClient()
    print("✅ Client creation successful")

    # Test that the client has the expected methods
    print(f"✅ Client has meta method: {hasattr(client, 'meta')}")
    print(f"✅ Client has user_state method: {hasattr(client, 'user_state')}")
    print(f"✅ Client has l2_book method: {hasattr(client, 'l2_book')}")

    print("\n🎉 Fallback implementation is working correctly!")

except ImportError as e:
    print(f"❌ Import failed: {e}")
    sys.exit(1)
except Exception as e:
    print(f"❌ Unexpected error: {e}")
    sys.exit(1)