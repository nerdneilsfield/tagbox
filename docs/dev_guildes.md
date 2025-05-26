# TagBox 开发规范与测试指南

## 一、项目目标与规范基调

TagBox 是一个面向个人文件管理的本地原生工具，强调：

* 模块化结构、清晰边界（每个模块职责单一）
* 最小依赖、跨平台兼容性（Rust 静态编译）
* 易测试、易扩展、支持 CLI / GUI / TUI 多入口
* 数据安全与可降级：不依赖服务端、不强绑定格式

---

## 二、Rust 项目结构与风格约定

### 1. 模块组织

* 每个核心子系统应为 `mod.rs` 或独立 crate（如 `tagbox-core`）
* 避免“巨型函数”，每个函数关注点单一、50 行内为宜
* 导出接口集中在 `lib.rs`，通过 `pub use mod::*` 暴露统一 API

### 2. 命名规范

* 函数名使用 snake\_case：`get_file_path`, `import_file`
* 类型名使用 CamelCase：`FileEntry`, `ImportMetadata`
* 模块名对应领域职责：`importer`, `search`, `editor`, `authors`

### 3. 配置管理

* 所有配置项集中在 `AppConfig`，使用 `toml` 加载 + `serde` 派发
* 配置结构推荐分层，支持 `validate()` 校验
* 统一使用 `.toml` 配置格式，避免混乱

### 4. 错误处理

* 所有模块返回 `Result<T, TagboxError>`
* 提供统一错误类型 `TagboxError`，使用 `thiserror` 实现

```rust
#[derive(thiserror::Error, Debug)]
pub enum TagboxError {
  #[error("数据库错误: {0}")]
  Database(Box<dyn std::error::Error + Send + Sync>),
  #[error("配置错误: {0}")]
  Config(String),
  #[error("I/O错误: {0}")]
  Io(#[from] std::io::Error),
  #[error("文件未找到: {path}")]
  FileNotFound { path: std::path::PathBuf },
  #[error("重复的文件哈希: {hash}")]
  DuplicateHash { hash: String },
  #[error("无效查询语法: {query}")]
  InvalidQuery { query: String },
  #[error("元信息提取失败: {0}")]
  MetaInfoExtraction(String),
}
```

---

## 三、异步编程建议

* 所有数据库与文件操作应采用 `async fn` 实现，便于并发
* 推荐使用 `tokio` + `sea-query` + `rusqlite` 或 `sqlx` + `tokio` 组合
* CLI/GUI 可通过 `tokio::main` 驱动 async API

---

## 四、单元测试与集成测试

### 1. 测试组织结构

* 每个模块带 `mod tests`，使用 `#[cfg(test)]`
* 可在 `tests/` 下增加集成测试（如导入-搜索全链路）

### 2. 样例推荐

```rust
#[tokio::test]
async fn test_import_and_search() {
  // 1. 使用 tempfile 创建数据库
  // 2. init_database()
  // 3. 构造 ImportMetadata → import_file()
  // 4. 调用 search_files() 验证记录是否写入
}
```

### 3. 工具与覆盖率

* 推荐工具：`tempfile`, `assert_cmd`, `predicates`, `serial_test`, `tokio`
* 覆盖率检测：使用 `cargo tarpaulin` + `cargo nextest`

### 4. 性能测试（可选）

* 使用 `criterion` 进行导入与搜索性能基准评估

---

## 五、CI/CD 流程建议

### 1. Git 工作流

* 分支命名：`feature/xxx`, `fix/xxx`, `refactor/xxx`
* 提交格式：Conventional Commits，如 `feat: 添加搜索分页`

### 2. GitHub Actions 工作流建议

```yaml
steps:
- uses: actions/checkout@v3
- uses: actions-rs/toolchain@v1
  with:
    toolchain: stable
    override: true
- run: cargo fmt --check
- run: cargo clippy -- -D warnings
- run: cargo test --all
- run: cargo audit
```

### 3. 版本发布建议

* CLI 可通过 `cargo install`
* GUI 可使用 `cargo-bundle`, `cargo-dist` 发布二进制
* Release 支持多平台构建与 artifact 上传

---

## 六、后续规范规划

| 方向   | 工具/方法                  | 说明              |
| ---- | ---------------------- | --------------- |
| 文档生成 | `cargo doc`, `mdbook`  | 自动生成 API + 使用手册 |
| 风格统一 | `rustfmt.toml`         | 格式约定一致          |
| 安全检查 | `cargo audit`          | 检测依赖漏洞          |
| 构建测试 | `nextest`, `tarpaulin` | 并行测试 + 覆盖率统计    |
| 插件系统 | 预留动态注册机制               | 导入器 / 编辑器扩展点    |
