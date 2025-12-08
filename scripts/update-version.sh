#!/bin/bash
# update-version.sh - Propagate VERSION to Rust and OpenAPI files
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION_FILE="$PROJECT_ROOT/VERSION"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
OPENAPI_FILE="$PROJECT_ROOT/docs/api/openapi.yaml"

# Read version from VERSION file
if [[ ! -f "$VERSION_FILE" ]]; then
    echo "ERROR: VERSION file not found at $VERSION_FILE"
    exit 1
fi

VERSION=$(cat "$VERSION_FILE" | tr -d '[:space:]')

# Validate semantic version format
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "ERROR: Invalid version format '$VERSION'. Expected X.Y.Z"
    exit 1
fi

echo "Updating Rust backend version to $VERSION"

# Update Cargo.toml workspace version
# Match: version = "X.Y.Z" under [workspace.package] (not rust-version)
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - use ^version to avoid matching rust-version
    sed -i '' -E '/\[workspace\.package\]/,/^\[/ s/^(version = ")[^"]+"/\1'"$VERSION"'"/' "$CARGO_TOML"
    sed -i '' -E 's/(version: ")[^"]+"/\1'"$VERSION"'"/' "$OPENAPI_FILE"
else
    # Linux - use ^version to avoid matching rust-version
    sed -i -E '/\[workspace\.package\]/,/^\[/ s/^(version = ")[^"]+"/\1'"$VERSION"'"/' "$CARGO_TOML"
    sed -i -E 's/(version: ")[^"]+"/\1'"$VERSION"'"/' "$OPENAPI_FILE"
fi

echo "Successfully updated:"
echo "  - Cargo.toml workspace.package.version = \"$VERSION\""
echo "  - openapi.yaml info.version = \"$VERSION\""

# Verify Cargo.lock gets updated
echo ""
echo "Running cargo check to update Cargo.lock..."
cd "$PROJECT_ROOT"
cargo check --quiet 2>/dev/null && echo "Cargo.lock updated" || echo "Note: cargo check skipped (may need full build)"
