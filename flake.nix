{
  description = "GnosisVPN client service";

  inputs = {
    flake-utils.url = github:numtide/flake-utils;
    flake-parts.url = github:hercules-ci/flake-parts;
    nixpkgs.url = github:NixOS/nixpkgs/release-24.05;
    rust-overlay.url = github:oxalica/rust-overlay/master;
    # using a fork with an added source filter
    crane.url = github:hoprnet/crane/tb/20240117-find-filter;
    # pin it to a version which we are compatible with
    pre-commit.url = github:cachix/pre-commit-hooks.nix;
    treefmt-nix.url = github:numtide/treefmt-nix;
    flake-root.url = github:srid/flake-root;

    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs-stable.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, flake-parts, flake-root, rust-overlay, crane, pre-commit, treefmt-nix, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.flake-root.flakeModule
      ];
      perSystem = { config, lib, self', inputs', system, ... }:
        let
          rev = toString (self.shortRev or self.dirtyShortRev);
          fs = lib.fileset;
          localSystem = system;
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit localSystem overlays;
          };
          rustNightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          craneLibNightly = (crane.mkLib pkgs).overrideToolchain rustNightly;

          depsSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor
              ./.cargo/config.toml
              ./Cargo.lock
              (fs.fileFilter (file: file.name == "Cargo.toml") ./.)
            ];
          };

          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor
              ./.cargo/config.toml
              ./Cargo.lock
              ./README.md
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
            ];
          };

          rust-builder-local = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
          };

          rust-builder-local-nightly = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
            useRustNightly = true;
          };

          rust-builder-x86_64-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
            crossSystem = pkgs.lib.systems.examples.gnu64;
            isCross = true;
          };

          rust-builder-x86_64-darwin = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
            crossSystem = pkgs.lib.systems.examples.x86_64-darwin;
            isCross = true;
          };

          rust-builder-aarch64-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
            crossSystem = pkgs.lib.systems.examples.aarch64-multiplatform;
            isCross = true;
          };

          rust-builder-aarch64-darwin = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
            crossSystem = pkgs.lib.systems.examples.aarch64-darwin;
            isCross = true;
          };

          rust-builder-armv7l-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
            crossSystem = pkgs.lib.systems.examples.armv7l-hf-multiplatform;
            isCross = true;
          };

          gnosisvpnBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "--all";
            cargoToml = ./Cargo.toml;
          };

          gnosisvpn = rust-builder-local.callPackage ./nix/rust-package.nix gnosisvpnBuildArgs;

          gnosisvpn-x86_64-linux = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix gnosisvpnBuildArgs;

          gnosisvpn-aarch64-linux = rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix gnosisvpnBuildArgs;

          gnosisvpn-armv7l-linux = rust-builder-armv7l-linux.callPackage ./nix/rust-package.nix gnosisvpnBuildArgs;
          # CAVEAT: must be built from a darwin system
          gnosisvpn-x86_64-darwin = rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix gnosisvpnBuildArgs;
          # CAVEAT: must be built from a darwin system
          gnosisvpn-aarch64-darwin = rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix gnosisvpnBuildArgs;

          gnosisvpn-debug = rust-builder-local.callPackage ./nix/rust-package.nix (gnosisvpnBuildArgs // { CARGO_PROFILE = "dev"; });

          defaultDevShell = import ./nix/shell.nix { inherit pkgs config crane; };

          run-check = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "run-check" ''
              set -e
              check=$1
              if [ -z "$check" ]; then
                nix flake show --json 2>/dev/null | \
                  jq -r '.checks."${system}" | to_entries | .[].key' | \
                  xargs -I '{}' nix build ".#checks."${system}".{}"
              else
              	nix build ".#checks."${system}".$check"
              fi
            '';
          };
        in
        {
          treefmt = {
            inherit (config.flake-root) projectRootFile;

            programs.yamlfmt.enable = true;
            settings.formatter.yamlfmt.includes = [ "./.github/workflows/*.yaml" ];
            settings.formatter.yamlfmt.excludes = [ "./vendor/*" ];

            programs.prettier.enable = true;
            settings.formatter.prettier.includes = [ "*.md" "*.json" ];
            settings.formatter.prettier.excludes = [ "./vendor/*" "*.yml" "*.yaml" ];

            programs.rustfmt.enable = true;
            settings.formatter.rustfmt.excludes = [ "./vendor/*" ];

            programs.nixpkgs-fmt.enable = true;
            settings.formatter.nixpkgs-fmt.excludes = [ "./vendor/*" ];
          };

          checks = {
            lint = rust-builder-local.callPackage ./nix/rust-package.nix (gnosisvpnBuildArgs // { runClippy = true; });

            tests = rust-builder-local.callPackage ./nix/rust-package.nix (gnosisvpnBuildArgs // { runTests = true; });

            fmt = pkgs.runCommand "fmt-check" { 
              buildInputs = [ pkgs.rustc pkgs.cargo pkgs.rustfmt ];
            } ''
              cargo test --manifest-path ./Cargo.toml
              cargo fmt --check
              touch $out
            '';
          };

          packages = {
            inherit gnosisvpn gnosisvpn-debug;
            inherit gnosisvpn-aarch64-linux gnosisvpn-armv7l-linux gnosisvpn-x86_64-linux;
            # FIXME: Darwin cross-builds are currently broken.
            # Follow https://github.com/nixos/nixpkgs/pull/256590
            inherit gnosisvpn-aarch64-darwin gnosisvpn-x86_64-darwin;
            default = gnosisvpn;
          };

          devShells.default = defaultDevShell;

          formatter = config.treefmt.build.wrapper;
        };
      # platforms which are supported as build environments
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
    };
}
