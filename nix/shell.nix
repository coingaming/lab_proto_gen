let nixpkgs = import ./nixpkgs20.nix;
in
{
  pkgs ? import nixpkgs {
    overlays = import ./overlay.nix;
  }
}:
with pkgs;

stdenv.mkDerivation {
  name = "lab-proto-gen-env";
  buildInputs = [
    nix
    git
    rustc
    openssh
    cargo2nix
  ];
  TERM="xterm-256color";
  NIX_SSL_CERT_FILE="${cacert}/etc/ssl/certs/ca-bundle.crt";
  GIT_SSL_CAINFO="${cacert}/etc/ssl/certs/ca-bundle.crt";
  NIX_PATH="/nix/var/nix/profiles/per-user/root/channels";
  CARGO_NET_GIT_FETCH_WITH_CLI="true";
}
