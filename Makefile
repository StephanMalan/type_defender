.DEFAULT_GOAL := run

build:
	cargo build --release

debug:
	cargo watch -x 'run'

lint:
	rustup toolchain install nightly --profile minimal --allow-downgrade --component clippy
	cargo clippy

run:
	cargo run