#!/bin/bash

# Monster-Player Installation Script for Arch Linux
# This script installs dependencies, builds the project, and installs the binary

set -e  # Exit on error

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Monster-Player Installation Script   ${NC}"
echo -e "${BLUE}========================================${NC}"
echo

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    echo -e "${YELLOW}Warning: Running as root. It's recommended to run this script as a regular user.${NC}"
    echo -e "${YELLOW}Press Ctrl+C to cancel or wait 3 seconds to continue...${NC}"
    sleep 3
fi

# Function to print status messages
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install package if not present
install_package() {
    local package=$1
    if ! pacman -Qi "$package" &>/dev/null; then
        echo -e "${BLUE}Installing $package...${NC}"
        sudo pacman -S --noconfirm "$package"
    else
        print_status "$package is already installed"
    fi
}

# Step 1: Check for sudo
if ! command_exists sudo; then
    print_error "sudo is not installed. Please install sudo and configure it first."
    exit 1
fi

# Step 2: Update system
echo -e "${BLUE}Updating system packages...${NC}"
sudo pacman -Syu --noconfirm

# Step 3: Install build dependencies
echo -e "${BLUE}Installing build dependencies...${NC}"
install_package "pkg-config"
install_package "alsa-lib"
install_package "dbus"
install_package "libchromaprint"
install_package "cava"  # For spectrum visualization
install_package "base-devel"  # For building

# Step 4: Install Rust if not present
if ! command_exists rustc; then
    echo -e "${BLUE}Installing Rust toolchain...${NC}"
    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        install_package "curl"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
else
    print_status "Rust is already installed"
fi

# Step 5: Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "src" ]]; then
    print_error "This script must be run from the monster-player project root directory."
    print_error "Current directory: $(pwd)"
    exit 1
fi

# Step 6: Build the project
echo -e "${BLUE}Building Monster-Player...${NC}"
echo -e "${YELLOW}This may take several minutes...${NC}"

# Clean previous builds
if [[ -d "target" ]]; then
    print_status "Cleaning previous builds..."
    cargo clean
fi

# Build release version
if cargo build --release; then
    print_status "Build successful!"
else
    print_error "Build failed. Please check the error messages above."
    exit 1
fi

# Step 7: Install binary
echo -e "${BLUE}Installing binary to /usr/local/bin/...${NC}"
if [[ -f "target/release/tmplayer" ]]; then
    sudo cp target/release/tmplayer /usr/local/bin/monsterplayer
    sudo chmod +x /usr/local/bin/monsterplayer
    print_status "Binary installed as 'monsterplayer'"
    
    # Create a symlink for backward compatibility
    sudo ln -sf /usr/local/bin/monsterplayer /usr/local/bin/tmplayer
    print_status "Created symlink 'tmplayer' for backward compatibility"
else
    print_error "Built binary not found at target/release/tmplayer"
    exit 1
fi

# Step 8: Create desktop entry (optional)
if [[ -d "/usr/share/applications" ]]; then
    echo -e "${BLUE}Creating desktop entry...${NC}"
    cat > /tmp/monsterplayer.desktop << EOF
[Desktop Entry]
Name=Monster-Player
Comment=TUI Music Player with Siren Records Streaming
Exec=monsterplayer
Icon=utilities-terminal
Terminal=true
Type=Application
Categories=AudioVideo;Audio;Player;
Keywords=music;player;tui;terminal;streaming;
EOF
    sudo cp /tmp/monsterplayer.desktop /usr/share/applications/
    print_status "Desktop entry created"
fi

# Step 9: Create configuration directory on first run
echo -e "${BLUE}Configuration will be created on first run at ~/.config/tmplayer/${NC}"

# Step 10: Verify installation
echo
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo
echo -e "You can now run Monster-Player using:"
echo -e "  ${YELLOW}monsterplayer${NC}"
echo
echo -e "Or using the original name:"
echo -e "  ${YELLOW}tmplayer${NC}"
echo
echo -e "${BLUE}Quick Start Guide:${NC}"
echo -e "1. Start the player: ${YELLOW}monsterplayer${NC}"
echo -e "2. Press ${YELLOW}M${NC} to open the Siren Records album browser"
echo -e "3. Use ${YELLOW}n/p${NC} or arrow keys to navigate albums"
echo -e "4. Use ${YELLOW}left/right${NC} arrows to select songs"
echo -e "5. Press ${YELLOW}Enter${NC} to play a selected song"
echo -e "6. Press ${YELLOW}Ctrl+K${NC} for full keyboard shortcuts"
echo
echo -e "${YELLOW}Note: Make sure your terminal uses a Nerd Font for proper icon display.${NC}"
echo -e "${BLUE}========================================${NC}"