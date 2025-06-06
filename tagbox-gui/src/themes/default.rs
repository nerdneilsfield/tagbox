use fltk::enums::Color;

pub struct DefaultTheme;

impl DefaultTheme {
    pub const BACKGROUND: Color = Color::from_rgb(248, 249, 250);
    pub const PRIMARY: Color = Color::from_rgb(0, 123, 255);
    pub const SUCCESS: Color = Color::from_rgb(40, 167, 69);
    pub const DANGER: Color = Color::from_rgb(220, 53, 69);
    pub const WARNING: Color = Color::from_rgb(255, 193, 7);
    pub const INFO: Color = Color::from_rgb(23, 162, 184);
    pub const TEXT: Color = Color::from_rgb(33, 37, 41);
    pub const MUTED: Color = Color::from_rgb(108, 117, 125);
    pub const WHITE: Color = Color::White;
    pub const BORDER: Color = Color::from_rgb(222, 226, 230);
}