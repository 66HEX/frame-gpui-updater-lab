use super::*;

pub(in crate::app) fn frame_list_item(
    id: impl Into<String>,
    selected: bool,
    enabled: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id.into())
        .h(px(SETTINGS_CONTROL_HEIGHT))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .rounded(px(theme::RADIUS_SM))
        .border_l(px(2.0))
        .border_color(color(if selected {
            theme::FRAME_GRAY_600
        } else {
            theme::TRANSPARENT
        }))
        .bg(color(if selected {
            theme::FRAME_GRAY_100
        } else {
            theme::TRANSPARENT
        }))
        .pl(px(if selected { 10.0 } else { 8.0 }))
        .pr(px(12.0))
        .text_size(px(theme::TEXT_LABEL_SIZE))
        .text_color(color(if selected {
            theme::FOREGROUND
        } else {
            theme::FRAME_GRAY_600
        }))
        .opacity(if enabled { 1.0 } else { 0.5 })
        .when(enabled, |this| {
            this.hover(|style| style.text_color(color(theme::FOREGROUND)).cursor_pointer())
        })
        .when(!enabled, |this| this.cursor_not_allowed())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            button_mouse_down(enabled, window, cx);
        })
}

pub(in crate::app) fn frame_list_item_with_caption(
    id: impl Into<String>,
    title: impl Into<String>,
    caption: impl Into<String>,
    selected: bool,
    enabled: bool,
) -> gpui::Stateful<gpui::Div> {
    let title = title.into();
    let caption = caption.into();

    frame_list_item(id, selected, enabled)
        .gap_3()
        .child(div().text_color(color(theme::FOREGROUND)).child(title))
        .child(
            div()
                .truncate()
                .text_size(px(theme::TEXT_LABEL_SIZE))
                .text_color(color(theme::FRAME_GRAY_600))
                .child(caption),
        )
}
