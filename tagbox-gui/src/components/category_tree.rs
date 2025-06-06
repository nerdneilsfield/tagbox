use fltk::{
    prelude::*,
    tree::Tree,
    enums::Color,
};
use std::sync::mpsc::Sender;
use std::collections::{HashMap, BTreeMap, BTreeSet};
use tagbox_core::config::AppConfig;
use crate::state::{AppEvent, CategoryTreeState};

// 分类层次结构管理器
#[derive(Debug, Clone)]
struct CategoryHierarchy {
    // 分类路径 -> 子分类集合
    categories: BTreeMap<String, BTreeSet<String>>,
}

impl CategoryHierarchy {
    fn new() -> Self {
        Self {
            categories: BTreeMap::new(),
        }
    }
    
    // 添加分类路径
    fn add_category(&mut self, path: &[String]) {
        if path.is_empty() {
            return;
        }
        
        // 添加根分类
        if path.len() == 1 {
            self.categories.entry("".to_string()).or_insert_with(BTreeSet::new).insert(path[0].clone());
            return;
        }
        
        // 构建完整路径并添加到对应的父分类
        for i in 1..path.len() {
            let parent_path = if i == 1 {
                path[0].clone()
            } else {
                path[..i].join("/")
            };
            
            let current_category = &path[i];
            self.categories.entry(parent_path).or_insert_with(BTreeSet::new).insert(current_category.clone());
        }
        
        // 确保所有父路径都存在
        for i in 1..=path.len() {
            let full_path = path[..i].join("/");
            self.categories.entry(full_path).or_insert_with(BTreeSet::new);
        }
    }
    
    // 获取根分类
    fn get_root_categories(&self) -> Vec<String> {
        self.categories.get("").map(|set| set.iter().cloned().collect()).unwrap_or_default()
    }
    
    // 获取指定分类的子分类
    fn get_subcategories(&self, parent_path: &str) -> Option<Vec<String>> {
        self.categories.get(parent_path).map(|set| set.iter().cloned().collect())
    }
    
    // 检查是否有子分类
    fn has_subcategories(&self, path: &str) -> bool {
        self.categories.get(path).map(|set| !set.is_empty()).unwrap_or(false)
    }
    
    // 统计分类数量
    fn count_categories(&self) -> usize {
        self.categories.values().map(|set| set.len()).sum()
    }
}

pub struct CategoryTree {
    tree: Tree,
    state: CategoryTreeState,
    event_sender: Sender<AppEvent>,
    file_counts: HashMap<String, i32>,
}

impl CategoryTree {
    pub fn new(
        x: i32,
        y: i32, 
        w: i32,
        h: i32,
        event_sender: Sender<AppEvent>
    ) -> Self {
        let mut tree = Tree::new(x, y, w, h, None);
        tree.set_show_root(false);
        tree.set_color(Color::White);
        tree.set_selection_color(Color::from_rgb(230, 240, 255));
        
        let mut category_tree = Self {
            tree,
            state: CategoryTreeState {
                expanded_nodes: std::collections::HashSet::new(),
                selected_category: None,
            },
            event_sender,
            file_counts: HashMap::new(),
        };
        
        category_tree.setup_callbacks();
        category_tree.load_default_categories();
        category_tree
    }
    
    pub async fn load_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 获取所有文件的分类信息
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        
        self.file_counts.clear();
        let mut category_structure = CategoryHierarchy::new();
        
        // 收集所有分类并构建层次结构
        for file in &search_result.entries {
            let cat1 = &file.category1;
            category_structure.add_category(&[cat1.clone()]);
            *self.file_counts.entry(cat1.clone()).or_insert(0) += 1;
            
            if let Some(cat2) = &file.category2 {
                category_structure.add_category(&[cat1.clone(), cat2.clone()]);
                let path2 = format!("{}/{}", cat1, cat2);
                *self.file_counts.entry(path2).or_insert(0) += 1;
                
                if let Some(cat3) = &file.category3 {
                    category_structure.add_category(&[cat1.clone(), cat2.clone(), cat3.clone()]);
                    let path3 = format!("{}/{}/{}", cat1, cat2, cat3);
                    *self.file_counts.entry(path3).or_insert(0) += 1;
                }
            }
        }
        
        // 重建树
        self.tree.clear();
        
        // 添加"全部文件"选项
        self.tree.add(&format!("📁 All Files ({})", search_result.entries.len()));
        
        // 构建分层的分类树
        self.build_tree_from_hierarchy(&category_structure, "");
        
