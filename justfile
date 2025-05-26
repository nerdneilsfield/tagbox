# Cross-platform Justfile for TagBox project
# Requires just (https://github.com/casey/just)

# Set shell for Windows compatibility
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Default recipe shows available commands
default:
    @just --list

# Variables
database_path := env_var_or_default("DATABASE_URL", "sqlite:${PWD}/.sqlx-data/tagbox.db")
rust_log := env_var_or_default("RUST_LOG", "info")

# Package names
core := "tagbox-core"
cli := "tagbox-cli"
gui := "tagbox-gui"
tui := "tagbox-tui"
config := "tagbox-config"
tools := "tagbox-tools"

# Setup development environment
setup:
    rustup component add rustfmt clippy rust-src
    cargo install sqlx-cli --no-default-features --features sqlite
    cargo install cargo-audit
    cargo install cargo-nextest
    cargo install cargo-tarpaulin
    cargo install cargo-benchcmp
    @echo "Development environment setup complete!"

# Initialize database
init-db:
    mkdir -p .sqlx-data
    rm -f .sqlx-data/tagbox.db
    touch .sqlx-data/tagbox.db
    DATABASE_URL={{database_path}} cargo run --bin tagbox-init-db
    cd tagbox-core && DATABASE_URL={{database_path}} cargo sqlx prepare -- --lib
    @echo "Database initialized successfully!"

# Build all packages or specific package
build package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        cargo build --all
    else
        cargo build -p tagbox-{{package}}
    fi

# Build in release mode
build-release package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        cargo build --all --release
    else
        cargo build -p tagbox-{{package}} --release
    fi

# Run tests for all packages or specific package
test package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        DATABASE_URL={{database_path}} cargo test --all
    else
        DATABASE_URL={{database_path}} cargo test -p tagbox-{{package}}
    fi

# Run tests with nextest (faster parallel execution)
test-nextest package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        DATABASE_URL={{database_path}} cargo nextest run --all
    else
        DATABASE_URL={{database_path}} cargo nextest run -p tagbox-{{package}}
    fi

# Run tests with coverage
test-coverage package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        DATABASE_URL={{database_path}} cargo tarpaulin --all --out Html
    else
        DATABASE_URL={{database_path}} cargo tarpaulin -p tagbox-{{package}} --out Html
    fi

# Run specific binary
run binary *args:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run --bin {{binary}} -- {{args}}

# Run CLI with arguments
run-cli *args:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run -p tagbox-cli -- {{args}}

# Run GUI
run-gui:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run -p tagbox-gui

# Run TUI
run-tui:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run -p tagbox-tui

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Run clippy lints
clippy:
    cargo clippy --all -- -D warnings

# Run security audit
audit:
    cargo audit

# Run all checks (format, clippy, test, audit)
check: fmt-check clippy test audit
    @echo "All checks passed!"

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/

# Deep clean including dependencies and database
clean-all: clean
    rm -rf .sqlx-data/
    rm -rf Cargo.lock
    @echo "Deep clean complete!"

# Run benchmarks
bench package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        cargo bench --all
    else
        cargo bench -p tagbox-{{package}}
    fi

# Watch and rebuild on changes (requires cargo-watch)
watch package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        cargo watch -x "build --all"
    else
        cargo watch -x "build -p tagbox-{{package}}"
    fi

# Update dependencies
update:
    cargo update
    @echo "Dependencies updated!"

# Show dependency tree
deps package="all":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        cargo tree
    else
        cargo tree -p tagbox-{{package}}
    fi

# Generate documentation
doc package="all" open="":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        cargo doc --all --no-deps {{open}}
    else
        cargo doc -p tagbox-{{package}} --no-deps {{open}}
    fi

# Generate and open documentation
doc-open package="all":
    just doc {{package}} --open

# Run CI pipeline locally
ci: check test-nextest
    @echo "CI pipeline passed!"

# Install TagBox CLI globally
install-cli:
    cargo install --path tagbox-cli
    @echo "TagBox CLI installed!"

# Uninstall TagBox CLI
uninstall-cli:
    cargo uninstall tagbox-cli
    @echo "TagBox CLI uninstalled!"

# Show environment info
info:
    @echo "Rust version:"
    @rustc --version
    @echo ""
    @echo "Cargo version:"
    @cargo --version
    @echo ""
    @echo "Database URL: {{database_path}}"
    @echo "Rust log level: {{rust_log}}"
    @echo ""
    @echo "Installed packages:"
    @ls -1 tagbox-*/Cargo.toml | sed 's/\/Cargo.toml//' | sed 's/tagbox-/  - /'

# Quick development build and test
dev package="all": build test
    @echo "Development build and test complete for {{package}}!"

# Release workflow
release: clean check test-nextest build-release
    @echo "Release build complete!"

# Platform-specific commands
[windows]
init-db-windows:
    if not exist .sqlx-data mkdir .sqlx-data
    if exist .sqlx-data\tagbox.db del .sqlx-data\tagbox.db
    type nul > .sqlx-data\tagbox.db
    set DATABASE_URL={{database_path}} && cargo run --bin tagbox-init-db
    cd tagbox-core && set DATABASE_URL={{database_path}} && cargo sqlx prepare -- --lib

[linux]
install-system-deps:
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev

[macos]
install-system-deps:
    brew install pkg-config openssl