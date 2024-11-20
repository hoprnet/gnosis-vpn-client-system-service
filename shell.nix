{ pkgs ? import <nixpkgs> { } }:
let
  linuxPkgs = with pkgs; lib.optional stdenv.isLinux (
    inotify-tools
  );
  macosPkgs = with pkgs; lib.optional stdenv.isDarwin (
    with darwin.apple_sdk.frameworks; [
      CoreFoundation
      CoreServices
      SystemConfiguration
    ]
  );
in
with pkgs;
mkShell {
  nativeBuildInputs = [
    cargo
    rustc
    clippy
    rust-analyzer
    rustfmt

    # openssl-sys crate
    pkg-config
    openssl

    # custom pkg groups
    linuxPkgs
    macosPkgs
  ];
}
