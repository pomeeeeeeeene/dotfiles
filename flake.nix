{
  description = "dotfiles keymap configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };

      keymapGnome = import ./keymap/linux/gnome.nix {
        inherit pkgs;
      };
    in
    {
      packages.${system} = {
        keymap-gnome = keymapGnome;
        default = keymapGnome;
      };
    };
}
