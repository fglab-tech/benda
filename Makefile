.PHONY: check rust-check rust-check-fmt rust-fmt
.PHONY: create-venv build

all: check build

check: rust-check rust-check-fmt

rust-check:
	cargo clippy --all-targets --all-features -- # -D warnings -W clippy::pedantic

rust-check-fmt:
	cargo fmt --all -- --check

rust-fmt:
	cargo fmt --all

create-venv:
	python3 -m venv venv
	@echo "Enter venv with:"
	@echo "source venv/bin/activate"

build:
	cd crates/benda; \
		maturin develop
