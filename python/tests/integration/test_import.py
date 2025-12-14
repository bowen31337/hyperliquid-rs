#!/usr/bin/env python3

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

try:
    import hyperliquid_rs
    print("✅ Hyperliquid Rust SDK imported successfully")

    # Test basic client creation
    client = hyperliquid_rs.HyperliquidClient()
    print("✅ Client created successfully")

    # Test basic API call
    meta = client.get_meta()
    print(f"✅ Basic API call successful - got {len(meta.universe)} assets")

except Exception as e:
    print(f"❌ Error: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)