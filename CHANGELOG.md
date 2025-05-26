# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive CI/CD pipeline with GitHub Actions
  - Main CI workflow for push and pull requests with multi-platform support
  - Nightly builds with extended test matrix
  - Release workflow with cross-platform binary builds
  - Dependabot configuration for automated dependency updates
- Local CI test script (`scripts/ci-test.sh`) with quick and full modes
- Cross-platform Justfile for unified build commands
- Caching strategy using sccache for faster Rust compilations
- Code coverage reporting with cargo-tarpaulin
- Test result artifacts upload to GitHub Actions

### Changed
- Migrated project documentation from AGENTS.md to CLAUDE.md
- Updated build scripts for better cross-platform compatibility

### Fixed
- Code formatting issues across all packages
- Multiple Clippy warnings (to be addressed in future commits)

## [2025-05-26] TagBox Rust 项目大规模修复与同步

### 主要修复内容

- **数据库结构同步**：
  - 补全并修正了所有表结构（categories, tags, file_tags, file_authors, author_aliases, file_links, file_metadata, file_access_log, files_fts），严格对齐 `database.md`。
  - 移除了 `tags` 表中不存在的 `description`、`category_id` 字段。
  - 修正了 FTS5 表名、字段名、索引等。

- **SQLx 校验与 Rust 类型同步**：
  - 统一了 Rust 结构体与数据库表字段的类型（如 `Option<i32>` → `Option<i64>`，`i32` → `i64`）。
  - 修正了所有 `sqlx::query!`、`query_as!` 的参数数量和类型。
  - 修正了 chrono `DateTime` 类型的解析与转换。
  - 修复了 HashMap、Option 等类型不匹配问题。

- **FTS5 兼容性与依赖处理**：
  - 注释掉了不可用的 signal_tokenizer 相关代码，保留标准 FTS5 分词器兼容。
  - 保证全文检索功能在无 signal_tokenizer 的情况下可用。

- **功能方法补全**：
  - 为 `Editor` 实现了 `get_file_path`、`get_file` 等缺失方法。
  - 修正了 `importer.rs`、`editor.rs`、`search.rs`、`authors.rs` 等核心模块的 SQL 逻辑和类型。

- **构建与校验**：
  - `cargo build --all`、`cargo sqlx prepare -- --lib` 全部通过。
  - 数据库初始化脚本（init-db.rs）可用，支持一键初始化。
  - 所有 SQL 查询均通过 SQLx 校验。

### 其他

- 统一了数据库路径为 `F:/WinFile/Source/langs/rust/tagbox/.sqlx-data/tagbox.db`，避免路径混乱导致的运行和校验失败。
- 仅剩部分未使用代码的警告，核心功能全部可用。

---

> 本次更新为 TagBox Rust 项目后续开发和测试打下坚实基础。
