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

build:
	cd crates/benda; \
		maturin develop --release

run_examples:
	python -m examples.quicksort
	python -m examples.radix_sort
	bend run examples/radix_sort.bend
	python -m examples.insertion_sort
	bend run examples/insertion_sort.bend
	python -m examples.bitonic_sort
	bend run examples/bitonic_sort.bend
	python -m examples.bubble_sort
	bend run examples/bubble_sort.bend
