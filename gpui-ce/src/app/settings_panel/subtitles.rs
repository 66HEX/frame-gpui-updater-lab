use super::*;

pub(in crate::app) fn settings_subtitles_tab(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    settings_disabled: bool,
    subtitle_fonts: &[String],
    window: &Window,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    let copy_mode = config.processing_mode == ProcessingMode::Copy;
    let burn_in_disabled = settings_disabled || copy_mode;
    let content = div().flex().flex_col().gap_4().child(
        settings_section("BURN-IN SUBTITLES")
            .child(settings_subtitle_burn_button(config, burn_in_disabled, cx))
            .child(settings_hint_text(if copy_mode {
                "Burn-in subtitles are disabled in stream copy mode."
            } else {
                "Burning in subtitles will force video re-encoding."
            })),
    );

    let content = if copy_mode {
        content
    } else {
        content.child(
            settings_section("STYLE").child(settings_subtitle_style_controls(
                config,
                burn_in_disabled,
                subtitle_fonts,
                window,
                cx,
            )),
        )
    };

    let track_options = subtitle_track_options(config, metadata, settings_disabled);
    if track_options.is_empty() {
        return content
            .child(settings_section("SOURCE TRACKS").child(settings_hint_text("No subtitles")));
    }

    let mut list = div().grid().grid_cols(1).gap_2();
    for option in track_options {
        list = list.child(settings_subtitle_track_button(option, cx));
    }

    content.child(settings_section("SOURCE TRACKS").child(list))
}

fn settings_subtitle_burn_button(
    config: &ConversionConfig,
    disabled: bool,
    cx: &mut Context<FrameRoot>,
) -> gpui::Stateful<gpui::Div> {
    let colors = button_colors(ButtonVariant::Secondary, false, !disabled);
    let label = subtitle_burn_file_label(config);
    let has_path = config.subtitle_burn_path.is_some();

    let button = div()
        .id("settings-subtitle-burn-file")
        .relative()
        .h(px(SETTINGS_CONTROL_HEIGHT))
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .px(px(10.0))
        .when(has_path, |this| this.pr(px(32.0)))
        .bg(color(colors.background))
        .text_size(px(theme::TEXT_LABEL_SIZE))
        .text_color(color(theme::FOREGROUND))
        .opacity(colors.opacity)
        .shadow(button_highlight_shadows())
        .when(!disabled, |this| {
            this.hover(move |style| {
                style
                    .bg(color(colors.hover_background))
                    .text_color(color(colors.hover_foreground))
                    .cursor_pointer()
            })
            .active(move |style| style.bg(color(colors.active_background)))
        })
        .when(disabled, |this| this.cursor_not_allowed())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            button_mouse_down(!disabled, window, cx);
        })
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            if disabled {
                return;
            }
            root.prompt_subtitle_burn_file(cx);
        }))
        .child(div().truncate().child(label));

    if has_path {
        button.child(settings_subtitle_clear_button(disabled, cx))
    } else {
        button
    }
}

fn settings_subtitle_clear_button(
    disabled: bool,
    cx: &mut Context<FrameRoot>,
) -> gpui::Stateful<gpui::Div> {
    let colors = button_colors(ButtonVariant::Default, true, !disabled);

    div()
        .id("settings-subtitle-clear-file")
        .absolute()
        .right(px(10.0))
        .w(px(20.0))
        .h(px(20.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .bg(color(theme::FRAME_RED))
        .text_color(color(colors.foreground))
        .opacity(if disabled { 0.5 } else { 1.0 })
        .when(!disabled, |this| {
            this.hover(|style| {
                style
                    .bg(color(theme::FRAME_RED.with_alpha(0.86)))
                    .cursor_pointer()
            })
        })
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            if disabled {
                return;
            }
            if root.update_selected_config(|config| apply_subtitle_burn_path(config, None)) {
                cx.notify();
            }
        }))
        .child(icon_svg(assets::ICON_CLOSE, 12.0, color(theme::FOREGROUND)))
}

