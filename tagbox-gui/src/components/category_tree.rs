use fltk::{
    prelude::*,
    tree::Tree,
    enums::{Color, Event},
    app::MouseButton,
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
        let sender_menu = self.event_sender.clone();
        
        // 左键点击回调
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
        
        // 右键菜单处理
        self.tree.handle(move |tree, event| {
            match event {
                Event::Push => {
                    if fltk::app::event_mouse_button() == MouseButton::Right {
                        if let Some(selected_items) = tree.get_selected_items() {
                            if let Some(selected_item) = selected_items.first() {
                                let label = selected_item.label().unwrap_or_default();
                                let category_path = Self::parse_category_from_label(&label);
                                
                                // 显示分类右键菜单
                                Self::show_category_context_menu(&category_path, &sender_menu);
                            }
                        }
                        true
                    } else {
                        false
                    }
                },
                _ => false,
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
        
        // 刷新树显示以更新计数
        self.refresh_tree_display();
    }
    
    // 刷新树显示
    fn refresh_tree_display(&mut self) {
        self.tree.clear();
        
        // 添加"所有文件"根节点
        let total_files = self.file_counts.values().sum::<i32>().max(1); // 避免重复计数，取最大值
        self.tree.add(&format!("📁 All Files ({})", total_files));
        
        // 按分类添加节点，显示文件计数
        let mut categories: Vec<_> = self.file_counts.iter().collect();
        categories.sort_by_key(|(name, _)| name.as_str());
        
        for (category_name, count) in categories {
            // 只显示一级分类，避免重复
            if !category_name.contains('/') {
                let icon = match category_name.as_str() {
                    "Documents" => "📄",
                    "Books" => "📚", 
                    "Research" => "🔬",
                    "Archive" => "📦",
                    "Uncategorized" | "未分类" => "❓",
                    _ => "📂",
                };
                
                self.tree.add(&format!("{} {} ({})", icon, category_name, count));
            }
        }
        
        self.tree.redraw();
    }
    
    // 显示分类右键菜单
    fn show_category_context_menu(category_path: &str, sender: &Sender<AppEvent>) {
        use fltk::menu::*;
        
        let mut menu = MenuButton::default();
        menu.set_pos(fltk::app::event_x(), fltk::app::event_y());
        
        let sender_search = sender.clone();
        let sender_new = sender.clone();
        let sender_edit = sender.clone();
        let sender_delete = sender.clone();
        
        // 根据分类类型创建不同的菜单项
        if category_path == "All Files" {
            // 全部文件的菜单
            menu.add_choice("📁 Show All Files");
            menu.add_choice("➕ Create New Category");
            menu.add_choice("🔄 Refresh Categories");
        } else {
            // 特定分类的菜单
            menu.add_choice(&format!("📂 View Files in '{}'", category_path));
            menu.add_choice(&format!("➕ Add Subcategory to '{}'", category_path));
            menu.add_choice(&format!("✏️ Rename '{}'", category_path));
            menu.add_choice(&format!("🗑️ Delete '{}'", category_path));
            menu.add_choice("🔄 Refresh");
        }
        
        let choice = menu.popup().map(|item| item.value() as usize);
        let category_owned = category_path.to_string();
        
        if category_path == "All Files" {
            match choice {
                Some(0) => { // Show All Files
                    let _ = sender_search.send(AppEvent::SearchQuery("".to_string()));
                },
                Some(1) => { // Create New Category
                    Self::create_new_category(sender_new);
                },
                Some(2) => { // Refresh Categories
                    let _ = sender.send(AppEvent::RefreshView);
                },
                _ => {}
            }
        } else {
            match choice {
                Some(0) => { // View Files
                    let _ = sender_search.send(AppEvent::CategorySelect(category_owned));
                },
                Some(1) => { // Add Subcategory
                    Self::add_subcategory(&category_owned, sender_new);
                },
                Some(2) => { // Rename Category
                    Self::rename_category(&category_owned, sender_edit);
                },
                Some(3) => { // Delete Category
                    Self::delete_category(&category_owned, sender_delete);
                },
                Some(4) => { // Refresh
                    let _ = sender.send(AppEvent::RefreshView);
                },
                _ => {}
            }
        }
    }
    
    // 创建新分类
    fn create_new_category(sender: Sender<AppEvent>) {
        let category_name = fltk::dialog::input_default("Create New Category", "Category Name:");
        if let Some(name) = category_name {
            if !name.trim().is_empty() {
                let _ = sender.send(AppEvent::CategoryCreated(name.trim().to_string()));
            }
        }
    }
    
    // 添加子分类
    fn add_subcategory(parent_category: &str, sender: Sender<AppEvent>) {
        let subcategory_name = fltk::dialog::input_default(
            &format!("Add Subcategory to '{}'", parent_category), 
            "Subcategory Name:"
        );
        if let Some(name) = subcategory_name {
            if !name.trim().is_empty() {
                let full_path = format!("{}/{}", parent_category, name.trim());
                let _ = sender.send(AppEvent::CategoryCreated(full_path));
            }
        }
    }
    
    // 重命名分类
    fn rename_category(category_path: &str, sender: Sender<AppEvent>) {
        let new_name = fltk::dialog::input_default(
            &format!("Rename Category '{}'", category_path),
            "New Name:"
        );
        if let Some(name) = new_name {
            if !name.trim().is_empty() && name.trim() != category_path {
                let _ = sender.send(AppEvent::CategoryUpdated(format!("{}:{}", category_path, name.trim())));
            }
        }
    }
    
    // 删除分类
    fn delete_category(category_path: &str, sender: Sender<AppEvent>) {
        let choice = fltk::dialog::choice2_default(
            &format!("Are you sure you want to delete category '{}'?\nFiles in this category will be moved to 'Uncategorized'.", category_path),
            "Cancel",
            "Delete",
            ""
        );
        
        if choice == Some(1) {
            let _ = sender.send(AppEvent::CategoryDeleted(category_path.to_string()));
        }
    }
}