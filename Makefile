check:
	cargo clippy --all-features -- -D warnings
	cargo test --features lol_html
	cargo test --features tl

fix:
	cargo fmt
	cargo clippy --all-features --fix --allow-dirty --allow-staged
