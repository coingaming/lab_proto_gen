#!/bin/sh

set -e

export NIXPKGS_ALLOW_BROKEN=1

./nix/bootstrap.sh

nix-build ./nix/default.nix -v --show-trace
