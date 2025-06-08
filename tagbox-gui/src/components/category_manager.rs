use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType, Group},
    input::Input,
    button::Button,
    tree::{Tree, TreeItem, TreeSelect},
    enums::{Color, FrameType},
    frame::Frame,
    menu::Choice,
    dialog,
};
use std::sync::mpsc::Sender;
use std::collections::{HashMap, HashSet};
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;

pub struct CategoryManager {
    window: Window,
    
    // 分类树显示
    category_tree: Tree,
    
    // 编辑区域
    category_name_input: Input,
    parent_category_choice: Choice,
    description_input: Input,
    
    // 操作按钮
    add_btn: Button,
    edit_btn: Button,
    delete_btn: Button,
    move_up_btn: Button,
    move_down_btn: Button,
    
    // 对话框按钮
    apply_btn: Button,
    cancel_btn: Button,
    
    // 状态
    categories: HashMap<String, CategoryInfo>,
    selected_category: Option<String>,
    event_sender: Sender<AppEvent>,
}

#[derive(Debug, Clone)]
struct CategoryInfo {
    name: String,
    parent: Option<String>,
    children: Vec<String>,
    description: Option<String>,
    file_count: usize,
    level: usize,
}

impl CategoryManager {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(150, 150, 800, 600, "Category Management");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let padding = 15;
        let mut main_flex = Flex::new(padding, padding, 770, 570, None);
        main_flex.set_type(FlexType::Row);
        main_flex.set_spacing(15);
        
        // 左侧：分类树视图
        let mut tree_group = Group::new(0, 0, 400, 570, "Category Hierarchy");
        tree_group.set_frame(FrameType::BorderBox);
        tree_group.set_color(Color::White);
        tree_group.set_label_color(Color::from_rgb(51, 51, 51));
        tree_group.set_label_size(14);
        
        let mut category_tree = Tree::new(10, 25, 380, 535, None);
        category_tree.set_color(Color::White);
        // category_tree.set_connector_style(TreeConnector::Dotted); // Not available in this FLTK version
        category_tree.set_select_mode(TreeSelect::Single);
        // category_tree.show_root(false); // Method signature different
        
        tree_group.end();
        main_flex.fixed(&tree_group, 400);
        
        // 右侧：编辑区域
        let mut edit_group = Group::new(0, 0, 355, 570, "Category Details");
        edit_group.set_frame(FrameType::BorderBox);
        edit_group.set_color(Color::White);
        edit_group.set_label_color(Color::from_rgb(51, 51, 51));
        edit_group.set_label_size(14);
        
        let mut edit_flex = Flex::new(10, 25, 335, 535, None);
        edit_flex.set_type(FlexType::Column);
        edit_flex.set_spacing(10);
        
        // 分类名称
        let name_label = Frame::new(0, 0, 335, 25, "Category Name:");
        edit_flex.fixed(&name_label, 25);
        
        let mut category_name_input = Input::new(0, 0, 335, 30, None);
        category_name_input.set_color(Color::White);
        category_name_input.set_tooltip("Enter the category name");
        edit_flex.fixed(&category_name_input, 30);
        
        // 父分类选择
        let parent_label = Frame::new(0, 0, 335, 25, "Parent Category:");
        edit_flex.fixed(&parent_label, 25);
        
        let mut parent_category_choice = Choice::new(0, 0, 335, 30, None);
        parent_category_choice.add_choice("Root Level");
        parent_category_choice.set_value(0);
        parent_category_choice.set_tooltip("Select parent category (empty for root level)");
        edit_flex.fixed(&parent_category_choice, 30);
        
        // 描述
        let desc_label = Frame::new(0, 0, 335, 25, "Description:");
        edit_flex.fixed(&desc_label, 25);
        
        let mut description_input = Input::new(0, 0, 335, 30, None);
        description_input.set_color(Color::White);
        description_input.set_tooltip("Optional description for this category");
        edit_flex.fixed(&description_input, 30);
        
        // 操作按钮区域
        let mut operations_flex = Flex::new(0, 0, 335, 150, None);
        operations_flex.set_type(FlexType::Column);
        operations_flex.set_spacing(8);
        
