use anyhow::{Context, Result};
use arboard::Clipboard;

/// 复制文本到剪贴板
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    
    clipboard.set_text(text)
        .context("Failed to set clipboard text")?;
    
    tracing::info!("Copied to clipboard: {}", text);
    Ok(())
}

/// 从剪贴板获取文本
pub fn get_clipboard_content() -> Result<String> {
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    
    let text = clipboard.get_text()
        .context("Failed to get clipboard text")?;
    
    Ok(text)
}

/// 检查剪贴板是否包含文本
pub fn has_clipboard_text() -> bool {
    match Clipboard::new() {
        Ok(mut clipboard) => clipboard.get_text().is_ok(),
        Err(_) => false,
    }
}

/// 清空剪贴板
pub fn clear_clipboard() -> Result<()> {
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    
    clipboard.clear()
        .context("Failed to clear clipboard")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_and_get() {
        let test_text = "Hello, TagBox!";
        
        // 复制到剪贴板
        assert!(copy_to_clipboard(test_text).is_ok());
        
        // 从剪贴板读取
        match get_clipboard_content() {
            Ok(content) => assert_eq!(content, test_text),
            Err(e) => {
                // 在 CI 环境中可能没有剪贴板访问权限
                eprintln!("Clipboard test skipped: {}", e);
            }
        }
    }

    #[test]
    fn test_has_clipboard_text() {
        // 这个测试可能在 CI 环境中失败
        if copy_to_clipboard("test").is_ok() {
            assert!(has_clipboard_text());
        }
    }
}