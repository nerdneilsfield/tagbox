use std::path::PathBuf;
use tagbox_core::types::{SearchOptions, SearchResult};

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
    SaveFile,
    DeleteFile,
    CancelEdit,
    
    // 分类树操作
    CategoryExpand(String),
    CategorySelect(String),
    
    // 系统事件
    LoadingStart,
    LoadingEnd,
    Error(String),
    
    // UI 事件
    WindowResize(i32, i32),
    RefreshView,
    OpenSettings,
    OpenLogViewer,
    ShowStatistics,
}