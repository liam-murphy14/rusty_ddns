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
          rustSrcPathStable = "./rust-toolchain";
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
              ln -sfn ${pkgs.rustPlatform.rustLibSrc} ${rustSrcPathStable}
            '';
            env = {
              # Required by RustRover
              RUST_SRC_PATH = "${rustSrcPathStable}";
            };
          };
        }
      );
      formatter = forEachSupportedSystem ({ pkgs }: pkgs.nixfmt-tree);
    };
}
