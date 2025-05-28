use clap::Parser;
use lopdf::Document;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tagbox_core::config::AppConfig;
use tagbox_core::metainfo::MetaInfoExtractor;
use tagbox_core::types::ImportMetadata;

#[derive(Parser)]
#[command(name = "show-pdf-metainfo")]
#[command(about = "提取并显示 PDF 文件的元信息")]
struct Args {
    /// PDF 文件路径
    #[arg(short, long)]
    file: PathBuf,

    /// 仅显示直接提取结果
    #[arg(long)]
    direct_only: bool,

    /// 仅显示通过 MetaInfoExtractor 提取的结果
    #[arg(long)]
    extractor_only: bool,

    /// 以 JSON 格式输出
    #[arg(long)]
    json: bool,

    /// 显示详细日志信息
    #[arg(short, long)]
    verbose: bool,
}

fn snip_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        text[..max_length].to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 根据 verbose 选项设置日志级别
    if args.verbose {
        // 显示所有日志
        tracing_subscriber::fmt().init();
    } else {
        // 只显示错误日志，过滤掉 lopdf 的噪音
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .init();
    }

    if !args.file.exists() {
        eprintln!("错误: 文件不存在: {}", args.file.display());
        std::process::exit(1);
    }

    if !args
        .file
        .extension()
        .map_or(false, |ext| ext.to_ascii_lowercase() == "pdf")
    {
        eprintln!("错误: 文件不是 PDF 格式");
        std::process::exit(1);
    }

    println!("正在分析 PDF 文件: {}", args.file.display());
    println!("文件大小: {} 字节", fs::metadata(&args.file)?.len());
    println!("{}", "=".repeat(80));

    if !args.extractor_only {
        // 方式1: 直接提取（模仿 metainfo.rs 的逻辑）
        println!("\n📄 方式1: 直接提取 PDF 元信息");
        println!("{}", "-".repeat(50));

        match extract_pdf_direct(&args.file).await {
            Ok(metadata) => {
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&metadata)?);
                } else {
                    print_metadata_formatted(&metadata, "直接提取");
                }
            }
            Err(e) => {
                eprintln!("直接提取失败: {}", e);
            }
        }
    }

    if !args.direct_only {
        // 方式2: 使用 MetaInfoExtractor
        println!("\n🔧 方式2: 使用 MetaInfoExtractor 提取");
        println!("{}", "-".repeat(50));

        let extractor = MetaInfoExtractor::new(AppConfig::default());
        match extractor.extract(&args.file).await {
            Ok(metadata) => {
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&metadata)?);
                } else {
                    print_metadata_formatted(&metadata, "MetaInfoExtractor");
                }
            }
            Err(e) => {
                eprintln!("MetaInfoExtractor 提取失败: {}", e);
            }
        }
    }

    Ok(())
}

