#!/bin/bash

# ports-tool installation script
set -e

echo "🔧 Installing ports-tool..."

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

# Download the latest binary from GitHub releases
if ! curl -L "https://github.com/eyalev/ports-tool/releases/latest/download/ports-tool" -o ports-tool; then
    echo "❌ Failed to download binary"
    echo "You can build from source instead:"
    echo "git clone https://github.com/eyalev/ports-tool.git && cd ports-tool && cargo build --release"
    exit 1
fi

echo "📦 Installing to /usr/local/bin..."
chmod +x ports-tool

if ! sudo mv ports-tool /usr/local/bin/; then
    echo "❌ Failed to install to /usr/local/bin (need sudo access)"
    echo "You can install to your home directory instead:"
    echo "mkdir -p ~/.local/bin && mv ports-tool ~/.local/bin/"
    echo "Then add ~/.local/bin to your PATH"
    exit 1
fi

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