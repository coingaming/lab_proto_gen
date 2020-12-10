let
  nixpkgsMozilla = builtins.fetchGit {
    url = https://github.com/mozilla/nixpkgs-mozilla;
    rev = "8c007b60731c07dd7a052cce508de3bb1ae849b4";
  };
  cargo2nix = builtins.fetchTarball {
    url="https://github.com/cargo2nix/cargo2nix/archive/v0.8.3.tar.gz";
    sha256="1iiphmjflr0qr4qlcbj6slk4918pgld21l5cwpzzp3fq1fppnfki";
  };
  rustOverlay = import "${nixpkgsMozilla}/rust-overlay.nix";
  cargo2nixOverlay = import "${cargo2nix}/overlay";
in
  [
    (_: _: {cargo2nix = (import cargo2nix {}).package;})
    rustOverlay
    cargo2nixOverlay
  ]
