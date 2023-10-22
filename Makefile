check:
	cargo test
	cargo clippy -- -D warnings

fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged
