.PHONY: build test lint fmt run clean bench

build:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --check

run:
	cargo run

clean:
	cargo clean

bench:
	cargo bench
