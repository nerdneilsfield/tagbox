//! URL 文件下载工具

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// 从 URL 下载文件到临时目录
pub async fn download_file_from_url(url: &str) -> Result<PathBuf> {
    // 验证 URL
    let parsed_url = url::Url::parse(url)
        .context("Invalid URL")?;
    
    // 只支持 http 和 https
    match parsed_url.scheme() {
        "http" | "https" => {},
        _ => return Err(anyhow::anyhow!("Only HTTP/HTTPS URLs are supported")),
    }
    
    // 创建临时目录
    let temp_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("tagbox-downloads");
    
    tokio::fs::create_dir_all(&temp_dir).await
        .context("Failed to create download directory")?;
    
    // 从 URL 提取文件名，如果没有则使用时间戳
    let filename = parsed_url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("download");
    
    // 添加时间戳避免冲突
    let timestamp = chrono::Utc::now().timestamp();
    let final_filename = format!("{}_{}", timestamp, filename);
    let file_path = temp_dir.join(final_filename);
    
    tracing::info!("Downloading {} to {}", url, file_path.display());
    
    // 下载文件
    let response = reqwest::get(url).await
        .context("Failed to download file")?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
    }
    
    // 获取文件内容
    let content = response.bytes().await
        .context("Failed to read response body")?;
    
    // 写入文件
    let mut file = tokio::fs::File::create(&file_path).await
        .context("Failed to create file")?;
    
    file.write_all(&content).await
        .context("Failed to write file")?;
    
    file.flush().await
        .context("Failed to flush file")?;
    
    tracing::info!("Downloaded {} bytes to {}", content.len(), file_path.display());
    
    Ok(file_path)
}

/// 验证 URL 是否有效
pub fn validate_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url)
        .context("Invalid URL format")?;
    
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => Err(anyhow::anyhow!("Only HTTP/HTTPS URLs are supported")),
    }
}

/// 清理下载的临时文件
pub async fn cleanup_downloads() -> Result<()> {
    let temp_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("tagbox-downloads");
    
    if temp_dir.exists() {
        // 删除超过24小时的文件
        let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(24 * 60 * 60);
        
        let mut entries = tokio::fs::read_dir(&temp_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        let _ = tokio::fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com/file.pdf").is_ok());
        assert!(validate_url("http://example.com/file.pdf").is_ok());
        assert!(validate_url("ftp://example.com/file.pdf").is_err());
        assert!(validate_url("not a url").is_err());
    }
}