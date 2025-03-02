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
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, pre-commit-hooks, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        toolchainFromFile = (pkgs.rust-bin.fromRustupToolchainFile "${self}/rust-toolchain.toml");
        toolchain = toolchainFromFile.override {
          extensions = [ "rust-src" "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      with pkgs;
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

        devShells.default = mkShell {
          name = cargoToml.package.name;
          inputsFrom = [self.packages.${system}.${cargoToml.package.name}];
          packages = [
            cargo-bloat
            toolchain
          ];

          shellHook= self.checks.${system}.pre-commit-check.shellHook;
          env.PRE_COMMIT_COLOR = "never";
        };

        packages.${cargoToml.package.name} = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = [
            openssl
          ];

          src = ./.;

          
          nativeBuildInputs = [
            pkg-config
          ];
        };
      }
    );
}
