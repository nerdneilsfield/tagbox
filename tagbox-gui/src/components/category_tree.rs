use fltk::{
    prelude::*,
    tree::Tree,
    enums::Color,
};
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use tagbox_core::config::AppConfig;
use crate::state::{AppEvent, CategoryTreeState};

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
        
        Self {
            tree,
            state: CategoryTreeState {
                expanded_nodes: std::collections::HashSet::new(),
                selected_category: None,
            },
            event_sender,
            file_counts: HashMap::new(),
        }
    }
    
    pub async fn load_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 获取所有文件的分类信息
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        
        self.file_counts.clear();
        let mut categories = std::collections::HashSet::new();
        
        // 收集所有分类
        for file in &search_result.entries {
            let category1 = &file.category1;
            *self.file_counts.entry(category1.clone()).or_insert(0) += 1;
            categories.insert(category1.clone());
            
            if let Some(cat2) = &file.category2 {
                let path2 = format!("{}/{}", category1, cat2);
                *self.file_counts.entry(path2.clone()).or_insert(0) += 1;
                categories.insert(path2);
                
                if let Some(cat3) = &file.category3 {
                    let path3 = format!("{}/{}/{}", category1, cat2, cat3);
                    *self.file_counts.entry(path3.clone()).or_insert(0) += 1;
                    categories.insert(path3);
                }
            }
        }
        
        // 重建树
        self.tree.clear();
        
        // 添加分类到树（简化版本）
        let mut sorted_categories: Vec<_> = categories.into_iter().collect();
        sorted_categories.sort();
        
        for category in sorted_categories {
            let count = self.file_counts.get(&category).unwrap_or(&0);
            let label = format!("{} ({})", category, count);
            self.tree.add(&label);
        }
        
        self.tree.redraw();
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
}