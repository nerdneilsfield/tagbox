# TagBox GUI 深度设计方案

## 一、架构设计

### 1.1 核心架构原则
- **异步优先**：完全基于 `tokio` 异步运行时，与 `tagbox-core` 保持一致
- **模块化组件**：每个页面作为独立组件，遵循单一职责原则
- **事件驱动**：基于 FLTK 的回调机制和 Rust 的 `channel` 构建响应式架构
- **状态管理**：集中式状态管理，支持跨组件数据共享

### 1.2 项目结构
```rust
tagbox-gui/
├── src/
│   ├── main.rs                 // 应用入口
│   ├── app.rs                  // 主应用控制器
│   ├── state/                  // 状态管理
│   │   ├── mod.rs
│   │   ├── app_state.rs       // 全局应用状态
│   │   └── events.rs          // 事件定义
│   ├── components/            // UI 组件
│   │   ├── mod.rs
│   │   ├── main_window.rs     // 主窗口
│   │   ├── search_bar.rs      // 搜索栏
│   │   ├── category_tree.rs   // 分类树
│   │   ├── file_preview.rs    // 文件预览面板
│   │   ├── import_dialog.rs   // 导入对话框
│   │   ├── edit_dialog.rs     // 编辑对话框
│   │   └── advanced_search.rs // 高级搜索
│   ├── utils/                 // 工具函数
│   │   ├── mod.rs
│   │   ├── async_bridge.rs    // 异步桥接
│   │   ├── clipboard.rs       // 剪贴板操作
│   │   └── system_open.rs     // 系统打开文件
│   └── themes/                // 主题样式
│       ├── mod.rs
│       └── default.rs
```

## 二、状态管理与事件系统

### 2.1 应用状态设计
```rust
// state/app_state.rs
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub current_files: Vec<FileEntry>,
    pub selected_file: Option<FileEntry>,
    pub current_query: String,
    pub category_tree: CategoryTreeState,
    pub is_loading: bool,
}

#[derive(Clone)]
pub struct CategoryTreeState {
    pub expanded_nodes: HashSet<String>,
    pub selected_category: Option<String>,
}
```

### 2.2 事件系统
```rust
// state/events.rs
#[derive(Debug, Clone)]
pub enum AppEvent {
    // 搜索相关
    SearchQuery(String),
    AdvancedSearch(SearchOptions),
    SearchResults(SearchResult),
    
    // 文件操作
    FileSelected(String), // file_id
    FileOpen(String),
    FileEdit(String),
    FileImport(PathBuf),
    
    // 分类树操作
    CategoryExpand(String),
    CategorySelect(String),
    
    // 系统事件
    LoadingStart,
    LoadingEnd,
    Error(String),
}
```

## 三、核心组件设计

### 3.1 主窗口布局 (components/main_window.rs)
```rust
pub struct MainWindow {
    window: Window,
    // 顶部搜索栏
    search_container: Flex,
    search_input: Input,
    advanced_btn: Button,
    import_btn: Button,
    
    // 主体区域 - 三栏布局
    main_container: Flex,
    category_tree: Tree,
    file_list: HoldBrowser,
    preview_panel: Group,
    
    // 状态
    state: AppState,
    event_sender: Sender<AppEvent>,
}

impl MainWindow {
    pub fn new(config: AppConfig) -> (Self, Receiver<AppEvent>) {
        // 创建主窗口 (1200x800)
        let mut window = Window::new(100, 100, 1200, 800, "TagBox");
        
        // 顶部搜索栏 (高度: 60px)
        let mut search_container = Flex::new(10, 10, 1180, 50, None);
        search_container.set_type(FlexType::Row);
        // 搜索输入框 (70% 宽度)
        // 高级搜索按钮 (15% 宽度) 
        // 导入按钮 (15% 宽度)
        
        // 主体三栏布局 (剩余空间)
        let mut main_container = Flex::new(10, 70, 1180, 720, None);
        main_container.set_type(FlexType::Row);
        // 左侧分类树 (25% 宽度)
        // 中间文件列表 (40% 宽度)
        // 右侧预览面板 (35% 宽度)
    }
}
```

