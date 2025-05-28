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
            if let Some(year_str) = year_part.strip_suffix(')') {
                let year_str = year_str.trim();
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
            let json_content = fs::read_to_string(&json_path).map_err(TagboxError::Io)?;

            let json_value: Value =
                serde_json::from_str(&json_content).map_err(TagboxError::Serialization)?;

            return self.parse_json_metadata(json_value);
        }
        Err(TagboxError::FileNotFound { path: json_path })
    }

    /// 获取与文件关联的元数据JSON文件路径
    fn get_metadata_json_path(&self, file_path: &Path) -> Result<PathBuf> {
        if let Some(filename) = file_path.file_name() {
            let parent = file_path.parent().unwrap_or(Path::new("."));
            let filename_str = filename.to_string_lossy();

            // 首先尝试查找 .meta 文件（推荐格式）
            let meta_path = parent.join(format!("{}.meta", filename_str));
            if meta_path.exists() {
                return Ok(meta_path);
            }

            // 然后尝试查找 .meta.json 文件
            let meta_json_path = parent.join(format!("{}.meta.json", filename_str));
            if meta_json_path.exists() {
                return Ok(meta_json_path);
            }

            // 最后尝试查找 .json 文件（用于兼容测试和简单场景）
            let json_path = parent.join(format!("{}.json", filename_str));
            if json_path.exists() {
                return Ok(json_path);
            }

            // 如果都不存在，返回首选的 .meta 路径（让调用者处理文件不存在的情况）
            Ok(meta_path)
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
            type_metadata: Some(json.clone()),
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

    /// 从PDF文件中提取完整元数据
    fn extract_from_pdf(&self, file_path: &Path) -> Result<ImportMetadata> {
        use lopdf::Document;
        use std::fs;

        debug!("从PDF文件提取元信息: {}", file_path.display());

        // 先从文件名获取基础元数据
        let mut meta = ImportMetadata {
            title: String::new(),
            authors: Vec::new(),
            year: None,
            publisher: None,
            source: None,
            category1: String::new(),
            category2: None,
            category3: None,
            tags: Vec::new(),
            summary: None,
            additional_info: HashMap::new(),
            file_metadata: None,
            type_metadata: None,
        };

        // 尝试打开并解析PDF文件
        match Document::load(file_path) {
            Ok(doc) => {
                // 提取PDF基本信息
                let pages = doc.get_pages();
                let page_count = pages.len();

                // 提取PDF元数据
                debug!("检查PDF Info字典...");
                if let Ok(info_dict) = doc.trailer.get(b"Info") {
                    debug!("找到PDF Info字典");
                    if let Ok(info_ref) = info_dict.as_reference() {
                        if let Ok(info_obj) = doc.get_object(info_ref) {
                            if let Ok(info_dict) = info_obj.as_dict() {
                                debug!("成功解析PDF Info字典，字段数: {}", info_dict.len());
                                // 提取标题
                                if let Ok(title_obj) = info_dict.get(b"Title") {
                                    debug!("找到PDF标题字段");
                                    if let Ok(title) = self.extract_pdf_string_value(title_obj) {
                                        debug!("提取的PDF标题: '{}'", title);
                                        if !title.is_empty() {
                                            meta.title = title;
                                        }
                                    } else {
                                        debug!("无法解析PDF标题字符串");
                                    }
                                } else {
                                    debug!("PDF Info字典中没有Title字段");
                                }

                                // 提取作者
                                if let Ok(author_obj) = info_dict.get(b"Author") {
                                    if let Ok(author) = self.extract_pdf_string_value(author_obj) {
                                        if !author.is_empty() {
                                            meta.authors = vec![author];
                                        }
                                    }
                                }

                                // 提取主题/摘要
                                if let Ok(subject_obj) = info_dict.get(b"Subject") {
                                    if let Ok(subject) = self.extract_pdf_string_value(subject_obj)
                                    {
                                        if !subject.is_empty() {
                                            meta.summary = Some(subject);
                                        }
                                    }
                                }

                                // 提取关键词作为标签
                                if let Ok(keywords_obj) = info_dict.get(b"Keywords") {
                                    if let Ok(keywords) =
                                        self.extract_pdf_string_value(keywords_obj)
                                    {
                                        if !keywords.is_empty() {
                                            meta.tags = keywords
                                                .split(&[',', ';', ' '][..])
                                                .map(|s| s.trim().to_string())
                                                .filter(|s| !s.is_empty())
                                                .collect();
                                        }
                                    }
                                }

                                // 提取创建者应用
                                if let Ok(creator_obj) = info_dict.get(b"Creator") {
                                    if let Ok(creator) = self.extract_pdf_string_value(creator_obj)
                                    {
                                        if !creator.is_empty() {
                                            meta.additional_info
                                                .insert("creator".to_string(), creator);
                                        }
                                    }
                                }

                                // 提取生产者应用
                                if let Ok(producer_obj) = info_dict.get(b"Producer") {
                                    if let Ok(producer) =
                                        self.extract_pdf_string_value(producer_obj)
                                    {
                                        if !producer.is_empty() {
                                            meta.additional_info
                                                .insert("producer".to_string(), producer);
                                        }
                                    }
                                }

                                // 提取创建时间
                                if let Ok(creation_date_obj) = info_dict.get(b"CreationDate") {
                                    if let Ok(creation_date) =
                                        self.extract_pdf_string_value(creation_date_obj)
                                    {
                                        // 解析PDF日期格式: D:YYYYMMDDHHmmSSOHH'mm
                                        if let Some(year) = self.parse_pdf_date_year(&creation_date)
                                        {
                                            meta.year = Some(year);
                                        }
                                        meta.additional_info
                                            .insert("creation_date".to_string(), creation_date);
                                    }
                                }

                                // 提取修改时间
                                if let Ok(mod_date_obj) = info_dict.get(b"ModDate") {
                                    if let Ok(mod_date) =
                                        self.extract_pdf_string_value(mod_date_obj)
                                    {
                                        meta.additional_info
                                            .insert("modification_date".to_string(), mod_date);
                                    }
                                }
                            } else {
                                debug!("无法解析PDF Info字典");
                            }
                        } else {
                            debug!("无法获取PDF Info对象");
                        }
                    } else {
                        debug!("PDF Info不是引用类型");
                    }
                } else {
                    debug!("PDF中没有Info字典");
                }

                // 尝试提取文本内容用于全文搜索（前1000字符）
                let mut extracted_text = String::new();
                let mut text_preview = String::new();

                debug!("开始提取PDF文本内容...");
                // 限制处理的页数以避免性能问题
                let max_pages = std::cmp::min(page_count, 5);
                debug!("将提取前{}页的文本", max_pages);
                for i in 1..=max_pages {
                    if let Ok(page_text) = doc.extract_text(&[i as u32]) {
                        debug!("第{}页提取了{}字符", i, page_text.len());
                        extracted_text.push_str(&page_text);
                        extracted_text.push('\n');

                        // 只保留前1000字符作为预览
                        if text_preview.len() < 1000 {
                            let remaining = 1000 - text_preview.len();
                            let to_add = if page_text.len() > remaining {
                                &page_text[..remaining]
                            } else {
                                &page_text
                            };
                            text_preview.push_str(to_add);
                        }
                    } else {
                        debug!("第{}页文本提取失败", i);
                    }
                }
                debug!(
                    "PDF文本提取完成，总字符数: {}, 预览字符数: {}",
                    extracted_text.len(),
                    text_preview.len()
                );

                // 如果没有提取到标题，尝试从文本内容中智能提取
                if meta.title.is_empty() && !text_preview.is_empty() {
                    // 尝试找到真正的标题（通常在前几行，但不是版权声明）
                    let lines: Vec<&str> = text_preview.lines().take(15).collect();
                    for line in lines {
                        let line = line.trim();
                        // 跳过版权声明、许可证信息等，寻找真正的标题
                        if line.len() > 5 && line.len() < 100 &&
                           !line.to_lowercase().contains("copyright") &&
                           !line.to_lowercase().contains("permission") &&
                           !line.to_lowercase().contains("attribution") &&
                           !line.to_lowercase().contains("proper") &&
                           !line.to_lowercase().contains("google hereby") &&
                           !line.contains("@") && // 跳过邮箱地址行
                           !line.contains("arXiv") && // 跳过 arXiv 信息
                           !line.starts_with("http") && // 跳过链接
                           !line.chars().all(|c| c.is_ascii_uppercase() || c.is_whitespace() || c.is_ascii_punctuation())
                        {
                            // 跳过全大写标题
                            meta.title = line.to_string();
                            debug!("智能提取标题: {}", meta.title);
                            break;
                        }
                    }

                    // 如果还是没找到，使用第一行非空行
                    if meta.title.is_empty() {
                        let first_line = text_preview.lines().next().unwrap_or("").trim();
                        if !first_line.is_empty() && first_line.len() < 200 {
                            meta.title = first_line.to_string();
                            debug!("使用第一行作为标题: {}", meta.title);
                        }
                    }
                }

                // 检查文件大小
                let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

                // 构建文件特定元数据
                let mut file_metadata = serde_json::json!({
                    "pdf": {
                        "page_count": page_count,
                        "file_size": file_size,
                        "has_text": !extracted_text.trim().is_empty(),
                        "text_length": extracted_text.len(),
                        "text_preview": if text_preview.len() > 200 {
                            format!("{}...", &text_preview[..200])
                        } else {
                            text_preview.clone()
                        }
                    }
                });

                // 如果提取到完整文本，添加到文件元数据中
                if !extracted_text.trim().is_empty() {
                    file_metadata["pdf"]["full_text"] = serde_json::json!(extracted_text);
                }

                meta.file_metadata = Some(file_metadata.clone());

                // 构建类型特定元数据（包含创建者、生产者等信息）
                let mut type_metadata = serde_json::json!({
                    "document": {
                        "format": "pdf",
                        "pages": page_count,
                        "extractable": !extracted_text.trim().is_empty()
                    }
                });

                // 将创建者、生产者等信息移动到 type_metadata
                if let Some(creator) = meta.additional_info.get("creator") {
                    type_metadata["document"]["creator"] = serde_json::json!(creator);
                }
                if let Some(producer) = meta.additional_info.get("producer") {
                    type_metadata["document"]["producer"] = serde_json::json!(producer);
                }
                if let Some(creation_date) = meta.additional_info.get("creation_date") {
                    type_metadata["document"]["creation_date"] = serde_json::json!(creation_date);
                }
                if let Some(modification_date) = meta.additional_info.get("modification_date") {
                    type_metadata["document"]["modification_date"] =
                        serde_json::json!(modification_date);
                }

                meta.type_metadata = Some(type_metadata.clone());

                debug!(
                    "构建的file_metadata: {}",
                    serde_json::to_string_pretty(&file_metadata).unwrap_or_default()
                );
                debug!(
                    "构建的type_metadata: {}",
                    serde_json::to_string_pretty(&type_metadata).unwrap_or_default()
                );
                debug!("成功提取PDF元数据: {} 页, 标题: {}", page_count, meta.title);
            }
            Err(e) => {
                warn!("无法打开PDF文件 {}: {:?}", file_path.display(), e);

                // 尝试使用pdf-extract作为备用方案
                let mut fallback_successful = false;
                let mut extracted_text = String::new();

                if let Ok(bytes) = fs::read(file_path) {
                    match pdf_extract::extract_text_from_mem(&bytes) {
                        Ok(text) => {
                            extracted_text = text;
                            fallback_successful = true;
                            debug!(
                                "使用pdf-extract成功提取文本，长度: {}",
                                extracted_text.len()
                            );

                            // 尝试从文本内容中智能提取标题
                            if meta.title.is_empty() && !extracted_text.is_empty() {
                                // 尝试找到真正的标题（通常在前几行，但不是版权声明）
                                let lines: Vec<&str> = extracted_text.lines().take(15).collect();
                                for line in lines {
                                    let line = line.trim();
                                    // 跳过版权声明、许可证信息等，寻找真正的标题
                                    if line.len() > 5 && line.len() < 100 &&
                                       !line.to_lowercase().contains("copyright") &&
                                       !line.to_lowercase().contains("permission") &&
                                       !line.to_lowercase().contains("attribution") &&
                                       !line.to_lowercase().contains("proper") &&
                                       !line.to_lowercase().contains("google hereby") &&
                                       !line.contains("@") && // 跳过邮箱地址行
                                       !line.contains("arXiv") && // 跳过 arXiv 信息
                                       !line.starts_with("http") && // 跳过链接
                                       !line.chars().all(|c| c.is_ascii_uppercase() || c.is_whitespace() || c.is_ascii_punctuation())
                                    {
                                        // 跳过全大写标题
                                        meta.title = line.to_string();
                                        debug!("pdf-extract 智能提取标题: {}", meta.title);
                                        break;
                                    }
                                }

                                // 如果还是没找到，使用第一行非空行
                                if meta.title.is_empty() {
                                    let first_line =
                                        extracted_text.lines().next().unwrap_or("").trim();
                                    if !first_line.is_empty() && first_line.len() < 200 {
                                        meta.title = first_line.to_string();
                                        debug!("pdf-extract 使用第一行作为标题: {}", meta.title);
                                    }
                                }
                            }
                        }
                        Err(extract_err) => {
                            warn!("pdf-extract也无法提取文本: {:?}", extract_err);
                        }
                    }
                }

                // 检查文件大小和基本信息
                let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

                let mut file_metadata = serde_json::json!({
                    "pdf": {
                        "error": format!("lopdf failed: {}", e),
                        "file_size": file_size,
                        "fallback_extraction": fallback_successful
                    }
                });

                // 如果备用提取成功，添加文本信息
                if fallback_successful && !extracted_text.trim().is_empty() {
                    let text_preview = if extracted_text.len() > 200 {
                        format!("{}...", &extracted_text[..200])
                    } else {
                        extracted_text.clone()
                    };

                    file_metadata["pdf"]["has_text"] = serde_json::json!(true);
                    file_metadata["pdf"]["text_length"] = serde_json::json!(extracted_text.len());
                    file_metadata["pdf"]["text_preview"] = serde_json::json!(text_preview);
                    file_metadata["pdf"]["full_text"] = serde_json::json!(extracted_text);
                    file_metadata["pdf"]["extraction_method"] = serde_json::json!("pdf-extract");
                } else {
                    // 检查是否是加密或损坏的PDF
                    let error_msg = format!("{}", e).to_lowercase();
                    let is_encrypted =
                        error_msg.contains("encrypt") || error_msg.contains("password");
                    let is_corrupted = error_msg.contains("corrupt")
                        || error_msg.contains("invalid")
                        || error_msg.contains("malformed");

                    file_metadata["pdf"]["encrypted"] = serde_json::json!(is_encrypted);
                    file_metadata["pdf"]["corrupted"] = serde_json::json!(is_corrupted);
                    file_metadata["pdf"]["has_text"] = serde_json::json!(false);
                }

                meta.file_metadata = Some(file_metadata);
            }
        }

        Ok(meta)
    }

    /// 从PDF对象中提取字符串值
    fn extract_pdf_string_value(&self, obj: &lopdf::Object) -> Result<String> {
        use lopdf::Object;

        match obj {
            Object::String(bytes, _) => {
                // 尝试解码为UTF-8，如果失败则使用ISO-8859-1
                String::from_utf8(bytes.clone())
                    .or_else(|_| {
                        // 尝试ISO-8859-1解码
                        Ok::<String, std::string::FromUtf8Error>(
                            bytes.iter().map(|&b| b as char).collect(),
                        )
                    })
                    .map_err(|e| TagboxError::Config(format!("PDF string decode error: {}", e)))
            }
            Object::Name(name) => String::from_utf8(name.clone())
                .map_err(|e| TagboxError::Config(format!("PDF name decode error: {}", e))),
            _ => Err(TagboxError::Config("Not a string object".to_string())),
        }
    }

    /// 从PDF日期字符串中解析年份
    fn parse_pdf_date_year(&self, date_str: &str) -> Option<i32> {
        // PDF日期格式: D:YYYYMMDDHHmmSSOHH'mm 或简化形式
        if date_str.starts_with("D:") && date_str.len() >= 6 {
            let year_part = &date_str[2..6];
            year_part.parse::<i32>().ok()
        } else if date_str.len() >= 4 {
            // 尝试解析前4个字符作为年份
            date_str[..4].parse::<i32>().ok()
        } else {
            None
        }
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
