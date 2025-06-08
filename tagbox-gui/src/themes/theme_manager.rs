use fltk::app;
use fltk_theme::{ColorTheme, WidgetTheme, WidgetScheme, ThemeType, SchemeType, color_themes};

#[derive(Debug, Clone, PartialEq)]
pub enum AppTheme {
    Default,
    Dark,
    Light,
    AquaClassic,
    HighContrast,
    Fleet,
    Sweet,
}

pub struct ThemeManager {
    current_theme: AppTheme,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current_theme: AppTheme::Default,
        }
    }
    
    pub fn apply_theme(&mut self, theme: AppTheme) {
        match theme {
            AppTheme::Default => {
                // 使用FLTK默认主题
                app::App::default().with_scheme(app::Scheme::Gtk);
            },
            AppTheme::Dark => {
                // 应用深色主题
                let color_theme = ColorTheme::new(color_themes::DARK_THEME);
                color_theme.apply();
                
                let widget_scheme = WidgetScheme::new(SchemeType::Fluent);
                widget_scheme.apply();
            },
            AppTheme::Light => {
                // 应用浅色主题
                let color_theme = ColorTheme::new(color_themes::TAN_THEME);
                color_theme.apply();
                
                let widget_scheme = WidgetScheme::new(SchemeType::Clean);
                widget_scheme.apply();
            },
            AppTheme::AquaClassic => {
                // 应用Aqua经典主题
                let widget_theme = WidgetTheme::new(ThemeType::AquaClassic);
                widget_theme.apply();
            },
            AppTheme::HighContrast => {
                // 应用高对比度主题
                let color_theme = ColorTheme::new(color_themes::BLACK_THEME);
                color_theme.apply();
                
                let widget_scheme = WidgetScheme::new(SchemeType::Clean);
                widget_scheme.apply();
            },
            AppTheme::Fleet => {
                // 应用Fleet主题
                let widget_scheme = WidgetScheme::new(SchemeType::Fleet1);
                widget_scheme.apply();
                
                // 可以配合颜色主题
                let color_theme = ColorTheme::new(color_themes::DARK_THEME);
                color_theme.apply();
            },
            AppTheme::Sweet => {
                // 应用Sweet主题（类似GNOME/KDE）
                let widget_scheme = WidgetScheme::new(SchemeType::Sweet);
                widget_scheme.apply();
                
                let color_theme = ColorTheme::new(color_themes::TAN_THEME);
                color_theme.apply();
            },
        }
        
        self.current_theme = theme;
        
        // 重绘所有窗口
        app::redraw();
    }
    
    pub fn get_current_theme(&self) -> &AppTheme {
        &self.current_theme
    }
    
    pub fn get_available_themes() -> Vec<AppTheme> {
        vec![
            AppTheme::Default,
            AppTheme::Dark,
            AppTheme::Light,
            AppTheme::AquaClassic,
            AppTheme::HighContrast,
            AppTheme::Fleet,
            AppTheme::Sweet,
        ]
    }
    
    pub fn theme_name(theme: &AppTheme) -> &'static str {
        match theme {
            AppTheme::Default => "Default",
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light", 
            AppTheme::AquaClassic => "Aqua Classic",
            AppTheme::HighContrast => "High Contrast",
            AppTheme::Fleet => "Fleet",
            AppTheme::Sweet => "Sweet",
        }
    }
    
    pub fn theme_description(theme: &AppTheme) -> &'static str {
        match theme {
            AppTheme::Default => "Standard FLTK theme with GTK styling",
            AppTheme::Dark => "Dark color scheme with modern fluent design",
            AppTheme::Light => "Light color scheme with clean appearance",
            AppTheme::AquaClassic => "macOS Aqua-style interface",
            AppTheme::HighContrast => "High contrast for better accessibility",
            AppTheme::Fleet => "Professional gradient-based theme",
            AppTheme::Sweet => "GNOME/KDE inspired theme",
        }
    }
    
    // 从字符串解析主题
    pub fn parse_theme(theme_str: &str) -> Option<AppTheme> {
        match theme_str.to_lowercase().as_str() {
            "default" => Some(AppTheme::Default),
            "dark" => Some(AppTheme::Dark),
            "light" => Some(AppTheme::Light),
            "aqua" | "aqua_classic" => Some(AppTheme::AquaClassic),
            "high_contrast" | "contrast" => Some(AppTheme::HighContrast),
            "fleet" => Some(AppTheme::Fleet),
            "sweet" => Some(AppTheme::Sweet),
            _ => None,
        }
    }
    
    // 应用适合文件管理的颜色配置
    pub fn apply_file_manager_styling(&self) {
        match self.current_theme {
            AppTheme::Dark => {
                // 深色主题的文件管理器样式调整
                app::set_color(fltk::enums::Color::Background, 30, 30, 30);
                app::set_color(fltk::enums::Color::Background2, 40, 40, 40);
                app::set_color(fltk::enums::Color::Foreground, 220, 220, 220);
            },
            AppTheme::Light => {
                // 浅色主题的文件管理器样式调整  
                app::set_color(fltk::enums::Color::Background, 248, 249, 250);
                app::set_color(fltk::enums::Color::Background2, 255, 255, 255);
                app::set_color(fltk::enums::Color::Foreground, 33, 37, 41);
            },
            AppTheme::AquaClassic => {
                // Aqua主题的文件管理器样式
                app::set_color(fltk::enums::Color::Background, 236, 236, 236);
                app::set_color(fltk::enums::Color::Selection, 56, 117, 215);
            },
            _ => {
                // 其他主题使用默认样式
            }
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}