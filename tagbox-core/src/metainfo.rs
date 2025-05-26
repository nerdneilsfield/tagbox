use crate::config::AppConfig;
use crate::errors::{Result, TagboxError};
use crate::types::ImportMetadata;
use imageinfo;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// 元信息提取器
pub struct MetaInfoExtractor {
    config: AppConfig,
}

impl MetaInfoExtractor {
    /// 创建一个新的元信息提取器
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// 从文件中提取元数据
    pub async fn extract(&self, file_path: &Path) -> Result<ImportMetadata> {
        debug!("从文件提取元信息: {}", file_path.display());

        if !file_path.exists() {
            return Err(TagboxError::FileNotFound {
                path: file_path.to_path_buf(),
            });
        }

        // 首先尝试从文件名提取信息
        let mut metadata = self.extract_from_filename(file_path);

        // 然后尝试从同目录下的元数据JSON文件提取
        if self.config.import.metadata.prefer_json {
            if let Ok(json_metadata) = self.extract_from_json_file(file_path) {
                metadata = self.merge_metadata(metadata, json_metadata);
            }
        }

        if let Some(ext) = file_path.extension().and_then(OsStr::to_str) {
            let ext_lc = ext.to_ascii_lowercase();
            match ext_lc.as_str() {
                "pdf" if self.config.import.metadata.fallback_pdf => {
                    if let Ok(m) = self.extract_from_pdf(file_path) {
                        metadata = self.merge_metadata(metadata, m);
                    }
                }
                "epub" => {
                    if let Ok(m) = self.extract_from_epub(file_path) {
                        metadata = self.merge_metadata(metadata, m);
                    }
                }
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => {
                    if let Ok(m) = self.extract_from_image(file_path) {
                        metadata = self.merge_metadata(metadata, m);
                    }
                }
                _ => {}
            }
        }

        // 设置默认分类（如果没有指定）
        if metadata.category1.is_empty() {
            metadata.category1 = self.config.import.metadata.default_category.clone();
        }

        Ok(metadata)
    }

    /// 从文件名提取基础信息
    fn extract_from_filename(&self, path: &Path) -> ImportMetadata {
        let filename = path.file_stem().unwrap_or_default().to_string_lossy();

        // 基本的文件名解析逻辑，可以扩展为更复杂的规则
        // 示例: "Title - Author (2023).pdf" 格式解析
        let parts: Vec<&str> = filename.split(" - ").collect();

        let mut metadata = ImportMetadata {
            title: if !parts.is_empty() {
                parts[0].to_string()
            } else {
                filename.to_string()
            },
            authors: Vec::new(),
            year: None,
            publisher: None,
            source: None,
            category1: self.config.import.metadata.default_category.clone(),
            category2: None,
            category3: None,
            tags: Vec::new(),
            summary: None,
            additional_info: HashMap::new(),
            file_metadata: None,
            type_metadata: None,
        };

        if parts.len() > 1 {
            // 提取作者和年份
            let author_part = parts[1].trim();
            if let Some((author, year)) = self.parse_author_year(author_part) {
                metadata.authors = vec![author];
                metadata.year = year;
            } else {
                metadata.authors = vec![author_part.to_string()];
            }
        }

        metadata
    }

    /// 解析作者和年份，格式如 "Author Name (2023)"
    fn parse_author_year(&self, s: &str) -> Option<(String, Option<i32>)> {
        let parts: Vec<&str> = s.rsplitn(2, '(').collect();
        if parts.len() == 2 {
            let author = parts[1].trim();
            let year_part = parts[0].trim();
            if year_part.ends_with(')') {
                let year_str = year_part[..year_part.len() - 1].trim();
                if let Ok(year) = year_str.parse::<i32>() {
                    return Some((author.to_string(), Some(year)));
                }
            }
        }
        None
    }

    /// 从配套的JSON文件提取元信息
    fn extract_from_json_file(&self, file_path: &Path) -> Result<ImportMetadata> {
        // 构造同名但扩展名为json的文件路径
        let json_path = self.get_metadata_json_path(file_path)?;

        if json_path.exists() {
            let json_content = fs::read_to_string(&json_path).map_err(|e| TagboxError::Io(e))?;

            let json_value: Value =
                serde_json::from_str(&json_content).map_err(|e| TagboxError::Serialization(e))?;

            return self.parse_json_metadata(json_value);
        }

        Err(TagboxError::FileNotFound { path: json_path })
    }

