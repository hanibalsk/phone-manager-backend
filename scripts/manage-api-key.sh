#!/bin/bash
# API Key Management CLI Tool for Phone Manager Backend
#
# Usage:
#   ./scripts/manage-api-key.sh create --name "Production Key"
#   ./scripts/manage-api-key.sh create --name "Admin Key" --admin
#   ./scripts/manage-api-key.sh list
#   ./scripts/manage-api-key.sh rotate --prefix pm_aBcDe
#   ./scripts/manage-api-key.sh deactivate --prefix pm_aBcDe
#   ./scripts/manage-api-key.sh info --prefix pm_aBcDe
#
# Requirements:
#   - openssl (for key generation and hashing)
#   - base64 (for encoding)
#   - psql (PostgreSQL client) for database operations
#
# Environment Variables:
#   DATABASE_URL - PostgreSQL connection string (required for list/rotate/deactivate)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Key prefix
KEY_PREFIX="pm_"

# Generate a new API key
generate_api_key() {
    # Generate 45 bytes of random data and encode as base64
    local random_bytes
    random_bytes=$(openssl rand -base64 45 | tr -d '\n' | tr '+/' '-_' | head -c 45)
    echo "${KEY_PREFIX}${random_bytes}"
}

# Extract prefix from a full key (first 8 chars after pm_)
extract_prefix() {
    local key="$1"
    echo "${key:0:11}"  # pm_ + 8 chars = 11 chars
}

# Compute SHA-256 hash of a key
compute_hash() {
    local key="$1"
    echo -n "$key" | openssl dgst -sha256 | awk '{print $2}'
}

# Print usage
usage() {
    echo "API Key Management CLI Tool for Phone Manager Backend"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  create     Create a new API key"
    echo "  list       List all API keys (requires DATABASE_URL)"
    echo "  rotate     Rotate an existing key (deactivate old, create new)"
    echo "  deactivate Deactivate an API key"
    echo "  info       Show info for a key by prefix"
    echo ""
    echo "Options:"
    echo "  --name <name>    Name for the API key (required for create)"
    echo "  --prefix <prefix> Key prefix (required for rotate/deactivate/info)"
    echo "  --admin          Create an admin key (for create)"
    echo "  --expires <days> Key expiration in days (optional)"
    echo ""
    echo "Examples:"
    echo "  $0 create --name \"Production App\""
    echo "  $0 create --name \"Admin Key\" --admin"
    echo "  $0 list"
    echo "  $0 rotate --prefix pm_aBcDeFgH --name \"New Production Key\""
    echo "  $0 deactivate --prefix pm_aBcDeFgH"
}

# Create a new API key
cmd_create() {
    local name=""
    local is_admin="false"
    local expires_days=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --name)
                name="$2"
                shift 2
                ;;
            --admin)
                is_admin="true"
                shift
                ;;
            --expires)
                expires_days="$2"
                shift 2
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done

    if [[ -z "$name" ]]; then
        echo -e "${RED}Error: --name is required${NC}"
        usage
        exit 1
    fi

    # Generate key
    local api_key
    api_key=$(generate_api_key)

    # Compute hash
    local key_hash
    key_hash=$(compute_hash "$api_key")

    # Extract prefix
    local key_prefix
    key_prefix=$(extract_prefix "$api_key")

    # Calculate expiration
    local expires_clause=""
    if [[ -n "$expires_days" ]]; then
        expires_clause=", expires_at = NOW() + INTERVAL '${expires_days} days'"
    fi

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}       NEW API KEY GENERATED${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "${YELLOW}IMPORTANT: Save the API key now!${NC}"
    echo -e "${YELLOW}It will NOT be shown again.${NC}"
    echo ""
    echo -e "${BLUE}API Key:${NC}     $api_key"
    echo -e "${BLUE}Key Prefix:${NC}  $key_prefix"
    echo -e "${BLUE}Key Hash:${NC}    $key_hash"
    echo -e "${BLUE}Name:${NC}        $name"
    echo -e "${BLUE}Is Admin:${NC}    $is_admin"
    if [[ -n "$expires_days" ]]; then
        echo -e "${BLUE}Expires:${NC}     In $expires_days days"
    else
        echo -e "${BLUE}Expires:${NC}     Never"
    fi
    echo ""
    echo -e "${GREEN}SQL INSERT Statement:${NC}"
    echo ""
    if [[ -n "$expires_days" ]]; then
        cat <<EOF
INSERT INTO api_keys (key_hash, key_prefix, name, is_active, is_admin, expires_at)
VALUES ('$key_hash', '$key_prefix', '$name', true, $is_admin, NOW() + INTERVAL '$expires_days days');
EOF
    else
        cat <<EOF
INSERT INTO api_keys (key_hash, key_prefix, name, is_active, is_admin)
VALUES ('$key_hash', '$key_prefix', '$name', true, $is_admin);
EOF
    fi
    echo ""
    echo -e "${GREEN}========================================${NC}"

    # If DATABASE_URL is set, offer to insert directly
    if [[ -n "$DATABASE_URL" ]]; then
        echo ""
        read -p "Insert into database now? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if [[ -n "$expires_days" ]]; then
                psql "$DATABASE_URL" -c "INSERT INTO api_keys (key_hash, key_prefix, name, is_active, is_admin, expires_at) VALUES ('$key_hash', '$key_prefix', '$name', true, $is_admin, NOW() + INTERVAL '$expires_days days');"
            else
                psql "$DATABASE_URL" -c "INSERT INTO api_keys (key_hash, key_prefix, name, is_active, is_admin) VALUES ('$key_hash', '$key_prefix', '$name', true, $is_admin);"
            fi
            echo -e "${GREEN}API key inserted into database.${NC}"
        fi
    fi
}

