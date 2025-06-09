//! 系统文件操作工具
//! 
//! 提供跨平台的文件和文件夹打开功能

use anyhow::Result;
use std::path::Path;

/// 使用系统默认程序打开文件
pub async fn open_file(path: &Path) -> Result<()> {
    // 确保文件存在
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }
    
    // 使用 open crate 打开文件
    open::that_detached(path)?;
    
    tracing::info!("Opened file: {}", path.display());
    Ok(())
}

/// 在文件管理器中显示文件（并选中它）
pub async fn reveal_in_folder(path: &Path) -> Result<()> {
    // 确保文件存在
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 explorer /select
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()?;
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 open -R
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试多种文件管理器
        let parent = path.parent().unwrap_or(path);
        
        // 尝试使用 xdg-open
        if let Ok(_) = std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
        {
            return Ok(());
        }
        
        // 备选方案
        let file_managers = ["nautilus", "dolphin", "thunar", "pcmanfm", "nemo"];
        for fm in &file_managers {
            if let Ok(_) = std::process::Command::new(fm)
                .arg(parent)
                .spawn()
            {
                return Ok(());
            }
        }
        
        // 如果都失败了，至少打开父文件夹
        open::that_detached(parent)?;
    }
    
    tracing::info!("Revealed in folder: {}", path.display());
    Ok(())
}

/// 打开文件夹
pub async fn open_folder(path: &Path) -> Result<()> {
    // 确保是文件夹
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Not a directory: {}", path.display()));
    }
    
    // 使用 open crate 打开文件夹
    open::that_detached(path)?;
    
    tracing::info!("Opened folder: {}", path.display());
    Ok(())
}

/// 获取文件的父文件夹路径
pub fn get_parent_folder(path: &Path) -> Option<&Path> {
    if path.is_file() {
        path.parent()
    } else {
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    
    #[tokio::test]
    async fn test_file_exists_check() {
        let result = open_file(Path::new("/nonexistent/file.txt")).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
    
    #[tokio::test]
    async fn test_folder_check() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        let result = open_folder(&file_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a directory"));
    }
    
    #[test]
    fn test_get_parent_folder() {
        let file_path = Path::new("/home/user/document.pdf");
        let parent = get_parent_folder(file_path);
        assert_eq!(parent, Some(Path::new("/home/user")));
        
        let folder_path = Path::new("/home/user/");
        let parent = get_parent_folder(folder_path);
        assert_eq!(parent, Some(folder_path));
    }
}