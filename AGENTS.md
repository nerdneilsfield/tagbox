# AGENTS.md – TagBox Agent Guidelines

## 🔍 Project Overview

TagBox is a modular, offline-first file management system written in Rust. It supports full-text search, tag/category classification, semantic linking, and metadata extraction. The project includes:

* `tagbox-core/`: Core async library with database logic, import pipeline, metadata extraction, and search
* `tagbox-cli/`: Command-line interface tool, supports JSON and stdio input
* `tagbox-gui/`: FLTK-based desktop UI
* `tagbox-tui/`: Optional terminal interface (future)
* `tagbox-config/`: TOML configuration structure
* `tagbox-tools/`: Utilities for metadata extraction

Data is stored locally in SQLite (with FTS5 + Signal tokenizer support). No cloud dependencies. The agent should follow async-first, type-safe, modular principles.

## 📂 Folder Responsibilities

| Folder         | Purpose                                |
| -------------- | -------------------------------------- |
| `tagbox-core/` | All business logic and data types      |
| `tagbox-cli/`  | Interface parsing, invokes core        |
| `schema.rs`    | DB schema creation via sea-query       |
| `search.rs`    | DSL → SQL query building               |
| `editor.rs`    | Metadata update & soft-delete          |
| `errors.rs`    | TagboxError definition (use thiserror) |
| `tests/`       | Use tokio, tempfile, assert\_cmd       |


## 🛠️ Development Environment

* Rust 1.70+
* Cargo workspace, Rust edition 2021+
* C++/C compiler for FFI

## Build

```bash
cargo install sqlx-cli --no-default-features --features sqlite

## fetch dependencies
rustup component add rustfmt
rustup component add clippy
rustup component add rust-src
cargo install cargo-audit
cargo install cargo-nextest
# for benchmark
cargo install cargo-benchmarks


# 提前获得依赖
cargo fetch
# 设置环境变量
export DATABASE_URL="sqlite:${PWD}/.sqlx-data/tagbox.db"

# 准备数据库文件
mkdir -p .sqlx-data
rm -rf .sqlx-data/tagbox.db
touch .sqlx-data/tagbox.db

#install sqlx-cli
cargo install sqlx-cli --no-default-features --features sqlite

# init database
cargo run --bin tagbox-init-db

# # 准备数据库 schema
cd tagbox-core
cargo sqlx prepare -- --lib

## Build all
cargo build --all --offline
```

See [Build Instructions](BUILDING.md)

## 🧪 Validation & Testing

### Local Validation

* Run all tests: `cargo test --all --offline`
* Check formatting: `cargo fmt --check`
* Lint warnings: `cargo clippy -- -D warnings`
* Audit: `cargo audit`
* Bench (opt): `cargo bench`

### Coverage (recommended)

* `cargo tarpaulin` for code coverage
* `cargo nextest` for parallel execution

### CI

See `.github/workflows/ci.yml` – all PRs must pass.

## ✅ Contribution Style

* Follow Conventional Commits: `feat/`, `fix/`, `test/`
* Follow Module name `feat/core/`, `feat/gui/`, `feat/tui/` and so on
* Use snake\_case for functions, CamelCase for types
* Structure every public module with `mod.rs` and matching `tests` section
* Prioritize async functions in all core APIs

## 🧠 Prompting Codex / Claude / Cursor

### Context & Style

* Use the full `lib.rs` API definitions for reference
* Point directly to modules like `importer`, `search`, `editor`, etc.
* Treat `types.rs` as source of truth for FileEntry and ImportMetadata
* Refer to `config.rs` for validating or modifying `AppConfig`

### Suggested Workflows

* For Codex:

  * Ask to refactor sync → async using `tokio`
  * Ask to generate unit tests using `tempfile`, `assert_cmd`
  * Ask to generate DSL parser with `pest` + tests
* For Claude:

  * Provide schema intent, ask for ER modeling
  * Ask to explain/refactor complex types or error handling logic
* For Cursor:

  * Tag `AGENTS.md` in `.cursor.json`
  * Enable project-wide context to link `tagbox-core` and CLI usage

## 🧩 Project Migration

* CLI and GUI are unified around `tagbox-core` – no logic duplication
* FTS5 search DSL is under active improvement (refactor in progress)
* Migration to async-std is **not** planned; use tokio ecosystem only

## 🧱 PR Rules

* Title: `[core] fix: extract_hash into reusable util`
* PR should only use English
* PR must link to specific module or function if logic is modified
* If modifying config or schema, update `config.toml` and ERD diagram
* Include tests or clear reason why it's not needed

## 🛠️ Commands

* Build all: `cargo build --all`
* CLI test: `cargo run -p tagbox-cli -- search --tag Rust`
* Interactive TUI (WIP): `cargo run -p tagbox-tui`

## 🔮 Experimental

* Claude 4 agents may generate `ImportMetadata` mapping from LLM-parsed JSON
* GPT-4 can attempt auto-tag suggestions from content with fallback rules
* Plugin system planned for v0.3+ (via dynamic trait loading in core)
