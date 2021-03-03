let nixpkgs = import ./nixpkgs20.nix;
in
{
  pkgs ? import nixpkgs {
    overlays = import ./overlay.nix;
  }
}:
with pkgs;
let
  rustPkgs = pkgs.rustBuilder.makePackageSet' {
    rustChannel = "stable";
    packageFun = import ./pkg.nix;
    workspaceSrc = ./..;
  };
in
  rustPkgs.workspace.lab_proto_gen {}
