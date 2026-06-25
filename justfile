default:
	nix develop . --command just run

run:
	nix develop . --command cargo run --bin cerebrum

test:
	nix develop . --command cargo test

fmt:
	nix develop . --command cargo fmt

lint:
	nix develop . --command cargo clippy -- -D warnings
