use crate::config::AppConfig;
use crate::errors::{Result, TagboxError};
use crate::types::ImportMetadata;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use tracing::debug;

lazy_static! {
    static ref TEMPLATE_VAR_RE: Regex = Regex::new(r"\{([a-zA-Z0-9_]+)\}")
        .expect("failed to compile TEMPLATE_VAR_RE regex");
}

/// 路径生成器
pub struct PathGenerator {
    config: AppConfig,
}

impl PathGenerator {
    /// 创建一个新的路径生成器
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
    
    /// 生成目标文件名
    pub fn generate_filename(&self, original_filename: &str, metadata: &ImportMetadata) -> Result<String> {
        let extension = Path::new(original_filename)
            .extension()
            .unwrap_or_default()
            .to_string_lossy();
        
        let mut filename = self.apply_template(&self.config.import.paths.rename_template, metadata)?;
        
        // 替换非法字符
        filename = self.sanitize_filename(&filename);
        
        // 添加扩展名
        if !extension.is_empty() {
            filename = format!("{}.{}", filename, extension);
        }
        
        Ok(filename)
    }
    
    /// 生成目标文件完整路径
    pub fn generate_path(
        &self, 
        filename: &str, 
        metadata: &ImportMetadata
    ) -> Result<PathBuf> {
        // 解析分类模板
        let mut relative_path = self.apply_template(
            &self.config.import.paths.classify_template, 
            metadata
        )?;
        
        // 替换分类模板中的 {filename} 占位符
        relative_path = relative_path.replace("{filename}", filename);
        
        // 构建完整路径
        let full_path = self.config.import.paths.storage_dir.join(relative_path);
        
        debug!("生成的目标路径: {}", full_path.display());
        
        Ok(full_path)
    }
    
    /// 应用模板替换变量
    fn apply_template(&self, template: &str, metadata: &ImportMetadata) -> Result<String> {
        let mut vars = HashMap::new();
        
        // 基础变量
        vars.insert("title".to_string(), metadata.title.clone());
        vars.insert("authors".to_string(), self.format_authors(&metadata.authors));
        
        if let Some(year) = metadata.year {
            vars.insert("year".to_string(), year.to_string());
        } else {
            vars.insert("year".to_string(), "unknown".to_string());
        }
        
        if let Some(publisher) = &metadata.publisher {
            vars.insert("publisher".to_string(), publisher.clone());
        }
        
        vars.insert("category1".to_string(), metadata.category1.clone());
        
        if let Some(category2) = &metadata.category2 {
            vars.insert("category2".to_string(), category2.clone());
        }
        
        if let Some(category3) = &metadata.category3 {
            vars.insert("category3".to_string(), category3.clone());
        }
        
        // 应用模板
        let mut result = template.to_string();
        for cap in TEMPLATE_VAR_RE.captures_iter(template) {
            let var_name = &cap[1];
            let replacement = vars.get(var_name).cloned().unwrap_or_else(|| {
                if var_name == "filename" {
                    // 特殊处理 filename，在 generate_path 中替换
                    "{filename}".to_string()
                } else {
                    format!("{{{}}}", var_name)
                }
            });
            result = result.replace(&format!("{{{}}}", var_name), &replacement);
        }
        
        Ok(result)
    }
    
    /// 格式化作者列表为单个字符串
    fn format_authors(&self, authors: &[String]) -> String {
        if authors.is_empty() {
            return "unknown".to_string();
        }
        
        if authors.len() == 1 {
            return authors[0].clone();
        }
        
        if authors.len() == 2 {
            return format!("{}_and_{}", authors[0], authors[1]);
        }
        
        format!("{}_et_al", authors[0])
    }
    
    /// 清理文件名中的非法字符
    fn sanitize_filename(&self, name: &str) -> String {
        // 替换常见的非法文件名字符
        let mut result = name.replace(&['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>', '.', ';'], "_");
        
        // 修剪前后空白
        result = result.trim().to_string();
        
        // 如果为空，使用默认名称
        if result.is_empty() {
            return "unnamed_file".to_string();
        }
        
        result
    }
}