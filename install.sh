#!/bin/bash

# ports-tool installation script
set -e

echo "🔧 Installing ports-tool..."

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        echo "Please build from source: https://github.com/eyalev/ports-tool#from-source"
        exit 1
        ;;
esac

# Check if we have required tools
if ! command -v curl &> /dev/null; then
    echo "❌ curl is required but not installed"
    echo "Install with: sudo apt update && sudo apt install curl"
    exit 1
fi

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "📥 Downloading ports-tool binary..."

# For now, build from source since we don't have releases yet
if ! command -v cargo &> /dev/null; then
    echo "⚠️  Rust/Cargo not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Clone and build
echo "🔨 Building from source..."
git clone https://github.com/eyalev/ports-tool.git
cd ports-tool
cargo build --release

echo "📦 Installing to /usr/local/bin..."
sudo cp target/release/ports-tool /usr/local/bin/

# Cleanup
cd /
rm -rf "$TMP_DIR"

echo "✅ ports-tool installed successfully!"
echo ""
echo "Usage:"
echo "  ports-tool          # Show localhost ports"
echo "  ports-tool -c       # Compact format"
echo "  ports-tool -d       # Detailed format"
echo "  ports-tool --help   # Show all options"
echo ""
echo "Repository: https://github.com/eyalev/ports-tool"