use fltk::{
    prelude::*,
    tree::{Tree, TreeItem, TreeReason},
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
    category_map: HashMap<String, Vec<String>>, // level1 -> [level2...]
    file_counts: HashMap<String, i32>, // category_path -> file_count
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
        
        // 设置树的回调
        let sender_clone = event_sender.clone();
        tree.set_callback(move |tree| {
            if let Some(item) = tree.get_selected_item() {
                if let Some(path) = item.label() {
                    let reason = tree.callback_reason();
                    match reason {
                        TreeReason::Selected => {
                            // 节点被选中
                            let _ = sender_clone.send(AppEvent::CategorySelect(path));
                        }
                        TreeReason::Opened => {
                            // 节点被展开
                            let _ = sender_clone.send(AppEvent::CategoryExpand(path));
                        }
                        _ => {}
                    }
                }
            }
        });
        
        Self {
            tree,
            state: CategoryTreeState {
                expanded_nodes: std::collections::HashSet::new(),
                selected_category: None,
            },
            event_sender,
            category_map: HashMap::new(),
            file_counts: HashMap::new(),
        }
    }
    
    pub async fn load_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 获取所有文件的分类信息
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        
        self.category_map.clear();
        self.file_counts.clear();
        
        // 构建分类层次结构
        for file in &search_result.entries {
            let category1 = &file.category1;
            let category2 = file.category2.as_deref();
            let category3 = file.category3.as_deref();
            
            // 统计一级分类文件数
            *self.file_counts.entry(category1.clone()).or_insert(0) += 1;
            
            if let Some(cat2) = category2 {
                let path2 = format!("{}/{}", category1, cat2);
                *self.file_counts.entry(path2.clone()).or_insert(0) += 1;
                
                // 添加到分类映射
                self.category_map
                    .entry(category1.clone())
                    .or_insert_with(Vec::new)
                    .push(cat2.to_string());
                
                if let Some(cat3) = category3 {
                    let path3 = format!("{}/{}/{}", category1, cat2, cat3);
                    *self.file_counts.entry(path3).or_insert(0) += 1;
                    
                    // 添加三级分类到映射
                    self.category_map
                        .entry(path2)
                        .or_insert_with(Vec::new)
                        .push(cat3.to_string());
                }
            }
        }
        
        // 去重并排序
        for categories in self.category_map.values_mut() {
            categories.sort();
            categories.dedup();
        }
        
        self.rebuild_tree();
        Ok(())
    }
    
    fn rebuild_tree(&mut self) {
        self.tree.clear();
        
        // 按字母顺序排序一级分类
        let mut level1_categories: Vec<_> = self.category_map.keys()
            .filter(|k| !k.contains('/'))
            .collect();
        level1_categories.sort();
        
        for category1 in level1_categories {
            let count1 = self.file_counts.get(category1).unwrap_or(&0);
            let label1 = format!("{} ({})", category1, count1);
            
            let item1 = self.tree.add(&label1);
            if let Some(mut item1) = item1 {
                item1.set_user_data(category1.clone());
                
                // 添加二级分类
                if let Some(level2_categories) = self.category_map.get(category1) {
                    for category2 in level2_categories {
                        let path2 = format!("{}/{}", category1, category2);
                        let count2 = self.file_counts.get(&path2).unwrap_or(&0);
                        let label2 = format!("{} ({})", category2, count2);
                        
                        let item2 = item1.add(&label2);
                        if let Some(mut item2) = item2 {
                            item2.set_user_data(path2.clone());
                            
                            // 添加三级分类
                            if let Some(level3_categories) = self.category_map.get(&path2) {
                                for category3 in level3_categories {
                                    let path3 = format!("{}/{}/{}", category1, category2, category3);
                                    let count3 = self.file_counts.get(&path3).unwrap_or(&0);
                                    let label3 = format!("{} ({})", category3, count3);
                                    
                                    let item3 = item2.add(&label3);
                                    if let Some(mut item3) = item3 {
                                        item3.set_user_data(path3);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 恢复展开状态
                if self.state.expanded_nodes.contains(category1) {
                    item1.open();
                }
            }
        }
        
        self.tree.redraw();
    }
    
    pub fn expand_category(&mut self, category_path: String) {
        self.state.expanded_nodes.insert(category_path.clone());
        
        // 查找并展开对应的树节点
        self.find_and_expand_item(&category_path);
    }
    
    pub fn collapse_category(&mut self, category_path: &str) {
        self.state.expanded_nodes.remove(category_path);
        
        // 查找并折叠对应的树节点
        self.find_and_collapse_item(category_path);
    }
    
    pub fn select_category(&mut self, category_path: Option<String>) {
        self.state.selected_category = category_path.clone();
        
        if let Some(path) = category_path {
            self.find_and_select_item(&path);
        } else {
            self.tree.deselect_all();
        }
    }
    
    fn find_and_expand_item(&mut self, category_path: &str) {
        if let Some(item) = self.find_item_by_path(category_path) {
            item.open();
            self.tree.redraw();
        }
    }
    
    fn find_and_collapse_item(&mut self, category_path: &str) {
        if let Some(item) = self.find_item_by_path(category_path) {
            item.close();
            self.tree.redraw();
        }
    }
    
    fn find_and_select_item(&mut self, category_path: &str) {
        if let Some(item) = self.find_item_by_path(category_path) {
            self.tree.select_item(Some(&item), true);
            self.tree.redraw();
        }
    }
    
    fn find_item_by_path(&self, category_path: &str) -> Option<TreeItem> {
        // 简化版本：遍历所有项目查找匹配的路径
        let root = self.tree.get_item("/");
        if let Some(root) = root {
            self.search_item_recursive(&root, category_path)
        } else {
            None
        }
    }
    
    fn search_item_recursive(&self, item: &TreeItem, target_path: &str) -> Option<TreeItem> {
        if let Some(user_data) = item.user_data() {
            if user_data == target_path {
                return Some(item.clone());
            }
        }
        
        // 递归搜索子项
        let mut child = item.first_child();
        while let Some(c) = child {
            if let Some(found) = self.search_item_recursive(&c, target_path) {
                return Some(found);
            }
            child = c.next_sibling();
        }
        
        None
    }
    
    pub fn get_selected_category(&self) -> Option<String> {
        self.state.selected_category.clone()
    }
    
    pub fn get_file_count(&self, category_path: &str) -> i32 {
        self.file_counts.get(category_path).copied().unwrap_or(0)
    }
    
    pub fn refresh(&mut self) {
        self.rebuild_tree();
    }
    
    // 获取分类的完整路径（用于过滤搜索）
    pub fn get_category_filter(&self) -> Option<String> {
        if let Some(selected) = &self.state.selected_category {
            Some(format!("category:{}", selected))
        } else {
            None
        }
    }
}