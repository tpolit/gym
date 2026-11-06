#!/bin/sh
# Photographie les planches HTML générées par planches.py (Chrome headless, hors sandbox).
cd "$(dirname "$0")/captures/_planches" || exit 1
shoot() {
  google-chrome --headless=new --disable-gpu --hide-scrollbars \
    --user-data-dir=/tmp/gym-chrome --force-device-scale-factor=1 \
    --window-size="$2" --screenshot="$PWD/$1.png" "file://$PWD/$1.html" 2>&1 | grep -o 'written.*'
}
shoot 01 1600,1260
shoot 02 2120,620
shoot 03 1850,900
shoot 04 1680,980
shoot 05 1480,980
shoot 06 1850,900
shoot 07 1680,980
shoot 08 1850,900
shoot 09 1850,900
shoot 10 1850,900
shoot 11 1480,1320
