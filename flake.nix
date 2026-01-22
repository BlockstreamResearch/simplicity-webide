{
  description = "Simplicity web IDE";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
  { self
  , nixpkgs
  , flake-utils
  , rust-overlay
  , ...
  }:
  flake-utils.lib.eachSystem [
    "x86_64-linux"
    "aarch64-linux"
    "x86_64-darwin"
    "aarch64-darwin"
  ] (system:
    let
      overlays = [
        (import rust-overlay)
      ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      targets = [
        "wasm32-unknown-unknown"
      ];
      rust-min = pkgs.rust-bin.stable.latest.minimal.override {
        inherit targets;
      };
      rust-dev = pkgs.rust-bin.stable.latest.default.override {
        inherit targets;
        extensions = [
          "rust-src"
        ];
      };
      inputs-min = [
        rust-min
        pkgs.trunk
        pkgs.dart-sass
      ];
      inputs-dev = [
        rust-dev
        pkgs.trunk
        pkgs.dart-sass
        pkgs.gdb
        pkgs.llvm
        pkgs.just
        pkgs.wasm-pack
        pkgs.wasm-bindgen-cli
      ];
      CC_wasm32_unknown_unknown = "${pkgs.llvmPackages_16.clang-unwrapped}/bin/clang-16";
      AR_wasm32_unknown_unknown = "${pkgs.llvmPackages_16.libllvm}/bin/llvm-ar";
      CFLAGS_wasm32_unknown_unknown = "-I ${pkgs.llvmPackages_16.libclang.lib}/lib/clang/16/include/";
    in
    {
      devShells = {
        default = pkgs.mkShell.override {
          stdenv = pkgs.clang16Stdenv;
        } {
          buildInputs = inputs-dev;

          # Constants for compiler
          inherit CC_wasm32_unknown_unknown;
          inherit AR_wasm32_unknown_unknown;
          inherit CFLAGS_wasm32_unknown_unknown;

          # Constants for IDE setup
          RUST_TOOLCHAIN = "${rust-dev}/bin";
          RUST_STDLIB = "${rust-dev}/lib/rustlib/src/rust";
          DEBUGGER = "${pkgs.gdb}";
        };
        # Temporary shell until deployment happens in a nix derivation
        deploy = pkgs.mkShell.override {
          stdenv = pkgs.clang16Stdenv;
        } {
          buildInputs = inputs-min;
          inherit CC_wasm32_unknown_unknown;
          inherit AR_wasm32_unknown_unknown;
          inherit CFLAGS_wasm32_unknown_unknown;
        };
      };
    }
  );
}
