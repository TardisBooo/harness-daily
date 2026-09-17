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
  *)
    echo "Unsupported platform: $(uname -s)-$(uname -m); build from source with cargo." >&2
    exit 1
    ;;
esac

ZIP="harness-daily-${TARGET}.zip"
API="https://api.github.com/repos/${REPO}/releases/latest"
URL="$(curl -fsSL -H 'User-Agent: harness-daily-install' "$API" | sed -n "s/.*\"browser_download_url\": \"\\(.*${ZIP}\\)\".*/\\1/p" | head -n1)"
if [ -z "$URL" ]; then
  echo "Could not find $ZIP on the latest GitHub release." >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
curl -fsSL "$URL" -o "$TMP/hd.zip"
mkdir -p "$INSTALL_DIR"
unzip -oq "$TMP/hd.zip" -d "$TMP/out"
BIN="$(find "$TMP/out" -type f -name harness-daily | head -n1)"
if [ -z "$BIN" ]; then
  echo "harness-daily binary missing inside $ZIP" >&2
  exit 1
fi
mv "$BIN" "$INSTALL_DIR/harness-daily"
chmod +x "$INSTALL_DIR/harness-daily"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Note: add $INSTALL_DIR to PATH if 'harness-daily' is not found." ;;
esac

"$INSTALL_DIR/harness-daily" --version
echo "Next: harness-daily init --host auto --out <your-logs-dir>"
echo "      harness-daily schedule install --time 08:00"
