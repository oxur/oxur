default: build

build:
	@echo "Building oxur..."
	@cargo build

lint:
	@echo "Running linter..."
	@cargo clippy
	@cargo fmt --all -- --check

test:
	@echo "Running tests..."
	@cargo test

coverage:
	@echo "Generating coverage report..."
	@cargo llvm-cov --summary-only

check: build lint test

format:
	@echo "Formatting code..."
	@cargo fmt --all
