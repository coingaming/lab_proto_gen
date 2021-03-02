#!/bin/sh

set -e

echo "starting nixos container..."
docker run -it --rm \
  -e NIXPKGS_ALLOW_BROKEN=1 \
  -e ROBOT_SSH_KEY="$ROBOT_SSH_KEY" \
  -v "$(pwd):/app" \
  -v "nix:/nix" \
  -v "nix-19.09-root:/root" \
  -w "/app" nixos/nix:2.3 sh -c "
  ./nix/bootstrap.sh &&
  nix-build ./nix/default.nix \
    -I ssh-config-file=/tmp/.ssh/config \
    --option sandbox false \
    --pure -v --show-trace
  "