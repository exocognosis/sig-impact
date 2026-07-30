.DEFAULT_GOAL := help

.PHONY: help build run test fmt lint check install report clean

help: ## Show available commands
	@awk 'BEGIN {FS = ":.*## "; print "sig-impact development commands:"} /^[a-zA-Z_-]+:.*## / {printf "  %-10s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build an optimized binary
	cargo build --release

run: ## Run the default scenario
	cargo run --release --

test: ## Run the test suite
	cargo test --all-targets

fmt: ## Check Rust formatting
	cargo fmt --all --check

lint: ## Run Clippy with warnings denied
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt lint test build ## Run every local quality gate

install: ## Install sig-impact from this checkout
	cargo install --path .

report: ## Write sample.csv and sample.md and print a table
	cargo run --release -- --schemes all --format table,csv,md --out sample

clean: ## Remove Cargo build output
	cargo clean
