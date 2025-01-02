check:
	cargo test --all-features
	cargo clippy --all-features -- -D warnings

fix:
	cargo fmt
	cargo clippy --all-features --fix --allow-dirty --allow-staged
