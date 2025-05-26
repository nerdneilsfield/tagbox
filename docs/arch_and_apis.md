# TagBox 架构与模块接口文档

## 一、项目结构总览（Cargo Workspace）

```bash
tagbox/                  # 项目根目录
├── Cargo.toml           # Workspace 配置
├── tagbox-core/         # 核心逻辑库（lib crate）
├── tagbox-cli/          # 命令行工具（bin crate）
├── tagbox-tui/          # TUI 界面（bin crate）
├── tagbox-gui/          # GUI 界面（fltk，bin crate）
└── tagbox-config/       # 配置解析库（可选 lib crate）
```

## 二、核心模块结构（tagbox-core）

```bash
tagbox-core/
├── lib.rs                # 对外统一接口
├── schema.rs             # 数据库初始化/迁移
├── config.rs             # TOML 配置加载器
├── importer.rs           # 导入逻辑：hash、移动、写入
├── metainfo.rs           # 提取文件名/meta/json 信息
├── pathgen.rs            # 文件名模板 & 分类路径生成
├── search.rs             # DSL → SQL/FTS5 查询器
├── editor.rs             # 修改/删除元信息
├── link.rs               # 文件双链关系管理
├── authors.rs            # 作者归一、别名合并
├── errors.rs             # 错误定义 TagboxError
├── utils.rs              # hash计算、路径处理、时间处理
```

## 三、模块依赖关系图（API 层级）

```ascii
             ┌────────────────────┐
             │     lib.rs         │
             └────────┬───────────┘
                      ▼
        ┌──────────────┬───────────────────────────────┐
        ▼              ▼                               ▼
    importer.rs     search.rs                      editor.rs
        │              │                               │
        ▼              ▼                               ▼
   metainfo.rs     pathgen.rs                    schema.rs
        │              │                               │
        ▼              ▼                               ▼
     utils.rs        config.rs                  authors.rs/link.rs/errors.rs
```

* `lib.rs` 暴露统一 API 接口，全部为 `async fn`
* 所有模块使用 `TagboxError` 作为统一错误类型

## 四、核心模块 API 设计（异步）

### lib.rs（对外接口）

```rust
pub async fn init_database(path: &Path) -> Result<(), TagboxError>;
pub async fn load_config(path: &Path) -> Result<AppConfig, TagboxError>;

pub async fn extract_metainfo(path: &Path, config: &AppConfig) -> Result<ImportMetadata>;
pub async fn import_file(path: &Path, metadata: ImportMetadata, config: &AppConfig) -> Result<FileEntry>;

pub async fn search_files(query: &str, config: &AppConfig) -> Result<Vec<FileEntry>>;
pub async fn search_files_advanced(query: &str, options: Option<SearchOptions>) -> Result<SearchResult>;

pub async fn get_file_path(file_id: &str, config: &AppConfig) -> Result<PathBuf>;
pub async fn edit_file(file_id: &str, update: FileUpdateRequest) -> Result<()>;

pub async fn link_files(a: &str, b: &str, relation: Option<String>) -> Result<()>;
pub async fn unlink_files(a: &str, b: &str) -> Result<()>;
```

### types.rs（核心结构）

```rust
pub struct FileEntry { ... }
pub struct ImportMetadata { ... }
pub struct FileUpdateRequest { ... }
pub struct SearchOptions { ... }
pub struct SearchResult { entries: Vec<FileEntry>, total_count: usize, offset: usize, limit: usize }
```

### config.rs

```rust
pub struct AppConfig {
  pub import: ImportConfig,
  pub search: SearchConfig,
  pub database: DatabaseConfig,
}
impl AppConfig { pub fn validate(&self) -> Result<(), String> }
```

---

## 五、配置文件结构（config.toml）

```toml
[database]
path = "./tagbox_data/meta.db"

[import.paths]
rename_template = "{title}_{authors}_{year}"
classify_template = "{category1}/{filename}"

[import.metadata]
prefer_json = true
fallback_pdf = true

[search]
default_limit = 50
```

## 六、后续建议

* 各模块需编写模块级文档注释与单元测试
* 支持 CLI / GUI 异步调度统一（由 tokio 驱动）
* Future: 导出 Plugin trait、支持插件式导入器
