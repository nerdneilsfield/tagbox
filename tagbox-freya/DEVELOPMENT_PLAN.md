# TagBox Freya 开发计划

## 当前状态总结

### 已完成功能
- ✅ 基础应用结构与异步状态初始化
- ✅ 主页面搜索和文件列表显示
- ✅ 导入页面的拖放和元数据提取
- ✅ 编辑页面的保存功能
- ✅ 基础路由系统

### 待实现功能
根据 `docs/gui_design_guide.md` 的要求，以下功能尚未实现：

1. **文件管理功能**
   - 文件删除（带确认对话框）
   - 重置到原始值
   - 重新提取元数据
   
2. **组织与发现功能**
   - 分类树的实际功能
   - 高级 DSL 搜索
   - 文件预览
   
3. **高级功能**
   - 语义链接视图页面
   - 批量操作
   - 导出功能

## 分阶段实施计划

### 第一阶段：核心文件管理功能（预计 1 天）

#### 1.1 删除确认对话框组件（1 小时）
**文件**: `src/components/confirm_dialog.rs`
```rust
#[component]
pub fn ConfirmDialog(
    title: String,
    message: String,
    is_open: Signal<bool>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element
```
- 模态对话框覆盖层
- 标题、消息和确认/取消按钮
- 键盘支持（ESC 取消，Enter 确认）

#### 1.2 文件删除功能（30 分钟）
- 在编辑页面集成确认对话框
- 调用 `TagBoxService.delete_file()`
- 成功后导航回主页面
- 失败时显示错误信息

#### 1.3 重置到原始值（1 小时）
- 在编辑页面存储原始 `FileEntry` 数据
- 添加重置按钮处理程序
- 恢复所有表单字段到原始值
- 添加确认提示

#### 1.4 重新提取元数据（1 小时）
- 调用 `TagBoxService.extract_metadata()`
- 显示加载状态
- 更新表单字段
- 处理提取失败情况

#### 1.5 通知系统（1.5 小时）
**文件**: `src/components/toast.rs`
- 全局通知组件
- 支持 success/error/info/warning 类型
- 自动消失（可配置时间）
- 支持堆叠多个通知
- 动画效果

#### 1.6 导航改进（1 小时）
- 实现 `navigate_to()` 和 `navigate_back()` 辅助函数
- 添加面包屑导航组件
- 所有页面添加返回按钮
- 操作成功后自动导航

### 第二阶段：组织与发现功能（预计 1-2 天）

#### 2.1 分类树实现（3 小时）
- 从搜索结果构建真实分类树
- 支持展开/折叠
- 点击分类过滤文件列表
- 显示每个分类的文件数量

#### 2.2 文件预览组件（2 小时）
**文件**: `src/components/file_preview.rs`
- 支持不同文件类型（PDF、图片、文本）
- 显示元数据摘要
- 快速操作按钮

#### 2.3 DSL 搜索解析器（4 小时）
- 实现基础 DSL 解析
- 搜索构建器 UI
- 与后端集成
- 搜索历史

#### 2.4 高级搜索界面（2 小时）
- 可视化查询构建器
- 保存常用搜索
- 搜索建议

### 第三阶段：高级功能（预计 2-3 天）

#### 3.1 语义链接页面（4 小时）
- 新建链接视图页面
- 可视化链接关系
- CRUD 操作
- 链接类型管理

#### 3.2 批量操作（3 小时）
- 多选文件
- 批量标签/分类编辑
- 批量删除
- 进度跟踪

#### 3.3 导出功能（2 小时）
- 导出为 JSON/CSV
- 选择导出字段
- 过滤条件

#### 3.4 设置页面（2 小时）
- 配置管理
- 主题切换
- 快捷键设置

## 技术考虑

### 状态管理
- 使用 Dioxus signals 管理组件状态
- AppState 仅存储共享数据
- 深层组件使用 use_context

### 错误处理
- 统一错误类型
- 用户友好的错误消息
- 优雅降级

### 性能优化
- 延迟加载文件列表
- 搜索输入防抖
- 大列表虚拟滚动

### 可访问性
- 键盘导航支持
- ARIA 标签
- 焦点管理

### 测试策略
- 每个组件的单元测试
- 工作流集成测试
- Mock 服务调用
- UI 更新验证

## 下一步行动

建议从第一阶段开始，因为：
1. 这些是用户最需要的基础功能
2. 实现相对简单，可快速交付价值
3. 为后续功能打下基础

第一阶段预计 1 天完成，可以立即开始实施。

## 时间估算汇总

- **第一阶段**：6.5 小时（约 1 工作日）
- **第二阶段**：11 小时（约 1.5 工作日）
- **第三阶段**：11 小时（约 1.5 工作日）
- **总计**：28.5 小时（约 4 工作日）

## 风险与挑战

1. **Freya 框架限制**：某些 UI 功能可能受框架限制
2. **性能问题**：大量文件时的渲染性能
3. **跨平台兼容**：不同操作系统的 UI 差异

## 成功标准

1. 所有基础 CRUD 操作正常工作
2. UI 响应流畅，无明显卡顿
3. 错误处理完善，用户体验良好
4. 代码测试覆盖率 > 80%