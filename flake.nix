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
          name = "crowdsec-gatekeeper";
          packages = [
            cargo-bloat
          ];

          buildInputs = [
            pkg-config
            openssl
            gcc
            cmake
            toolchain
          ];
          shellHook= self.checks.${system}.pre-commit-check.shellHook;
          env.PRE_COMMIT_COLOR = "never";
        };
      }
    );
}
