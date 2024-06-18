alias r := run
alias n := nix

run:
    cargo clippy
    cargo fmt
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run 

nix:
    nix flake check --all-systems
