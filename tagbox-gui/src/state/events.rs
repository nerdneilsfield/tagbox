use std::path::PathBuf;
use tagbox_core::types::{SearchOptions, SearchResult, FileEntry};

#[derive(Debug, Clone)]
pub enum AppEvent {
    // 搜索相关
    SearchQuery(String),
    AdvancedSearch(SearchOptions),
    SearchResults(SearchResult),
    
    // 文件操作
    FileSelected(String), // file_id
    FileLoaded(FileEntry), // 文件详情加载完成
    FileImported(FileEntry), // 文件导入成功
    FileOpen(String),
    FileEdit(String),
    FileImport(PathBuf),
    SaveFile,
    DeleteFile(String), // 指定文件ID或索引
    CancelEdit,
    
    // 右键菜单相关事件
    OpenFile(String), // 打开文件
    EditFile(String), // 编辑文件元数据
    CopyFilePath(String), // 复制文件路径
    ShowInFolder(String), // 在文件夹中显示
    
    // 分类树操作
    CategoryExpand(String),
    CategorySelect(String),
    
    // 系统事件
    LoadingStart,
    LoadingEnd,
    Error(String),
    ConfigUpdated(PathBuf), // 配置文件已更新
    
    // UI 事件
    WindowResize(i32, i32),
    RefreshView,
    OpenSettings,
    OpenLogViewer,
    ShowStatistics,
    OpenAdvancedSearch,
    ShowAdvancedSearchDialog,
    OpenCategoryManager,
    CategoryCreated(String),
    CategoryUpdated(String),
    CategoryDeleted(String),
    
    // 键盘快捷键事件
    FocusSearchBar,
    EditSelectedFile,
    DeleteSelectedFile,
}