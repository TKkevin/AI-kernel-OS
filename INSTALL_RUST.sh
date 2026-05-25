#!/bin/bash
# NEXUS OS - Rust Installation Script
# "Sometimes you gotta run before you can walk." - Stark Philosophy

echo "╔══════════════════════════════════════════════════════════╗"
echo "║                                                          ║"
echo "║   ${CYAN}NEXUS${RESET} OS - Rust Environment Setup                        ║"
echo "║                                                          ║"
echo "╚══════════════════════════════════════════════════════════╝"

# Check if rust is already installed
if command -v rustc &> /dev/null; then
    echo "✓ Rust is already installed"
    rustc --version
    cargo --version
else
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source the cargo environment
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    echo "✓ Rust installed successfully"
    rustc --version
    cargo --version
fi

echo ""
echo "Next steps:"
echo "  cd /workspace"
echo "  cargo build --release"
echo ""
