{
  description = "hoprnet monorepo";

  inputs = {
    flake-utils.url = github:numtide/flake-utils;
    flake-parts.url = github:hercules-ci/flake-parts;
    nixpkgs.url = github:NixOS/nixpkgs/release-24.05;
    rust-overlay.url = github:oxalica/rust-overlay/master;
    # using a fork with an added source filter
    crane.url = github:hoprnet/crane/tb/20240117-find-filter;
    # pin it to a version which we are compatible with
    foundry.url = github:shazow/foundry.nix/e4c79767b4d2e51179d1975a9f0553ef30d82711;
    # use change to add solc 0.8.24
    solc.url = github:hoprnet/solc.nix/tb/20240129-solc-0.8.24;
    pre-commit.url = github:cachix/pre-commit-hooks.nix;
    treefmt-nix.url = github:numtide/treefmt-nix;
    flake-root.url = github:srid/flake-root;

    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    foundry.inputs.flake-utils.follows = "flake-utils";
    foundry.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs-stable.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    solc.inputs.flake-utils.follows = "flake-utils";
    solc.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, flake-parts, flake-root, rust-overlay, crane, foundry, solc, pre-commit, treefmt-nix, ... }@inputs:
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
          overlays = [ (import rust-overlay) foundry.overlay solc.overlay ];
          pkgs = import nixpkgs {
            inherit localSystem overlays;
          };
          solcDefault = solc.mkDefault pkgs pkgs.solc_0_8_19;
          rustNightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          craneLibNightly = (crane.mkLib pkgs).overrideToolchain rustNightly;
          hoprdCrateInfoOriginal = craneLibNightly.crateNameFromCargoToml {
            cargoToml = ./hopr/hopr-lib/Cargo.toml;
          };
          hoprdCrateInfo = {
            pname = "hoprd";
            # normalize the version to not include any suffixes so the cache
            # does not get busted
            version = pkgs.lib.strings.concatStringsSep "."
              (pkgs.lib.lists.take 3 (builtins.splitVersion hoprdCrateInfoOriginal.version));
          };
          depsSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor/cargo
              ./.cargo/config.toml
              ./Cargo.lock
              (fs.fileFilter (file: file.name == "Cargo.toml") ./.)
            ];
          };

          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor/cargo
              ./.cargo/config.toml
              ./Cargo.lock
              ./README.md
              ./hopr/hopr-lib/data
              ./ethereum/contracts/contracts-addresses.json
              ./ethereum/contracts/foundry.toml.in
              ./ethereum/contracts/remappings.txt
              ./hoprd/hoprd/example_cfg.yaml
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
            ];
          };

          rust-builder-local = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
          };

          rust-builder-local-nightly = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            useRustNightly = true;
          };

          rust-builder-x86_64-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.gnu64;
          };

          rust-builder-x86_64-darwin = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.x86_64-darwin;
          };

          rust-builder-aarch64-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.aarch64-multiplatform;
          };

          rust-builder-aarch64-darwin = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.aarch64-darwin;
          };

          rust-builder-armv7l-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.armv7l-hf-multiplatform;
          };

          gnovpnBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "-p gnosis-vpn";
            cargoToml = ./gnosis-vpn/Cargo.toml;
          };

          gnovpnctlBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "-p gnosis-vpn-ctl";
            cargoToml = ./gnosis-vpn-ctl/Cargo.toml;
          };

          gnovpn = rust-builder-local.callPackage ./nix/rust-package.nix gnovpnBuildArgs;
          gnovpnctl = rust-builder-local.callPackage ./nix/rust-package.nix gnovpnctlBuildArgs;

          gnovpn-x86_64-linux = rust-builder-local.callPackage ./nix/rust-package.nix gnovpnBuildArgs;
          gnovpnctl-x86_64-linux = rust-builder-local.callPackage ./nix/rust-package.nix gnovpnctlBuildArgs;
          # hoprd-x86_64-linux = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # hoprd-aarch64-linux = rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # hoprd-armv7l-linux = rust-builder-armv7l-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # CAVEAT: must be built from a darwin system
          # hoprd-x86_64-darwin = rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # CAVEAT: must be built from a darwin system
          # hoprd-aarch64-darwin = rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix hoprdBuildArgs;

          gnovpn-clippy = rust-builder-local.callPackage ./nix/rust-package.nix (gnovpnBuildArgs // { runClippy = true; });
          gnovpnctl-clippy = rust-builder-local.callPackage ./nix/rust-package.nix (gnovpnctlBuildArgs // { runClippy = true; });
          gnovpn-debug = rust-builder-local.callPackage ./nix/rust-package.nix (gnovpnBuildArgs // { CARGO_PROFILE = "dev"; });
          gnovpnctl-debug = rust-builder-local.callPackage ./nix/rust-package.nix (gnovpnctlBuildArgs // { CARGO_PROFILE = "dev"; });

          pre-commit-check = pre-commit.lib.${system}.run {
            src = ./.;
            hooks = {
              treefmt.enable = false;
              treefmt.package = config.treefmt.build.wrapper;
              immutable-files = {
                enable = false;
                name = "Immutable files - the files should not change";
                entry = "bash .github/scripts/immutable-files-check.sh";
                files = "";
                language = "system";
              };
            };
            tools = pkgs;
          };

          defaultDevShell = import ./nix/shell.nix { inherit pkgs config crane pre-commit-check solcDefault; };

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
          update-github-labels = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "update-github-labels" ''
              set -eu
              # remove existing crate entries (to remove old crates)
              yq 'with_entries(select(.key != "crate:*"))' .github/labeler.yml > labeler.yml.new
              # add new crate entries for known crates
              for f in `find . -mindepth 2 -name "Cargo.toml" -type f ! -path "./vendor/*" -printf '%P\n'`; do
              	env \
              		name="crate:`yq '.package.name' $f`" \
              		dir="`dirname $f`/**" \
              		yq -n '.[strenv(name)][0]."changed-files"[0]."any-glob-to-any-file" = env(dir)' >> labeler.yml.new
              done
              mv labeler.yml.new .github/labeler.yml
            '';
          };
        in
        {
          treefmt = {
            inherit (config.flake-root) projectRootFile;

            programs.yamlfmt.enable = true;
            settings.formatter.yamlfmt.includes = [ "./.github/labeler.yml" "./.github/workflows/*.yaml" ];
            settings.formatter.yamlfmt.excludes = [ "./vendor/*" ];

            programs.prettier.enable = true;
            settings.formatter.prettier.includes = [ "*.md" "*.json" "./ethereum/contracts/README.md" ];
            settings.formatter.prettier.excludes = [ "./vendor/*" "./ethereum/contracts/*" "*.yml" "*.yaml" ];

            programs.rustfmt.enable = true;
            settings.formatter.rustfmt.excludes = [ "./vendor/*" "./db/entity/src/codegen/*" "./ethereum/bindings/src/codegen/*" ];

            programs.nixpkgs-fmt.enable = true;
            settings.formatter.nixpkgs-fmt.excludes = [ "./vendor/*" ];

            programs.taplo.enable = true;
            settings.formatter.taplo.excludes = [ "./vendor/*" "./ethereum/contracts/*" ];

            settings.formatter.solc = {
              command = "sh";
              options = [
                "-euc"
                ''
                  # must generate the foundry.toml here, since this step could
                  # be executed in isolation
                  if ! grep -q "solc = \"${solcDefault}/bin/solc\"" ethereum/contracts/foundry.toml; then
                    echo "solc = \"${solcDefault}/bin/solc\""
                    echo "Generating foundry.toml file!"
                    sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
                      ethereum/contracts/foundry.toml.in >| \
                      ethereum/contracts/foundry.toml
                  else
                    echo "foundry.toml file already exists!"
                  fi

                  for file in "$@"; do
                    ${pkgs.foundry-bin}/bin/forge fmt $file \
                      --root ./ethereum/contracts;
                  done
                ''
                "--"
              ];
              includes = [ "*.sol" ];
              excludes = [ "./vendor/*" "./ethereum/contracts/src/static/*" ];
            };
          };

          checks = {
            inherit gnovpn-clippy gnovpnctl-clippy;
          };

          apps = {
            inherit update-github-labels;
            check = run-check;
          };

          packages = {
            inherit gnovpn gnovpn-debug;
            inherit gnovpnctl gnovpnctl-debug;
            inherit gnovpn-x86_64-linux;
            inherit gnovpnctl-x86_64-linux;
            # inherit gnovpn-aarch64-linux gnovpn-armv7l-linux gnovpn-x86_64-linux;
            # inherit gnovpnctl-aarch64-linux gnovpnctl-armv7l-linux gnovpnctl-x86_64-linux;
            # FIXME: Darwin cross-builds are currently broken.
            # Follow https://github.com/nixos/nixpkgs/pull/256590
            # inherit hoprd-aarch64-darwin hoprd-x86_64-darwin;
            # inherit hopli-aarch64-darwin hopli-x86_64-darwin;
            default = gnovpn;
          };

          devShells.default = defaultDevShell;

          formatter = config.treefmt.build.wrapper;
        };
      # platforms which are supported as build environments
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
    };
}
