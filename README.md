# Walrus Bar
### Contributions and Bugs Reports are welcome!
A Wayland Status Bar in Rust based on Smithay

## Running
If you have a executable, then nothing else is needed (other than a wayland session to connect to)

If you have the source code, then use either
`nix run` if you have nix installed, or
`cargo run --release` if you don't

## Building 
### Nix
For running, I recommend using nix.

You just need to have nix installed, then you can just do the normal flake commands.

`nix develop` to make a shell that contains all the packages needed
`nix run` to just run the program
`nix build` to compile the program

### Otherwise
If you don't have nix, or are developing it, you should have available
`pkg-config`, `libxkbcommon`, and `alsa-lib`.
The first two are likely already installed in a non-minimal distro, but are still needed.

Then for the build system, cargo and rustc are needed. rustc is normally installed with cargo

To run either use the justfile (with the `just` package), or just run `cargo run`


## Customization
### Fonts
The fonts are located in ./fonts/
Whichever font used is compiled into the program so it can just be a static 
executable with no dynamic dependencies (including a runtime font).

Right now, it only contains nerd font's FiraCode, which is what I use.
If you want to use another font, you can either specify a path to load it from
(font searching not supported)
Or **(recommended)** change the embedded font, change the `DEFAULT_FONT_DATA` variable
in the drawing module to the (relative) path of the font you want.
It may complain if the font is not within the project, so you may want to move it into the project.
You will then have to recompile

**Non Mono space fonts should be supported, but are not tested.** (make a bug report to tell me how they work)

