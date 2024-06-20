alias r := run
alias n := nix

run:
    cargo clippy
    cargo fmt
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run --features=textbox-all -- --updated-last=`date +%s` --height=128

nix:
    nix flake check --all-systems