        let mut add_btn = Button::new(0, 0, 335, 30, "Add New Category");
        add_btn.set_color(Color::from_rgb(40, 167, 69));
        add_btn.set_label_color(Color::White);
        operations_flex.fixed(&add_btn, 30);
        
        let mut edit_btn = Button::new(0, 0, 335, 30, "Update Selected Category");
        edit_btn.set_color(Color::from_rgb(0, 123, 255));
        edit_btn.set_label_color(Color::White);
        edit_btn.deactivate();
        operations_flex.fixed(&edit_btn, 30);
        
        let mut delete_btn = Button::new(0, 0, 335, 30, "Delete Selected Category");
        delete_btn.set_color(Color::from_rgb(220, 53, 69));
        delete_btn.set_label_color(Color::White);
        delete_btn.deactivate();
        operations_flex.fixed(&delete_btn, 30);
        
        // 排序按钮
        let mut sort_flex = Flex::new(0, 0, 335, 30, None);
        sort_flex.set_type(FlexType::Row);
        sort_flex.set_spacing(8);
        
        let mut move_up_btn = Button::new(0, 0, 0, 30, "Move Up");
        move_up_btn.set_color(Color::from_rgb(108, 117, 125));
        move_up_btn.set_label_color(Color::White);
        move_up_btn.deactivate();
        
        let mut move_down_btn = Button::new(0, 0, 0, 30, "Move Down");
        move_down_btn.set_color(Color::from_rgb(108, 117, 125));
        move_down_btn.set_label_color(Color::White);
        move_down_btn.deactivate();
        
        sort_flex.end();
        operations_flex.fixed(&sort_flex, 30);
        
        operations_flex.end();
        edit_flex.fixed(&operations_flex, 150);
        
        // 间隔
        let spacer = Frame::new(0, 0, 335, 0, None);
        edit_flex.resizable(&spacer);
        
        // 对话框控制按钮
        let mut dialog_flex = Flex::new(0, 0, 335, 40, None);
        dialog_flex.set_type(FlexType::Row);
        dialog_flex.set_spacing(10);
        
        let mut apply_btn = Button::new(0, 0, 0, 40, "Apply Changes");
        apply_btn.set_color(Color::from_rgb(40, 167, 69));
        apply_btn.set_label_color(Color::White);
        apply_btn.set_label_size(14);
        
        let mut cancel_btn = Button::new(0, 0, 0, 40, "Cancel");
        cancel_btn.set_color(Color::from_rgb(220, 53, 69));
        cancel_btn.set_label_color(Color::White);
        cancel_btn.set_label_size(14);
        
        dialog_flex.end();
        edit_flex.fixed(&dialog_flex, 40);
        
        edit_flex.end();
        edit_group.end();
        main_flex.fixed(&edit_group, 355);
        
        main_flex.end();
        window.end();
        
