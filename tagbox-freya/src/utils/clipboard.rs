use anyhow::Result;
use arboard::Clipboard;

/// 复制文本到剪贴板
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    tracing::info!("Copied to clipboard: {}", text);
    Ok(())
}

/// 从剪贴板获取文本
pub fn get_clipboard_text() -> Result<String> {
    let mut clipboard = Clipboard::new()?;
    let text = clipboard.get_text()?;
    Ok(text)
}