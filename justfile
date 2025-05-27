# Cross-platform Justfile for TagBox project
# Requires just (https://github.com/casey/just)

# Set shell for Windows compatibility
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Default recipe shows available commands
default:
    @just --list

# Variables
database_path := env_var_or_default("DATABASE_URL", "sqlite:.sqlx-data/tagbox.db")
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
[unix]
init-db:
    mkdir -p .sqlx-data
    rm -f .sqlx-data/tagbox.db
    touch .sqlx-data/tagbox.db
    DATABASE_URL={{database_path}} cargo run --bin tagbox-init-db
    cd tagbox-core && DATABASE_URL={{database_path}} cargo sqlx prepare -- --lib
    @echo "Database initialized successfully!"

[windows]
init-db:
    if not exist .sqlx-data mkdir .sqlx-data
    if exist .sqlx-data\tagbox.db del .sqlx-data\tagbox.db
    type nul > .sqlx-data\tagbox.db
    $env:DATABASE_URL="{{database_path}}"; cargo run --bin tagbox-init-db
    cd tagbox-core; $env:DATABASE_URL="{{database_path}}"; cargo sqlx prepare -- --lib
    @echo "Database initialized successfully!"

# Build all packages
build-all:
    cargo build --all

# Build specific package
build-package package:
    cargo build -p tagbox-{{package}}

# Build command with package selection
[unix]
build package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just build-all
    else
        just build-package {{package}}
    fi

[windows]
build package="all":
    @if "{{package}}" == "all" (just build-all) else (just build-package {{package}})

# Build all packages in release mode
build-all-release:
    cargo build --all --release

# Build specific package in release mode
build-package-release package:
    cargo build -p tagbox-{{package}} --release

# Build release command with package selection
[unix]
build-release package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just build-all-release
    else
        just build-package-release {{package}}
    fi

[windows]
build-release package="all":
    @if "{{package}}" == "all" (just build-all-release) else (just build-package-release {{package}})

# Test all packages
[unix]
test-all:
    DATABASE_URL={{database_path}} cargo test --all

[windows]
test-all:
    $env:DATABASE_URL="{{database_path}}"; cargo test --all

# Test specific package
[unix]
test-package package:
    DATABASE_URL={{database_path}} cargo test -p tagbox-{{package}}

[windows]
test-package package:
    $env:DATABASE_URL="{{database_path}}"; cargo test -p tagbox-{{package}}

# Test command with package selection
[unix]
test package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just test-all
    else
        just test-package {{package}}
    fi

[windows]
test package="all":
    @if "{{package}}" == "all" (just test-all) else (just test-package {{package}})

# Run tests with nextest for all packages
[unix]
test-all-nextest:
    DATABASE_URL={{database_path}} cargo nextest run --all

[windows]
test-all-nextest:
    $env:DATABASE_URL="{{database_path}}"; cargo nextest run --all

# Run tests with nextest for specific package
[unix]
test-package-nextest package:
    DATABASE_URL={{database_path}} cargo nextest run -p tagbox-{{package}}

[windows]
test-package-nextest package:
    $env:DATABASE_URL="{{database_path}}"; cargo nextest run -p tagbox-{{package}}

# Run tests with nextest
[unix]
test-nextest package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just test-all-nextest
    else
        just test-package-nextest {{package}}
    fi

[windows]
test-nextest package="all":
    @if "{{package}}" == "all" (just test-all-nextest) else (just test-package-nextest {{package}})

# Run tests with coverage for all packages
[unix]
test-all-coverage:
    DATABASE_URL={{database_path}} cargo tarpaulin --all --out Html

[windows]
test-all-coverage:
    $env:DATABASE_URL="{{database_path}}"; cargo tarpaulin --all --out Html

# Run tests with coverage for specific package
[unix]
test-package-coverage package:
    DATABASE_URL={{database_path}} cargo tarpaulin -p tagbox-{{package}} --out Html

[windows]
test-package-coverage package:
    $env:DATABASE_URL="{{database_path}}"; cargo tarpaulin -p tagbox-{{package}} --out Html

# Run tests with coverage
[unix]
test-coverage package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just test-all-coverage
    else
        just test-package-coverage {{package}}
    fi

[windows]
test-coverage package="all":
    @if "{{package}}" == "all" (just test-all-coverage) else (just test-package-coverage {{package}})

# Run specific binary
[unix]
run binary *args:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run --bin {{binary}} -- {{args}}

[windows]
run binary *args:
    $env:DATABASE_URL="{{database_path}}"; $env:RUST_LOG="{{rust_log}}"; cargo run --bin {{binary}} -- {{args}}

# Run CLI with arguments
[unix]
run-cli *args:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run -p tagbox-cli -- {{args}}