# List all API keys
cmd_list() {
    if [[ -z "$DATABASE_URL" ]]; then
        echo -e "${RED}Error: DATABASE_URL environment variable is required for list command${NC}"
        exit 1
    fi

    echo ""
    echo -e "${BLUE}API Keys in Database:${NC}"
    echo ""
    psql "$DATABASE_URL" -c "SELECT id, key_prefix, name, is_active, is_admin,
        CASE WHEN last_used_at IS NOT NULL THEN to_char(last_used_at, 'YYYY-MM-DD HH24:MI') ELSE 'Never' END as last_used,
        to_char(created_at, 'YYYY-MM-DD') as created,
        CASE WHEN expires_at IS NOT NULL THEN to_char(expires_at, 'YYYY-MM-DD') ELSE 'Never' END as expires
        FROM api_keys ORDER BY created_at DESC;"
}

# Deactivate an API key
cmd_deactivate() {
    local prefix=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --prefix)
                prefix="$2"
                shift 2
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done

    if [[ -z "$prefix" ]]; then
        echo -e "${RED}Error: --prefix is required${NC}"
        usage
        exit 1
    fi

    if [[ -z "$DATABASE_URL" ]]; then
        echo -e "${RED}Error: DATABASE_URL environment variable is required${NC}"
        exit 1
    fi

    echo -e "${YELLOW}Deactivating API key with prefix: $prefix${NC}"

    psql "$DATABASE_URL" -c "UPDATE api_keys SET is_active = false, updated_at = NOW() WHERE key_prefix = '$prefix';"

    echo -e "${GREEN}API key deactivated.${NC}"
}

# Rotate an API key
cmd_rotate() {
    local prefix=""
    local name=""
    local is_admin="false"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --prefix)
                prefix="$2"
                shift 2
                ;;
            --name)
                name="$2"
                shift 2
                ;;
            --admin)
                is_admin="true"
                shift
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done

    if [[ -z "$prefix" ]]; then
        echo -e "${RED}Error: --prefix is required${NC}"
        usage
        exit 1
    fi

    if [[ -z "$DATABASE_URL" ]]; then
        echo -e "${RED}Error: DATABASE_URL environment variable is required${NC}"
        exit 1
    fi

    # Get existing key info
    local old_key_info
    old_key_info=$(psql "$DATABASE_URL" -t -c "SELECT name, is_admin FROM api_keys WHERE key_prefix = '$prefix' AND is_active = true;")

    if [[ -z "$old_key_info" ]]; then
        echo -e "${RED}Error: No active key found with prefix $prefix${NC}"
        exit 1
    fi

    # Use existing name if not provided
    if [[ -z "$name" ]]; then
        name=$(echo "$old_key_info" | awk -F'|' '{print $1}' | xargs)
        name="${name} (rotated)"
    fi

    # Preserve admin status
    local old_is_admin
    old_is_admin=$(echo "$old_key_info" | awk -F'|' '{print $2}' | xargs)
    if [[ "$old_is_admin" == "t" ]]; then
        is_admin="true"
    fi

    echo -e "${YELLOW}Rotating API key: $prefix${NC}"
    echo ""

    # Deactivate old key
    echo -e "${BLUE}Step 1: Deactivating old key...${NC}"
    psql "$DATABASE_URL" -c "UPDATE api_keys SET is_active = false, updated_at = NOW() WHERE key_prefix = '$prefix';"

    # Create new key
    echo -e "${BLUE}Step 2: Creating new key...${NC}"
    echo ""
    cmd_create --name "$name" $(if [[ "$is_admin" == "true" ]]; then echo "--admin"; fi)
}

# Show info for a key
cmd_info() {
    local prefix=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --prefix)
                prefix="$2"
                shift 2
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done

    if [[ -z "$prefix" ]]; then
        echo -e "${RED}Error: --prefix is required${NC}"
        usage
        exit 1
    fi

    if [[ -z "$DATABASE_URL" ]]; then
        echo -e "${RED}Error: DATABASE_URL environment variable is required${NC}"
        exit 1
    fi

    echo ""
    echo -e "${BLUE}API Key Info for prefix: $prefix${NC}"
    echo ""
    psql "$DATABASE_URL" -c "SELECT id, key_prefix, name, is_active, is_admin,
        created_at, updated_at, last_used_at, expires_at
        FROM api_keys WHERE key_prefix = '$prefix';"
}

# Main entry point
main() {
    if [[ $# -lt 1 ]]; then
        usage
        exit 1
    fi

    local command="$1"
    shift

    case $command in
        create)
            cmd_create "$@"
            ;;
        list)
            cmd_list "$@"
            ;;
        deactivate)
            cmd_deactivate "$@"
            ;;
        rotate)
            cmd_rotate "$@"
            ;;
        info)
            cmd_info "$@"
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            echo -e "${RED}Unknown command: $command${NC}"
            usage
            exit 1
            ;;
    esac
}

main "$@"
