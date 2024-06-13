alias r := run
alias n := nix

run:
	cargo clippy
	export RUST_LOG=trace; cargo run 


	 

nix:
	nix flake check --all-systems