[windows]
run-cli *args:
    $env:DATABASE_URL="{{database_path}}"; $env:RUST_LOG="{{rust_log}}"; cargo run -p tagbox-cli -- {{args}}

# Run GUI
[unix]
run-gui:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run -p tagbox-gui

[windows]
run-gui:
    $env:DATABASE_URL="{{database_path}}"; $env:RUST_LOG="{{rust_log}}"; cargo run -p tagbox-gui

# Run TUI
[unix]
run-tui:
    DATABASE_URL={{database_path}} RUST_LOG={{rust_log}} cargo run -p tagbox-tui

[windows]
run-tui:
    $env:DATABASE_URL="{{database_path}}"; $env:RUST_LOG="{{rust_log}}"; cargo run -p tagbox-tui

# Check code formatting
fmt-check:
    cargo fmt -p tagbox-core -p tagbox-cli -p tagbox-tui -p tagbox-gui -p tagbox-config -p tagbox-tools -- --check

# Format code
fmt:
    cargo fmt -p tagbox-core -p tagbox-cli -p tagbox-tui -p tagbox-gui -p tagbox-config -p tagbox-tools

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
[unix]
clean-all: clean
    rm -rf .sqlx-data/
    rm -rf Cargo.lock
    @echo "Deep clean complete!"

[windows]
clean-all: clean
    if exist .sqlx-data rmdir /s /q .sqlx-data
    if exist Cargo.lock del Cargo.lock
    @echo "Deep clean complete!"

# Run benchmarks for all packages
bench-all:
    cargo bench --all

# Run benchmarks for specific package
bench-package package:
    cargo bench -p tagbox-{{package}}

# Run benchmarks
[unix]
bench package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just bench-all
    else
        just bench-package {{package}}
    fi

[windows]
bench package="all":
    @if "{{package}}" == "all" (just bench-all) else (just bench-package {{package}})

# Watch and rebuild all packages
watch-all:
    cargo watch -x "build --all"

# Watch and rebuild specific package
watch-package package:
    cargo watch -x "build -p tagbox-{{package}}"

# Watch and rebuild
[unix]
watch package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just watch-all
    else
        just watch-package {{package}}
    fi

[windows]
watch package="all":
    @if "{{package}}" == "all" (just watch-all) else (just watch-package {{package}})

# Update dependencies
update:
    cargo update
    @echo "Dependencies updated!"

# Show dependency tree for all packages
deps-all:
    cargo tree

# Show dependency tree for specific package
deps-package package:
    cargo tree -p tagbox-{{package}}

# Show dependency tree
[unix]
deps package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just deps-all
    else
        just deps-package {{package}}
    fi

[windows]
deps package="all":
    @if "{{package}}" == "all" (just deps-all) else (just deps-package {{package}})

# Generate documentation for all packages
doc-all:
    cargo doc --all --no-deps

# Generate documentation for specific package
doc-package package:
    cargo doc -p tagbox-{{package}} --no-deps

# Generate documentation
[unix]
doc package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just doc-all
    else
        just doc-package {{package}}
    fi

[windows]
doc package="all":
    @if "{{package}}" == "all" (just doc-all) else (just doc-package {{package}})

# Generate and open documentation for all packages
doc-open-all:
    cargo doc --all --no-deps --open

# Generate and open documentation for specific package
doc-open-package package:
    cargo doc -p tagbox-{{package}} --no-deps --open

# Generate and open documentation
[unix]
doc-open package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just doc-open-all
    else
        just doc-open-package {{package}}
    fi

[windows]
doc-open package="all":
    @if "{{package}}" == "all" (just doc-open-all) else (just doc-open-package {{package}})

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
[unix]
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

[windows]
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
    @dir /b tagbox-*\Cargo.toml 2>nul | for /f "tokens=1 delims=\" %i in ('more') do @echo   - %i | powershell -c "$input -replace 'tagbox-',''"

# Quick development build and test
[unix]
dev package="all":
    #!/usr/bin/env sh
    if [ "{{package}}" = "all" ]; then
        just build-all
        just test-all
    else
        just build-package {{package}}
        just test-package {{package}}
    fi
    echo "Development build and test complete for {{package}}!"

[windows]
dev package="all":
    @if "{{package}}" == "all" (just build-all && just test-all) else (just build-package {{package}} && just test-package {{package}})
    @echo "Development build and test complete for {{package}}!"

# Release workflow
release: clean check test-nextest build-release
    @echo "Release build complete!"

# Platform-specific dependency installation
[linux]
install-system-deps:
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev

[macos]
install-system-deps:
    brew install pkg-config openssl

[windows]
install-system-deps:
    @echo "Please ensure you have Visual Studio Build Tools installed"
    @echo "Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022"