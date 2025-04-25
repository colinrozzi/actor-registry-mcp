{
  description = "{{actor_name}} - A Theater actor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        pname = cargoToml.package.name;
        version = cargoToml.package.version;

        buildActor = args: rustPlatform.buildRustPackage (rec {
          inherit pname version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
            };
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          postBuild = ''
            mkdir -p $out/lib
            cp target/wasm32-unknown-unknown/release/*.wasm $out/lib/
          '';
        } // args);
      in
      {
        packages = {
          default = buildActor {};

          ${pname} = buildActor {};
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rustVersion.override { targets = [ "wasm32-unknown-unknown" ]; })
            pkg-config
            openssl
          ];

          shellHook = ''
            echo "{{actor_name}} development environment"
            echo "Run 'cargo build --target wasm32-unknown-unknown --release' to build"
          '';
        };
      }
    );
}