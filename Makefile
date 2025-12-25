default: build

build:
	@echo "Building oxur..."
	@cargo build

lint:
	@echo "Running linter..."
	@cargo clippy

test:
	@echo "Running tests..."
	@cargo test

check: build lint test
