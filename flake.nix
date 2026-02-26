{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane .url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      appName = "cerebro";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.nightly.latest.default);
      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;

        nativeBuildInputs = [
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.openssl
        ];
      };

      cerebro = craneLib.buildPackage (commonArgs
        // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });

      dockerImage = pkgs.dockerTools.buildImage {
        name = appName;
        tag = "latest";
        copyToRoot = pkgs.buildEnv {
          name = "ffmpeg";
          paths = [
            pkgs.ffmpeg-full
            pkgs.libva-utils
            pkgs.intel-media-driver
            pkgs.intel-vaapi-driver
          ];
          pathsToLink = ["/bin"];
        };
        config = {
          Entrypoint = ["${cerebro}/bin/${appName}"];
          Env = [
            "DATA_PATH=/data"
            "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"

            "LIBVA_DRIVERS_PATH=${pkgs.intel-media-driver}/lib/dri:${pkgs.intel-vaapi-driver}/lib/dri"
            "LIBVA_DRIVER_NAME=iHD"
          ];
          Volumes = {"/data" = {};};
        };
      };
    in {
      checks = {
        inherit cerebro;
      };

      packages = {
        default = cerebro;
        docker = dockerImage;
      };

      apps.default = flake-utils.lib.mkApp {
        drv = cerebro;
      };

      devShells.default = craneLib.devShell {
        checks = self.checks.${system};

        RUST_LOG = "info";
        packages = with pkgs; [
          ffmpeg
          openssl
        ];
      };
    });
}
