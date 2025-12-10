.PHONY: install build lint format test run docker-up docker-down

install:
	cargo fetch

build:
	cargo build

lint:
	cargo clippy --all-targets -- -D warnings

format:
	cargo fmt --all

test:
	cargo test

run:
	cargo run

start:
	docker-compose down && docker-compose up -d

