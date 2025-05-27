use crate::errors::{Result, TagboxError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 应用配置总结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub import: ImportConfig,
    pub search: SearchConfig,
    pub database: DatabaseConfig,
    pub hash: HashConfig,
}

/// 导入相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    pub paths: ImportPathsConfig,
    pub metadata: ImportMetadataConfig,
}

/// 导入路径配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPathsConfig {
    pub storage_dir: PathBuf,
    pub rename_template: String,
    pub classify_template: String,
}

/// 元数据提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadataConfig {
    pub prefer_json: bool,
    pub fallback_pdf: bool,
    pub default_category: String,
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_limit: usize,
    pub enable_fts: bool,
    pub fts_language: String,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub journal_mode: String,
    pub sync_mode: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HashConfig {
    /// 使用的哈希算法
    /// 可选值:
    /// - "md5": 最快但已被破解，仅用于非安全场景
    /// - "sha256": 广泛使用的安全哈希 (较慢)
    /// - "sha512": 更强的SHA变体 (较慢)
    /// - "blake2b": 快速且安全 (推荐)
    /// - "blake3": 最新最快的安全哈希 (推荐)
    /// - "xxh3_64": 极快的非加密哈希
    /// - "xxh3_128": 极快的非加密哈希，更低碰撞率
    #[serde(default = "default_hash_algorithm")]
    pub algorithm: String,

    /// 是否在导入时验证文件完整性
    #[serde(default = "default_verify_on_import")]
    pub verify_on_import: bool,
}

fn default_hash_algorithm() -> String {
    "blake3".to_string()
}

fn default_verify_on_import() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            import: ImportConfig {
                paths: ImportPathsConfig {
                    storage_dir: PathBuf::from("./tagbox_data/files"),
                    rename_template: "{title}_{authors}_{year}".to_string(),
                    classify_template: "{category1}/{filename}".to_string(),
                },
                metadata: ImportMetadataConfig {
                    prefer_json: true,
                    fallback_pdf: true,
                    default_category: "未分类".to_string(),
                },
            },
            search: SearchConfig {
                default_limit: 50,
                enable_fts: true,
                fts_language: "simple".to_string(),
            },
            database: DatabaseConfig {
                path: PathBuf::from("./tagbox_data/meta.db"),
                journal_mode: "WAL".to_string(),
                sync_mode: "NORMAL".to_string(),
            },
            hash: HashConfig {
                algorithm: "blake3".to_string(),
                verify_on_import: true,
            },
        }
    }
}

impl AppConfig {
    /// 从TOML文件加载配置
    pub async fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(TagboxError::Io)?;

        let config: AppConfig = toml::from_str(&content).map_err(TagboxError::TomlParse)?;

        config.validate()?;
        Ok(config)
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        // 检查路径模板是否包含必要的变量
        if !self.import.paths.rename_template.contains("{title}") {
            return Err(TagboxError::Config(
                "重命名模板必须包含 {title} 变量".to_string(),
            ));
        }

        // 检查分类模板是否包含文件名变量
        if !self.import.paths.classify_template.contains("{filename}") {
            return Err(TagboxError::Config(
                "分类模板必须包含 {filename} 变量".to_string(),
            ));
        }

        // 其他验证规则...

        Ok(())
    }
}
