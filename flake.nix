{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    let
      makeGitRenameStash =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          name = "git-rename-stash";
          pname = "git-rename-stash";
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          buildInputs = with pkgs; [ openssl ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
          src = builtins.path {
            path = ./.;
            name = "git-rename-stash-src";
          };
        };
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.git-rename-stash = makeGitRenameStash pkgs;
        packages.default = self.packages.${system}.git-rename-stash;
      }
    );
}
