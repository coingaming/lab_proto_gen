#!/bin/sh

set -e

PLATFORM=`uname -m`
if [ "$PLATFORM" = "aarch64" ]; then
  PLATFORM="arm64"
fi

NIX_CONF="http2 = false
extra-substituters = https://cache.nixos.org https://hydra.iohk.io https://all-hies.cachix.org file:///app/nix_ci_cache
trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ= all-hies.cachix.org-1:JjrzAOEUsD9ZMt8fdFbzo3jNAyEWlPAwdVuHw4RD43k=
"

echo "starting nixos container..."
docker run -it --rm \
  -e NIXPKGS_ALLOW_BROKEN=1 \
  -e ROBOT_SSH_KEY="$ROBOT_SSH_KEY" \
  -v "$(pwd):/app" \
  -v "nix:/nix" \
  -v "nix-19.09-root:/root" \
  -w "/app" "heathmont/nix:alpine-$PLATFORM-2.3.15" sh -c "
  echo \"$NIX_CONF\" >> /etc/nix/nix.conf && \
  ./nix/bootstrap.sh && \
  nix-shell ./nix/shell.nix \
    -I ssh-config-file=/tmp/.ssh/config \
    --argstr robotSshKey $ROBOT_SSH_KEY \
    --option sandbox false \
    --pure -v --show-trace \
    $@
  "
