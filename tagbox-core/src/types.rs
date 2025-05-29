use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 文件实体，表示数据库中的一个已索引文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub publisher: Option<String>,
    pub source: Option<String>,
    pub path: PathBuf,
    pub original_path: Option<PathBuf>,
    pub original_filename: String,
    pub hash: String,
    pub current_hash: Option<String>,
    pub category1: String,
    pub category2: Option<String>,
    pub category3: Option<String>,
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub full_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub file_metadata: Option<serde_json::Value>,
    pub type_metadata: Option<serde_json::Value>,
}

/// 导入文件时的元数据信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub publisher: Option<String>,
    pub source: Option<String>,
    pub category1: String,
    pub category2: Option<String>,
    pub category3: Option<String>,
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub full_text: Option<String>,
    pub additional_info: HashMap<String, String>,
    pub file_metadata: Option<serde_json::Value>,
    pub type_metadata: Option<serde_json::Value>,
}

/// 文件更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUpdateRequest {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub year: Option<i32>,
    pub publisher: Option<String>,
    pub source: Option<String>,
    pub category1: Option<String>,
    pub category2: Option<String>,
    pub category3: Option<String>,
    pub tags: Option<Vec<String>>,
    pub summary: Option<String>,
    pub full_text: Option<String>,
    pub is_deleted: Option<bool>,
    pub file_metadata: Option<serde_json::Value>,
    pub type_metadata: Option<serde_json::Value>,
}

/// 搜索选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub offset: usize,
    pub limit: usize,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub include_deleted: bool,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entries: Vec<FileEntry>,
    pub total_count: usize,
    pub offset: usize,
    pub limit: usize,
}

/// 作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// 文件关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationType {
    References,
    DerivedFrom,
    Relates,
    Depends,
    Custom(String),
}

impl From<Option<String>> for RelationType {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(s) => match s.as_str() {
                "references" => RelationType::References,
                "derived_from" => RelationType::DerivedFrom,
                "relates" => RelationType::Relates,
                "depends" => RelationType::Depends,
                custom => RelationType::Custom(custom.to_string()),
            },
            None => RelationType::Relates,
        }
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationType::References => write!(f, "references"),
            RelationType::DerivedFrom => write!(f, "derived_from"),
            RelationType::Relates => write!(f, "relates"),
            RelationType::Depends => write!(f, "depends"),
            RelationType::Custom(s) => write!(f, "{}", s),
        }
    }
}

// Added QueryParam enum for dynamic query argument binding
#[derive(Debug)] // Added Debug for convenience
pub enum QueryParam {
    String(String),
    Int(i64),
    // Add other types as needed, e.g., Bool(bool), Float(f64)
}

impl std::fmt::Display for QueryParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryParam::String(s) => write!(f, "{}", s),
            QueryParam::Int(i) => write!(f, "{}", i),
        }
    }
}
