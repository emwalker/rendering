check:
	cargo clippy --all-features -- -D warnings
	cargo test --all-features

fix:
	cargo fmt
	cargo clippy --all-features --fix --allow-dirty --allow-staged
