#!/bin/sh
# Trunk `post_build` hook (Trunk.toml): fills `sw.js` in the build directory with
#  - the list of produced files (hashed names), cached as soon as it installs;
#  - a version derived from their content (new cache whenever it changes).
# Trunk provides TRUNK_STAGING_DIR. Portable across Linux / macOS (no `sed -i`).
set -eu

cd "${TRUNK_STAGING_DIR:?variable provided by Trunk}"

if command -v sha256sum >/dev/null 2>&1; then
  sha256() { sha256sum "$@"; }
else
  sha256() { shasum -a 256 "$@"; }
fi

files=$(find . -type f ! -name sw.js | sed 's|^\./||' | LC_ALL=C sort)
# Build file names contain neither spaces nor "|".
# shellcheck disable=SC2086
version=$(sha256 $files | sha256 | cut -c1-16)
# shellcheck disable=SC2086
list=$(printf '"%s",' $files)

sed -e "s|__SW_VERSION__|$version|" -e "s|/\* __SW_PRECACHE__ \*/|$list|" sw.js > sw.js.tmp
mv sw.js.tmp sw.js
echo "sw.js : version $version"
