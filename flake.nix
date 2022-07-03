{
  description = "A full `const fn` SQL query builder";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";

    devshell.url = "github:numtide/devshell"; devshell.inputs.flake-utils.follows = "flake-utils";
    devshell.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {self, devshell, flake-utils, nixpkgs, rust-overlay}: let
    inherit (flake-utils.lib) eachDefaultSystem;

    nixpkgsFor = let
      overlays = {
        rust-bin = import rust-overlay;
        devshell = devshell.overlay;
      };

      allPkgs = eachDefaultSystem (system: {
        pkgs = import nixpkgs {
          inherit system;
          overlays = builtins.attrValues overlays;
        };
      });
    in sys: allPkgs.pkgs.${sys};

    shellWith = { pkgs, rust }: let
      attrsToList = pkgs.lib.mapAttrsToList (name: value: {inherit name value;});
      attrsToSubmodulesList = attrs:
        map
        ({
          name,
          value,
        }:
          value // {inherit name;})
        (attrsToList attrs);
    in pkgs.devshell.mkShell {
      name = "sql-builder";

      commands = attrsToSubmodulesList {
        t = {
          command = "cargo test --workspace";
          help = "run tests";
          category = "tests";
        };
        c = {
          command = "cargo clippy --workspace";
          help = "run clippy";
          category = "linters";
        };
      };

      devshell.packages = [rust pkgs.clang_14];
    };

    flakeForSystem = system: let
      pkgs = nixpkgsFor system;
      rust = pkgs.rust-bin.nightly.latest.default;
    in {
      devShells.default = shellWith { inherit pkgs rust; };
    };
  in eachDefaultSystem flakeForSystem;
}
