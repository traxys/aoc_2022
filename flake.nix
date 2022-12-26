{
  description = "A basic flake with a shell";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.naersk.url = "github:nix-community/naersk";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rust = pkgs.rust-bin.beta.latest.default;
    in {
      devShell = pkgs.mkShell {
        nativeBuildInputs = [rust pkgs.cargo-criterion];
        RUST_PATH = "${rust}";
        shellHook = ''
          alias rstddoc="firefox ${rust}/share/doc/rust/html/std/index.html"
        '';
      };
    });
}
