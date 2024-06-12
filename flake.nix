{
  description = "kindelia-node";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        python3 = pkgs.python3;
        rust_toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        nativeBuildInputs = [
          pkgs.git
          # Build toolchain
          pkgs.pkg-config
          pkgs.clang
          rust_toolchain
          # Python
          python3
        ];

        dev_packages = [
          pkgs.gnumake
          # GitHub Actions runner
          pkgs.act
          # Build tools
          pkgs.maturin
          # Python
          (pkgs.pipenv.override { inherit python3; })
        ];

        naersk' = pkgs.callPackage naersk {
          cargo = rust_toolchain;
          rustc = rust_toolchain;
        };
      in
      {
        devShells.default = pkgs.stdenvNoCC.mkDerivation {
          name = "shell";
          inherit nativeBuildInputs;
          buildInputs = dev_packages;
          shellHook = ''
            export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${python3}/lib
          '';
        };

        packages.default = naersk'.buildPackage {
          inherit nativeBuildInputs;
          src = ./.;
        };
      }
    );
}
