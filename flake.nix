{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Remove when 1.3.0 version lands to nixpkgs-unstable
        dav1d-dev = pkgs.dav1d.dev.overrideAttrs (oldAttrs: rec {
          inherit (oldAttrs) pname;
          version = "1.3.0";
          src = pkgs.fetchFromGitHub {
            owner = "videolan";
            repo = pname;
            rev = version;
            hash = "sha256-c7Dur+0HpteI7KkR9oo3WynoH/FCRaBwZA7bJmPDJp8=";
          };
        });
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = [
              openssl
              pkg-config
              dav1d-dev
              rust-bin.stable.latest.default
              rust-analyzer
              rustfmt
            ];
          };
        }
    );
}
