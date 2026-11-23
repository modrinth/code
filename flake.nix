{
  description = "Labrinth backend development environment";

  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devenv.flakeModule ];
      systems = [ "x86_64-linux" "aarch64-linux" ];

      perSystem = { pkgs, system, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };

        devenv.shells.default = { config, ... }: {
          packages = with pkgs; [
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            rustPlatform.bindgenHook
            pkg-config
            cmake
            gnumake
            perl
            nasm
            clang
            openssl
            zlib
            postgresql_15
            redis
          ];

          # Fortify requires optimization, but Cargo debug builds compile native code with -O0.
          hardeningDisable = [ "fortify" ];

          env.SQLX_OFFLINE = "true";
        };
      };
    };
}
