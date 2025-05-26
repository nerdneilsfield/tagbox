use std::path::PathBuf;
use thiserror::Error;

/// TagBox 核心库的统一错误类型
#[derive(Error, Debug)]
pub enum TagboxError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("I/O错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML解析错误: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("文件未找到: {path}")]
    FileNotFound { path: PathBuf },

    #[error("重复的文件哈希: {hash}")]
    DuplicateHash { hash: String },

    #[error("无效查询语法: {query}")]
    InvalidQuery { query: String },

    #[error("元信息提取失败: {0}")]
    MetaInfoExtraction(String),
    
    #[error("无效的文件ID: {0}")]
    InvalidFileId(String),
    
    #[error("路径生成错误: {0}")]
    PathGeneration(String),
    
    #[error("未找到关联: 文件 {file_id_a} 和 {file_id_b}")]
    LinkNotFound { file_id_a: String, file_id_b: String },
}

pub type Result<T> = std::result::Result<T, TagboxError>;