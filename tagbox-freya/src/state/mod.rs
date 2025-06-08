use serde::{Deserialize, Serialize};
use tagbox_core::types;

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub files: Vec<FileEntry>,
    pub selected_file: Option<FileEntry>,
    pub categories: Vec<Category>,
    pub search_query: String,
    pub is_loading: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
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
            category: None, // TODO: 映射分类
            size: 0, // TODO: 从元数据获取
            modified_at: entry.updated_at.to_string(),
            imported_at: entry.created_at.to_string(),
        }
    }
}