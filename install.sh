#!/usr/bin/env sh
# harness-daily installer (macOS / Linux)
# Usage: curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash
set -eu

REPO="TardisBooo/harness-daily"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) TARGET="aarch64-apple-darwin" ;;
  Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
  Linux-x86_64)
    if ldd --version 2>/dev/null | grep -q musl; then
      TARGET="x86_64-unknown-linux-musl"
    else
      TARGET="x86_64-unknown-linux-gnu"
    fi
    ;;
  *) echo "Unsupported platform: $(uname -s)-$(uname -m); build from source with cargo." >&2; exit 1 ;;
esac

if ! command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI (gh) is required: https://cli.github.com" >&2
  exit 1
fi

TAG="$(gh api "repos/$REPO/releases/latest" --jq .tag_name)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

gh release download "$TAG" --repo "$REPO" --pattern "harness-daily-$TARGET.zip" --output "$TMP/hd.zip"
mkdir -p "$INSTALL_DIR"
unzip -oq "$TMP/hd.zip" -d "$TMP/out"
BIN="$(find "$TMP/out" -type f -name harness-daily | head -n1)"
mv "$BIN" "$INSTALL_DIR/harness-daily"
chmod +x "$INSTALL_DIR/harness-daily"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Note: $INSTALL_DIR is not on your PATH." ;;
esac

"$INSTALL_DIR/harness-daily" --version
echo "Next: harness-daily init --host grok --out <your-logs-dir>"
