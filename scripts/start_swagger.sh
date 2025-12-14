#!/bin/bash
# Start Swagger UI server using uv

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

# Default port
PORT=${1:-8081}

echo "🚀 Starting Swagger UI server on port $PORT..."
echo "📄 OpenAPI spec: openapi/openapi.json"
echo ""

# Ensure dependencies are installed via uv
uv pip install pydantic pydantic-settings httpx > /dev/null 2>&1 || true

# Run the script (uses standard library only, no Rust module needed)
python3 scripts/serve_swagger.py --port "$PORT"

