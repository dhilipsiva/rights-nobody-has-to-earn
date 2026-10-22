# Dev shell for the verifier, the generator and the reading interface, cut from the portfolio's
# template (products/templates/flake.nix).
#
#   nix develop          # the pinned Rust toolchain with the wasm32 target, git-lfs,
#                        # wasm-bindgen-cli, uv and Python for the book tools, just, cargo-deny
#
# The Nibli engine itself is a separate checkout beside this one (./bootstrap.sh).
{
  description = "rights-nobody-has-to-earn: verifier, generator and reading interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Equal to `rust-version` in Cargo.toml and ui/Cargo.toml, and to the toolchain the Nibli
        # engine pins. Move them together.
        rustVersion = "1.94.0";

        rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [
            "rust-src"
            "clippy"
            "rustfmt"
          ];
          # The reading interface's web build.
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "rights-dev";
          packages = [
            rustToolchain
            pkgs.just
            pkgs.cargo-deny
            # The constitution is stored with Git LFS (README, Verify).
            pkgs.git-lfs
            # The version ui/Cargo.lock pins for wasm-bindgen.
            pkgs.wasm-bindgen-cli_0_2_128
            # The book tools run as uv scripts, pinned by their *.py.lock files.
            pkgs.uv
            pkgs.python3
          ];
          shellHook = ''
            echo "$(cargo --version) | $(just --version) | $(git lfs version | cut -d" " -f1)"
          '';
        };
      }
    );
}
