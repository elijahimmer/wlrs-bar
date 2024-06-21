alias r := run
alias n := nix

run: prelude
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run --features= -- --updated-last=`date +%s` --height=64

test FEATURES: prelude
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run --no-default-features --features={{FEATURES}} -- --height=128

nix:
    nix flake check --all-systems

prelude:
    cargo clippy
    cargo fmt
