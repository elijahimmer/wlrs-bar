{
  description = "Hyprland Status Bar";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    # Very nice to use
    flake-utils.url = "github:numtide/flake-utils";

    # Great rust build system
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
  }: let
    supportedSystems = with flake-utils.lib.system; [
      x86_64-linux
      aarch64-linux
      # no mac, wayland isn't on mac (as far as I know...)
      # also, bsd users can fix this themselves. There are too many options...
    ];
  in flake-utils.lib.eachSystem supportedSystems (system: let
    pkgs = (import nixpkgs) {
      inherit system;
    };

    naersk' = pkgs.callPackage naersk {};

    buildInputs = with pkgs; [
      # makeBinaryWrapper
      pkg-config
      libxkbcommon
    ];
  in {
    packages.default = naersk'.buildPackage {
      inherit buildInputs;
      src = ./.;
      meta = with pkgs.lib; {
        description = "A status bar for Hyprland.";
        homepage = "https://github.com/elijahimmer/bar-wlrs";
        license = licenses.mit;
        mainProgram = "bar-wlrs";
      };

      /*postInstall = ''
        wrapProgram $out/bin/bar-wlrs \
      '';*/
    };

    devShells.default = pkgs.mkShell {
      buildInputs =
        buildInputs
        ++ (with pkgs; [
          cargo
          rustc
          clippy
        ]);
    };

    formatter = pkgs.alejandra;
  });
}
