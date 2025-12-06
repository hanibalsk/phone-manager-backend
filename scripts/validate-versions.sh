#!/bin/bash
# validate-versions.sh - Verify all version files are in sync
set -e

# Configure paths (adjust if needed)
ANDROID_ROOT="${ANDROID_ROOT:-/Users/martinjanci/cursor/phone-manager}"
BACKEND_ROOT="${BACKEND_ROOT:-/Users/martinjanci/cursor/phone-manager-backend}"

echo "=== Version Validation Report ==="
echo ""

# Read source versions
ANDROID_VERSION=$(cat "$ANDROID_ROOT/VERSION" 2>/dev/null | tr -d '[:space:]' || echo "MISSING")
BACKEND_VERSION=$(cat "$BACKEND_ROOT/VERSION" 2>/dev/null | tr -d '[:space:]' || echo "MISSING")

# Read target file versions
CARGO_VERSION=$(grep -A5 '\[workspace.package\]' "$BACKEND_ROOT/Cargo.toml" 2>/dev/null | grep 'version' | head -1 | sed -E 's/.*version = "([^"]+)".*/\1/' || echo "MISSING")
OPENAPI_VERSION=$(grep -E '^\s+version:' "$BACKEND_ROOT/docs/api/openapi.yaml" 2>/dev/null | head -1 | sed -E 's/.*version: "([^"]+)".*/\1/' || echo "MISSING")

echo "Source Files:"
echo "  Android VERSION:  $ANDROID_VERSION"
echo "  Backend VERSION:  $BACKEND_VERSION"
echo ""
echo "Target Files:"
echo "  Cargo.toml:       $CARGO_VERSION"
echo "  openapi.yaml:     $OPENAPI_VERSION"
echo ""

# Check sync status
ERRORS=0

if [[ "$ANDROID_VERSION" == "MISSING" ]]; then
    echo "ERROR: Android VERSION file not found!"
    ERRORS=$((ERRORS + 1))
fi

if [[ "$BACKEND_VERSION" == "MISSING" ]]; then
    echo "ERROR: Backend VERSION file not found!"
    ERRORS=$((ERRORS + 1))
fi

if [[ "$ANDROID_VERSION" != "$BACKEND_VERSION" && "$ANDROID_VERSION" != "MISSING" && "$BACKEND_VERSION" != "MISSING" ]]; then
    echo "ERROR: VERSION files out of sync! Android=$ANDROID_VERSION, Backend=$BACKEND_VERSION"
    ERRORS=$((ERRORS + 1))
fi

if [[ "$BACKEND_VERSION" != "$CARGO_VERSION" && "$BACKEND_VERSION" != "MISSING" ]]; then
    echo "ERROR: Backend VERSION != Cargo.toml version"
    ERRORS=$((ERRORS + 1))
fi

if [[ "$BACKEND_VERSION" != "$OPENAPI_VERSION" && "$BACKEND_VERSION" != "MISSING" ]]; then
    echo "ERROR: Backend VERSION != OpenAPI version"
    ERRORS=$((ERRORS + 1))
fi

echo ""
if [[ $ERRORS -eq 0 ]]; then
    echo "SUCCESS: All versions in sync at $ANDROID_VERSION"
    exit 0
else
    echo "FAILED: $ERRORS version mismatch(es) found"
    exit 1
fi
