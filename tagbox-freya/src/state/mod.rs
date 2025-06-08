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
    pub selected_category: Option<String>,  // 选中的分类ID
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
            selected_category: None,
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
    
    /// 获取当前显示的文件列表（根据选中的分类过滤）
    pub fn get_filtered_files(&self) -> Vec<FileEntry> {
        let all_files: Vec<FileEntry> = self.search_results.entries.iter()
            .map(|e| e.clone().into())
            .collect();
            
        if let Some(category_id) = &self.selected_category {
            // 根据分类ID过滤文件
            all_files.into_iter()
                .filter(|file| {
                    if let Some(cat) = &file.category {
                        // 检查是否匹配一级分类
                        if &cat.level1 == category_id {
                            return true;
                        }
                        // 检查是否匹配二级分类
                        if let Some(level2) = &cat.level2 {
                            let cat2_id = format!("{}/{}", cat.level1, level2);
                            if &cat2_id == category_id {
                                return true;
                            }
                            // 检查是否匹配三级分类
                            if let Some(level3) = &cat.level3 {
                                let cat3_id = format!("{}/{}/{}", cat.level1, level2, level3);
                                if &cat3_id == category_id {
                                    return true;
                                }
                            }
                        }
                        false
                    } else {
                        // 未分类文件
                        category_id == "uncategorized"
                    }
                })
                .collect()
        } else {
            all_files
        }
    }
    
    /// 从搜索结果构建分类树
    fn build_category_tree(results: &SearchResult) -> Vec<Category> {
        use std::collections::HashMap;
        
        // 收集所有分类路径
        let mut category_map: HashMap<String, Vec<FileEntry>> = HashMap::new();
        
        for entry in &results.entries {
            let file_entry: FileEntry = entry.clone().into();
            
            // 处理分类路径
            if let Some(cat) = &file_entry.category {
                // 一级分类
                let cat1_key = cat.level1.clone();
                category_map.entry(cat1_key.clone()).or_insert_with(Vec::new);
                
                // 二级分类
                if let Some(level2) = &cat.level2 {
                    let cat2_key = format!("{}/{}", cat1_key, level2);
                    category_map.entry(cat2_key.clone()).or_insert_with(Vec::new);
                    
                    // 三级分类
                    if let Some(level3) = &cat.level3 {
                        let cat3_key = format!("{}/{}/{}", cat1_key, level2, level3);
                        category_map.entry(cat3_key.clone()).or_insert_with(Vec::new).push(file_entry.clone());
                    } else {
                        category_map.get_mut(&cat2_key).unwrap().push(file_entry.clone());
                    }
                } else {
                    category_map.get_mut(&cat1_key).unwrap().push(file_entry.clone());
                }
            }
        }
        
        // 构建分类树
        let mut root_categories = Vec::new();
        let mut processed = std::collections::HashSet::new();
        
        // 先处理一级分类
        for (path, files) in &category_map {
            if !path.contains('/') && !processed.contains(path) {
                processed.insert(path.clone());
                
                let mut category = Category {
                    id: path.clone(),
                    name: path.clone(),
                    level: 1,
                    parent_id: None,
                    children: vec![],
                    files: files.clone(),
                };
                
                // 查找二级分类
                for (sub_path, sub_files) in &category_map {
                    if sub_path.starts_with(&format!("{}/", path)) && sub_path.matches('/').count() == 1 {
                        let parts: Vec<&str> = sub_path.split('/').collect();
                        if parts.len() >= 2 {
                            let mut sub_category = Category {
                                id: sub_path.clone(),
                                name: parts[1].to_string(),
                                level: 2,
                                parent_id: Some(path.clone()),
                                children: vec![],
                                files: sub_files.clone(),
                            };
                            
                            // 查找三级分类
                            for (sub_sub_path, sub_sub_files) in &category_map {
                                if sub_sub_path.starts_with(&format!("{}/", sub_path)) && sub_sub_path.matches('/').count() == 2 {
                                    let parts: Vec<&str> = sub_sub_path.split('/').collect();
                                    if parts.len() >= 3 {
                                        let sub_sub_category = Category {
                                            id: sub_sub_path.clone(),
                                            name: parts[2].to_string(),
                                            level: 3,
                                            parent_id: Some(sub_path.clone()),
                                            children: vec![],
                                            files: sub_sub_files.clone(),
                                        };
                                        sub_category.children.push(sub_sub_category);
                                    }
                                }
                            }
                            
                            category.children.push(sub_category);
                        }
                    }
                }
                
                root_categories.push(category);
            }
        }
        
        // 添加未分类的文件
        let uncategorized_files: Vec<FileEntry> = results.entries.iter()
            .filter_map(|e| {
                let file_entry: FileEntry = e.clone().into();
                if file_entry.category.is_none() {
                    Some(file_entry)
                } else {
                    None
                }
            })
            .collect();
            
        if !uncategorized_files.is_empty() {
            root_categories.push(Category {
                id: "uncategorized".to_string(),
                name: "未分类".to_string(),
                level: 1,
                parent_id: None,
                children: vec![],
                files: uncategorized_files,
            });
        }
        
        // 按名称排序
        root_categories.sort_by(|a, b| a.name.cmp(&b.name));
        
        root_categories
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
            selected_category: None,
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
        // 构建分类路径
        let category = if !entry.category1.is_empty() {
            Some(CategoryPath {
                level1: entry.category1,
                level2: entry.category2,
                level3: entry.category3,
            })
        } else {
            None
        };
        
        Self {
            id: entry.id.to_string(),
            title: entry.title,
            path: entry.path.to_string_lossy().to_string(),
            tags: entry.tags,
            summary: entry.summary,
            authors: entry.authors,
            category,
            size: 0, // TODO: 从元数据获取文件大小
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