# TagBox 架构与模块接口文档

## 一、项目结构总览（Cargo Workspace）

```
tagbox/                  # 项目根目录
├── Cargo.toml           # Workspace 配置
├── tagbox-core/         # 核心逻辑库（lib crate）
├── tagbox-cli/          # 命令行工具（bin crate）
├── tagbox-tui/          # TUI 界面（bin crate）
├── tagbox-gui/          # GUI 界面（fltk，bin crate）
└── tagbox-config/       # 配置解析库（可选 lib crate）
```

## 二、核心模块结构（tagbox-core）

```
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
├── utils.rs              # hash计算、路径处理、时间处理
```

## 三、模块依赖关系图（API 层级）

```
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
     utils.rs        config.rs                  authors.rs/link.rs
```

* `lib.rs` 暴露统一 API 接口
* CLI / GUI / TUI 统一依赖 `tagbox-core`

## 四、核心模块 API 设计

### lib.rs（对外接口）

```rust
pub fn init_database(path: &Path) -> Result<()>;
pub fn load_config(path: &Path) -> Result<AppConfig>;

pub fn extract_metainfo(path: &Path, config: &AppConfig) -> Result<ImportMetadata>;
pub fn import_file(path: &Path, metadata: ImportMetadata, config: &AppConfig) -> Result<FileEntry>;

pub fn search_files(query: &str, config: &AppConfig) -> Result<Vec<FileEntry>>;
pub fn get_file_path(file_id: &str, config: &AppConfig) -> Result<PathBuf>;
pub fn edit_file(file_id: &str, update: FileUpdateRequest) -> Result<()>;

pub fn link_files(a: &str, b: &str, relation: Option<String>) -> Result<()>;
pub fn unlink_files(a: &str, b: &str) -> Result<()>;
```

### importer.rs

```rust
pub fn import_file(...);
```

### metainfo.rs

```rust
pub fn from_filename(...);
pub fn from_json(...);
pub fn from_pdf(...);
```

### pathgen.rs

```rust
pub fn generate_filename(meta: &ImportMetadata, config: &AppConfig) -> String;
pub fn generate_path(meta: &ImportMetadata, config: &AppConfig) -> PathBuf;
```

### search.rs

```rust
pub fn search_files(query: &str, config: &AppConfig) -> Result<Vec<FileEntry>>;
```

### editor.rs

```rust
pub fn edit_file(...);
pub fn soft_delete(...);
```

### link.rs

```rust
pub fn link_files(...);
pub fn unlink_files(...);
```

### authors.rs

```rust
pub fn resolve_author_aliases(...);
pub fn get_author_files(...);
```

## 五、配置结构（AppConfig）

```toml
[rename]
template = "{title}_{authors}_{year}_{publisher}"

[classify]
pattern = "{category1}/{category2}/{filename}"

[meta]
prefer_json = true
fallback_pdf = true
```

```rust
pub struct AppConfig {
  pub rename_template: String,
  pub classify_template: String,
  pub prefer_json: bool,
  pub fallback_pdf: bool,
  ...
}
```

## 六、下一步建议

* 完善每个模块的入参结构（如 `ImportMetadata`, `FileEntry`）
* 为每个核心操作设计 CLI 命令对应参数结构
* 构建最小工作流测试链：导入 → 搜索 → 获取路径
