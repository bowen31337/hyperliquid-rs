#!/usr/bin/env python3
"""
Verification script to test current state of the Hyperliquid Rust SDK implementation.
This script will verify that the existing codebase is working correctly before implementing new features.
"""

import asyncio
import sys
import subprocess
import os
from pathlib import Path

def run_command(cmd, cwd=None, capture_output=True):
    """Run a command and return the result"""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            cwd=cwd,
            capture_output=capture_output,
            text=True,
            timeout=120
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Command timed out"
    except Exception as e:
        return False, "", str(e)

def check_rust_environment():
    """Check if Rust toolchain is available"""
    print("🔍 Checking Rust environment...")
    
    success, stdout, stderr = run_command("rustc --version")
    if not success:
        print("❌ Rust compiler (rustc) not found")
        return False
    print(f"✅ Rust compiler: {stdout.strip()}")
    
    success, stdout, stderr = run_command("cargo --version")
    if not success:
        print("❌ Cargo not found")
        return False
    print(f"✅ Cargo: {stdout.strip()}")
    
    return True

def check_python_environment():
    """Check if Python environment is set up"""
    print("🔍 Checking Python environment...")
    
    success, stdout, stderr = run_command("python3 --version")
    if not success:
        print("❌ Python not found")
        return False
    print(f"✅ Python: {stdout.strip()}")
    
    # Check if uv is available
    success, stdout, stderr = run_command("which uv")
    if success:
        print("✅ uv package manager is available")
        return True
    
    # Check if pip is available
    success, stdout, stderr = run_command("pip3 --version")
    if success:
        print("✅ pip package manager is available")
        return True
    
    print("⚠️  No package manager found (uv or pip)")
    return True

def check_project_structure():
    """Check if project structure is correct"""
    print("🔍 Checking project structure...")
    
    required_files = [
        "Cargo.toml",
        "pyproject.toml", 
        "crates/hyperliquid-core/Cargo.toml",
        "crates/hyperliquid-python/Cargo.toml",
        "crates/hyperliquid-core/src/lib.rs",
        "python/hyperliquid_rs/__init__.py",
    ]
    
    missing_files = []
    for file_path in required_files:
        if not Path(file_path).exists():
            missing_files.append(file_path)
    
    if missing_files:
        print("❌ Missing required files:")
        for file_path in missing_files:
            print(f"   - {file_path}")
        return False
    
    print("✅ All required files exist")
    return True

def build_rust_workspace():
    """Build the Rust workspace"""
    print("🔧 Building Rust workspace...")
    
    success, stdout, stderr = run_command("cargo check --workspace", cwd=".")
    if not success:
        print("❌ Rust workspace check failed")
        print("STDERR:", stderr)
        return False
    
    print("✅ Rust workspace check passed")
    return True

def run_rust_tests():
    """Run Rust tests"""
    print("🧪 Running Rust tests...")
    
    success, stdout, stderr = run_command("cargo test --workspace --lib -- --nocapture", cwd=".")
    if not success:
        print("❌ Rust tests failed")
        print("STDERR:", stderr)
        return False
    
    print("✅ Rust tests passed")
    return True

def build_python_bindings():
    """Build Python bindings with maturin"""
    print("🐍 Building Python bindings...")
    
    success, stdout, stderr = run_command("maturin develop", cwd="crates/hyperliquid-python")
    if not success:
        print("❌ Python bindings build failed")
        print("STDERR:", stderr)
        return False
    
    print("✅ Python bindings built successfully")
    return True

def run_python_tests():
    """Run Python tests"""
    print("🧪 Running Python tests...")
    
    success, stdout, stderr = run_command("python3 -m pytest tests/ -v", cwd="python")
    if not success:
        print("❌ Python tests failed")
        print("STDERR:", stderr)
        return False
    
    print("✅ Python tests passed")
    return True

def test_imports():
    """Test Python imports"""
    print("📦 Testing Python imports...")
    
    try:
        # Try importing the main module
        import sys
        sys.path.insert(0, "python")
        
        import hyperliquid_rs
        print("✅ hyperliquid_rs imported successfully")
        
        # Try importing the client
        from hyperliquid_rs.client import HyperliquidClient
        print("✅ HyperliquidClient imported successfully")
        
        return True
    except Exception as e:
        print(f"❌ Import failed: {e}")
        return False

def main():
    """Main verification function"""
    print("🚀 Starting Hyperliquid Rust SDK verification...")
    print("=" * 60)
    
    # Track verification results
    results = {}
    
    # Check environments
    results['rust_env'] = check_rust_environment()
    results['python_env'] = check_python_environment()
    
    if not results['rust_env'] or not results['python_env']:
        print("\n❌ Environment checks failed. Cannot proceed with verification.")
        return False
    
    # Check project structure
    results['project_structure'] = check_project_structure()
    
    # Build and test Rust
    results['rust_build'] = build_rust_workspace()
    results['rust_tests'] = run_rust_tests()
    
    # Build and test Python
    results['python_build'] = build_python_bindings()
    results['python_tests'] = run_python_tests()
    
    # Test imports
    results['python_imports'] = test_imports()
    
    # Print summary
    print("\n" + "=" * 60)
    print("📊 VERIFICATION SUMMARY")
    print("=" * 60)
    
    for test_name, passed in results.items():
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"{test_name:<20}: {status}")
    
    total_tests = len(results)
    passed_tests = sum(results.values())
    
    print(f"\nTotal: {total_tests}, Passed: {passed_tests}, Failed: {total_tests - passed_tests}")
    
    if passed_tests == total_tests:
        print("\n🎉 All verifications passed! The codebase is ready for new feature implementation.")
        return True
    else:
        print(f"\n⚠️  {total_tests - passed_tests} verification(s) failed. Please fix issues before implementing new features.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
