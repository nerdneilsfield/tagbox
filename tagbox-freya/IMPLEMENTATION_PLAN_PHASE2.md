# TagBox Freya GUI - 中级功能实现计划

## 深度分析总结

经过使用 MCP Sequential Thinking 工具的深度分析，我确定了以下核心问题和实现路径：

### 当前状态评估
1. **UI框架已完成**：基础界面、路由、状态管理都已就绪
2. **核心问题**：大量功能只有UI壳，缺少实际实现
3. **技术债务**：Button组件问题已解决，但仍有许多TODO标记
4. **集成不完整**：TagBoxService 与 tagbox-core 的连接需要完善

### 实施策略
基于用户需求和技术可行性，按以下优先级实现：

## 第一阶段：核心文件操作功能（最高优先级，2天）

### 1.1 添加必要依赖
```toml
# tagbox-freya/Cargo.toml
[dependencies]
open = "5.0"              # 跨平台打开文件/文件夹
rfd = "0.14"              # 异步文件对话框
arboard = "3.3"           # 强大的剪贴板支持
```

### 1.2 实现系统集成功能
```rust
// src/utils/system_open.rs
pub async fn open_file(path: &Path) -> Result<()>
pub async fn reveal_in_folder(path: &Path) -> Result<()>
pub async fn open_folder(path: &Path) -> Result<()>
```

### 1.3 升级剪贴板功能
```rust
// src/utils/clipboard.rs - 使用 arboard
pub fn copy_to_clipboard(text: &str) -> Result<()>
pub fn get_clipboard_content() -> Result<String>
```

### 1.4 集成文件选择器
```rust
// src/components/drag_drop.rs
async fn select_files() -> Option<Vec<PathBuf>> {
    rfd::AsyncFileDialog::new()
        .add_filter("Documents", &["pdf", "epub", "txt", "md"])
        .pick_files()
        .await
}
```

## 第二阶段：搜索功能完善（3天）

### 2.1 实现搜索DSL解析器增强
- 在 tagbox-core/src/search.rs 完善 DSL 语法
- 支持复杂查询：`tag:rust AND (author:john OR year:2024)`
- 实现查询验证和错误提示

### 2.2 创建搜索构建器组件
```rust
// src/components/search_builder.rs
pub fn SearchBuilder() -> Element {
    // 可视化构建搜索条件
    // 支持添加/删除条件
    // 实时预览查询语句
}
```

### 2.3 搜索历史和建议
- 在 AppState 中维护搜索历史
- 实现自动完成功能
- 保存常用搜索

## 第三阶段：分类管理系统（3天）

### 3.1 级联分类选择器
```rust
// src/components/category_picker.rs
#[component]
pub fn CategoryPicker(
    selected: Signal<(Option<i32>, Option<i32>, Option<i32>)>,
    on_change: EventHandler<(Option<i32>, Option<i32>, Option<i32>)>,
) -> Element
```

### 3.2 分类管理页面
```rust
// src/pages/category_manager.rs
pub fn CategoryManagerPage() -> Element {
    // 完整的分类CRUD界面
    // 拖放排序
    // 批量操作
}
```

### 3.3 分类数据结构优化
- 实现高效的树形结构
- 支持移动/合并分类
- 维护分类统计信息

## 第四阶段：批量操作功能（2天）

### 4.1 多选机制
```rust
// 在 AppState 中添加
pub selected_files: Signal<HashSet<String>>,
pub selection_mode: Signal<bool>,
```

### 4.2 批量操作栏
```rust
// src/components/batch_operation_bar.rs
pub fn BatchOperationBar() -> Element {
    // 显示选中数量
    // 批量操作按钮
    // 全选/反选
}
```

### 4.3 批量操作实现
- 批量删除（事务处理）
- 批量标签修改
- 批量分类移动
- 批量导出

## 第五阶段：用户体验增强（2天）

### 5.1 键盘快捷键系统
```rust
// src/utils/keyboard.rs
pub fn use_global_shortcuts() {
    // 注册全局快捷键
    // 处理快捷键事件
}
```

### 5.2 右键菜单（如Freya支持）
```rust
// src/components/context_menu.rs
pub fn ContextMenu() -> Element
```

### 5.3 虚拟列表优化
```rust
// src/components/virtual_list.rs
pub fn VirtualFileList() -> Element {
    // 只渲染可见项
    // 平滑滚动
    // 保持选择状态
}
```

## 技术实现要点

### 使用 MCP 工具辅助开发
1. **Sequential Thinking**：复杂功能的逻辑规划
2. **Context7**：获取最新的 Freya/Dioxus API 文档
3. **WebSearch**：查找特定问题的解决方案

### 错误处理模式
```rust
// 统一的错误处理
match operation().await {
    Ok(result) => {
        app_state.show_success("操作成功");
        // 更新UI
    }
    Err(e) => {
        app_state.show_error(&format!("操作失败: {}", e));
        // 恢复状态
    }
}
```

### 性能优化策略
1. 使用 `use_memo` 缓存计算结果
2. 实现分页加载
3. 延迟加载大型组件
4. 使用 Web Workers（如Freya支持）

## 测试计划

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use freya_testing::*;
    
    #[tokio::test]
    async fn test_file_operations() {
        // 测试文件打开
        // 测试剪贴板
        // 测试错误处理
    }
}
```

### 集成测试
- 完整用户流程测试
- 跨平台兼容性测试
- 性能基准测试

## 时间线

| 阶段 | 时间 | 交付物 |
|------|------|--------|
| 第一阶段 | 2天 | 文件操作功能完整可用 |
| 第二阶段 | 3天 | 高级搜索功能上线 |
| 第三阶段 | 3天 | 分类管理系统完成 |
| 第四阶段 | 2天 | 批量操作功能可用 |
| 第五阶段 | 2天 | UX增强完成 |

**总计：12天完成所有中级功能**

## 立即行动项

1. **今天**：添加依赖，实现 system_open.rs
2. **明天**：完成文件选择器集成，修复 FilePreview 按钮
3. **后天**：开始搜索功能增强

## 成功指标

- [ ] 所有文件操作按钮功能正常
- [ ] 搜索响应时间 < 500ms
- [ ] 支持 10000+ 文件流畅操作
- [ ] 零崩溃率
- [ ] 用户满意度提升

## 风险与缓解

1. **Freya限制**：某些功能可能不支持
   - 缓解：提前测试，准备替代方案
   
2. **跨平台兼容**：不同OS行为差异
   - 缓解：条件编译，平台特定代码
   
3. **性能瓶颈**：大数据量时卡顿
   - 缓解：虚拟列表，分页，缓存

## 结论

通过这个详细的实施计划，TagBox Freya GUI 将从一个基础UI框架转变为功能完整、用户友好的桌面应用。每个阶段都有明确的目标和可测量的成果，确保项目稳步推进。