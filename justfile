alias r := run
alias n := nix

run:
    cargo clippy
    cargo fmt
    BAR_WLRS_LOG=trace RUST_BACKTRACE=1 cargo run -- --updated-last `date +%s`

nix:
    nix flake check --all-systems
