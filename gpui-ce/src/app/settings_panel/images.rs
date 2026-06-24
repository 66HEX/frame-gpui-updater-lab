use super::*;

pub(in crate::app) fn settings_images_tab(
    config: &ConversionConfig,
    settings_disabled: bool,
    available_encoders: &AvailableEncoders,
    video_width_focus: Option<&FocusHandle>,
    video_height_focus: Option<&FocusHandle>,
    window: &Window,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(settings_video_resolution_section(
            config,
            settings_disabled,
            video_width_focus,
            video_height_focus,
            window,
            cx,
        ))
        .child(settings_video_ml_section(
            config,
            settings_disabled,
            available_encoders,
            cx,
        ))
        .child(settings_video_scaling_section(
            config,
            settings_disabled,
            cx,
        ))
        .child(settings_images_pixel_format_section(
            config,
            settings_disabled,
            cx,
        ))
}

fn settings_images_pixel_format_section(
    config: &ConversionConfig,
    settings_disabled: bool,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    let mut list = div().grid().grid_cols(1);
    for option in video_pixel_format_options(config) {
        let pixel_format = option.id;
        let enabled = !settings_disabled && !option.is_disabled;
        list = list.child(
            settings_images_list_item(
                format!("images-pixel-format-{pixel_format}"),
                option.label,
                option.caption,
                option.is_selected,
                enabled,
            )
            .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                cx.stop_propagation();
                if !enabled {
                    return;
                }
                if root.update_selected_config(|config| apply_pixel_format(config, pixel_format)) {
                    cx.notify();
                }
            })),
        );
    }

    settings_section("PIXEL FORMAT").child(list)
}

fn settings_images_list_item(
    id: impl Into<String>,
    title: impl Into<String>,
    caption: impl Into<String>,
    selected: bool,
    enabled: bool,
) -> gpui::Stateful<gpui::Div> {
    let colors = button_colors(ButtonVariant::Secondary, selected, enabled);

    div()
        .id(id.into())
        .h(px(SETTINGS_CONTROL_HEIGHT))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .rounded(px(theme::RADIUS_SM))
        .px(px(10.0))
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
        .child(
            div()
                .text_color(color(theme::FOREGROUND))
                .child(title.into()),
        )
        .child(
            div()
                .truncate()
                .text_size(px(theme::TEXT_LABEL_SIZE))
                .text_color(color(theme::FRAME_GRAY_600))
                .child(caption.into()),
        )
}