/// 直接提取 PDF 元信息（复制 metainfo.rs 的逻辑）
async fn extract_pdf_direct(
    file_path: &Path,
) -> Result<ImportMetadata, Box<dyn std::error::Error>> {
    let mut meta = ImportMetadata {
        title: String::new(),
        authors: Vec::new(),
        year: None,
        publisher: None,
        source: None,
        category1: "未分类".to_string(),
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
            println!("✅ 成功打开 PDF 文档");

            // 提取PDF基本信息
            let pages = doc.get_pages();
            let page_count = pages.len();
            println!("📖 页数: {}", page_count);

            // 提取PDF元数据
            if let Ok(info_dict) = doc.trailer.get(b"Info") {
                if let Ok(info_ref) = info_dict.as_reference() {
                    if let Ok(info_obj) = doc.get_object(info_ref) {
                        if let Ok(info_dict) = info_obj.as_dict() {
                            println!("✅ 找到 PDF Info 字典，字段数: {}", info_dict.len());

                            // 提取标题
                            if let Ok(title_obj) = info_dict.get(b"Title") {
                                if let Ok(title) = extract_pdf_string_value(title_obj) {
                                    if !title.is_empty() {
                                        meta.title = title;
                                        println!("📌 标题: {}", meta.title);
                                    }
                                }
                            }

                            // 提取作者
                            if let Ok(author_obj) = info_dict.get(b"Author") {
                                if let Ok(author) = extract_pdf_string_value(author_obj) {
                                    if !author.is_empty() {
                                        meta.authors = vec![author];
                                        println!("👤 作者: {}", meta.authors.join(", "));
                                    }
                                }
                            }

                            // 提取主题/摘要
                            if let Ok(subject_obj) = info_dict.get(b"Subject") {
                                if let Ok(subject) = extract_pdf_string_value(subject_obj) {
                                    if !subject.is_empty() {
                                        meta.summary = Some(subject);
                                        println!("📝 主题: {}", meta.summary.as_ref().unwrap());
                                    }
                                }
                            }

                            // 提取关键词作为标签
                            if let Ok(keywords_obj) = info_dict.get(b"Keywords") {
                                if let Ok(keywords) = extract_pdf_string_value(keywords_obj) {
                                    if !keywords.is_empty() {
                                        meta.tags = keywords
                                            .split(&[',', ';', ' '][..])
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        println!("🏷️  关键词: {}", meta.tags.join(", "));
                                    }
                                }
                            }

                            // 提取创建者应用
                            if let Ok(creator_obj) = info_dict.get(b"Creator") {
                                if let Ok(creator) = extract_pdf_string_value(creator_obj) {
                                    if !creator.is_empty() {
                                        meta.additional_info
                                            .insert("creator".to_string(), creator.clone());
                                        println!("🛠️  创建者: {}", creator);
                                    }
                                }
                            }

                            // 提取生产者应用
                            if let Ok(producer_obj) = info_dict.get(b"Producer") {
                                if let Ok(producer) = extract_pdf_string_value(producer_obj) {
                                    if !producer.is_empty() {
                                        meta.additional_info
                                            .insert("producer".to_string(), producer.clone());
                                        println!("⚙️  生产者: {}", producer);
                                    }
                                }
                            }

                            // 提取创建时间
                            if let Ok(creation_date_obj) = info_dict.get(b"CreationDate") {
                                if let Ok(creation_date) =
                                    extract_pdf_string_value(creation_date_obj)
                                {
                                    if let Some(year) = parse_pdf_date_year(&creation_date) {
                                        meta.year = Some(year);
                                    }
                                    meta.additional_info
                                        .insert("creation_date".to_string(), creation_date.clone());
                                    println!("📅 创建时间: {}", creation_date);
                                }
                            }

                            // 提取修改时间
                            if let Ok(mod_date_obj) = info_dict.get(b"ModDate") {
                                if let Ok(mod_date) = extract_pdf_string_value(mod_date_obj) {
                                    meta.additional_info
                                        .insert("modification_date".to_string(), mod_date.clone());
                                    println!("📅 修改时间: {}", mod_date);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("⚠️  PDF 中没有 Info 字典");
            }

            // 尝试提取文本内容
            let mut extracted_text = String::new();
            let max_pages = std::cmp::min(page_count, 3);
            println!("📄 正在提取前 {} 页的文本内容...", max_pages);

            for i in 1..=max_pages {
                if let Ok(page_text) = doc.extract_text(&[i as u32]) {
                    println!("   第 {} 页: {} 字符", i, page_text.len());
                    extracted_text.push_str(&page_text);
                    extracted_text.push('\n');
                }
            }

            // 如果没有提取到标题，尝试从文本内容中智能提取
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
                        println!("📌 智能提取标题: {}", meta.title);
                        break;
                    }
                }

                // 如果还是没找到，使用第一行非空行
                if meta.title.is_empty() {
                    let first_line = extracted_text.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() && first_line.len() < 200 {
                        meta.title = first_line.to_string();
                        println!("📌 使用第一行作为标题: {}", meta.title);
                    }
                }
            }

            // 检查文件大小
            let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

            // 构建文件特定元数据
            let text_preview = if extracted_text.len() > 200 {
                format!("{}...", &extracted_text[..200])
            } else {
                extracted_text.clone()
            };

            let mut file_metadata = serde_json::json!({
                "pdf": {
                    "page_count": page_count,
                    "file_size": file_size,
                    "has_text": !extracted_text.trim().is_empty(),
                    "text_length": extracted_text.len(),
                    "text_preview": text_preview
                }
            });

            // 如果提取到完整文本，添加到文件元数据中
            if !extracted_text.trim().is_empty() {
                file_metadata["pdf"]["full_text"] = serde_json::json!(extracted_text);
            }

            meta.file_metadata = Some(file_metadata);

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

            meta.type_metadata = Some(type_metadata);

            println!("✅ 文本提取完成，总字符数: {}", extracted_text.len());
        }
        Err(e) => {
            println!("❌ 无法打开 PDF 文件: {:?}", e);

            // 尝试使用 pdf-extract 作为备用方案
            println!("🔄 尝试使用 pdf-extract 备用方案...");
            let mut fallback_successful = false;
            let mut extracted_text = String::new();

            if let Ok(bytes) = fs::read(file_path) {
                match pdf_extract::extract_text_from_mem(&bytes) {
                    Ok(text) => {
                        extracted_text = text;
                        fallback_successful = true;
                        println!(
                            "✅ pdf-extract 成功提取文本，长度: {}",
                            extracted_text.len()
                        );

                        // 尝试从文本内容中提取标题
                        if meta.title.is_empty() && !extracted_text.is_empty() {
                            let first_line = extracted_text.lines().next().unwrap_or("").trim();
                            if !first_line.is_empty() && first_line.len() < 200 {
                                meta.title = first_line.to_string();
                                println!("📌 从文本提取标题: {}", meta.title);
                            }
                        }
                    }
                    Err(extract_err) => {
                        println!("❌ pdf-extract 也无法提取文本: {:?}", extract_err);
                    }
                }
            }

            // 构建错误元数据
            let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            let mut file_metadata = serde_json::json!({
                "pdf": {
                    "error": format!("lopdf failed: {}", e),
                    "file_size": file_size,
                    "fallback_extraction": fallback_successful
                }
            });

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
            }

            meta.file_metadata = Some(file_metadata);
        }
    }

    Ok(meta)
}