fn settings_subtitle_style_controls(
    config: &ConversionConfig,
    disabled: bool,
    subtitle_fonts: &[String],
    _window: &Window,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .grid()
                .grid_cols(2)
                .gap_2()
                .child(settings_subtitle_font_select(
                    config,
                    disabled,
                    subtitle_fonts,
                    cx,
                ))
                .child(settings_subtitle_font_size_select(config, disabled, cx)),
        )
        .child(
            div()
                .grid()
                .grid_cols(2)
                .gap_2()
                .child(settings_subtitle_color_field(
                    "TEXT COLOR",
                    "settings-subtitle-font-color",
                    subtitle_color_value(
                        config.subtitle_font_color.as_ref(),
                        DEFAULT_SUBTITLE_FONT_COLOR,
                    ),
                    disabled,
                    true,
                    cx,
                ))
                .child(settings_subtitle_color_field(
                    "OUTLINE COLOR",
                    "settings-subtitle-outline-color",
                    subtitle_color_value(
                        config.subtitle_outline_color.as_ref(),
                        DEFAULT_SUBTITLE_OUTLINE_COLOR,
                    ),
                    disabled,
                    false,
                    cx,
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(settings_field_label("POSITION"))
                .child(settings_subtitle_position_grid(config, disabled, cx)),
        )
        .child(settings_hint_text(
            "Style applies to burned-in subtitles only.",
        ))
}

fn settings_subtitle_font_select(
    config: &ConversionConfig,
    disabled: bool,
    subtitle_fonts: &[String],
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    let selected = config
        .subtitle_font_name
        .as_deref()
        .unwrap_or("Default (e.g. Arial)")
        .to_string();
    let mut visible_fonts = subtitle_font_options(config, subtitle_fonts, disabled)
        .into_iter()
        .take(8)
        .collect::<Vec<_>>();
    if let Some(font) = config.subtitle_font_name.as_ref()
        && !visible_fonts.iter().any(|option| option.name == *font)
    {
        visible_fonts.insert(
            0,
            SubtitleFontOption {
                name: font.clone(),
                is_selected: true,
                is_disabled: disabled,
            },
        );
    }

    let mut list = div().flex().flex_col().gap_1();
    for option in visible_fonts {
        let name = option.name.clone();
        let is_enabled = !option.is_disabled;
        list = list.child(
            settings_choice_button(
                format!("subtitle-font-{name}"),
                option.name,
                option.is_selected,
                is_enabled,
            )
            .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                cx.stop_propagation();
                if !is_enabled {
                    return;
                }
                if root.update_selected_config(|config| apply_subtitle_font_name(config, &name)) {
                    cx.notify();
                }
            })),
        );
    }

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(settings_field_label("FONT"))
        .child(
            div()
                .h(px(SETTINGS_CONTROL_HEIGHT))
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .rounded(px(theme::RADIUS_SM))
                .px(px(10.0))
                .bg(color(theme::FRAME_GRAY_100))
                .shadow(button_highlight_shadows())
                .text_color(color(theme::FOREGROUND))
                .child(div().truncate().child(selected))
                .child(icon_svg(
                    assets::ICON_CHEVRONS_UP_DOWN,
                    12.0,
                    color(theme::FOREGROUND),
                )),
        )
        .child(list)
}

fn settings_subtitle_font_size_select(
    config: &ConversionConfig,
    disabled: bool,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    let mut grid = div().grid().grid_cols(4).gap_1();
    for option in subtitle_font_size_options(config, disabled) {
        let size = option.size;
        let is_enabled = !option.is_disabled;
        grid = grid.child(
            settings_choice_button(
                format!("subtitle-size-{size}"),
                size,
                option.is_selected,
                is_enabled,
            )
            .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                cx.stop_propagation();
                if !is_enabled {
                    return;
                }
                if root.update_selected_config(|config| apply_subtitle_font_size(config, size)) {
                    cx.notify();
                }
            })),
        );
    }

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(settings_field_label("SIZE"))
        .child(grid)
}

