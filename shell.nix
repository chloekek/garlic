let
    nixpkgs = import ./nix/nixpkgs.nix;
in
    nixpkgs.mkShell {
        nativeBuildInputs = [
            nixpkgs.cargo
        ];
    }
