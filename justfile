# masonry_baseview justfile

# Run the hello example
hello:
    cargo run --example hello

# Build all examples
build:
    cargo build --examples

# Check compilation
check:
    cargo check

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt

# Run clippy lints
lint:
    cargo clippy --all-targets
