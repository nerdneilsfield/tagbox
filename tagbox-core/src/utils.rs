use crate::errors::{Result, TagboxError};
use blake2::{Blake2b512, Digest as Blake2Digest};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// 计算文件的 SHA-256 哈希值
pub async fn calculate_file_hash(path: &Path) -> Result<String> {
    let file_content = fs::read(path).map_err(|e| TagboxError::Io(e))?;

    let mut hasher = Sha256::new();
    hasher.update(&file_content);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// 计算文件的 Blake2b 哈希值
pub async fn calculate_file_blake2b(path: &Path) -> Result<String> {
    let file_content = fs::read(path).map_err(|e| TagboxError::Io(e))?;

    let mut hasher = Blake2b512::new();
    hasher.update(&file_content);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// 通用哈希计算函数，支持多种哈希算法
pub async fn calculate_hash(path: &Path, hash_type: HashType) -> Result<String> {
    let file_content = fs::read(path).map_err(|e| TagboxError::Io(e))?;

    match hash_type {
        HashType::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(&file_content);
            let result = hasher.finalize();
            Ok(format!("{:x}", result))
        }
        HashType::Blake2b => {
            let mut hasher = Blake2b512::new();
            hasher.update(&file_content);
            let result = hasher.finalize();
            Ok(format!("{:x}", result))
        }
    }
}

/// 支持的哈希算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    Sha256,
    Blake2b,
}

/// 归一化路径（转换为绝对路径）
pub fn normalize_path(path: &Path) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path).map_err(|_| TagboxError::FileNotFound {
        path: path.to_path_buf(),
    })?;

    Ok(canonical)
}

/// 确保目录存在，如果不存在则创建
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| TagboxError::Io(e))?;
    } else if !path.is_dir() {
        return Err(TagboxError::Config(format!(
            "路径 {} 存在但不是目录",
            path.display()
        )));
    }

    Ok(())
}

/// 生成一个新的 UUID
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// 获取当前的 UTC 时间，格式化为 ISO 8601 字符串
pub fn current_time() -> DateTime<Utc> {
    Utc::now()
}

/// 格式化 DateTime 为数据库存储格式
pub fn format_datetime_for_db(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// 从数据库格式解析 DateTime
pub fn parse_datetime_from_db(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| TagboxError::Config(format!("无效的日期时间格式: {}", s)))
}

/// 安全地复制文件到目标位置
pub async fn safe_copy_file(source: &Path, dest: &Path) -> Result<()> {
    // 确保目标目录存在
    if let Some(parent) = dest.parent() {
        ensure_dir_exists(parent)?;
    }

    // 如果目标文件已存在，先进行备份
    if dest.exists() {
        let backup_path = generate_backup_path(dest)?;
        fs::rename(dest, &backup_path).map_err(|e| TagboxError::Io(e))?;
    }

    // 复制文件
    fs::copy(source, dest).map_err(|e| TagboxError::Io(e))?;

    Ok(())
}

/// 生成备份路径
fn generate_backup_path(path: &Path) -> Result<PathBuf> {
    let file_stem = path
        .file_stem()
        .ok_or_else(|| TagboxError::Config(format!("无效的文件路径: {}", path.display())))?
        .to_string_lossy();

    let extension = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();

    let timestamp = Utc::now().timestamp();
    let backup_filename = format!("{}.{}{}", file_stem, timestamp, extension);

    if let Some(parent) = path.parent() {
        Ok(parent.join(backup_filename))
    } else {
        Ok(PathBuf::from(backup_filename))
    }
}

/// Ensure an Option contains a value, otherwise return `TagboxError::MissingField`.
pub fn require_field<T>(opt: Option<T>, field: &str) -> Result<T> {
    opt.ok_or_else(|| TagboxError::MissingField {
        field: field.to_string(),
    })
}
