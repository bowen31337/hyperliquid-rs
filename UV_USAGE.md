# Using uv for Python Commands

This project uses `uv` as the Python package manager. All Python-related commands should use `uv` for dependency management.

## Quick Reference

### Installing Dependencies

```bash
# Install all project dependencies
uv pip install pydantic pydantic-settings httpx

# Install development dependencies
uv pip install pytest maturin
```

### Running Scripts

**Swagger UI Server:**
```bash
# Recommended: Use convenience script
./scripts/start_swagger.sh 8081

# Manual:
uv pip install pydantic pydantic-settings httpx
python3 scripts/serve_swagger.py --port 8081
```

**Testnet Tests:**
```bash
# Recommended: Use convenience script
./scripts/run_testnet.sh

# Manual:
uv pip install pydantic pydantic-settings httpx
PYTHONPATH=./python:$PYTHONPATH python3 test_testnet.py
```

**Python Tests:**
```bash
cd python
uv pip install pytest
uv run pytest tests/
```

### Building Rust-Python Bindings

```bash
cd crates/hyperliquid-python
uv pip install maturin
uv run maturin develop
```

## Why uv instead of uv run?

For scripts that don't require the Rust module (like `serve_swagger.py`), we use:
1. `uv pip install` to manage dependencies
2. `python3` directly to run scripts

This avoids the build overhead when `uv run` tries to build the Rust package via maturin.

For scripts that need the full project environment, use `uv run` which will:
1. Create a virtual environment
2. Install dependencies from `pyproject.toml`
3. Build the Rust module if needed
4. Run the script

## Convenience Scripts

All convenience scripts in `scripts/` use `uv pip install` to ensure dependencies are available before running Python commands:

- `scripts/start_swagger.sh` - Start Swagger UI server
- `scripts/run_testnet.sh` - Run testnet tests
- `scripts/run_python.sh` - Generic Python script runner

## Environment Variables

When running Python scripts manually, ensure `PYTHONPATH` includes the `python/` directory:

```bash
export PYTHONPATH=./python:$PYTHONPATH
python3 your_script.py
```

Or inline:
```bash
PYTHONPATH=./python:$PYTHONPATH python3 your_script.py
```

