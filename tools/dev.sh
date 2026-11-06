#!/bin/bash
# Runs a command in the development Docker container, with the tools from `.toolchain/`.
# Usage: tools/dev.sh <command...>         e.g. tools/dev.sh cargo test
#        tools/dev.sh                      (interactive shell)
# The container (`simak_$USER`, created by the `simak_run` alias) mounts /home/$USER: the repository has the same path.
# On a machine without this container (personal computer), source `tools/env.sh` directly.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTAINER="${GYM_CONTAINER:-simak_$USER}"

if ! docker ps --filter "name=^${CONTAINER}$" --filter "status=running" --format '{{.Names}}' | grep -q .; then
  echo "Container $CONTAINER is stopped: start it with \`simak_run\`, or source tools/env.sh outside Docker." >&2
  exit 1
fi

TTY_FLAGS="-i"
[ -t 0 ] && [ -t 1 ] && TTY_FLAGS="-it"

if [ $# -eq 0 ]; then
  exec docker exec $TTY_FLAGS -w "$PWD" --user "$USER" "$CONTAINER" bash --rcfile "$ROOT/tools/env.sh" -i
fi
exec docker exec $TTY_FLAGS -w "$PWD" --user "$USER" "$CONTAINER" bash -c '. "$0/tools/env.sh" && exec "$@"' "$ROOT" "$@"
