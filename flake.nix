{
  description = "A rust packaging flake based on https://github.com/cpu/rust-flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } ({ self, withSystem, flake-parts-lib, ... }:
      let
        inherit (flake-parts-lib) importApply;
      in
      {
    systems = [ "x86_64-linux" ];
    perSystem = { config, self', pkgs, lib, system, ... }:
    let
      runtimeDeps = with pkgs; [ openssl ];
      buildDeps = with pkgs; [ pkg-config rustPlatform.bindgenHook ];
      devDeps = with pkgs; [ gdb ];

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      msrv = cargoToml.package.rust-version;

      rustPackage = features:
      (pkgs.makeRustPlatform {
        cargo = pkgs.rust-bin.nightly.latest.minimal;
        rustc = pkgs.rust-bin.nightly.latest.minimal;
      }).buildRustPackage {
        inherit (cargoToml.package) name version;
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        buildFeatures = features;
        buildInputs = runtimeDeps;
        nativeBuildInputs = buildDeps;
        # Uncomment if your cargo tests require networking or otherwise
        # don't play nicely with the Nix build sandbox:
        # doCheck = false;
        # PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };

      mkDevShell = rustc:
      pkgs.mkShell {
        shellHook = ''
          export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
        '';
        buildInputs = runtimeDeps;
        nativeBuildInputs = buildDeps ++ devDeps ++ [ rustc ];
      };
    in {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ (import inputs.rust-overlay) ];
      };

      packages.default = self'.packages.toricelli;
      devShells.default = self'.devShells.nightly;

      packages.toricelli = (rustPackage "");

      devShells.nightly = (mkDevShell (pkgs.rust-bin.selectLatestNightlyWith
      (toolchain: toolchain.default)));
      devShells.stable = (mkDevShell pkgs.rust-bin.stable.latest.default);
      devShells.msrv = (mkDevShell pkgs.rust-bin.stable.${msrv}.default);
    };
    # flake.nixosModules.default = importApply ./nixos-module.nix { localFlake = self; inherit withSystem; };
  });
}
