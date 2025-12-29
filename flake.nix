{
    description = "Crabby Programming Language";
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };
    outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem(system:
        let pkgs = import nixpkgs {
            inherit system;
        };
        in {
            packages.default = pkgs.callPackage ./pkgs/default.nix { };
            apps.default = {
                type = "app";
                program = "${self.packages.${system}.default}/bin/crabby";
            };

            devShells.default = pkgs.mkShell {
                buildInputs = with pkgs; [
                    rustc
                    cargo
                    clang
                    llvmPackages.bintools
                ];
            };
        }
    );
}
