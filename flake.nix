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
      alsa-lib
    ];
  in {
    packages.default = naersk'.buildPackage {
      inherit buildInputs;
      src = ./.;
      meta = with pkgs.lib; {
        description = "A status bar for Hyprland.";
        homepage = "https://github.com/elijahimmer/wlrs-bar";
        license = licenses.mit;
        mainProgram = "wlrs-bar";
      };

      /*postInstall = ''
        wrapProgram $out/bin/wlrs-bar \
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
