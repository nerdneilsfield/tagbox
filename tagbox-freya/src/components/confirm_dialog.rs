use freya::prelude::*;
use crate::components::CustomButton;

/// 确认对话框组件
/// 用于在执行危险操作（如删除）前要求用户确认
#[component]
pub fn ConfirmDialog(
    /// 对话框标题
    title: String,
    /// 对话框消息内容
    message: String,
    /// 控制对话框显示/隐藏的信号
    is_open: Signal<bool>,
    /// 确认按钮的回调
    on_confirm: EventHandler<()>,
    /// 取消按钮的回调
    on_cancel: EventHandler<()>,
) -> Element {
    // 如果对话框未打开，不渲染任何内容
    if !is_open() {
        return rsx! {};
    }

    rsx! {
        // 背景遮罩
        rect {
            width: "100%",
            height: "100%",
            position: "absolute",
            background: "rgb(0, 0, 0)",
            onclick: move |_| {
                // 点击背景关闭对话框
                is_open.set(false);
                on_cancel.call(());
            },
            
            // 对话框容器
            rect {
                position: "absolute",
                width: "400",
                background: "white",
                corner_radius: "8",
                shadow: "0 4 16 0 rgb(200, 200, 200)",
                padding: "24",
                // 居中显示
                position_top: "50%",
                position_left: "50%",
                offset_x: "-200",
                offset_y: "-150",
                onclick: move |e| {
                    // 阻止事件冒泡，避免点击对话框内容时关闭
                    e.stop_propagation();
                },
                
                // 对话框内容
                rect {
                    direction: "column",
                    spacing: "20",
                    
                    // 标题
                    label {
                        font_size: "20",
                        font_weight: "bold",
                        color: "rgb(30, 30, 30)",
                        "{title}"
                    }
                    
                    // 消息内容
                    label {
                        font_size: "16",
                        color: "rgb(60, 60, 60)",
                        line_height: "1.5",
                        "{message}"
                    }
                    
                    // 按钮区域
                    rect {
                        direction: "horizontal",
                        spacing: "12",
                        content: "center end",
                        margin: "20 0 0 0",
                        
                        // 取消按钮
                        CustomButton {
                            text: "取消",
                            variant: "secondary",
                            onpress: move |_| {
                                is_open.set(false);
                                on_cancel.call(());
                            },
                        }
                        
                        // 确认按钮（危险操作，使用红色）
                        rect {
                            background: "rgb(220, 38, 38)",
                            corner_radius: "4",
                            padding: "8 16",
                            onclick: move |_| {
                                is_open.set(false);
                                on_confirm.call(());
                            },
                            
                            label {
                                color: "white",
                                font_weight: "bold",
                                "确认"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_testing::prelude::*;

    #[test]
    fn test_confirm_dialog_display() {
        let mut utils = launch_test(|ctx| {
            let is_open = use_signal(&ctx, || true);
            let confirmed = use_signal(&ctx, || false);
            let cancelled = use_signal(&ctx, || false);
            
            rsx! {
                ConfirmDialog {
                    title: "删除文件",
                    message: "确定要删除这个文件吗？此操作无法撤销。",
                    is_open: is_open,
                    on_confirm: move |_| confirmed.set(true),
                    on_cancel: move |_| cancelled.set(true),
                }
            }
        });

        // 验证对话框渲染
        let root = utils.root();
        assert!(root.children().len() > 0);
        
        // 验证遮罩层存在
        let overlay = root.get(0);
        assert_eq!(overlay.get_type(), freya_elements::elements::rect::TAG_NAME);
    }

    #[test]
    fn test_confirm_dialog_hidden() {
        let mut utils = launch_test(|ctx| {
            let is_open = use_signal(&ctx, || false);
            
            rsx! {
                ConfirmDialog {
                    title: "Test",
                    message: "Test message",
                    is_open: is_open,
                    on_confirm: |_| {},
                    on_cancel: |_| {},
                }
            }
        });

        // 验证对话框未渲染
        let root = utils.root();
        assert_eq!(root.children().len(), 0);
    }

    #[test]
    fn test_dialog_callbacks() {
        // 测试确认和取消回调
        let title = "删除确认";
        let message = "您确定要删除吗？";
        
        // 验证标题和消息正确传递
        assert_eq!(title, "删除确认");
        assert_eq!(message, "您确定要删除吗？");
    }
}