        self.tree.redraw();
        println!("Loaded {} categories with {} total files", category_structure.count_categories(), search_result.entries.len());
        Ok(())
    }
    
    pub fn expand_category(&mut self, category_path: String) {
        self.state.expanded_nodes.insert(category_path);
    }
    
    pub fn collapse_category(&mut self, category_path: &str) {
        self.state.expanded_nodes.remove(category_path);
    }
    
    pub fn select_category(&mut self, category_path: Option<String>) {
        self.state.selected_category = category_path;
    }
    
    pub fn get_selected_category(&self) -> Option<String> {
        self.state.selected_category.clone()
    }
    
    pub fn get_file_count(&self, category_path: &str) -> i32 {
        self.file_counts.get(category_path).copied().unwrap_or(0)
    }
    
    pub fn refresh(&mut self) {
        self.tree.redraw();
    }
    
    // 获取分类的完整路径（用于过滤搜索）
    pub fn get_category_filter(&self) -> Option<String> {
        if let Some(selected) = &self.state.selected_category {
            Some(format!("category:{}", selected))
        } else {
            None
        }
    }
    
    // 获取树组件引用（用于主窗口布局）
    pub fn widget(&mut self) -> &mut Tree {
        &mut self.tree
    }
    
    // 设置回调函数
    fn setup_callbacks(&mut self) {
        let sender = self.event_sender.clone();
        
        self.tree.set_callback(move |tree| {
            if let Some(selected_items) = tree.get_selected_items() {
                if let Some(selected_item) = selected_items.first() {
                    let label = selected_item.label().unwrap_or_default();
                    
                    // 解析分类路径
                    let category_path = Self::parse_category_from_label(&label);
                    
                    if category_path == "All Files" {
                        // 显示所有文件
                        let _ = sender.send(AppEvent::SearchQuery("".to_string()));
                    } else if !category_path.is_empty() {
                        // 按分类筛选
                        let _ = sender.send(AppEvent::CategorySelect(category_path));
                    }
                }
            }
        });
    }
    
    // 从标签文本中解析分类路径
    fn parse_category_from_label(label: &str) -> String {
        // 移除表情符号和文件计数
        let clean_label = label
            .trim_start_matches("📁 ")
            .trim_start_matches("📄 ")
            .trim_start_matches("📂 ");
            
        if let Some(paren_pos) = clean_label.rfind(" (") {
            clean_label[..paren_pos].to_string()
        } else {
            clean_label.to_string()
        }
    }
    
    // 从分类层次结构构建树
    fn build_tree_from_hierarchy(&mut self, hierarchy: &CategoryHierarchy, parent_path: &str) {
        let categories = if parent_path.is_empty() {
            hierarchy.get_root_categories()
        } else {
            hierarchy.get_subcategories(parent_path).unwrap_or_default()
        };
        
        for category in categories {
            let full_path = if parent_path.is_empty() {
                category.clone()
            } else {
                format!("{}/{}", parent_path, category)
            };
            
            let count = self.file_counts.get(&full_path).unwrap_or(&0);
            let has_subcategories = hierarchy.has_subcategories(&full_path);
            
            let icon = if has_subcategories { "📂" } else { "📄" };
            let label = format!("{} {} ({})", icon, category, count);
            
            let tree_path = if parent_path.is_empty() {
                label
            } else {
                format!("{}/{}", parent_path, label)
            };
            
            self.tree.add(&tree_path);
            
            // 递归添加子分类
            if has_subcategories {
                self.build_tree_from_hierarchy(hierarchy, &full_path);
            }
        }
    }
    
    // 加载默认分类（在没有文件时显示）
    fn load_default_categories(&mut self) {
        self.tree.clear();
        self.tree.add("📁 All Files (0)");
        self.tree.add("📂 Documents (0)");
        self.tree.add("📂 Books (0)");
        self.tree.add("📂 Research (0)");
        self.tree.add("📂 Archive (0)");
        self.tree.add("📂 Uncategorized (0)");
        self.tree.redraw();
    }
    
    // 更新文件计数
    pub fn update_file_counts(&mut self, files: &[tagbox_core::types::FileEntry]) {
        self.file_counts.clear();
        
        // 重新计算每个分类的文件数量
        for file in files {
            let cat1 = &file.category1;
            *self.file_counts.entry(cat1.clone()).or_insert(0) += 1;
            
            if let Some(cat2) = &file.category2 {
                let path2 = format!("{}/{}", cat1, cat2);
                *self.file_counts.entry(path2).or_insert(0) += 1;
                
                if let Some(cat3) = &file.category3 {
                    let path3 = format!("{}/{}/{}", cat1, cat2, cat3);
                    *self.file_counts.entry(path3).or_insert(0) += 1;
                }
            }
        }
    }
}