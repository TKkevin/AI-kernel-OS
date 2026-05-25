.PHONY: build test clean run

# Build all components
build:
	@echo "Building NEXUS Kernel OS..."
	cargo build --release

# Run tests
test:
	@echo "Running tests..."
	cargo test --lib

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Run clippy linter
lint:
	@echo "Running clippy..."
	cargo clippy --all -- -D warnings

# Display help
help:
	@echo "NEXUS Kernel Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  make build  - Build NEXUS kernel and modules"
	@echo "  make test   - Run tests"
	@echo "  make clean  - Clean build artifacts"
	@echo "  make fmt    - Format code"
	@echo "  make lint   - Run clippy linter"
