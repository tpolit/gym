#!/bin/sh
# Generates the app PNG icons from icon.html (headless Chrome; outside the sandbox).
# Usage: sh design/icon/shoot.sh
cd "$(dirname "$0")" || exit 1
OUT="$PWD/../../public/icons"
mkdir -p "$OUT"
shoot() {
  google-chrome --headless=new --disable-gpu --hide-scrollbars \
    --user-data-dir=/tmp/gym-chrome --force-device-scale-factor=1 --window-size="$1,$1" \
    --virtual-time-budget=2000 --screenshot="$OUT/$2" "file://$PWD/icon.html" 2>&1 | grep -o 'written.*'
}
shoot 180 apple-touch-icon.png
shoot 192 icon-192.png
shoot 512 icon-512.png
