#!/bin/bash
set -e

# AgentMem One-Line Installer
# Usage: curl -sSL https://install.agentmem.dev | sh

REPO="jiyujie2006/agentmem"
BINARY="agentmem"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
            aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    darwin)
        case "$ARCH" in
            x86_64) TARGET="x86_64-apple-darwin" ;;
            arm64) TARGET="aarch64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# For local development, just copy from cargo build
if [ -f "target/release/$BINARY" ]; then
    echo "📦 Installing from local build..."
    cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
    chmod +x "$INSTALL_DIR/$BINARY"
    echo "✅ Installed to $INSTALL_DIR/$BINARY"
    echo ""
    echo "💡 Make sure $INSTALL_DIR is in your PATH"
    echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
    exit 0
fi

# Download from GitHub Release
VERSION="${VERSION:-latest}"
if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/$REPO/releases/latest/download/${BINARY}-${TARGET}.tar.gz"
else
    URL="https://github.com/$REPO/releases/download/$VERSION/${BINARY}-${TARGET}.tar.gz"
fi

echo "📥 Downloading AgentMem for $TARGET..."
mkdir -p "$INSTALL_DIR"

if command -v curl >/dev/null 2>&1; then
    curl -sSL "$URL" | tar -xz -C "$INSTALL_DIR" "$BINARY"
elif command -v wget >/dev/null 2>&1; then
    wget -qO- "$URL" | tar -xz -C "$INSTALL_DIR" "$BINARY"
else
    echo "❌ curl or wget is required"
    exit 1
fi

chmod +x "$INSTALL_DIR/$BINARY"

echo ""
echo "✅ AgentMem installed to $INSTALL_DIR/$BINARY"
echo ""
echo "🚀 Get started:"
echo "   agentmem init"
echo "   agentmem --help"
echo ""
echo "💡 Make sure $INSTALL_DIR is in your PATH:"
echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
