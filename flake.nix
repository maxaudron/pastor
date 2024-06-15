{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci.url = "github:yusdacra/nix-cargo-integration";
    nci.inputs.nixpkgs.follows = "nixpkgs";
    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };

  outputs =
    inputs@{ parts, nci, ... }:
    parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      imports = [ nci.flakeModule ];
      perSystem =
        {
          pkgs,
          config,
          lib,
          ...
        }:
        let
          # shorthand for accessing this crate's outputs
          # you can access crate outputs under `config.nci.outputs.<crate name>` (see documentation)
          crateOutputs = config.nci.outputs."pastor";
        in
        {
          nci = {
            projects."pastor".path = ./.;
            crates."pastor" =
              let
                mkDerivation = {
                  nativeBuildInputs = [ pkgs.file.dev ];
                };
                env = {
                  PASTOR_MIME_DB = "${pkgs.file}/share/misc/magic.mgc";
                };
              in
              {
                drvConfig = {
                  inherit mkDerivation env;
                };
                depsDrvConfig = {
                  inherit mkDerivation env;
                };
              };

            toolchainConfig = {
              channel = "stable";
              targets = [ "x86_64-unknown-linux-musl" ];
              components = [
                "rustfmt"
                "rust-src"
              ];
            };
          };

          devShells.default = crateOutputs.devShell;
          packages.default = crateOutputs.packages.release;
        };
    };
}
