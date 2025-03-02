{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane .url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      appName = "cerebro";
      pkgs = nixpkgs.legacyPackages.${system};
      craneLib = crane.mkLib pkgs;
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
          paths = [pkgs.ffmpeg-headless];
          pathsToLink = ["/bin"];
        };
        config = {
          Entrypoint = ["${cerebro}/bin/${appName}"];
          Env = [
            "DATA_PATH=/data"
            "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
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
