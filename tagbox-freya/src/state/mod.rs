use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::services::TagBoxService;
use crate::components::{ToastMessage, ToastType, create_toast};
use tagbox_core::types::{self, SearchResult};

#[derive(Clone)]
pub struct AppState {
    // 服务层
    pub service: Arc<TagBoxService>,
    
    // 搜索状态
    pub search_query: String,
    pub search_results: SearchResult,
    
    // UI 状态
    pub selected_file: Option<FileEntry>,
    pub categories: Vec<Category>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub toast_messages: Vec<ToastMessage>,
}

impl AppState {
    pub async fn new(config_path: Option<&str>) -> anyhow::Result<Self> {
        let service = Arc::new(TagBoxService::new(config_path).await?);
        
        // 加载初始数据
        let search_results = service.list_all(Some(50)).await.unwrap_or(SearchResult {
            entries: vec![],
            total_count: 0,
            offset: 0,
            limit: 50,
        });
        let categories = Self::build_category_tree(&search_results);
        
        Ok(Self {
            service,
            search_query: String::new(),
            search_results,
            selected_file: None,
            categories,
            is_loading: false,
            error_message: None,
            toast_messages: Vec::new(),
        })
    }
    
    /// 执行搜索
    pub async fn search(&mut self, query: &str) -> anyhow::Result<()> {
        self.search_query = query.to_string();
        self.is_loading = true;
        self.error_message = None;
        
        match self.service.search(query, None).await {
            Ok(results) => {
                self.search_results = results;
                self.categories = Self::build_category_tree(&self.search_results);
                self.is_loading = false;
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
                self.is_loading = false;
                Err(e)
            }
        }
    }
    
    /// 刷新文件列表
    pub async fn refresh_files(&mut self) -> anyhow::Result<()> {
        let query = self.search_query.clone();
        if query.is_empty() {
            self.search_results = self.service.list_all(Some(50)).await?;
        } else {
            self.search(&query).await?;
        }
        Ok(())
    }
    
    /// 显示成功通知
    pub fn show_success(&mut self, message: &str) {
        self.toast_messages.push(create_toast(ToastType::Success, message));
    }
    
    /// 显示错误通知
    pub fn show_error(&mut self, message: &str) {
        self.toast_messages.push(create_toast(ToastType::Error, message));
    }
    
    /// 显示信息通知
    pub fn show_info(&mut self, message: &str) {
        self.toast_messages.push(create_toast(ToastType::Info, message));
    }
    
    /// 显示警告通知
    pub fn show_warning(&mut self, message: &str) {
        self.toast_messages.push(create_toast(ToastType::Warning, message));
    }
    
    /// 从搜索结果构建分类树
    fn build_category_tree(_results: &SearchResult) -> Vec<Category> {
        // TODO: 实现真实的分类树构建逻辑
        // 暂时返回示例分类
        vec![
            Category {
                id: "tech".to_string(),
                name: "技术".to_string(),
                level: 1,
                parent_id: None,
                children: vec![
                    Category {
                        id: "programming".to_string(),
                        name: "编程".to_string(),
                        level: 2,
                        parent_id: Some("tech".to_string()),
                        children: vec![],
                        files: vec![],
                    }
                ],
                files: vec![],
            },
            Category {
                id: "literature".to_string(),
                name: "文学".to_string(),
                level: 1,
                parent_id: None,
                children: vec![],
                files: vec![],
            }
        ]
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            service: Arc::new(TagBoxService::new(None).now_or_never().unwrap().unwrap()),
            search_query: String::new(),
            search_results: SearchResult {
                entries: vec![],
                total_count: 0,
                offset: 0,
                limit: 50,
            },
            selected_file: None,
            categories: vec![],
            is_loading: false,
            error_message: None,
            toast_messages: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub title: String,
    pub path: String,
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub authors: Vec<String>,
    pub category: Option<CategoryPath>,
    pub size: u64,
    pub modified_at: String,
    pub imported_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CategoryPath {
    pub level1: String,
    pub level2: Option<String>,
    pub level3: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub level: u8,
    pub parent_id: Option<String>,
    pub children: Vec<Category>,
    pub files: Vec<FileEntry>,
}

impl From<types::FileEntry> for FileEntry {
    fn from(entry: types::FileEntry) -> Self {
        Self {
            id: entry.id.to_string(),
            title: entry.title,
            path: entry.path.to_string_lossy().to_string(),
            tags: entry.tags,
            summary: entry.summary,
            authors: entry.authors,
            category: None, // TODO: 从分类字段解析
            size: 0, // TODO: 从元数据获取
            modified_at: entry.updated_at.to_string(),
            imported_at: entry.created_at.to_string(),
        }
    }
}

use std::future::Future;
use std::task::{Context, Poll};

/// 扩展 trait 用于同步执行异步代码
trait FutureExt: Future {
    fn now_or_never(self) -> Option<Self::Output>
    where
        Self: Sized,
    {
        use std::task::Waker;
        use std::sync::Arc;
        use std::task::Wake;
        
        struct NoopWaker;
        impl Wake for NoopWaker {
            fn wake(self: Arc<Self>) {}
        }
        
        let waker = Waker::from(Arc::new(NoopWaker));
        let mut context = Context::from_waker(&waker);
        let mut future = Box::pin(self);
        
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => Some(output),
            Poll::Pending => None,
        }
    }
}

impl<F: Future> FutureExt for F {}