fn settings_subtitle_color_field(
    label: &'static str,
    id: &'static str,
    value: String,
    disabled: bool,
    is_font_color: bool,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(settings_field_label(label))
        .child(
            div()
                .id(id)
                .h(px(SETTINGS_CONTROL_HEIGHT))
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .rounded(px(theme::RADIUS_SM))
                .px(px(10.0))
                .bg(color(theme::FRAME_GRAY_100))
                .opacity(if disabled { 0.5 } else { 1.0 })
                .shadow(button_highlight_shadows())
                .when(!disabled, |this| this.cursor_pointer())
                .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                    cx.stop_propagation();
                    if disabled {
                        return;
                    }
                    let changed = if is_font_color {
                        root.update_selected_config(|config| {
                            let next = if subtitle_color_value(
                                config.subtitle_font_color.as_ref(),
                                DEFAULT_SUBTITLE_FONT_COLOR,
                            ) == DEFAULT_SUBTITLE_FONT_COLOR
                            {
                                "#ffd166"
                            } else {
                                DEFAULT_SUBTITLE_FONT_COLOR
                            };
                            apply_subtitle_font_color(config, next)
                        })
                    } else {
                        root.update_selected_config(|config| {
                            let next = if subtitle_color_value(
                                config.subtitle_outline_color.as_ref(),
                                DEFAULT_SUBTITLE_OUTLINE_COLOR,
                            ) == DEFAULT_SUBTITLE_OUTLINE_COLOR
                            {
                                "#1d3557"
                            } else {
                                DEFAULT_SUBTITLE_OUTLINE_COLOR
                            };
                            apply_subtitle_outline_color(config, next)
                        })
                    };
                    if changed {
                        cx.notify();
                    }
                }))
                .child(
                    div()
                        .w(px(18.0))
                        .h(px(18.0))
                        .rounded(px(theme::RADIUS_SM))
                        .bg(parse_hex(&value))
                        .shadow(input_highlight_shadows()),
                )
                .child(div().text_color(color(theme::FOREGROUND)).child(value)),
        )
}

fn settings_subtitle_position_grid(
    config: &ConversionConfig,
    disabled: bool,
    cx: &mut Context<FrameRoot>,
) -> gpui::Div {
    let mut grid = div().grid().grid_cols(3).gap_2();
    for option in subtitle_position_options(config, disabled) {
        let position = option.position;
        let is_enabled = !option.is_disabled;
        grid = grid.child(
            settings_choice_button(
                format!("subtitle-position-{}", position.id()),
                option.label,
                option.is_selected,
                is_enabled,
            )
            .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                cx.stop_propagation();
                if !is_enabled {
                    return;
                }
                if root.update_selected_config(|config| apply_subtitle_position(config, position)) {
                    cx.notify();
                }
            })),
        );
    }

    grid
}

pub(in crate::app) fn settings_subtitle_track_button(
    option: crate::settings::SubtitleTrackOption,
    cx: &mut Context<FrameRoot>,
) -> gpui::Stateful<gpui::Div> {
    let colors = button_colors(
        ButtonVariant::Secondary,
        option.is_selected,
        !option.is_disabled,
    );
    let index = option.index;
    let is_enabled = !option.is_disabled;
    let is_selected = option.is_selected;
    let detail = option.detail;

    div()
        .id(format!("subtitle-track-{index}"))
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
        .when(is_enabled, |this| {
            this.hover(move |style| {
                style
                    .bg(color(colors.hover_background))
                    .text_color(color(colors.hover_foreground))
                    .cursor_pointer()
            })
            .active(move |style| style.bg(color(colors.active_background)))
        })
        .when(!is_enabled, |this| this.cursor_not_allowed())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            button_mouse_down(is_enabled, window, cx);
        })
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            if !is_enabled {
                return;
            }
            if root.update_selected_config(|config| toggle_subtitle_track_selection(config, index))
            {
                cx.notify();
            }
        }))
        .child(
            div()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_color(color(theme::FRAME_GRAY_600))
                        .child(option.index_label),
                )
                .child(
                    div()
                        .text_color(color(theme::FOREGROUND))
                        .child(option.codec),
                )
                .when(!detail.is_empty(), |this| {
                    this.child(
                        div()
                            .truncate()
                            .text_color(color(theme::FRAME_GRAY_600))
                            .child(format!("• {detail}")),
                    )
                }),
        )
        .child(selection_dot(is_selected))
}
