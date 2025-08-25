#!/bin/bash

# ports-tool installation/uninstallation script
set -e

INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="ports-tool"
REPO_URL="https://github.com/eyalev/ports-tool"

show_usage() {
    echo "ports-tool installer"
    echo ""
    echo "Usage:"
    echo "  $0 [install]    Install ports-tool (default)"
    echo "  $0 uninstall    Uninstall ports-tool"
    echo "  $0 --help       Show this help"
    echo ""
    echo "Installation location: $INSTALL_DIR"
}

install_ports_tool() {
    echo "🔧 Installing ports-tool..."

    # Check if we have required tools
    if ! command -v curl &> /dev/null; then
        echo "❌ curl is required but not installed"
        echo "Install with: sudo apt update && sudo apt install curl"
        exit 1
    fi

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"

    echo "📥 Downloading ports-tool binary..."

    # Download the latest binary from GitHub releases
    if ! curl -L "$REPO_URL/releases/latest/download/$BINARY_NAME" -o "$BINARY_NAME"; then
        echo "❌ Failed to download binary"
        echo "You can build from source instead:"
        echo "git clone $REPO_URL.git && cd ports-tool && cargo build --release"
        exit 1
    fi

    echo "📦 Installing to $INSTALL_DIR..."
    chmod +x "$BINARY_NAME"
    mv "$BINARY_NAME" "$INSTALL_DIR/"

    # Cleanup
    cd /
    rm -rf "$TMP_DIR"

    echo "✅ ports-tool installed successfully!"
    echo ""
    
    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "⚠️  $INSTALL_DIR is not in your PATH"
        echo "Add this to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
        echo "Or run: echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
        echo "Then restart your terminal or run: source ~/.bashrc"
        echo ""
    fi

    echo "Usage:"
    echo "  ports-tool          # Show localhost ports"
    echo "  ports-tool -c       # Compact format"
    echo "  ports-tool -d       # Detailed format"  
    echo "  ports-tool --help   # Show all options"
    echo ""
    echo "To uninstall: curl -sSL https://raw.githubusercontent.com/eyalev/ports-tool/master/install.sh | bash -s uninstall"
    echo "Repository: $REPO_URL"
}

uninstall_ports_tool() {
    echo "🗑️  Uninstalling ports-tool..."
    
    if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        rm "$INSTALL_DIR/$BINARY_NAME"
        echo "✅ ports-tool removed from $INSTALL_DIR"
    else
        echo "❌ ports-tool not found in $INSTALL_DIR"
        
        # Check common system locations
        SYSTEM_LOCATIONS=("/usr/local/bin/$BINARY_NAME" "/usr/bin/$BINARY_NAME")
        for location in "${SYSTEM_LOCATIONS[@]}"; do
            if [ -f "$location" ]; then
                echo "Found ports-tool at $location"
                echo "Remove with: sudo rm $location"
            fi
        done
        exit 1
    fi
    
    echo "🧹 Uninstall complete!"
}

# Parse command line arguments
case "${1:-install}" in
    "install"|"")
        install_ports_tool
        ;;
    "uninstall")
        uninstall_ports_tool
        ;;
    "--help"|"-h")
        show_usage
        ;;
    *)
        echo "Unknown command: $1"
        show_usage
        exit 1
        ;;
esac