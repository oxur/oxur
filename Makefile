default: build

BIN_DIR := ./bin

$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

build: $(BIN_DIR)
	@echo "Building oxur..."
	@cargo build
	@cp target/debug/oxd $(BIN_DIR)/oxd

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