    /// 获取与文件关联的元数据JSON文件路径
    fn get_metadata_json_path(&self, file_path: &Path) -> Result<PathBuf> {
        if let Some(stem) = file_path.file_stem() {
            let parent = file_path.parent().unwrap_or(Path::new("."));
            Ok(parent.join(format!("{}.meta.json", stem.to_string_lossy())))
        } else {
            Err(TagboxError::Config(format!(
                "无法获取文件名: {}",
                file_path.display()
            )))
        }
    }

    /// 解析JSON元数据
    fn parse_json_metadata(&self, json: Value) -> Result<ImportMetadata> {
        let mut metadata = ImportMetadata {
            title: json
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            authors: json
                .get("authors")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            year: json.get("year").and_then(|v| v.as_i64()).map(|y| y as i32),
            publisher: json
                .get("publisher")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            source: json
                .get("source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            category1: json
                .get("category1")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or(self.config.import.metadata.default_category.clone()),
            category2: json
                .get("category2")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            category3: json
                .get("category3")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tags: json
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            summary: json
                .get("summary")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            additional_info: HashMap::new(),
            file_metadata: None,
            type_metadata: None,
        };

        // 处理额外信息
        if let Some(obj) = json.as_object() {
            for (key, value) in obj.iter() {
                if ![
                    "title",
                    "authors",
                    "year",
                    "publisher",
                    "source",
                    "category1",
                    "category2",
                    "category3",
                    "tags",
                    "summary",
                ]
                .contains(&key.as_str())
                {
                    if let Some(value_str) = value.as_str() {
                        metadata
                            .additional_info
                            .insert(key.clone(), value_str.to_string());
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// 从PDF文件中提取元数据
    fn extract_from_pdf(&self, file_path: &Path) -> Result<ImportMetadata> {
        let mut meta = self.extract_from_filename(file_path);

        // TODO: 实现完整的PDF元数据提取
        // 当前pdf crate的API较复杂，暂时返回基本信息
        warn!("PDF元数据提取功能暂未完全实现");

        // 构建基本的文件元数据
        let file_metadata = serde_json::json!({
            "pdf": {
                "note": "Full PDF metadata extraction pending implementation"
            }
        });

        meta.file_metadata = Some(file_metadata);

        Ok(meta)
    }

    /// 从图片文件读取尺寸等信息
    fn extract_from_image(&self, file_path: &Path) -> Result<ImportMetadata> {
        let mut meta = self.extract_from_filename(file_path);

        match imageinfo::ImageInfo::from_file_path(file_path) {
            Ok(info) => {
                // 构建文件特定元数据
                let file_metadata = serde_json::json!({
                    "image": {
                        "width": info.size.width,
                        "height": info.size.height,
                        "format": format!("{:?}", info.format),
                        "mimetype": info.mimetype
                    }
                });

                meta.file_metadata = Some(file_metadata);

                // 在additional_info中也保留基本信息，方便查询
                meta.additional_info
                    .insert("width".into(), info.size.width.to_string());
                meta.additional_info
                    .insert("height".into(), info.size.height.to_string());
                meta.additional_info
                    .insert("format".into(), format!("{:?}", info.format));
            }
            Err(e) => {
                warn!("读取图片信息失败: {:?}", e);
            }
        }

        Ok(meta)
    }

    /// 从EPUB文件提取完整元数据
    fn extract_from_epub(&self, file_path: &Path) -> Result<ImportMetadata> {
        use epub::doc::EpubDoc;

        let mut meta = self.extract_from_filename(file_path);

        // 尝试打开EPUB文件
        match EpubDoc::new(file_path) {
            Ok(mut doc) => {
                // 提取基本元数据
                if let Some(title) = doc.mdata("title") {
                    meta.title = title;
                }

                // 提取作者
                if let Some(creator) = doc.mdata("creator") {
                    meta.authors = vec![creator];
                }

                // 提取出版商
                if let Some(publisher) = doc.mdata("publisher") {
                    meta.publisher = Some(publisher);
                }

                // 提取出版日期
                if let Some(date) = doc.mdata("date") {
                    // 尝试解析年份
                    if let Some(year_str) = date.split('-').next() {
                        if let Ok(year) = year_str.parse::<i32>() {
                            meta.year = Some(year);
                        }
                    }
                }

                // 提取语言
                if let Some(language) = doc.mdata("language") {
                    meta.additional_info
                        .insert("language".to_string(), language.clone());
                }

                // 提取描述/摘要
                if let Some(description) = doc.mdata("description") {
                    meta.summary = Some(description);
                }

                // 提取主题/标签
                if let Some(subject) = doc.mdata("subject") {
                    // 将主题转换为标签
                    let subjects: Vec<String> = subject
                        .split(';')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    meta.tags.extend(subjects);
                }

                // 提取标识符（ISBN等）
                if let Some(identifier) = doc.mdata("identifier") {
                    meta.additional_info
                        .insert("identifier".to_string(), identifier.clone());
                    // 如果是ISBN，特别标记
                    if identifier.contains("ISBN") || identifier.contains("isbn") {
                        meta.additional_info.insert("isbn".to_string(), identifier);
                    }
                }

                // 提取权利信息
                if let Some(rights) = doc.mdata("rights") {
                    meta.additional_info.insert("rights".to_string(), rights);
                }

                // 提取贡献者
                if let Some(contributor) = doc.mdata("contributor") {
                    meta.additional_info
                        .insert("contributor".to_string(), contributor);
                }

                // 构建文件特定元数据
                let has_cover = doc.get_cover().is_some();
                let mut file_metadata = serde_json::json!({
                    "epub": {
                        "spine_count": doc.get_num_pages(),
                        "has_cover": has_cover
                    }
                });

                // 如果能获取封面，保存封面信息
                if let Some((cover_data, _mime)) = doc.get_cover() {
                    file_metadata["epub"]["cover_size"] = serde_json::json!(cover_data.len());
                }

                meta.file_metadata = Some(file_metadata);

                // 构建类型特定元数据（书籍）
                let mut type_metadata = serde_json::json!({
                    "book": {}
                });

                // 添加ISBN到类型元数据
                if let Some(isbn) = meta.additional_info.get("isbn") {
                    type_metadata["book"]["isbn"] = serde_json::json!(isbn);
                }

                // 添加语言到类型元数据
                if let Some(language) = meta.additional_info.get("language") {
                    type_metadata["book"]["language"] = serde_json::json!(language);
                }

                meta.type_metadata = Some(type_metadata);
            }
            Err(e) => {
                warn!("无法打开EPUB文件 {}: {:?}", file_path.display(), e);
                // 返回基于文件名的基本元数据
            }
        }

        Ok(meta)
    }

    /// 合并两个元数据结构，优先使用第二个非空值
    fn merge_metadata(
        &self,
        base: ImportMetadata,
        override_data: ImportMetadata,
    ) -> ImportMetadata {
        ImportMetadata {
            title: if override_data.title.is_empty() {
                base.title
            } else {
                override_data.title
            },
            authors: if override_data.authors.is_empty() {
                base.authors
            } else {
                override_data.authors
            },
            year: override_data.year.or(base.year),
            publisher: override_data.publisher.or(base.publisher),
            source: override_data.source.or(base.source),
            category1: if override_data.category1.is_empty() {
                base.category1
            } else {
                override_data.category1
            },
            category2: if override_data.category2.is_none() {
                base.category2
            } else {
                override_data.category2
            },
            category3: if override_data.category3.is_none() {
                base.category3
            } else {
                override_data.category3
            },
            tags: {
                let mut merged_tags = base.tags;
                for tag in override_data.tags {
                    if !merged_tags.contains(&tag) {
                        merged_tags.push(tag);
                    }
                }
                merged_tags
            },
            summary: override_data.summary.or(base.summary),
            additional_info: {
                let mut merged_info = base.additional_info;
                merged_info.extend(override_data.additional_info);
                merged_info
            },
            file_metadata: override_data.file_metadata.or(base.file_metadata),
            type_metadata: override_data.type_metadata.or(base.type_metadata),
        }
    }
}
