//! 文件操作助手函数
//! 
//! 提供在 Freya 组件中使用的文件操作功能

use anyhow::Result;
use std::path::Path;
use crate::utils::system_open::{open_file, reveal_in_folder};

/// 在 Freya 组件中打开文件
/// 这个函数会自动在后台线程中运行，不会阻塞 UI
pub fn open_file_async(path: String) {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let path = Path::new(&path);
            match open_file(path).await {
                Ok(_) => tracing::info!("Opened file: {}", path.display()),
                Err(e) => {
                    tracing::error!("Failed to open file: {}", e);
                    // TODO: 通过消息传递显示错误
                }
            }
        });
    });
}

/// 在 Freya 组件中显示文件夹
pub fn reveal_in_folder_async(path: String) {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let path = Path::new(&path);
            match reveal_in_folder(path).await {
                Ok(_) => tracing::info!("Revealed in folder: {}", path.display()),
                Err(e) => {
                    tracing::error!("Failed to reveal in folder: {}", e);
                    // TODO: 通过消息传递显示错误
                }
            }
        });
    });
}

/// 使用协程处理文件操作的枚举
#[derive(Debug, Clone)]
pub enum FileOperation {
    Open(String),
    RevealInFolder(String),
}

/// 处理文件操作
pub async fn handle_file_operation(op: FileOperation) -> Result<()> {
    match op {
        FileOperation::Open(path) => {
            let path = Path::new(&path);
            open_file(path).await
        }
        FileOperation::RevealInFolder(path) => {
            let path = Path::new(&path);
            reveal_in_folder(path).await
        }
    }
}