#!/usr/bin/env python3
"""Test what methods are available on the client"""

import sys
import os

# Add the python directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

from hyperliquid_rs import HyperliquidClient

client = HyperliquidClient()

print("Available methods on HyperliquidClient:")
methods = [method for method in dir(client) if not method.startswith('_')]
for method in sorted(methods):
    print(f"  - {method}")

print(f"\nTotal methods: {len(methods)}")

# Test a simple method call
print(f"\n✅ Client created successfully with {len(methods)} methods available")

# Test a simple info method (without making actual API calls)
print("✅ Available Info API methods:")
info_methods = [m for m in methods if m.startswith('get_') and not m.startswith('exchange_')]
for method in info_methods:
    print(f"  - {method}")

print("\n✅ Available Exchange API methods:")
exchange_methods = [m for m in methods if m.startswith('exchange_') or m.startswith('place_') or m.startswith('cancel_')]
for method in exchange_methods:
    print(f"  - {method}")