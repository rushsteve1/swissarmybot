#!/usr/bin/env bash
set -euo pipefail

FOSSIL_VERSION=$(fossil timeline -n 1 -F %h | head -1)

# Uses Podman through the symlink
docker build -t swiss_army_bot \
    --build-arg FOSSIL_VERSION="$FOSSIL_VERSION" \
    -f "$(dirname "$0")/../Dockerfile-cross" \
    "$(dirname "$0")/.."

docker save swiss_army_bot -o "$(dirname "$0")/../swiss_army_bot.image"
