{
  description = "Nix flake for rusty ddns client";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs { inherit system; };
          }
        );
    in
    {
      packages = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.callPackage ./default.nix { };
        }
      );
      devShells = forEachSupportedSystem (
        { pkgs }:
        let
          rustToolchain = pkgs.symlinkJoin {
            name = "rust-toolchain";
            paths = with pkgs; [
              rustc
              cargo
              rustPlatform.rustLibSrc
            ];
          };
          rustToolchainPathStable = "./rust-toolchain";
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
            ];
            shellHook = ''
              ln -sfn ${rustToolchain} ${rustToolchainPathStable}
            '';
            env = {
              # Required by RustRover
              RUST_SRC_PATH = "${rustToolchainPathStable}";
              RUST_BACKTRACE = 1;
            };
          };
        }
      );
      formatter = forEachSupportedSystem ({ pkgs }: pkgs.nixfmt-tree);
    };
}
