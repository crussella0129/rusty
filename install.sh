#!/usr/bin/env bash

# Rusty installer & launcher script
# Paced, pedagogical bootstrap experience (prompt §0 / §7)

set -e

# ANSI styling
BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BOLD}========================================================================${NC}"
echo -e "${BOLD}                      Rusty — Learn Rust by Doing                       ${NC}"
echo -e "${BOLD}========================================================================${NC}"
echo ""
echo -e "Welcome! I am Rusty's setup guide."
echo -e "The first lesson of learning Rust is compilation: you will build Rusty from"
echo -e "source using your own local Rust compiler."
echo ""

# 1. Detect Rust Toolchain
echo -e "${BLUE}[1/3] Detecting Rust compiler toolchain...${NC}"

HAS_CARGO=false
if command -v cargo >/dev/null 2>&1; then
    HAS_CARGO=true
fi

if [ "$HAS_CARGO" = true ]; then
    VERSION=$(cargo --version)
    echo -e "${GREEN}✓ Found Rust toolchain: ${VERSION}${NC}"
else
    echo -e "${YELLOW}⚠ Rust toolchain was not found on your system.${NC}"
    echo -e "To compile and run Rusty, we need to install the official Rust compiler."
    echo ""
    read -p "Would you like to install Rust via rustup now? (y/N): " choice
    case "$choice" in
        [yY][eE][sS]|[yY])
            echo ""
            echo -e "${BLUE}Running official rustup installer...${NC}"
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            
            # Load cargo environment in current shell session
            if [ -f "$HOME/.cargo/env" ]; then
                source "$HOME/.cargo/env"
            fi
            
            if command -v cargo >/dev/null 2>&1; then
                echo -e "${GREEN}✓ Rust successfully installed!${NC}"
            else
                echo -e "${YELLOW}⚠ Rust was installed, but 'cargo' is not yet in your PATH.${NC}"
                echo -e "Please open a new terminal, navigate to this directory, and run this script again."
                exit 0
            fi
            ;;
        *)
            echo ""
            echo -e "Understood. You chose not to install Rust automatically."
            echo -e "To continue, you can install it manually by running:"
            echo -e "  ${BOLD}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
            echo -e "After installing, please run this script again to build and launch Rusty."
            exit 0
            ;;
    esac
fi

# 2. Build Rusty from Source
echo ""
echo -e "${BLUE}[2/3] Compiling Rusty from source...${NC}"
echo -e "Your toolchain will compile all crates in the workspace in release mode."
echo -e "This might take a few minutes as Cargo builds and optimizes everything."
echo -e "Enjoy watching the build process — this is the real compiler at work!"
echo ""

cargo build --release

# 3. Launch Rusty
echo ""
echo -e "${GREEN}✓ Build complete!${NC}"
echo -e "${BLUE}[3/3] Launching Rusty...${NC}"

if [ -f "./target/release/rusty" ]; then
    ./target/release/rusty
else
    echo -e "${YELLOW}⚠ Could not locate the built binary at './target/release/rusty'.${NC}"
    exit 1
fi
