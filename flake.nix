{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix = {
        url = "github:nix-community/fenix";
        inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, fenix, nixpkgs }: 
    let
        system = "x86_64-linux";
        pkgs = nixpkgs.legacyPackages.${system};
        fenixStable = fenix.packages.${system}.stable;
    in {
        defaultPackage.${system} =
            (pkgs.makeRustPlatform { cargo = fenixStable.toolchain; rustc = fenixStable.toolchain; }).buildRustPackage (finalAttrs: {
                pname = "bosh-rs";
                version = "0.4.1";

                cargoLock.lockFile = ./Cargo.lock;

                src = ./.;

                meta = {
                    description = "bosh-rs is a highly configurable physics engine for the game Line Rider.";
                    homepage = "https://codeberg.org/lipfang/bosh-rs";
                    license = pkgs.lib.licenses.lgpl3Plus;
                };
            });

        devShell.${system} = pkgs.mkShell {
            buildInputs = [
                fenixStable.toolchain
                fenixStable.rust-analyzer
            ];
        };
    };
}
