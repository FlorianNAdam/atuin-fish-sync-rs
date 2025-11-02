{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      naersk,
      ...
    }:
    {
      homeModules = rec {
        atuin-fish-sync-rs =
          {
            lib,
            config,
            pkgs,
            ...
          }:
          let
            sync = self.packages.${pkgs.system}.atuin-fish-sync-rs;
          in
          {
            options.programs.atuin-fish-sync-rs.enable = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Enable automatic Fish history sync with atuin-fish-sync-rs";
            };

            config = lib.mkIf config.programs.atuin-fish-sync-rs.enable {
              home.packages = [ sync ];

              programs.fish.interactiveShellInit = ''
                ${sync}/bin/atuin-fish-sync-rs &>/dev/null &

                function _sync_atuin_fish --on-event fish_postexec
                    if not set -q fish_private_mode
                        ${sync}/bin/atuin-fish-sync-rs &>/dev/null &
                    end
                end
              '';
            };
          };
        default = atuin-fish-sync-rs;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk-lib = pkgs.callPackage naersk { };

        atuin-fish-sync-rs = naersk-lib.buildPackage {
          pname = "atuin-fish-sync-rs";
          src = ./.;
        };
      in
      {
        packages = {
          inherit atuin-fish-sync-rs;
          default = atuin-fish-sync-rs;
        };

        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
          ];
          packages = [
            pkgs.rust-analyzer
            pkgs.sqlx-cli
          ];
        };
      }
    );
}
