{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs.buildPackages; [
    rustc cargo rustfmt clippy
    automake autoconf269 gnumake gcc libtool
  ];

  buildInputs = with pkgs.buildPackages; [
    openssl_1_1 libevent zlib pkg-config
  ];

  shellHook = ''
  '';
}

