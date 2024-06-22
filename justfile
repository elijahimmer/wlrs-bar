alias r := run
alias n := nix

run:
    cargo fmt
    cargo clippy
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run --features= -- --updated-last=`date +%s` --height=64

test FEATURES:
    cargo fmt 
    cargo clippy --no-default-features
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run --no-default-features --features={{FEATURES}} -- --height=128

nix:
    nix flake check --all-systems

