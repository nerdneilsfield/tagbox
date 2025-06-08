# TagBox Freya GUI 开发计划

## 分析总结

### CLI 与 Core 的集成模式

1. **配置管理**
   - 使用 `AppConfig::from_file()` 加载配置
   - 支持多个配置文件位置，优先级从高到低
   - 在执行命令前检查数据库是否存在

2. **核心功能调用模式**
   - **搜索**: 使用 `search_files_advanced()` 和 `SearchOptions`
   - **导入**: 先 `extract_metainfo()`，后 `import_file()`
   - **编辑**: 使用 `Editor` 类，支持预览更改
   - **列表**: 使用通配符搜索 `*` 获取所有文件
   - **链接**: 使用 `LinkManager` 管理文件关联

3. **异步处理**
   - 所有核心 API 都是异步的
   - 批量导入使用并行元数据提取 + 串行数据库写入

4. **错误处理**
   - 使用 `Result<T, TagboxError>` 处理错误
   - CLI 将 Core 错误包装为 `CliError`

## GUI 集成架构设计

### 1. 状态管理层
```rust
// state/app_state.rs
pub struct AppState {
    config: Arc<Mutex<AppConfig>>,
    search_results: Arc<Mutex<SearchResult>>,
    selected_files: Arc<Mutex<Vec<String>>>,
    current_view: Arc<Mutex<ViewType>>,
    import_queue: Arc<Mutex<Vec<PathBuf>>>,
    // ... 其他状态
}
```

### 2. API 适配层
```rust
// utils/api.rs
pub struct TagboxApi {
    config: AppConfig,
}

impl TagboxApi {
    pub async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResult>
    pub async fn import_files(&self, paths: Vec<PathBuf>) -> Result<Vec<FileEntry>>
    pub async fn edit_file(&self, id: &str, update: FileUpdateRequest) -> Result<()>
    // ... 其他方法
}
```

### 3. 组件架构
每个组件负责特定功能，通过 API 层与 Core 交互：
- `SearchBar`: 搜索输入和执行
- `FileList`: 文件列表显示
- `FilePreview`: 文件预览
- `ImportDialog`: 导入界面
- `EditDialog`: 编辑界面

## 核心功能开发计划

### 第一阶段：基础框架集成（1-2天）

1. **创建 API 适配层**
   - [ ] 实现 `TagboxApi` 结构体
   - [ ] 封装所有核心功能调用
   - [ ] 添加错误处理和日志

2. **更新状态管理**
   - [ ] 集成 `AppConfig` 到全局状态
   - [ ] 添加搜索结果缓存
   - [ ] 实现文件选择状态

3. **组件测试框架**
   - [ ] 创建测试工具模块
   - [ ] 实现 mock API 层
   - [ ] 设置组件测试环境

### 第二阶段：搜索功能（2-3天）

1. **SearchBar 组件**
   - [ ] 实现搜索输入框
   - [ ] 支持 DSL 和自然语言搜索
   - [ ] 添加搜索历史
   - [ ] 实现实时搜索建议

2. **FileList 组件**
   - [ ] 显示搜索结果
   - [ ] 支持分页
   - [ ] 实现排序功能
   - [ ] 添加文件选择

3. **组件测试**
   - [ ] 测试搜索输入验证
   - [ ] 测试搜索结果显示
   - [ ] 测试分页和排序
   - [ ] 测试文件选择逻辑

### 第三阶段：文件导入（2-3天）

1. **DragDropArea 组件**
   - [ ] 实现拖拽文件检测
   - [ ] 显示拖拽状态反馈
   - [ ] 支持多文件拖拽

2. **ImportDialog 组件**
   - [ ] 文件选择界面
   - [ ] 元数据编辑表单
   - [ ] 导入进度显示
   - [ ] 错误处理和重试

3. **组件测试**
   - [ ] 测试拖拽功能
   - [ ] 测试元数据表单验证
   - [ ] 测试批量导入
   - [ ] 测试错误恢复

### 第四阶段：文件管理（2-3天）

1. **FilePreview 组件**
   - [ ] 显示文件元数据
   - [ ] 集成文件预览
   - [ ] 添加快速操作按钮

2. **EditDialog 组件**
   - [ ] 元数据编辑表单
   - [ ] 分类选择器
   - [ ] 标签管理
   - [ ] 更改预览

3. **CategoryTree 组件**
   - [ ] 显示分类树
   - [ ] 支持展开/折叠
   - [ ] 分类过滤功能

4. **组件测试**
   - [ ] 测试预览显示
   - [ ] 测试编辑表单
   - [ ] 测试分类导航
   - [ ] 测试数据保存

### 第五阶段：高级功能（1-2天）

1. **AdvancedSearch 组件**
   - [ ] DSL 构建器界面
   - [ ] 过滤器选择
   - [ ] 搜索预设管理

2. **文件关联功能**
   - [ ] 链接管理界面
   - [ ] 关联文件显示
   - [ ] 批量关联操作

3. **组件测试**
   - [ ] 测试 DSL 构建
   - [ ] 测试过滤器
   - [ ] 测试关联管理

### 第六阶段：完善和优化（1-2天）

1. **性能优化**
   - [ ] 实现虚拟滚动
   - [ ] 添加结果缓存
   - [ ] 优化渲染性能

2. **用户体验**
   - [ ] 添加快捷键
   - [ ] 实现撤销/重做
   - [ ] 改进错误提示

3. **集成测试**
   - [ ] 端到端测试
   - [ ] 性能测试
   - [ ] 用户流程测试

## 组件测试策略

### 1. 单元测试
每个组件都应该有对应的单元测试：
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_search_input_validation() {
        // 测试搜索输入验证
    }
    
    #[test]
    fn test_file_selection() {
        // 测试文件选择逻辑
    }
}
```

### 2. 集成测试
测试组件与 API 层的集成：
```rust
#[tokio::test]
async fn test_search_integration() {
    let api = create_test_api();
    let result = api.search("test", None).await;
    assert!(result.is_ok());
}
```

### 3. UI 测试
使用 Freya 的测试工具：
```rust
#[test]
fn test_search_bar_ui() {
    let mut app = launch_test(SearchBar);
    app.push_event(PlatformEvent::Keyboard { 
        // 模拟键盘输入
    });
    // 验证 UI 状态
}
```

## 开发优先级

1. **最高优先级**：搜索和文件列表（核心功能）
2. **高优先级**：文件导入和预览
3. **中优先级**：编辑和分类管理
4. **低优先级**：高级搜索和文件关联

## 技术要点

1. **异步处理**
   - 使用 `tokio::spawn` 处理后台任务
   - 实现加载状态和进度反馈
   - 避免阻塞 UI 线程

2. **错误处理**
   - 统一错误处理机制
   - 用户友好的错误提示
   - 支持错误恢复和重试

3. **状态同步**
   - 使用 `use_coroutine` 处理异步操作
   - 实现状态变更通知
   - 保持 UI 响应性

4. **代码复用**
   - 尽可能复用 CLI 的逻辑
   - 创建共享的工具函数
   - 模块化组件设计

## 时间估算

- 总时间：10-15 天
- 核心功能：7-10 天
- 测试和优化：3-5 天

## 下一步行动

1. 创建 API 适配层
2. 实现基础搜索功能
3. 添加文件列表组件
4. 编写第一批组件测试