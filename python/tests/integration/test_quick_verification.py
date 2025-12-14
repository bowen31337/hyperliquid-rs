#!/usr/bin/env python3
"""Quick test to verify Hyperliquid SDK functionality"""

import sys
sys.path.insert(0, './python')

try:
    import hyperliquid_rs
    print("✅ Hyperliquid SDK imported successfully")

    # Test client initialization
    client = hyperliquid_rs.HyperliquidClient()
    print("✅ Client initialized")

    # Test API connection
    meta = client.get_meta()
    print(f"✅ Connected to API - {len(meta.universe)} assets available")

    # Test error handling
    try:
        client.get_user_state("invalid_address")
    except Exception as e:
        print(f"✅ Error handling works: {type(e).__name__}")

    print("\n🎉 All basic functionality tests passed!")

except Exception as e:
    print(f"❌ Error: {e}")
    import traceback
    traceback.print_exc()