/// 从PDF对象中提取字符串值
fn extract_pdf_string_value(obj: &lopdf::Object) -> Result<String, Box<dyn std::error::Error>> {
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
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Object::Name(name) => {
            String::from_utf8(name.clone()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        _ => Err("Not a string object".into()),
    }
}

/// 从PDF日期字符串中解析年份
fn parse_pdf_date_year(date_str: &str) -> Option<i32> {
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

/// 格式化打印元数据
fn print_metadata_formatted(metadata: &ImportMetadata, source: &str) {
    println!("\n📊 {} - 提取结果:", source);
    println!("{}", "─".repeat(40));

    if !metadata.title.is_empty() {
        println!("📌 标题: {}", metadata.title);
    }

    if !metadata.authors.is_empty() {
        println!("👤 作者: {}", metadata.authors.join(", "));
    }

    if let Some(year) = metadata.year {
        println!("📅 年份: {}", year);
    }

    if let Some(publisher) = &metadata.publisher {
        println!("🏢 出版商: {}", publisher);
    }

    if let Some(source) = &metadata.source {
        println!("🔗 来源: {}", source);
    }

    if !metadata.category1.is_empty() {
        println!("📂 分类1: {}", metadata.category1);
    }

    if let Some(category2) = &metadata.category2 {
        println!("📂 分类2: {}", category2);
    }

    if let Some(category3) = &metadata.category3 {
        println!("📂 分类3: {}", category3);
    }

    if !metadata.tags.is_empty() {
        println!("🏷️  标签: {}", metadata.tags.join(", "));
    }

    if let Some(summary) = &metadata.summary {
        println!("📝 摘要: {}", summary);
    }

    if !metadata.additional_info.is_empty() {
        println!("\n📋 附加信息:");
        for (key, value) in &metadata.additional_info {
            println!("   {}: {}", key, value);
        }
    }

    if let Some(file_metadata) = &metadata.file_metadata {
        println!("\n📄 文件元数据:");
        println!(
            "{}",
            serde_json::to_string_pretty(file_metadata).unwrap_or_default()
        );
    }

    if let Some(type_metadata) = &metadata.type_metadata {
        println!("\n🔧 类型元数据:");
        println!(
            "{}",
            serde_json::to_string_pretty(type_metadata).unwrap_or_default()
        );
    }
}
