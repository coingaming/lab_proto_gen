let
  nixpkgsMozilla = builtins.fetchGit {
    url = https://github.com/mozilla/nixpkgs-mozilla;
    rev = "8c007b60731c07dd7a052cce508de3bb1ae849b4";
  };
  cargo2nix = builtins.fetchTarball {
    url="https://github.com/cargo2nix/cargo2nix/archive/0dec2a8d6313347dfd183f3cd52aa9d3b43e7ed2.tar.gz";
    sha256="045cfvvwirlyp0h36ppbpp8ygqj3nzwzxgmsijh5cm9ilslxm3ym";
  };
  rustOverlay = import "${nixpkgsMozilla}/rust-overlay.nix";
  cargo2nixOverlay = import "${cargo2nix}/overlay";
in
  [
    (_: _: {cargo2nix = (import cargo2nix {}).package;})
    rustOverlay
    cargo2nixOverlay
  ]
