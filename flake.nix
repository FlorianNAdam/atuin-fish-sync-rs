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

        atuin-fish-sync-rs = naersk-lib.buildPackage {
          pname = "atuin-fish-sync-rs";
          src = ./.;
        };
      in
      {
        packages = {
          atuin-fish-sync = atuin-fish-sync-rs;
          default = atuin-fish-sync-rs;
        };

        nixosModules.atuin-fish-sync-rs =
          { lib, ... }:
          {
            options.programs.atuin-fish-sync-rs.enable = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Enable automatic Fish history sync with atuin-fish-sync-rs";
            };

            config =
              {
                config,
                lib,
                ...
              }:
              let
                sync = "${atuin-fish-sync-rs}/bin/atuin-fish-sync";
              in
              {
                options = { };
                config = lib.mkIf config.programs.atuin-fish-sync-rs.enable {
                  environment.systemPackages = [ sync ];

                  programs.fish.interactiveShellInit = ''
                    # Run atuin-fish-sync-rs once per shell startup
                    ${sync} &>/dev/null &
                  '';

                  programs.fish.initExtra = ''
                    function _sync_atuin_fish --on-event fish_postexec
                        if not set -q fish_private_mode
                            ${sync} &>/dev/null &
                        end
                    end
                  '';
                };
              };
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
