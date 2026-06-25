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

coverage:
	nix develop . --command cargo tarpaulin --out Html --output-dir coverage --timeout 300 --exclude-files tests/* --fail-under 90

