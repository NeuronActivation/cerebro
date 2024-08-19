{
  description = "Discord bot that proxies AV1 to Discord supported formats";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      appName = "yliproxy";
      pkgs = nixpkgs.legacyPackages.${system};
      craneLib = crane.mkLib pkgs;
      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;

        nativeBuildInputs = [
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.dav1d
          pkgs.openssl
        ];
      };

      yliproxy = craneLib.buildPackage (commonArgs
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
          Entrypoint = ["${yliproxy}/bin/${appName}"];
          Env = ["SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"];
        };
      };
    in {
      checks = {
        inherit yliproxy;
      };

      packages = {
        default = yliproxy;
        docker = dockerImage;
      };

      apps.default = flake-utils.lib.mkApp {
        drv = yliproxy;
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
