{
  description = "Guts - A Git-like version control tool written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      rustPlatform = pkgs.rustPlatform;
    in {
      # main binary
      packages.default = rustPlatform.buildRustPackage {
        pname = "guts";
        version = "0.1.0";

        # source code workspace
        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        # build crate `guts/`
        cargoBuildFlags = [ "--package" "guts" ];

        meta = {
          description = "Git in Rust";
          license = pkgs.lib.licenses.mit;
          maintainers = [ ];
        };
      };

      # `nix run`
      apps.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/guts";
      };

      # environment dev Rust `nix develop`
      devShells.default = pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.openssl
          pkgs.pkg-config
        ];
        shellHook = ''
          echo "Welcome to Guts, you can look at the wiki for more information on commands, or guts help"
        '';
      };
    }
  );
}
