{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      naersk,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk-lib = pkgs.callPackage naersk { };

        atuin-fish-sync = naersk-lib.buildPackage {
          pname = "atuin-fish-sync";
          src = ./.;
        };
      in
      {
        packages = {
          inherit atuin-fish-sync;
          default = atuin-fish-sync;
        };

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
          ];

          packages = with pkgs; [
            rust-analyzer
            sqlx-cli
          ];
        };
      }
    );
}
