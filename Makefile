.PHONY: check rust-check rust-check-fmt rust-fmt

check: rust-check rust-check-fmt

rust-check:
	cargo clippy --all-targets --all-features -- # -D warnings -W clippy::pedantic

rust-check-fmt:
	cargo fmt --all -- --check

rust-fmt:
	cargo fmt --all
