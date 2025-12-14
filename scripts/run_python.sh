#!/bin/bash
# Run Python scripts using uv
# This script ensures uv is used for all Python commands

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

# For scripts that don't need Rust module, use Python directly
# but ensure dependencies are installed via uv
if [ -f "pyproject.toml" ]; then
    # Install dependencies if needed
    uv pip install pydantic pydantic-settings httpx > /dev/null 2>&1 || true
fi

# Run the script with Python (uv manages the environment)
exec python3 "$@"

