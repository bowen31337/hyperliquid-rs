#!/bin/bash
# Run testnet tests using uv

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

echo "🧪 Running testnet tests..."
echo ""

# Ensure dependencies are installed via uv
uv pip install pydantic pydantic-settings httpx > /dev/null 2>&1 || true

# Run tests with PYTHONPATH set
PYTHONPATH=./python:$PYTHONPATH python3 python/tests/integration/test_testnet.py

