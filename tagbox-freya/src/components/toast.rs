use freya::prelude::*;
use std::time::Duration;

/// 通知类型
#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

/// 通知消息
#[derive(Debug, Clone, PartialEq)]
pub struct ToastMessage {
    pub id: String,
    pub message: String,
    pub toast_type: ToastType,
}

/// Toast 通知组件
/// 用于显示临时的成功、错误、信息或警告消息
#[component]
pub fn Toast(
    /// 通知消息
    message: ToastMessage,
    /// 关闭通知的回调
    on_close: EventHandler<String>,
) -> Element {
    let mut opacity = use_signal(|| 1.0);
    let message_id = message.id.clone();
    
    // 自动消失计时器
    use_effect(move || {
        let message_id = message_id.clone();
        let on_close = on_close.clone();
        spawn(async move {
            // 等待 3 秒
            tokio::time::sleep(Duration::from_secs(3)).await;
            
            // 淡出动画
            for i in (0..=10).rev() {
                opacity.set(i as f32 / 10.0);
                tokio::time::sleep(Duration::from_millis(30)).await;
            }
            
            // 触发关闭回调
            on_close.call(message_id);
        });
    });
    
    let (background, icon_color, icon) = match message.toast_type {
        ToastType::Success => ("rgb(220, 252, 231)", "rgb(34, 197, 94)", "✓"),
        ToastType::Error => ("rgb(254, 226, 226)", "rgb(239, 68, 68)", "✕"),
        ToastType::Info => ("rgb(224, 231, 255)", "rgb(99, 102, 241)", "ℹ"),
        ToastType::Warning => ("rgb(254, 243, 199)", "rgb(245, 158, 11)", "⚠"),
    };
    
    rsx! {
        rect {
            width: "350",
            padding: "16",
            margin: "8",
            background: "{background}",
            corner_radius: "6",
            shadow: "0 2 8 rgba(0, 0, 0, 0.1)",
            opacity: "{opacity}",
            direction: "horizontal",
            spacing: "12",
            
            // 图标
            rect {
                width: "24",
                height: "24",
                corner_radius: "12",
                background: "{icon_color}",
                content: "center",
                
                label {
                    color: "white",
                    font_size: "14",
                    font_weight: "bold",
                    "{icon}"
                }
            }
            
            // 消息内容
            label {
                width: "flex",
                color: "rgb(55, 65, 81)",
                font_size: "14",
                line_height: "1.5",
                "{message.message}"
            }
            
            // 关闭按钮
            rect {
                width: "20",
                height: "20",
                corner_radius: "4",
                content: "center",
                onclick: move |_| {
                    on_close.call(message.id.clone());
                },
                
                label {
                    color: "rgb(107, 114, 128)",
                    font_size: "16",
                    "×"
                }
            }
        }
    }
}

/// Toast 容器组件
/// 管理多个通知的显示
#[component]
pub fn ToastContainer(
    /// 通知消息列表
    messages: Signal<Vec<ToastMessage>>,
) -> Element {
    rsx! {
        rect {
            position: "absolute",
            position_top: "20",
            position_right: "20",
            direction: "column",
            spacing: "0",
            
            for message in messages.read().iter() {
                Toast {
                    key: "{message.id}",
                    message: message.clone(),
                    on_close: move |id: String| {
                        messages.write().retain(|m| m.id != id);
                    },
                }
            }
        }
    }
}

/// 创建新的通知消息
pub fn create_toast(toast_type: ToastType, message: &str) -> ToastMessage {
    ToastMessage {
        id: uuid::Uuid::new_v4().to_string(),
        message: message.to_string(),
        toast_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_toast() {
        let toast = create_toast(ToastType::Success, "操作成功");
        assert_eq!(toast.message, "操作成功");
        assert_eq!(toast.toast_type, ToastType::Success);
        assert!(!toast.id.is_empty());
    }

    #[test]
    fn test_toast_types() {
        let success = ToastType::Success;
        let error = ToastType::Error;
        let info = ToastType::Info;
        let warning = ToastType::Warning;
        
        assert_ne!(success, error);
        assert_ne!(info, warning);
        assert_eq!(success, ToastType::Success);
    }
}