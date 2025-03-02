{
  description = "A crowdsec bouncer using nftables and telegram";

  inputs = {
    nixpkgs.url = "nixpkgs";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      pre-commit-hooks,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        toolchainFromFile = (pkgs.rust-bin.fromRustupToolchainFile "${self}/rust-toolchain.toml");
        toolchain = toolchainFromFile.override {
          extensions = [
            "rust-src"
            "rustc"
            "cargo"
            "clippy"
            "rustfmt"
            "rust-analyzer"
          ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        mkDerivation =
          {
            pkgs,
            rustPlatform,
            openssl,
          }:
          rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            buildInputs = with pkgs; [
              openssl.dev
            ];

            src = ./.;

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            env = {
              OPENSSL_DIR = "${openssl.dev}";
              OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
              OPENSSL_LIB_DIR = "${openssl.out}/lib";
            };

          };
      in
      {
        checks.pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            convco.enable = true;
            rustfmt.enable = true;
            rustfmt.package = toolchain;
            rustfmt.packageOverrides.cargo = toolchain;
            rustfmt.packageOverrides.rustfmt = toolchain;
          };
        };

        devShells.default = pkgs.mkShell {
          name = cargoToml.package.name;
          inputsFrom = [ self.packages.${system}.${cargoToml.package.name} ];
          packages = with pkgs; [
            cargo-bloat
          ];

          shellHook = self.checks.${system}.pre-commit-check.shellHook;
          env.PRE_COMMIT_COLOR = "never";
        };

        packages.${cargoToml.package.name} = pkgs.callPackage mkDerivation { };

        cross."aarch64-unknown-linux-gnu"."${cargoToml.package.name}" =

          let
            pkgs' = import nixpkgs {
              inherit system overlays;
              crossSystem = {
                config = "aarch64-unknown-linux-gnu";
                rustc.config = "aarch64-unknown-linux-gnu";
              };
            };

            toolchainFromFile = (pkgs.rust-bin.fromRustupToolchainFile "${self}/rust-toolchain.toml");
            toolchain = toolchainFromFile.override {
              extensions = [
                "rustc"
                "cargo"
              ];
              targets = [ "aarch64-unknown-linux-gnu" ];
            };
            rustPlatform = pkgs'.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };
          in
          (pkgs'.pkgsCross.aarch64-multiplatform.callPackage mkDerivation { inherit rustPlatform; });
      }
    );
}
