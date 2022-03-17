{
  description = "The official Modrinth launcher";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.utils.lib.eachDefaultSystem (system: let
      pkgs = import inputs.nixpkgs { inherit system; };
      fenix = inputs.fenix.packages.${system};
      utils = inputs.utils.lib;

      toolchain = with fenix;
        combine [
          minimal.rustc minimal.cargo
        ];

      naersk = inputs.naersk.lib.${system}.override {
        rustc = toolchain;
        cargo = toolchain;
      };

      deps = with pkgs; {
        global = [
          openssl pkg-config
        ];
        gui = [
          gtk4 gdk-pixbuf atk webkitgtk
        ];
        shell = [
          toolchain fenix.default.clippy git
          jdk17 jdk8
        ];
      };
    in {
      packages = {
        theseus-cli = naersk.buildPackage {
          pname = "theseus_cli";
          src = ./.;
          buildInputs = deps.global;
          cargoBuildOptions = x: x ++ ["-p" "theseus_cli"];
        };
      };

      apps = {
        theseus-cli = utils.mkApp {
          drv = inputs.self.packages.${system}.theseus-cli;
        };
      };

      devShell = pkgs.mkShell {
        buildInputs = with deps;
          global ++ gui ++ shell;
      };
    });
}
