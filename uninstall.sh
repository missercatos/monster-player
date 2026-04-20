#!/bin/bash

# Monster-Player Uninstallation Script

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Monster-Player Uninstallation Script ${NC}"
echo -e "${BLUE}========================================${NC}"
echo

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    echo -e "${YELLOW}Warning: Running as root. Press Ctrl+C to cancel or wait 3 seconds to continue...${NC}"
    sleep 3
fi

# Function to print status messages
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Step 1: Remove binary
if [[ -f "/usr/local/bin/monsterplayer" ]]; then
    echo -e "${BLUE}Removing monsterplayer binary...${NC}"
    sudo rm -f /usr/local/bin/monsterplayer
    print_status "Removed /usr/local/bin/monsterplayer"
else
    print_warning "monsterplayer binary not found in /usr/local/bin/"
fi

# Step 2: Remove tmplayer symlink
if [[ -L "/usr/local/bin/tmplayer" ]]; then
    echo -e "${BLUE}Removing tmplayer symlink...${NC}"
    sudo rm -f /usr/local/bin/tmplayer
    print_status "Removed /usr/local/bin/tmplayer symlink"
fi

# Step 3: Remove desktop entry
if [[ -f "/usr/share/applications/monsterplayer.desktop" ]]; then
    echo -e "${BLUE}Removing desktop entry...${NC}"
    sudo rm -f /usr/share/applications/monsterplayer.desktop
    print_status "Removed desktop entry"
fi

# Step 4: Ask about configuration files
echo
echo -e "${YELLOW}Configuration files are stored in ~/.config/tmplayer/${NC}"
read -p "Do you want to remove configuration files? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [[ -d "$HOME/.config/tmplayer" ]]; then
        rm -rf "$HOME/.config/tmplayer"
        print_status "Removed configuration directory ~/.config/tmplayer"
    else
        print_warning "Configuration directory not found"
    fi
fi

# Step 5: Optional - remove Rust installation
echo
read -p "Do you want to uninstall Rust toolchain? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v rustup >/dev/null 2>&1; then
        rustup self uninstall -y
        print_status "Uninstalled Rust toolchain"
    else
        print_warning "Rust toolchain not found or not installed via rustup"
    fi
fi

echo
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Uninstallation Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo
echo -e "${YELLOW}Note: System dependencies (cava, alsa-lib, etc.) were not removed.${NC}"
echo -e "${YELLOW}If you want to remove them, use:${NC}"
echo -e "  sudo pacman -Rs cava alsa-lib dbus libchromaprint"
echo -e "${BLUE}========================================${NC}"