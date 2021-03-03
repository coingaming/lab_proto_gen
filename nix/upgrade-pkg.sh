#!/bin/sh

set -e

rm -rf ./nix/pkg.nix
cargo2nix -f ./nix/pkg.nix
