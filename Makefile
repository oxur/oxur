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

coverage:
	@echo "Generating coverage report..."
	@cargo llvm-cov --summary-only

check: build lint test
