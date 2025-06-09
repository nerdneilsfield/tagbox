use freya::prelude::*;

/// 自定义按钮组件，替代有问题的原生Button
#[component]
pub fn CustomButton(
    onpress: EventHandler<()>,
    text: String,
    #[props(default = "primary".to_string())] variant: String,
    #[props(default = false)] disabled: bool,
) -> Element {
    let mut is_hovered = use_signal(|| false);
    let mut is_pressed = use_signal(|| false);
    
    // 根据变体和状态确定颜色
    let (bg_color, _hover_color, text_color) = match variant.as_str() {
        "primary" => {
            if disabled {
                ("rgb(200, 200, 200)", "rgb(200, 200, 200)", "rgb(150, 150, 150)")
            } else if is_pressed() {
                ("rgb(50, 100, 200)", "rgb(50, 100, 200)", "white")
            } else if is_hovered() {
                ("rgb(70, 130, 240)", "rgb(70, 130, 240)", "white")
            } else {
                ("rgb(59, 130, 246)", "rgb(70, 130, 240)", "white")
            }
        }
        "secondary" => {
            if disabled {
                ("rgb(200, 200, 200)", "rgb(200, 200, 200)", "rgb(150, 150, 150)")
            } else if is_pressed() {
                ("rgb(210, 210, 210)", "rgb(210, 210, 210)", "rgb(50, 50, 50)")
            } else if is_hovered() {
                ("rgb(230, 230, 230)", "rgb(230, 230, 230)", "rgb(50, 50, 50)")
            } else {
                ("rgb(245, 245, 245)", "rgb(230, 230, 230)", "rgb(50, 50, 50)")
            }
        }
        _ => ("rgb(245, 245, 245)", "rgb(230, 230, 230)", "rgb(50, 50, 50)")
    };
    
    rsx! {
        rect {
            padding: "8 16",
            background: bg_color,
            corner_radius: "6",
            direction: "horizontal",
            content: "center",
            min_width: "80",
            height: "36",
            border: "1 solid rgb(220, 220, 220)",
            shadow: "0 1 3 0 rgb(220, 220, 220)",
            
            onmouseenter: move |_| {
                if !disabled {
                    is_hovered.set(true);
                }
            },
            onmouseleave: move |_| {
                is_hovered.set(false);
                is_pressed.set(false);
            },
            onmousedown: move |_| {
                if !disabled {
                    is_pressed.set(true);
                }
            },
            onmouseup: move |_| {
                is_pressed.set(false);
            },
            onclick: move |_| {
                if !disabled {
                    onpress.call(());
                }
            },
            
            label {
                font_size: "14",
                font_weight: "500",
                color: text_color,
                "{text}"
            }
        }
    }
}

/// 图标按钮组件
#[component]
pub fn IconButton(
    onpress: EventHandler<()>,
    icon: String,
) -> Element {
    let mut is_hovered = use_signal(|| false);
    
    rsx! {
        rect {
            width: "36",
            height: "36",
            padding: "8",
            background: if is_hovered() {
                "rgb(240, 240, 240)"
            } else {
                "transparent"
            },
            corner_radius: "6",
            content: "center",
            
            onmouseenter: move |_| {
                is_hovered.set(true);
            },
            onmouseleave: move |_| {
                is_hovered.set(false);
            },
            onclick: move |_| {
                onpress.call(());
            },
            
            label {
                font_size: "18",
                color: "rgb(50, 50, 50)",
                "{icon}"
            }
        }
    }
}