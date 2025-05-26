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
* 不在各功能模块中直接解析环境变量或路径，应统一入口

### 4. 错误处理

* 所有模块返回 `Result<T, TagboxError>`
* 提供统一错误类型 `TagboxError`，支持 `thiserror`
* 避免 `unwrap` / `expect`，全部使用显式异常传播

---

## 三、单元测试规范

### 1. 测试组织结构

* 每个模块必须带有 `mod tests`，存于同一文件尾部或 `tests/` 目录
* 公共结构体和函数需覆盖以下测试：

  * 正常行为（正常导入、查询等）
  * 边界行为（空输入、无结果等）
  * 错误行为（数据库错误、字段缺失等）

### 2. 推荐工具

* 使用 Rust 自带测试框架：`#[test]`
* 支持使用 `tempfile`, `assert_cmd`, `serial_test` 等做集成测试
* 未来支持 `cargo nextest` 并行测试框架加速 CI

### 3. 覆盖率要求

* 各模块核心路径需覆盖 80% 以上逻辑分支
* 可使用 `cargo tarpaulin` 做代码覆盖率检测

---

## 四、集成测试建议

### 场景建议（放在 `tests/cli.rs`, `tests/integration.rs`）

* 导入 → 搜索 → 获取路径 → 打开（完整流程）
* 导入重复文件（hash 判重）
* 模糊查询 + DSL 过滤组合测试
* CLI `--stdio` 模式 JSON 交互测试

---

## 五、CI/CD 规范建议

### 构建建议

* 使用 GitHub Actions 自动构建：

  * `cargo check`
  * `cargo fmt --check`
  * `cargo clippy -- -D warnings`
  * `cargo test --all`

### 发布策略

* CLI 工具支持 `cargo install` 发布方式
* GUI 可构建为压缩包或单文件（静态链接）
* 支持通过 GitHub Release 分发二进制

---

## 六、未来可引入规范

| 方向   | 工具/做法                  | 说明                 |
| ---- | ---------------------- | ------------------ |
| 文档生成 | `cargo doc` + `mdbook` | 自动生成 API 文档 + 用户手册 |
| 代码风格 | `rustfmt.toml`         | 统一缩进、换行等           |
| 静态检查 | `clippy.toml`          | 控制 lint 级别         |
| 安全审计 | `cargo audit`          | 第三方依赖漏洞检测          |
