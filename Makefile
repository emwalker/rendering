check:
	cargo clippy -- -D warnings
	cargo test

fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged
