alias r := run
alias n := nix

run:
	cargo clippy --release
	cargo build --release


	export RUST_LOG=trace; ./target/release/bar-wlrs 

nix:
	nix flake check --all-systems
