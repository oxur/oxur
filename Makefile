default: build

BIN_DIR := ./bin
MODE := debug
TARGET := ./target/$(MODE)

$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

build: clean $(BIN_DIR)
	@echo "Building oxur..."
	@cargo build
	@cp $(TARGET)/aster $(BIN_DIR)/aster
	@cp $(TARGET)/oxd $(BIN_DIR)/oxd

clean:
	@echo "Cleaning project..."
	@rm -rf $(BIN_DIR) $(TARGET_OXD)

clean-all: clean
	@echo "Performing full clean..."
	@cargo clean

lint:
	@echo "Running linter..."
	@cargo clippy --all-features --workspace -- -D warnings
	@cargo fmt --all -- --check

test:
	@echo "Running tests..."
	@cargo test --all-features --workspace

coverage:
	@echo "Generating coverage report..."
	@cargo llvm-cov --summary-only

check: build lint test

check-all: build lint coverage

format:
	@echo "Formatting code..."
	@cargo fmt --all

tracked-files:
	@echo "Saving tracked files..."
	@git ls-files > $(TARGET)/git-tracked-files.txt