### 3.2 分类树组件 (components/category_tree.rs)
```rust
pub struct CategoryTree {
    tree: Tree,
    state: CategoryTreeState,
    event_sender: Sender<AppEvent>,
}

impl CategoryTree {
    // 加载分类树数据
    pub async fn load_categories(&mut self, config: &AppConfig) -> Result<()> {
        // 调用 tagbox_core::search_files_advanced 获取分类数据
        // 构建三级分类树结构
        // 更新 FLTK Tree 组件显示
    }
    
    // 处理节点展开/折叠
    fn on_tree_select(&mut self, path: &str) {
        // 发送 CategorySelect 事件
        // 更新选中状态
    }
}
```

### 3.3 文件预览面板 (components/file_preview.rs)
```rust
pub struct FilePreview {
    container: Group,
    
    // 文件信息区域
    title_label: Output,
    path_label: Output,
    authors_label: Output,
    tags_container: Flex,
    summary_text: TextDisplay,
    
    // 关联文件区域
    links_browser: Browser,
    
    // 操作按钮
    open_btn: Button,
    edit_btn: Button,
    copy_path_btn: Button,
    cd_btn: Button,
    
    current_file: Option<FileEntry>,
    event_sender: Sender<AppEvent>,
}

impl FilePreview {
    pub async fn display_file(&mut self, file_id: &str, config: &AppConfig) -> Result<()> {
        // 调用 tagbox_core::get_file 获取文件详情
        // 调用 tagbox_core::LinkManager 获取关联文件
        // 更新所有显示组件
    }
}
```

## 四、页面功能实现

### 4.1 搜索功能 (components/search_bar.rs)
```rust
pub struct SearchBar {
    input: Input,
    suggestion_popup: MenuButton, // 自动补全
}

impl SearchBar {
    // DSL 搜索
    pub async fn perform_search(&self, query: &str, config: &AppConfig) -> Result<SearchResult> {
        tagbox_core::search_files_advanced(query, None, config).await
    }
    
    // 模糊搜索（用户输入时实时建议）
    pub async fn get_suggestions(&self, partial: &str, config: &AppConfig) -> Result<Vec<String>> {
        tagbox_core::fuzzy_search_files(partial, None, config).await
            .map(|result| result.entries.into_iter()
                .map(|entry| entry.title.unwrap_or(entry.original_name))
                .collect())
    }
}
```

### 4.2 高级搜索对话框 (components/advanced_search.rs)
```rust
pub struct AdvancedSearchDialog {
    dialog: Window,
    
    // 搜索字段
    title_input: Input,
    author_choice: Choice,  // 下拉选择
    tag_input: Input,       // 支持多标签输入
    category_l1: Choice,
    category_l2: Choice,
    category_l3: Choice,
    year_from: IntInput,
    year_to: IntInput,
    
    search_btn: Button,
    cancel_btn: Button,
}

impl AdvancedSearchDialog {
    pub async fn populate_dropdowns(&mut self, config: &AppConfig) -> Result<()> {
        // 调用 tagbox_core 获取所有作者、分类等数据
        // 填充下拉选择框
    }
    
    pub fn build_search_options(&self) -> SearchOptions {
        // 将表单数据转换为 SearchOptions
    }
}
```

### 4.3 文件导入对话框 (components/import_dialog.rs)
```rust
pub struct ImportDialog {
    dialog: Window,
    
    // 文件选择区域
    file_path_input: Input,
    browse_btn: Button,
    url_input: Input,        // 支持 URL 下载
    download_btn: Button,
    
    // 元数据表单（与 GUI 设计指南一致）
    title_input: Input,
    authors_input: Input,    // 逗号分隔或标签形式
    year_input: IntInput,
    publisher_input: Input,
    tags_input: Input,       // 多标签输入
    summary_text: TextEditor,
    
    // 分类选择
    category_l1: Choice,
    category_l2: Choice,
    category_l3: Choice,
    
    // 操作按钮
    extract_meta_btn: Button,
    import_move_btn: Button,
    import_keep_btn: Button,
    cancel_btn: Button,
}

impl ImportDialog {
    pub async fn extract_metadata(&mut self, path: &Path, config: &AppConfig) -> Result<()> {
        let metadata = tagbox_core::extract_metainfo(path, config).await?;
        // 填充表单字段
        self.populate_form(metadata);
        Ok(())
    }
    
    pub async fn import_file(&self, config: &AppConfig, move_file: bool) -> Result<FileEntry> {
        let metadata = self.collect_form_data();
        let path = PathBuf::from(self.file_path_input.value());
        
        tagbox_core::import_file(&path, metadata, config).await
    }
}
```

