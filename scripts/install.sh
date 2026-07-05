#!/usr/bin/env bash
# Install the ait release binary (agent-it-tools) for this platform.
#
# Usage: bash install.sh [version]   (default: latest release)
# Destination: ~/.local/bin/ait
#
# Prefers `gh release download` (works for private repos with gh auth),
# falls back to anonymous curl from the public release URL.

set -euo pipefail

REPO="mck/agent-it-tools"
VERSION="${1:-latest}"
DEST_DIR="${AGENT_IT_TOOLS_BIN:-$HOME/.local/bin}"

if command -v ait >/dev/null 2>&1 && [ "$VERSION" = "latest" ]; then
    echo "already installed: $(command -v ait) ($(ait --version))"
    exit 0
fi

case "$(uname -s)" in
    Darwin) os="apple-darwin" ;;
    Linux)  os="unknown-linux-musl" ;;
    *) echo '{"error":"unsupported OS; build from source: cargo install --git https://github.com/mck/agent-it-tools"}' >&2; exit 1 ;;
esac
case "$(uname -m)" in
    arm64|aarch64) arch="aarch64" ;;
    x86_64|amd64)  arch="x86_64" ;;
    *) echo '{"error":"unsupported architecture"}' >&2; exit 1 ;;
esac
target="${arch}-${os}"
asset="ait-${target}.tar.gz"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    if [ "$VERSION" = "latest" ]; then
        gh release download --repo "$REPO" --pattern "$asset" --dir "$tmp"
    else
        gh release download "$VERSION" --repo "$REPO" --pattern "$asset" --dir "$tmp"
    fi
else
    if [ "$VERSION" = "latest" ]; then
        url="https://github.com/$REPO/releases/latest/download/$asset"
    else
        url="https://github.com/$REPO/releases/download/$VERSION/$asset"
    fi
    curl -fsSL "$url" -o "$tmp/$asset"
fi

tar -xzf "$tmp/$asset" -C "$tmp"
mkdir -p "$DEST_DIR"
install -m 755 "$tmp/ait" "$DEST_DIR/ait"
echo "installed: $DEST_DIR/ait ($("$DEST_DIR/ait" --version))"

case ":$PATH:" in
    *":$DEST_DIR:"*) ;;
    *) echo "note: $DEST_DIR is not on PATH; invoke via the full path or add it" ;;
esac