        Self {
            window,
            category_tree,
            category_name_input,
            parent_category_choice,
            description_input,
            add_btn,
            edit_btn,
            delete_btn,
            move_up_btn,
            move_down_btn,
            apply_btn,
            cancel_btn,
            categories: HashMap::new(),
            selected_category: None,
            event_sender,
        }
    }
    
    pub fn show(&mut self) {
        self.window.show();
        self.setup_callbacks();
    }
    
    pub fn hide(&mut self) {
        self.window.hide();
    }
    
    pub fn shown(&self) -> bool {
        self.window.shown()
    }
    
    pub async fn load_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 从数据库加载分类信息
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        
        let mut category_stats: HashMap<String, (usize, Option<String>, HashSet<String>)> = HashMap::new();
        
        // 统计文件数量并构建层次结构
        for file in &search_result.entries {
            // 一级分类
            category_stats.entry(file.category1.clone())
                .or_insert((0, None, HashSet::new()))
                .0 += 1;
                
            // 二级分类
            if let Some(cat2) = &file.category2 {
                category_stats.entry(cat2.clone())
                    .or_insert((0, Some(file.category1.clone()), HashSet::new()))
                    .0 += 1;
                    
                // 添加到父分类的子分类集合
                category_stats.entry(file.category1.clone())
                    .or_insert((0, None, HashSet::new()))
                    .2.insert(cat2.clone());
                    
                // 三级分类
                if let Some(cat3) = &file.category3 {
                    category_stats.entry(cat3.clone())
                        .or_insert((0, Some(cat2.clone()), HashSet::new()))
                        .0 += 1;
                        
                    // 添加到父分类的子分类集合
                    category_stats.entry(cat2.clone())
                        .or_insert((0, Some(file.category1.clone()), HashSet::new()))
                        .2.insert(cat3.clone());
                }
            }
        }
        
        // 构建CategoryInfo映射
        self.categories.clear();
        for (name, (file_count, parent, children)) in &category_stats {
            let level = if parent.is_none() {
                1
            } else if let Some(parent_name) = &parent {
                if category_stats.get(parent_name).and_then(|(_, grandparent, _)| grandparent.as_ref()).is_none() {
                    2
                } else {
                    3
                }
            } else {
                1
            };
            
            self.categories.insert(name.clone(), CategoryInfo {
                name: name.clone(),
                parent: parent.clone(),
                children: children.iter().cloned().collect(),
                description: None,
                file_count: *file_count,
                level,
            });
        }
        
        self.refresh_tree();
        self.refresh_parent_choices();
        
        Ok(())
    }
    
    fn refresh_tree(&mut self) {
        self.category_tree.clear();
        
        // 收集根级分类名称
        let mut root_category_names: Vec<_> = self.categories.iter()
            .filter_map(|(name, info)| {
                if info.parent.is_none() {
                    Some((name.clone(), info.file_count))
                } else {
                    None
                }
            })
            .collect();
        root_category_names.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (name, file_count) in root_category_names {
            let label = format!("{} ({})", name, file_count);
            let _item = self.category_tree.add(&label);
            // TODO: 添加子项需要Tree的更高级API
        }
    }
    
    fn add_children_to_tree(&mut self, parent_item: TreeItem, parent_name: &str) {
        if let Some(parent_info) = self.categories.get(parent_name) {
            let mut children: Vec<_> = parent_info.children.iter()
                .filter_map(|child_name| self.categories.get(child_name).map(|info| (child_name, info)))
                .collect();
            children.sort_by(|a, b| a.1.name.cmp(&b.1.name));
            
            for (child_name, child_info) in children {
                let label = format!("{} ({})", child_name, child_info.file_count);
                // TODO: 实现子项添加（FLTK Tree API 限制）
                // if let Some(child_item) = self.category_tree.add_child(&parent_item, &label) {
                //     self.add_children_to_tree(child_item, child_name);
                // }
            }
        }
    }
    
    fn refresh_parent_choices(&mut self) {
        self.parent_category_choice.clear();
        self.parent_category_choice.add_choice("Root Level");
        
        // 添加所有现有分类作为可能的父分类
        let mut categories: Vec<_> = self.categories.keys().collect();
        categories.sort();
        
        for category in categories {
            self.parent_category_choice.add_choice(category);
        }
    }
    
    fn setup_callbacks(&mut self) {
        // 树选择回调
        self.category_tree.set_callback(|tree| {
            if let Some(item) = tree.first_selected_item() {
                if let Some(label) = item.label() {
                    // 解析标签获取分类名称（去掉文件计数）
                    let category_name = label.split(" (").next().unwrap_or(&label).to_string();
                    // TODO: 更新选中状态和编辑区域
                    println!("Selected category: {}", category_name);
                }
            }
        });
        
        // 添加按钮回调
        self.add_btn.set_callback(|_| {
            // TODO: 验证输入并添加新分类
            println!("Add category button clicked");
        });
        
        // 编辑按钮回调
        self.edit_btn.set_callback(|_| {
            // TODO: 更新选中的分类
            println!("Edit category button clicked");
        });
        
        // 删除按钮回调
        self.delete_btn.set_callback(|_| {
            // TODO: 删除选中的分类（需要确认）
            println!("Delete category button clicked");
        });
        
        // 应用更改按钮回调
        self.apply_btn.set_callback(move |btn| {
            // TODO: 保存所有更改到数据库
            println!("Apply changes button clicked");
            if let Some(mut window) = btn.window() {
                window.hide();
            }
        });
        
        // 取消按钮回调
        self.cancel_btn.set_callback(move |btn| {
            // 关闭对话框而不保存
            if let Some(mut window) = btn.window() {
                window.hide();
            }
        });
        
        // Escape键处理
        self.window.set_callback(move |win| {
            if fltk::app::event() == fltk::enums::Event::KeyDown 
                && fltk::app::event_key() == fltk::enums::Key::Escape {
                win.hide();
            }
        });
    }
    
    fn update_edit_area(&mut self) {
        if let Some(category_name) = &self.selected_category {
            if let Some(info) = self.categories.get(category_name) {
                self.category_name_input.set_value(&info.name);
                self.description_input.set_value(info.description.as_deref().unwrap_or(""));
                
                // 设置父分类选择
                if let Some(parent) = &info.parent {
                    for i in 0..self.parent_category_choice.size() {
                        if let Some(choice_text) = self.parent_category_choice.at(i).and_then(|item| item.label()) {
                            if choice_text == *parent {
                                self.parent_category_choice.set_value(i);
                                break;
                            }
                        }
                    }
                } else {
                    self.parent_category_choice.set_value(0); // Root Level
                }
                
                // 激活编辑和删除按钮
                self.edit_btn.activate();
                self.delete_btn.activate();
                self.move_up_btn.activate();
                self.move_down_btn.activate();
            }
        } else {
            // 清空编辑区域
            self.category_name_input.set_value("");
            self.description_input.set_value("");
            self.parent_category_choice.set_value(0);
            
            // 禁用编辑相关按钮
            self.edit_btn.deactivate();
            self.delete_btn.deactivate();
            self.move_up_btn.deactivate();
            self.move_down_btn.deactivate();
        }
    }
    
    fn validate_category_input(&self) -> Result<(), String> {
        let category_name_value = self.category_name_input.value();
        let name = category_name_value.trim();
        if name.is_empty() {
            return Err("Category name cannot be empty".to_string());
        }
        
        // 检查名称是否已存在（编辑时除外）
        if let Some(selected) = &self.selected_category {
            if name != selected && self.categories.contains_key(name) {
                return Err("Category name already exists".to_string());
            }
        } else if self.categories.contains_key(name) {
            return Err("Category name already exists".to_string());
        }
        
        // 检查是否会创建循环引用
        let parent_index = self.parent_category_choice.value();
        if parent_index > 0 {
            if let Some(parent_item) = self.parent_category_choice.at(parent_index) {
                if let Some(parent_name) = parent_item.label() {
                    if self.would_create_cycle(name, &parent_name) {
                        return Err("Cannot create circular reference in category hierarchy".to_string());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn would_create_cycle(&self, category: &str, potential_parent: &str) -> bool {
        // 检查是否会创建循环引用
        let mut current = potential_parent;
        let mut visited = HashSet::new();
        
        while let Some(info) = self.categories.get(current) {
            if visited.contains(current) || current == category {
                return true;
            }
            visited.insert(current);
            
            if let Some(parent) = &info.parent {
                current = parent;
            } else {
                break;
            }
        }
        
        false
    }
    
    fn add_category(&mut self) -> Result<(), String> {
        self.validate_category_input()?;
        
        let category_name_value = self.category_name_input.value();
        let name = category_name_value.trim().to_string();
        let description_value = self.description_input.value();
        let description = description_value.trim();
        let description = if description.is_empty() { None } else { Some(description.to_string()) };
        
        let parent = if self.parent_category_choice.value() > 0 {
            self.parent_category_choice.choice()
        } else {
            None
        };
        
        let level = if parent.is_none() {
            1
        } else if let Some(parent_name) = &parent {
            if let Some(parent_info) = self.categories.get(parent_name) {
                parent_info.level + 1
            } else {
                2
            }
        } else {
            1
        };
        
        // 检查层级不超过3级
        if level > 3 {
            return Err("Maximum category depth is 3 levels".to_string());
        }
        
        let category_info = CategoryInfo {
            name: name.clone(),
            parent: parent.clone(),
            children: Vec::new(),
            description,
            file_count: 0,
            level,
        };
        
        self.categories.insert(name.clone(), category_info);
        
        // 更新父分类的children列表
        if let Some(parent_name) = &parent {
            if let Some(parent_info) = self.categories.get_mut(parent_name) {
                parent_info.children.push(name.clone());
            }
        }
        
        self.refresh_tree();
        self.refresh_parent_choices();
        
        // 清空输入
        self.category_name_input.set_value("");
        self.description_input.set_value("");
        self.parent_category_choice.set_value(0);
        
        Ok(())
    }
    
    fn update_category(&mut self) -> Result<(), String> {
        let selected = self.selected_category.clone().ok_or("No category selected")?;
        self.validate_category_input()?;
        
        let category_name_value = self.category_name_input.value();
        let new_name = category_name_value.trim().to_string();
        let description_value = self.description_input.value();
        let description = description_value.trim();
        let description = if description.is_empty() { None } else { Some(description.to_string()) };
        
        let parent = if self.parent_category_choice.value() > 0 {
            self.parent_category_choice.choice()
        } else {
            None
        };
        
        // 更新分类信息
        if let Some(mut info) = self.categories.remove(&selected) {
            // 更新旧父分类的children列表
            if let Some(old_parent) = &info.parent {
                if let Some(old_parent_info) = self.categories.get_mut(old_parent) {
                    old_parent_info.children.retain(|child| child != &selected);
                }
            }
            
            // 更新信息
            info.name = new_name.clone();
            info.description = description;
            info.parent = parent.clone();
            
            // 计算新层级
            info.level = if parent.is_none() {
                1
            } else if let Some(parent_name) = &parent {
                if let Some(parent_info) = self.categories.get(parent_name) {
                    parent_info.level + 1
                } else {
                    2
                }
            } else {
                1
            };
            
            // 检查层级
            if info.level > 3 {
                return Err("Maximum category depth is 3 levels".to_string());
            }
            
            self.categories.insert(new_name.clone(), info);
            
            // 更新新父分类的children列表
            if let Some(parent_name) = &parent {
                if let Some(parent_info) = self.categories.get_mut(parent_name) {
                    parent_info.children.push(new_name.clone());
                }
            }
            
            // 如果名称改变了，需要更新所有引用
            if new_name != selected {
                self.update_category_references(&selected, &new_name);
            }
            
            self.selected_category = Some(new_name);
        }
        
        self.refresh_tree();
        self.refresh_parent_choices();
        
        Ok(())
    }
    
    fn update_category_references(&mut self, old_name: &str, new_name: &str) {
        // 更新所有引用旧名称的分类
        let categories_to_update: Vec<String> = self.categories.iter()
            .filter_map(|(name, info)| {
                if info.parent.as_deref() == Some(old_name) || info.children.iter().any(|s| s == old_name) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
            
        for category_name in categories_to_update {
            if let Some(info) = self.categories.get_mut(&category_name) {
                // 更新父分类引用
                if info.parent.as_deref() == Some(old_name) {
                    info.parent = Some(new_name.to_string());
                }
                
                // 更新子分类引用
                for child in &mut info.children {
                    if child == old_name {
                        *child = new_name.to_string();
                    }
                }
            }
        }
    }
    
    fn delete_category(&mut self) -> Result<(), String> {
        let selected = self.selected_category.clone().ok_or("No category selected")?;
        
        // 检查是否有子分类
        if let Some(info) = self.categories.get(&selected) {
            if !info.children.is_empty() {
                return Err("Cannot delete category with subcategories. Move or delete subcategories first.".to_string());
            }
            
            if info.file_count > 0 {
                // 显示确认对话框
                let choice = dialog::choice2_default(
                    &format!("Category '{}' contains {} files. These files will be moved to the parent category or 'Default'. Continue?", 
                             selected, info.file_count),
                    "Yes",
                    "No",
                    ""
                );
                
                if choice != Some(0) {
                    return Ok(()); // 用户取消
                }
            }
        }
        
        // 删除分类
        if let Some(info) = self.categories.remove(&selected) {
            // 从父分类的children列表中移除
            if let Some(parent) = &info.parent {
                if let Some(parent_info) = self.categories.get_mut(parent) {
                    parent_info.children.retain(|child| child != &selected);
                }
            }
        }
        
        self.selected_category = None;
        self.refresh_tree();
        self.refresh_parent_choices();
        self.update_edit_area();
        
        Ok(())
    }
}