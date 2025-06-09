# TagBox Freya - 快速开始实现中级功能

## 立即可以开始的任务

### 1. 修复 FilePreview 中的文件操作按钮

```rust
// src/components/file_preview.rs
// 在 CustomButton 的 onpress 中调用：

// Open File 按钮
onpress: move |_| {
    let file_path = file_path_open.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::utils::system_open::open_file(&file_path).await {
            tracing::error!("Failed to open file: {}", e);
        }
    });
},

// CD 按钮 - 打开文件所在文件夹
onpress: move |_| {
    let file_path = file_path3.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::utils::system_open::reveal_in_folder(&file_path).await {
            tracing::error!("Failed to reveal in folder: {}", e);
        }
    });
},
```

### 2. 实现文件选择对话框

```rust
// src/components/drag_drop.rs
// 在 FileSelectButton 中：

onpress: move |_| {
    let onfile = onfile.clone();
    tokio::spawn(async move {
        if let Some(files) = rfd::AsyncFileDialog::new()
            .add_filter("Documents", &["pdf", "epub", "txt", "md", "djvu"])
            .add_filter("All files", &["*"])
            .set_directory(dirs::document_dir().unwrap_or_default())
            .pick_files()
            .await
        {
            for file in files {
                onfile.call(file.path().to_path_buf());
            }
        }
    });
},
```

### 3. 升级剪贴板功能使用 arboard

```rust
// src/utils/clipboard.rs
use arboard::Clipboard;
use anyhow::Result;

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    tracing::info!("Copied to clipboard: {}", text);
    Ok(())
}

pub fn get_clipboard_content() -> Result<String> {
    let mut clipboard = Clipboard::new()?;
    Ok(clipboard.get_text()?)
}
```

### 4. 完成 Import 页面的 URL 下载功能

```rust
// src/pages/import.rs
// 在 Download 按钮的 onpress 中：

onpress: move |_| {
    let url = download_url.read().clone();
    if !url.is_empty() {
        // TODO: 实现实际的下载逻辑
        // 1. 验证 URL
        // 2. 下载文件到临时目录
        // 3. 调用导入逻辑
        tracing::info!("Download from URL: {}", url);
    }
},
```

### 5. 实现编辑页面的保存功能

```rust
// src/pages/edit.rs
// 在 save_coroutine 中完成实际的保存逻辑：

let save_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<()>| async move {
    while let Some(_) = rx.next().await {
        is_saving.set(true);
        
        // 构建更新的元数据
        let metadata = ImportMetadata {
            title: Some(title.read().clone()),
            authors: authors.read().split(',').map(|s| s.trim().to_string()).collect(),
            tags: tags.read().split(',').map(|s| s.trim().to_string()).collect(),
            // ... 其他字段
        };
        
        // 调用 TagBoxService 更新
        match app_state.read().as_ref() {
            Some(state) => {
                match state.tagbox_service.update_file(&file_id, metadata).await {
                    Ok(_) => {
                        save_message.set(Some("保存成功！".to_string()));
                        // 3秒后清除消息
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        save_message.set(None);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("保存失败: {}", e)));
                    }
                }
            }
            None => {
                error_message.set(Some("应用状态未初始化".to_string()));
            }
        }
        
        is_saving.set(false);
    }
});
```

## 今日任务清单

- [ ] 1. 实现 system_open.rs 的三个函数 ✅
- [ ] 2. 修复 FilePreview 的按钮功能
- [ ] 3. 实现文件选择对话框
- [ ] 4. 升级剪贴板到 arboard
- [ ] 5. 完成编辑页面的保存功能
- [ ] 6. 测试跨平台兼容性

## 测试命令

```bash
# 下载依赖
cargo fetch

# 检查编译
cargo check

# 运行测试
cargo test --all

# 运行应用（需要图形环境）
cargo run --bin tagbox-freya
```

## 注意事项

1. **错误处理**：所有异步操作都要有适当的错误处理和用户反馈
2. **跨平台**：使用条件编译处理平台差异
3. **用户体验**：操作要有加载状态和成功/失败反馈
4. **性能**：文件操作放在 tokio::spawn 中避免阻塞 UI

完成这些任务后，TagBox Freya 将具备基本的文件管理功能，用户可以真正使用它来管理文件了！