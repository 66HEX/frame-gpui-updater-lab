use super::*;

pub(in crate::app) struct SettingsPresetsTabState<'a> {
    pub(in crate::app) config: &'a ConversionConfig,
    pub(in crate::app) metadata: Option<&'a SourceMetadata>,
    pub(in crate::app) settings_disabled: bool,
    pub(in crate::app) preset_name: &'a str,
    pub(in crate::app) preset_name_focus: Option<&'a FocusHandle>,
    pub(in crate::app) presets: &'a [PresetDefinition],
    pub(in crate::app) notice: Option<&'a PresetNotice>,
}

pub(in crate::app) fn settings_presets_tab(
    state: SettingsPresetsTabState<'_>,
    window: &Window,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    let mut list = div().grid().grid_cols(1);
    for option in preset_options(state.config, state.presets, state.metadata) {
        list = list.child(settings_preset_row(
            option,
            state.settings_disabled,
            window,
            cx,
        ));
    }

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(settings_presets_header(state.notice))
        .child(settings_presets_save_row(
            state.preset_name,
            state.settings_disabled,
            state.preset_name_focus,
            window,
            cx,
        ))
        .child(list)
}

fn settings_presets_header(notice: Option<&PresetNotice>) -> gpui::Div {
    let mut header = div()
        .relative()
        .w_full()
        .child(settings_section_label("PRESET LIBRARY"));
    if let Some(notice) = notice {
        header = header.child(
            div()
                .absolute()
                .top_0()
                .right_0()
                .text_size(px(theme::TEXT_LABEL_SIZE))
                .text_color(color(match notice.tone {
                    PresetNoticeTone::Success => theme::FOREGROUND,
                    PresetNoticeTone::Error => theme::FRAME_RED,
                }))
                .child(notice.text.clone()),
        );
    }

    header
}

fn settings_presets_save_row(
    preset_name: &str,
    settings_disabled: bool,
    preset_name_focus: Option<&FocusHandle>,
    window: &Window,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    let save_enabled = !settings_disabled && !preset_name.trim().is_empty();
    div()
        .flex()
        .gap_2()
        .child(div().flex_1().child(frame_text_input(
            FrameTextInputSpec {
                id: "settings-preset-name-field",
                value: preset_name,
                placeholder: "Preset Label",
                disabled: settings_disabled,
                focus: preset_name_focus,
                kind: FrameTextInputKind::PresetName,
            },
            window,
            cx,
        )))
        .child(settings_save_preset_button(save_enabled, cx))
}

fn settings_save_preset_button(
    enabled: bool,
    cx: &mut Context<FrameRoot>,
) -> gpui::Stateful<gpui::Div> {
    let colors = button_colors(ButtonVariant::Secondary, false, enabled);

    div()
        .id("settings-save-preset")
        .h(px(SETTINGS_CONTROL_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
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
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            if enabled && root.save_preset_from_draft() {
                cx.notify();
            }
        }))
        .child("Save")
}

fn settings_preset_row(
    option: PresetOption,
    settings_disabled: bool,
    _window: &Window,
    cx: &mut Context<FrameRoot>,
) -> gpui::Stateful<gpui::Div> {
    let preset = option.preset;
    let preset_id = preset.id.clone();
    let apply_all_id = preset.id.clone();
    let delete_id = preset.id.clone();
    let is_enabled = !settings_disabled && option.is_compatible;
    let selected = option.is_selected;
    let status = option.status;

    div()
        .id(format!("preset-{}", preset.id))
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
        .pr(px(4.0))
        .text_size(px(theme::TEXT_LABEL_SIZE))
        .text_color(color(if selected {
            theme::FOREGROUND
        } else {
            theme::FRAME_GRAY_600
        }))
        .opacity(if !option.is_compatible || settings_disabled {
            0.5
        } else {
            1.0
        })
        .when(is_enabled, |this| {
            this.hover(|style| style.text_color(color(theme::FOREGROUND)).cursor_pointer())
        })
        .when(!is_enabled, |this| this.cursor_not_allowed())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            button_mouse_down(is_enabled, window, cx);
        })
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            if is_enabled && root.apply_preset_to_selected(&preset_id) {
                cx.notify();
            }
        }))
        .child(div().min_w_0().truncate().child(preset.name.clone()))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .pr(px(8.0))
                        .text_size(px(theme::TEXT_LABEL_SIZE))
                        .text_color(color(theme::FRAME_GRAY_600))
                        .child(status.unwrap_or_default()),
                )
                .when(option.is_compatible, |this| {
                    this.child(settings_preset_icon_button(
                        format!("settings-preset-apply-all-{}", apply_all_id),
                        assets::ICON_LIST_CHECKS,
                        ButtonVariant::Ghost,
                        !settings_disabled,
                        move |root, window, cx| {
                            root.confirm_apply_preset_to_all(apply_all_id.clone(), window, cx);
                        },
                        cx,
                    ))
                })
                .when(!preset.built_in, |this| {
                    this.child(settings_preset_icon_button(
                        format!("settings-preset-delete-{}", delete_id),
                        assets::ICON_TRASH,
                        ButtonVariant::Default,
                        !settings_disabled,
                        move |root, _window, cx| {
                            if root.delete_preset(&delete_id) {
                                cx.notify();
                            }
                        },
                        cx,
                    ))
                }),
        )
}

fn settings_preset_icon_button(
    id: String,
    icon: &'static str,
    variant: ButtonVariant,
    enabled: bool,
    action: impl Fn(&mut FrameRoot, &mut Window, &mut Context<FrameRoot>) + 'static,
    cx: &mut Context<FrameRoot>,
) -> gpui::Stateful<gpui::Div> {
    let colors = button_colors(variant, false, enabled);
    let icon_color = if matches!(variant, ButtonVariant::Default) {
        color(theme::FOREGROUND)
    } else {
        color(colors.foreground)
    };

    div()
        .id(id)
        .w(px(24.0))
        .h(px(24.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .bg(color(if matches!(variant, ButtonVariant::Default) {
            theme::FRAME_RED
        } else {
            colors.background
        }))
        .text_color(color(colors.foreground))
        .opacity(colors.opacity)
        .when(enabled, |this| {
            this.hover(move |style| {
                style
                    .bg(color(if matches!(variant, ButtonVariant::Default) {
                        theme::FRAME_RED.with_alpha(0.86)
                    } else {
                        colors.hover_background
                    }))
                    .cursor_pointer()
            })
        })
        .when(!enabled, |this| this.cursor_not_allowed())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            button_mouse_down(enabled, window, cx);
        })
        .on_click(cx.listener(move |root, _: &ClickEvent, window, cx| {
            cx.stop_propagation();
            if enabled {
                action(root, window, cx);
            }
        }))
        .child(icon_svg(icon, 16.0, icon_color))
}
