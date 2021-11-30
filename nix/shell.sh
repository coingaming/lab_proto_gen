#!/bin/sh

set -e

PLATFORM=`uname -m`

echo "starting nixos container..."
docker run -it --rm \
  -e NIXPKGS_ALLOW_BROKEN=1 \
  -e ROBOT_SSH_KEY="$ROBOT_SSH_KEY" \
  -v "$(pwd):/app" \
  -v "nix:/nix" \
  -v "nix-19.09-root:/root" \
  -w "/app" "heathmont/nix:alpine-$PLATFORM-2.3.15" sh -c "
  ./nix/bootstrap.sh &&
  nix-shell ./nix/shell.nix \
    -I ssh-config-file=/tmp/.ssh/config \
    --argstr robotSshKey $ROBOT_SSH_KEY \
    --option sandbox false \
    --pure -v --show-trace \
    $@
  "
