use crate::errors::{Result, TagboxError};
use blake2::{Blake2b512, Digest as Blake2Digest};
use blake3;
use chrono::{DateTime, Utc};
use md5;
use sha2::{Sha256, Sha512};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use xxhash_rust::xxh3::{xxh3_128, xxh3_64};

/// 支持的哈希算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    /// MD5 - 最快但已被破解，仅用于非安全场景
    Md5,
    /// SHA-256 - 广泛使用的安全哈希
    Sha256,
    /// SHA-512 - 更强的SHA变体
    Sha512,
    /// Blake2b - 快速且安全
    Blake2b,
    /// Blake3 - 最新最快的安全哈希
    Blake3,
    /// XXHash3 64位 - 极快的非加密哈希
    XXH3_64,
    /// XXHash3 128位 - 极快的非加密哈希，更低碰撞率
    XXH3_128,
}

impl HashType {
    /// 从字符串解析哈希类型
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "md5" => Ok(HashType::Md5),
            "sha256" => Ok(HashType::Sha256),
            "sha512" => Ok(HashType::Sha512),
            "blake2b" => Ok(HashType::Blake2b),
            "blake3" => Ok(HashType::Blake3),
            "xxh3" | "xxh3_64" | "xxhash3" => Ok(HashType::XXH3_64),
            "xxh3_128" | "xxhash3_128" => Ok(HashType::XXH3_128),
            _ => Err(TagboxError::Config(format!("不支持的哈希算法: {}", s))),
        }
    }

    /// 获取算法描述
    pub fn description(&self) -> &'static str {
        match self {
            HashType::Md5 => "MD5 (快速但不安全)",
            HashType::Sha256 => "SHA-256 (标准安全哈希)",
            HashType::Sha512 => "SHA-512 (更强的安全哈希)",
            HashType::Blake2b => "Blake2b (快速安全哈希)",
            HashType::Blake3 => "Blake3 (最快的安全哈希)",
            HashType::XXH3_64 => "XXHash3-64 (极快非加密哈希)",
            HashType::XXH3_128 => "XXHash3-128 (极快非加密哈希，低碰撞)",
        }
    }

    /// 是否为加密安全的哈希
    pub fn is_cryptographic(&self) -> bool {
        match self {
            HashType::Md5 => false, // MD5已被破解
            HashType::Sha256 | HashType::Sha512 | HashType::Blake2b | HashType::Blake3 => true,
            HashType::XXH3_64 | HashType::XXH3_128 => false,
        }
    }
}

/// 计算文件哈希的通用函数
pub async fn calculate_file_hash(path: &Path) -> Result<String> {
    // 默认使用 Blake3
    calculate_file_hash_with_type(path, HashType::Blake3).await
}