## 五、异步集成方案

### 5.1 异步桥接 (utils/async_bridge.rs)
```rust
pub struct AsyncBridge {
    runtime: tokio::runtime::Runtime,
    event_sender: Sender<AppEvent>,
}

impl AsyncBridge {
    pub fn spawn_task<F, R>(&self, future: F) 
    where
        F: Future<Output = Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            match future.await {
                Ok(result) => {
                    // 发送成功事件
                }
                Err(e) => {
                    sender.send(AppEvent::Error(e.to_string())).unwrap();
                }
            }
        });
    }
}
```

### 5.2 FLTK 与异步集成
```rust
// main.rs
#[tokio::main]
async fn main() -> Result<()> {
    let app = fltk::app::App::default();
    let config = tagbox_core::load_config(Path::new("config.toml")).await?;
    
    let (mut main_window, event_receiver) = MainWindow::new(config.clone());
    let async_bridge = AsyncBridge::new();
    
    // 事件循环
    while app.wait() {
        // 处理 FLTK 事件
        if let Ok(event) = event_receiver.try_recv() {
            handle_app_event(event, &mut main_window, &async_bridge, &config).await;
        }
    }
    
    Ok(())
}
```

## 六、关键技术要点

### 6.1 性能优化
- **虚拟化列表**：当文件数量超过 1000 时，使用虚拟滚动
- **延迟加载**：分类树和文件列表按需加载
- **缓存机制**：搜索结果和文件元数据缓存

### 6.2 用户体验
- **响应式搜索**：输入时实时显示建议
- **拖拽支持**：文件可拖拽到导入区域
- **键盘导航**：支持全键盘操作
- **状态保存**：窗口大小、分栏比例等记住用户偏好

### 6.3 错误处理
- **优雅降级**：网络错误时回退到本地功能
- **用户友好提示**：具体的错误信息和建议操作
- **日志记录**：详细的操作日志用于调试

## 七、实现计划

### 阶段一：基础架构 (1-2 周)
1. 搭建主窗口和基础布局
2. 实现状态管理和事件系统
3. 完成异步桥接层

### 阶段二：核心功能 (2-3 周)
1. 实现搜索栏和基础搜索
2. 完成分类树组件
3. 实现文件预览面板

### 阶段三：高级功能 (1-2 周)
1. 实现文件导入对话框
2. 完成高级搜索功能
3. 添加文件编辑功能

### 阶段四：优化完善 (1 周)
1. 性能优化和测试
2. 用户体验细节优化
3. 文档和部署

这个设计方案充分利用了 `tagbox-core` 的 API，遵循了 GUI 设计指南的要求，并考虑了 FLTK 的特性和最佳实践。整个架构支持异步操作、模块化扩展，并提供了良好的用户体验。

## 核心亮点

1. **完全异步架构** - 与 `tagbox-core` 无缝集成
2. **模块化设计** - 每个组件独立可测试
3. **事件驱动** - 响应式用户界面
4. **性能优化** - 虚拟化列表、缓存机制
5. **用户体验** - 拖拽、键盘导航、实时搜索

## 技术栈选择理由

- **FLTK** - 轻量级、跨平台、单文件部署
- **Tokio** - 与 core 库保持一致的异步运行时
- **Channel** - 组件间通信的最佳实践

这个方案严格遵循了项目的设计原则，所有功能都通过 `tagbox-core` 的公开 API 实现，确保了架构的一致性和可维护性。