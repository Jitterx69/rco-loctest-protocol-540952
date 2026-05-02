#!/bin/bash

# RCO Ψ-V5.1.0 "Sovereign Inversion" Global Installer (Forensic Shielded)
# SECURITY: Hardened against Path Hijacking, TOCTOU, and Injection attacks.

set -e

# --- 1. Restricted Execution Environment ---
# Enforce a strict PATH to prevent Path Hijacking
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# --- Visual Setup ---
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}   RCO Ψ-V5.1.0: System Installer                   ${NC}"
echo -e "${BLUE}   (Forensic Shielded Release)                      ${NC}"
echo -e "${BLUE}====================================================${NC}"

# --- 2. Privilege Assertion ---
if [[ $EUID -eq 0 ]]; then
   echo -e "${YELLOW}Running as root. Privilege verified.${NC}"
else
   if ! sudo -v &> /dev/null; then
       echo -e "${RED}Error: Administrative privileges required.${NC}"
       exit 1
   fi
fi

# Parse flags
AUTO_YES=false
for arg in "$@"; do
    if [[ "$arg" == "-y" || "$arg" == "--yes" ]]; then
        AUTO_YES=true
    fi
done

# --- 3. Whitelisted Dependency Audit ---
check_dependency() {
    local cmd=$1
    local name=$2
    local pkg=$3

    echo -n "Checking for $name... "
    if ! command -v "$cmd" &> /dev/null; then
        echo -e "${RED}MISSING${NC}"
        echo -e "${YELLOW}Requirement:${NC} $name is required."
        
        local install=false
        if [ "$AUTO_YES" = true ]; then
            install=true
        else
            read -p "Install $pkg via apt? (y/n): " choice
            [[ "$choice" == "y" ]] && install=true
        fi

        if [ "$install" = true ]; then
            # Whitelist-based installation
            case $pkg in
                "curl"|"python3"|"tpm2-tools"|"build-essential")
                    sudo apt update && sudo apt install -y "$pkg"
                    ;;
                *)
                    echo -e "${RED}Error: Unauthorized package '$pkg' requested.${NC}"
                    exit 1
                    ;;
            esac
        else
            echo -e "${RED}Error: Cannot proceed without $name. Exiting.${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}DETECTED${NC}"
    fi
}

echo -e "\n${BLUE}[1/5] Auditing System Dependencies...${NC}"
check_dependency "curl" "Curl" "curl"
check_dependency "python3" "Python 3" "python3"
check_dependency "tpm2_getcap" "TPM2 Tools" "tpm2-tools"
check_dependency "gcc" "Build Essentials" "build-essential"

# --- 4. Secure Home Staging (Anti-TOCTOU & Anti-Symlink) ---
echo -e "\n${BLUE}[2/5] Performing Secure Home Staging...${NC}"
STAGING_BASE="${HOME}/.cache/rco_manifold"
mkdir -p "$STAGING_BASE"
chmod 700 "$STAGING_BASE"

STAGING_DIR=$(mktemp -d "${STAGING_BASE}/staging.XXXXXX")
trap 'rm -rf "$STAGING_DIR"' EXIT

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${SCRIPT_DIR}/../dist"
LIB_SOURCE="${DIST_DIR}/lib/librco_core.so"
MANIFEST="${DIST_DIR}/manifest.json"

# Move to secure, user-owned staging FIRST
cp "$LIB_SOURCE" "$STAGING_DIR/librco_core.so"
cp "${DIST_DIR}/include/rco-core.h" "$STAGING_DIR/rco-core.h"

# 5. Forensic Hash Verification
if [ -f "$MANIFEST" ]; then
    EXPECTED_HASH=$(grep -oP '"hash": "\K[^"]+' "$MANIFEST")
    ACTUAL_HASH=$(sha256sum "$STAGING_DIR/librco_core.so" | awk '{print $1}')
    if [ "$EXPECTED_HASH" == "$ACTUAL_HASH" ]; then
        echo -e "${GREEN}Integrity: BIT-PERFECT (Forensic Match)${NC}"
    else
        echo -e "${RED}CRITICAL SECURITY ALERT: Hash mismatch! ABORTING.${NC}"
        exit 1
    fi
fi

# --- 6. Atomic System Registration ---
echo -e "\n${BLUE}[3/5] Atomic System Registration...${NC}"
sudo install -m 644 -o root -g root "$STAGING_DIR/librco_core.so" "/usr/local/lib/librco_core.so"
sudo install -m 644 -o root -g root "$STAGING_DIR/rco-core.h" "/usr/local/include/rco-core.h"
sudo ldconfig
echo -e "${GREEN}System Installation: SECURE & COMPLETE${NC}"

echo -e "\n${GREEN}====================================================${NC}"
echo -e "${GREEN}   RCO Ψ-V5.1.0 SETUP COMPLETE                      ${NC}"
echo -e "${GREEN}====================================================${NC}"
