#!/bin/bash

# Test script for multi-sig envelope signing implementation

echo "Testing Multi-sig Envelope Signing Implementation"
echo "================================================="

# Check if Rust toolchain is available
echo "Checking Rust toolchain..."
if ! command -v rustc &> /dev/null; then
    echo "ERROR: rustc not found. Please install Rust toolchain."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found. Please install Rust toolchain."
    exit 1
fi

echo "✓ Rust toolchain found"

# Change to the hyperliquid-core crate directory
cd crates/hyperliquid-core

# Run basic compilation check
echo "Running compilation check..."
if cargo check --lib; then
    echo "✓ Compilation successful"
else
    echo "ERROR: Compilation failed"
    exit 1
fi

# Run unit tests
echo "Running unit tests..."
if cargo test --lib; then
    echo "✓ Unit tests passed"
else
    echo "ERROR: Unit tests failed"
    exit 1
fi

# Run integration tests
echo "Running integration tests..."
if cargo test --test multi_sig_tests; then
    echo "✓ Integration tests passed"
else
    echo "ERROR: Integration tests failed"
    exit 1
fi

echo ""
echo "All tests passed! Multi-sig envelope signing implementation is working correctly."
echo ""
echo "Implementation Summary:"
echo "- MultiSigEnvelope struct with signature management"
echo "- MULTI_SIG_ENVELOPE_SIGN_TYPES with proper EIP-712 types"
echo "- sign_multi_sig_envelope() function"
echo "- create_multi_sig_envelope() helper"
echo "- verify_multi_sig_envelope() threshold checking"
echo "- Comprehensive test coverage"