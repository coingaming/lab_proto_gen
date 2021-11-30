#!/bin/sh

set -e

# Enable HTTPS support in wget and set nsswitch.conf to make resolution work within containers
apt update -y \
  && apt install -y openssl git wget xz-utils \
  && echo hosts: files dns > /etc/nsswitch.conf

# Download Nix and install it into the system.
NIX_VERSION=2.3.15
wget https://nixos.org/releases/nix/nix-${NIX_VERSION}/nix-${NIX_VERSION}-$(uname -m)-linux.tar.xz \
  && tar xf nix-${NIX_VERSION}-$(uname -m)-linux.tar.xz \
  && addgroup --system --gid 30000 nixbld \
  && for i in $(seq 1 30); do useradd --system --home-dir /var/empty --shell $(which nologin) -g nixbld -G nixbld --uid $((30000 + i)) nixbld$i ; done \
  && mkdir -m 0755 /etc/nix \
  && echo 'sandbox = false' > /etc/nix/nix.conf \
  && echo 'filter-syscalls = false' >> /etc/nix/nix.conf \
  && mkdir -m 0755 /nix && USER=root sh nix-${NIX_VERSION}-$(uname -m)-linux/install \
  && ln -s /nix/var/nix/profiles/default/etc/profile.d/nix.sh /etc/profile.d/ \
  && rm -r /nix-${NIX_VERSION}-$(uname -m)-linux* \
  && rm -rf /var/cache/apt/* \
  && /nix/var/nix/profiles/default/bin/nix-collect-garbage --delete-old \
  && /nix/var/nix/profiles/default/bin/nix-store --optimise \
  && /nix/var/nix/profiles/default/bin/nix-store --verify --check-contents
