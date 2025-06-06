use std::collections::HashSet;
use tagbox_core::{config::AppConfig, types::FileEntry};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub current_files: Vec<FileEntry>,
    pub selected_file: Option<FileEntry>,
    pub current_query: String,
    pub category_tree: CategoryTreeState,
    pub is_loading: bool,
}

#[derive(Clone)]
pub struct CategoryTreeState {
    pub expanded_nodes: HashSet<String>,
    pub selected_category: Option<String>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            current_files: Vec::new(),
            selected_file: None,
            current_query: String::new(),
            category_tree: CategoryTreeState {
                expanded_nodes: HashSet::new(),
                selected_category: None,
            },
            is_loading: false,
        }
    }
    
    pub fn set_files(&mut self, files: Vec<FileEntry>) {
        self.current_files = files;
    }
    
    pub fn get_files(&self) -> &Vec<FileEntry> {
        &self.current_files
    }
    
    pub fn select_file(&mut self, file_id: &str) {
        self.selected_file = self.current_files
            .iter()
            .find(|f| f.id == file_id)
            .cloned();
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }
    
    pub fn expand_category(&mut self, category: String) {
        self.category_tree.expanded_nodes.insert(category);
    }
    
    pub fn collapse_category(&mut self, category: &str) {
        self.category_tree.expanded_nodes.remove(category);
    }
    
    pub fn select_category(&mut self, category: Option<String>) {
        self.category_tree.selected_category = category;
    }
}