/// 使用指定算法计算文件哈希
pub async fn calculate_file_hash_with_type(path: &Path, hash_type: HashType) -> Result<String> {
    // 对于大文件，使用流式读取以节省内存
    const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer

    let mut file = fs::File::open(path).map_err(TagboxError::Io)?;
    let metadata = file.metadata().map_err(TagboxError::Io)?;

    // 小文件直接读入内存
    if metadata.len() < 10 * 1024 * 1024 {
        // 10MB
        let mut content = Vec::new();
        file.read_to_end(&mut content).map_err(TagboxError::Io)?;
        return calculate_hash_from_bytes(&content, hash_type);
    }

    // 大文件使用流式处理
    let mut buffer = vec![0u8; BUFFER_SIZE];

    match hash_type {
        HashType::Md5 => {
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(TagboxError::Io)?;
            let digest = md5::compute(&content);
            Ok(format!("{:x}", digest))
        }
        HashType::Sha256 => {
            let mut hasher = Sha256::new();
            loop {
                let bytes_read = file.read(&mut buffer).map_err(TagboxError::Io)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashType::Sha512 => {
            let mut hasher = Sha512::new();
            loop {
                let bytes_read = file.read(&mut buffer).map_err(TagboxError::Io)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashType::Blake2b => {
            let mut hasher = Blake2b512::new();
            loop {
                let bytes_read = file.read(&mut buffer).map_err(TagboxError::Io)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashType::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let bytes_read = file.read(&mut buffer).map_err(TagboxError::Io)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(hasher.finalize().to_hex().to_string())
        }
        HashType::XXH3_64 => {
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(TagboxError::Io)?;
            let hash = xxh3_64(&content);
            Ok(format!("{:016x}", hash))
        }
        HashType::XXH3_128 => {
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(TagboxError::Io)?;
            let hash = xxh3_128(&content);
            Ok(format!("{:032x}", hash))
        }
    }
}

/// 从字节数组计算哈希
pub fn calculate_hash_from_bytes(data: &[u8], hash_type: HashType) -> Result<String> {
    match hash_type {
        HashType::Md5 => {
            let digest = md5::compute(data);
            Ok(format!("{:x}", digest))
        }
        HashType::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashType::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashType::Blake2b => {
            let mut hasher = Blake2b512::new();
            hasher.update(data);
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashType::Blake3 => {
            let hash = blake3::hash(data);
            Ok(hash.to_hex().to_string())
        }
        HashType::XXH3_64 => {
            let hash = xxh3_64(data);
            Ok(format!("{:016x}", hash))
        }
        HashType::XXH3_128 => {
            let hash = xxh3_128(data);
            Ok(format!("{:032x}", hash))
        }
    }
}

/// 计算文件的 Blake2b 哈希值（保留以兼容旧代码）
pub async fn calculate_file_blake2b(path: &Path) -> Result<String> {
    calculate_file_hash_with_type(path, HashType::Blake2b).await
}

/// 通用哈希计算函数（保留以兼容旧代码）
pub async fn calculate_hash(path: &Path, hash_type: HashType) -> Result<String> {
    calculate_file_hash_with_type(path, hash_type).await
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
        fs::create_dir_all(path).map_err(TagboxError::Io)?;
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
        fs::rename(dest, &backup_path).map_err(TagboxError::Io)?;
    }

    // 复制文件
    fs::copy(source, dest).map_err(TagboxError::Io)?;

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

/// 解析分类字符串，支持格式：
/// - "category1"            -> (category1, None, None)
/// - "category1/category2"  -> (category1, Some(category2), None)  
/// - "category1/category2/category3" -> (category1, Some(category2), Some(category3))
/// - "category1/category2/category3/ignored" -> (category1, Some(category2), Some(category3))
///
/// 分类名不能包含 '/' 字符
pub fn parse_category_string(
    category_str: &str,
) -> Result<(String, Option<String>, Option<String>)> {
    let category_str = category_str.trim();

    if category_str.is_empty() {
        return Err(TagboxError::Config("分类字符串不能为空".to_string()));
    }

    // 检查是否包含连续的斜杠或以斜杠开头/结尾
    if category_str.starts_with('/') || category_str.ends_with('/') || category_str.contains("//") {
        return Err(TagboxError::Config(
            "分类格式错误，不能以斜杠开头/结尾或包含连续斜杠".to_string(),
        ));
    }

    let parts: Vec<&str> = category_str.split('/').collect();

    // 检查每个部分是否为空
    for (i, part) in parts.iter().enumerate() {
        if part.trim().is_empty() {
            return Err(TagboxError::Config(format!("第{}级分类不能为空", i + 1)));
        }
    }

    match parts.len() {
        1 => Ok((parts[0].trim().to_string(), None, None)),
        2 => Ok((
            parts[0].trim().to_string(),
            Some(parts[1].trim().to_string()),
            None,
        )),
        3 => Ok((
            parts[0].trim().to_string(),
            Some(parts[1].trim().to_string()),
            Some(parts[2].trim().to_string()),
        )),
        n if n > 3 => {
            // 超过3级的情况，只取前3级，发出警告
            tracing::warn!("分类层级超过3级，只使用前3级: {}", category_str);
            Ok((
                parts[0].trim().to_string(),
                Some(parts[1].trim().to_string()),
                Some(parts[2].trim().to_string()),
            ))
        }
        _ => unreachable!("split应该至少返回1个元素"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_category_string() {
        // 单级分类
        assert_eq!(
            parse_category_string("Tech").unwrap(),
            ("Tech".to_string(), None, None)
        );

        // 二级分类
        assert_eq!(
            parse_category_string("Tech/Programming").unwrap(),
            ("Tech".to_string(), Some("Programming".to_string()), None)
        );

        // 三级分类
        assert_eq!(
            parse_category_string("Tech/Programming/Rust").unwrap(),
            (
                "Tech".to_string(),
                Some("Programming".to_string()),
                Some("Rust".to_string())
            )
        );

        // 超过三级（只取前三级）
        assert_eq!(
            parse_category_string("Tech/Programming/Rust/Web/Backend").unwrap(),
            (
                "Tech".to_string(),
                Some("Programming".to_string()),
                Some("Rust".to_string())
            )
        );

        // 带空格的分类名
        assert_eq!(
            parse_category_string(" Tech / Programming / Rust ").unwrap(),
            (
                "Tech".to_string(),
                Some("Programming".to_string()),
                Some("Rust".to_string())
            )
        );

        // 错误情况
        assert!(parse_category_string("").is_err());
        assert!(parse_category_string("/Tech").is_err());
        assert!(parse_category_string("Tech/").is_err());
        assert!(parse_category_string("Tech//Programming").is_err());
        assert!(parse_category_string("Tech/ /Programming").is_err());
    }
}
