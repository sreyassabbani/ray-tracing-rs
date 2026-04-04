{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/DeterminateSystems/nixpkgs-weekly/0.tar.gz";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };

        llvm = pkgs.llvmPackages_21;

        # Stable toolchain (already pinned by flake.lock via the fenix input)
        pinned = pkgs.fenix.stable;

        # One derivation containing exactly the components we want.
        rust = pinned.withComponents [
          "cargo"
          "rustc"
          "rustfmt"
          "clippy"
          "rust-src"
        ];

        # Nightly rust-analyzer from fenix overlay (pkgs.rust-analyzer is also fine if you prefer)
        ra = pkgs.rust-analyzer-nightly;
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            with pkgs; [
              rust
              ra
              imagemagick
              llvm.lldb
              bacon
            ];

          # shellHook = ''
          #   echo "Nix dev shell activated"
          #   export RUST_SRC_PATH="${pinned.rust-src}/lib/rustlib/src/rust/library"

          #   echo "rustc: $(rustc --version 2>/dev/null || echo 'not found')"
          #   echo "cargo: $(cargo --version 2>/dev/null || echo 'not found')"
          #   echo "rust-analyzer: $(rust-analyzer --version 2>/dev/null || echo 'not found')"
          # '';
        };
      }
    );
}
