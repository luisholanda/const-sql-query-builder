{
  description = "A full `const fn` SQL query builder";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {self, flake-utils, nixpkgs, rust-overlay}: let
    inherit (flake-utils.lib) eachDefaultSystem;

    nixpkgsFor = let
      overlays = {
        rust-bin = import rust-overlay;
      };

      allPkgs = eachDefaultSystem (system: {
        pkgs = import nixpkgs {
          inherit system;
          overlays = builtins.attrValues overlays;
        };
      });
    in sys: allPkgs.pkgs.${sys};

    flakeForSystem = system: let
      pkgs = nixpkgsFor system;
      rust = pkgs.rust-bin.nightly."2023-04-10".default;
    in {
      devShells.default = pkgs.mkShell {
        name = "const-sql-query-builder";

        buildInputs = [ rust ];
      };
    };
  in eachDefaultSystem flakeForSystem;
}
