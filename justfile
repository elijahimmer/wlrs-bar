alias r := run
alias n := nix

features := ""
height := "64"

run:
    cargo fmt
    cargo clippy
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run --features={{features}} -- --updated-last=`date +%s` --height={{height}}

test FEATURES:
    cargo fmt 
    cargo clippy --no-default-features
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run --no-default-features --features={{FEATURES}} -- --height={{height}}

nix:
    nix flake check --all-systems

