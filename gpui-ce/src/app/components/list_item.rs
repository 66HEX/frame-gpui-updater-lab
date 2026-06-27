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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::app) enum FrameTrackListItemLayout {
    Inline,
    Stacked,
}

pub(in crate::app) fn frame_track_list_item(
    id: impl Into<String>,
    index_label: impl Into<String>,
    primary: impl Into<String>,
    detail: impl Into<String>,
    selected: bool,
    enabled: bool,
    layout: FrameTrackListItemLayout,
) -> gpui::Stateful<gpui::Div> {
    let colors = button_colors(ButtonVariant::Secondary, selected, enabled);
    let index_label = index_label.into();
    let primary = primary.into();
    let detail = detail.into();

    let label_row = div()
        .min_w_0()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_color(color(theme::FRAME_GRAY_600))
                .child(index_label),
        )
        .child(div().text_color(color(theme::FOREGROUND)).child(primary));

    let content = match layout {
        FrameTrackListItemLayout::Inline => label_row.when(!detail.is_empty(), |this| {
            this.child(
                div()
                    .truncate()
                    .text_color(color(theme::FRAME_GRAY_600))
                    .child(detail),
            )
        }),
        FrameTrackListItemLayout::Stacked => div()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_1()
            .child(label_row)
            .when(!detail.is_empty(), |this| {
                this.child(
                    div()
                        .truncate()
                        .text_color(color(theme::FRAME_GRAY_600))
                        .child(detail),
                )
            }),
    };

    div()
        .id(id.into())
        .min_h(px(SETTINGS_CONTROL_HEIGHT))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .rounded(px(theme::RADIUS_SM))
        .px(px(10.0))
        .py(px(6.0))
        .bg(color(colors.background))
        .text_size(px(theme::TEXT_LABEL_SIZE))
        .text_color(color(colors.foreground))
        .opacity(colors.opacity)
        .shadow(button_highlight_shadows())
        .when(enabled, |this| {
            this.hover(move |style| {
                style
                    .bg(color(colors.hover_background))
                    .text_color(color(colors.hover_foreground))
                    .cursor_pointer()
            })
            .active(move |style| style.bg(color(colors.active_background)))
        })
        .when(!enabled, |this| this.cursor_not_allowed())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            button_mouse_down(enabled, window, cx);
        })
        .child(content)
        .child(frame_selection_dot(selected))
}
