default: build

BIN_DIR := ./bin
TARGET := ./target
TARGET_OXD := $(TARGET)/debug/oxd
BIN_OXD := $(BIN_DIR)/oxd

$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

build: $(BIN_DIR)
	@echo "Building oxur..."
	@cargo build
	@cp $(TARGET_OXD) $(BIN_OXD)

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

format:
	@echo "Formatting code..."
	@cargo fmt --all

tracked-files:
	@echo "Saving tracked files..."
	@git ls-files > $(TARGET)/git-tracked-files.txt
