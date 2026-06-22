use frame_gpui_ce::{
    ActiveView, CONTENT_PADDING, FILE_LIST_ROW_SPAN, FILE_ROW_HEIGHT, FrameAppState,
    LEFT_COLUMN_SPAN, LEFT_GRID_ROWS, PANEL_HEADER_HEIGHT, PREVIEW_ROW_SPAN, RIGHT_COLUMN_SPAN,
    TITLEBAR_ACTION_ICON_SIZE, TITLEBAR_BUTTON_HEIGHT, TITLEBAR_DIVIDER_HEIGHT, TITLEBAR_HEIGHT,
    TITLEBAR_ICON_BUTTON_SIZE, TITLEBAR_ICON_SIZE, TITLEBAR_LOGO_SIZE, TITLEBAR_NAV_BUTTON_HEIGHT,
    TITLEBAR_SEGMENT_HEIGHT, TITLEBAR_TOP_PADDING, TITLEBAR_TRAFFIC_LIGHT_SIZE, WINDOW_MIN_HEIGHT,
    WINDOW_MIN_WIDTH, WORKSPACE_COLUMNS, WORKSPACE_GAP,
    assets::{self, FrameAssets},
    file_queue::{
        BatchSelectionState, FileItem, FileQueue, FileStateTone, RowActionAvailability,
        format_file_size,
    },
    format_total_size, theme,
};
use gpui::{
    App, Bounds, BoxShadow, ClickEvent, Context, InteractiveElement, IntoElement, KeyBinding, Menu,
    MenuItem, Render, Rgba, SharedString, StatefulInteractiveElement, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowBounds, WindowControlArea, WindowDecorations, WindowOptions,
    actions, div, hsla, point, prelude::*, px, size, svg,
};

actions!(frame_gpui_ce, [Quit]);

const FILE_LIST_ACTIONS_WIDTH: f32 = 64.0;
const FILE_LIST_ACTION_BUTTON_SIZE: f32 = 24.0;
const FILE_LIST_CHECKBOX_SIZE: f32 = 12.0;

struct FrameRoot {
    active_view: ActiveView,
    file_queue: FileQueue,
    is_processing: bool,
    is_settings_open: bool,
}

impl FrameRoot {
    fn new() -> Self {
        Self {
            active_view: ActiveView::Workspace,
            file_queue: FileQueue::new(),
            is_processing: false,
            is_settings_open: false,
        }
    }

    fn app_state(&self) -> FrameAppState {
        FrameAppState::from_file_queue(self.active_view, self.is_processing, &self.file_queue)
    }
}

impl Render for FrameRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state();
        let content = div().flex_1().p(px(CONTENT_PADDING));
        let content = match state.active_view {
            ActiveView::Workspace => content.child(workspace_view(&self.file_queue, cx)),
            ActiveView::Logs => content.child(logs_view()),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(color(theme::BACKGROUND))
            .text_color(color(theme::FOREGROUND))
            .child(titlebar(state, cx))
            .child(content)
    }
}

fn titlebar(state: FrameAppState, cx: &mut Context<FrameRoot>) -> impl IntoElement {
    div()
        .h(px(TITLEBAR_HEIGHT))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .pt(px(TITLEBAR_TOP_PADDING))
        .window_control_area(WindowControlArea::Drag)
        .text_size(px(theme::TEXT_LABEL_SIZE))
        .child(
            div()
                .flex()
                .items_center()
                .mt_2()
                .gap_6()
                .child(macos_window_controls(cx))
                .child(frame_logo())
                .child(titlebar_divider())
                .child(titlebar_navigation(state.active_view, cx))
                .child(titlebar_divider())
                .child(titlebar_stats(state)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .mt_2()
                .gap_2()
                .child(
                    action_button(assets::ICON_SETTINGS, None, ButtonVariant::Secondary, true)
                        .id("titlebar-settings")
                        .on_click(cx.listener(|root, _: &ClickEvent, _window, cx| {
                            root.is_settings_open = !root.is_settings_open;
                            cx.notify();
                        })),
                )
                .child(
                    action_button(
                        assets::ICON_PLUS,
                        Some("ADD SOURCE"),
                        ButtonVariant::Secondary,
                        true,
                    )
                    .id("titlebar-add-source")
                    .on_click(cx.listener(|_, _: &ClickEvent, _window, cx| {
                        cx.stop_propagation();
                    })),
                )
                .child(action_button(
                    assets::ICON_PLAY,
                    Some(if state.is_processing {
                        "PROCESSING"
                    } else {
                        "START"
                    }),
                    ButtonVariant::Default,
                    state.can_start_conversion(),
                )),
        )
}

fn macos_window_controls(cx: &mut Context<FrameRoot>) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .mr_2()
        .child(
            traffic_light("#ff5f56", "#e0443e")
                .id("titlebar-close")
                .window_control_area(WindowControlArea::Close)
                .on_click(cx.listener(|_, _: &ClickEvent, window, cx| {
                    cx.stop_propagation();
                    window.remove_window();
                })),
        )
        .child(
            traffic_light("#ffbd2e", "#dea123")
                .id("titlebar-minimize")
                .window_control_area(WindowControlArea::Min)
                .on_click(cx.listener(|_, _: &ClickEvent, window, cx| {
                    cx.stop_propagation();
                    window.minimize_window();
                })),
        )
        .child(
            traffic_light("#27c93f", "#1aab29")
                .id("titlebar-zoom")
                .window_control_area(WindowControlArea::Max)
                .on_click(cx.listener(|_, _: &ClickEvent, window, cx| {
                    cx.stop_propagation();
                    window.zoom_window();
                })),
        )
}

fn traffic_light(fill: &'static str, border: &'static str) -> gpui::Div {
    div()
        .w(px(TITLEBAR_TRAFFIC_LIGHT_SIZE))
        .h(px(TITLEBAR_TRAFFIC_LIGHT_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .cursor_pointer()
        .child(
            div()
                .w(px(12.0))
                .h(px(12.0))
                .rounded_full()
                .bg(parse_hex(fill))
                .border_1()
                .border_color(parse_hex(border)),
        )
}

fn frame_logo() -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .px_2()
        .text_color(color(theme::FRAME_GRAY_600))
        .child(
            svg()
                .path(assets::ICON_FRAME)
                .w(px(TITLEBAR_LOGO_SIZE))
                .h(px(TITLEBAR_LOGO_SIZE))
                .text_color(color(theme::FRAME_GRAY_600)),
        )
}

fn titlebar_divider() -> gpui::Div {
    div()
        .h(px(TITLEBAR_DIVIDER_HEIGHT))
        .w(px(1.0))
        .bg(color(theme::FRAME_GRAY_100))
}

fn titlebar_navigation(active_view: ActiveView, cx: &mut Context<FrameRoot>) -> gpui::Div {
    div()
        .h(px(TITLEBAR_SEGMENT_HEIGHT))
        .flex()
        .items_center()
        .gap_1()
        .rounded(px(theme::RADIUS_MD))
        .bg(color(theme::FRAME_GRAY_100))
        .px(px(3.0))
        .py(px(2.0))
        .shadow(input_highlight_shadows())
        .child(titlebar_segment(
            assets::ICON_LAYOUT_LIST,
            "WORKSPACE",
            ActiveView::Workspace,
            active_view == ActiveView::Workspace,
            cx,
        ))
        .child(titlebar_segment(
            assets::ICON_TERMINAL,
            "LOGS",
            ActiveView::Logs,
            active_view == ActiveView::Logs,
            cx,
        ))
}

fn titlebar_stats(state: FrameAppState) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap_4()
        .text_color(color(theme::FRAME_GRAY_600))
        .child(titlebar_stat(
            assets::ICON_HARD_DRIVE,
            format!("STORAGE {}", format_total_size(state.total_size_bytes)),
        ))
        .child(titlebar_stat(
            assets::ICON_FILE_VIDEO,
            format!("ITEMS {}", state.file_count),
        ))
}

fn titlebar_stat(icon: &'static str, label: String) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(icon_svg(
            icon,
            TITLEBAR_ICON_SIZE,
            color(theme::FRAME_GRAY_600),
        ))
        .child(label)
}

fn titlebar_segment(
    icon: &'static str,
    label: &'static str,
    view: ActiveView,
    selected: bool,
    cx: &mut Context<FrameRoot>,
) -> impl IntoElement {
    div()
        .h(px(TITLEBAR_NAV_BUTTON_HEIGHT))
        .flex()
        .items_center()
        .gap_2()
        .rounded(px(theme::RADIUS_SM))
        .id(match view {
            ActiveView::Workspace => "titlebar-workspace",
            ActiveView::Logs => "titlebar-logs",
        })
        .px_2()
        .bg(if selected {
            color(theme::FRAME_GRAY_400)
        } else {
            color(theme::TRANSPARENT)
        })
        .text_color(if selected {
            color(theme::FOREGROUND)
        } else {
            color(theme::FRAME_GRAY_600)
        })
        .when(selected, |this| this.shadow(button_highlight_shadows()))
        .hover(|style| style.text_color(color(theme::FOREGROUND)).cursor_pointer())
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            if root.active_view != view {
                root.active_view = view;
                cx.notify();
            }
            cx.stop_propagation();
        }))
        .child(icon_svg(
            icon,
            TITLEBAR_ICON_SIZE,
            if selected {
                color(theme::FOREGROUND)
            } else {
                color(theme::FRAME_GRAY_600)
            },
        ))
        .child(label)
}

#[derive(Clone, Copy)]
enum ButtonVariant {
    Default,
    Secondary,
}

fn action_button(
    icon: &'static str,
    label: Option<&'static str>,
    variant: ButtonVariant,
    enabled: bool,
) -> gpui::Div {
    let is_icon_only = label.is_none();
    let background = match (variant, enabled) {
        (ButtonVariant::Default, true) => theme::FRAME_GRAY_400,
        (ButtonVariant::Default, false) => theme::FRAME_GRAY_400.with_alpha(0.10),
        (ButtonVariant::Secondary, true | false) => theme::FRAME_GRAY_100,
    };
    let button_icon_color = if enabled {
        color(theme::FOREGROUND)
    } else {
        color(theme::FOREGROUND.with_alpha(0.50))
    };

    let button = div()
        .h(px(TITLEBAR_BUTTON_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .rounded(px(theme::RADIUS_SM))
        .bg(color(background))
        .shadow(button_highlight_shadows())
        .text_color(if enabled {
            color(theme::FOREGROUND)
        } else {
            color(theme::FOREGROUND.with_alpha(0.50))
        })
        .when(enabled, |this| {
            this.hover(|style| style.bg(color(theme::FRAME_GRAY_200)).cursor_pointer())
        });

    if is_icon_only {
        button.w(px(TITLEBAR_ICON_BUTTON_SIZE)).child(icon_svg(
            icon,
            TITLEBAR_ACTION_ICON_SIZE,
            button_icon_color,
        ))
    } else {
        button
            .px(px(10.0))
            .child(icon_svg(icon, TITLEBAR_ICON_SIZE, button_icon_color))
            .child(label.unwrap_or_default())
    }
}

fn icon_svg(path: &'static str, size: f32, icon_color: Rgba) -> impl IntoElement {
    svg()
        .path(path)
        .w(px(size))
        .h(px(size))
        .text_color(icon_color)
}

fn parse_hex(hex: &str) -> Rgba {
    let hex = hex.trim_start_matches('#');
    let red = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let green = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let blue = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    color(theme::RgbaToken::from_rgb(red, green, blue))
}

fn input_highlight_shadows() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.20),
            offset: point(px(0.0), px(0.5)),
            blur_radius: px(0.0),
            spread_radius: px(0.0),
            inset: true,
        },
        BoxShadow {
            color: color(theme::FRAME_GRAY_400).into(),
            offset: point(px(0.0), px(-0.5)),
            blur_radius: px(0.0),
            spread_radius: px(0.0),
            inset: true,
        },
    ]
}

fn button_highlight_shadows() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: color(theme::FRAME_GRAY_400).into(),
            offset: point(px(0.0), px(0.5)),
            blur_radius: px(0.0),
            spread_radius: px(0.0),
            inset: true,
        },
        BoxShadow {
            color: color(theme::FRAME_GRAY_200).into(),
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(0.5),
            inset: true,
        },
    ]
}

fn workspace_view(file_queue: &FileQueue, cx: &mut Context<FrameRoot>) -> gpui::Div {
    div()
        .grid()
        .grid_cols(WORKSPACE_COLUMNS)
        .gap(px(WORKSPACE_GAP))
        .size_full()
        .child(
            div()
                .col_span(LEFT_COLUMN_SPAN)
                .grid()
                .grid_rows(LEFT_GRID_ROWS)
                .gap(px(WORKSPACE_GAP))
                .size_full()
                .child(
                    panel("PREVIEW")
                        .row_span(PREVIEW_ROW_SPAN)
                        .items_center()
                        .justify_center(),
                )
                .child(file_list_panel(file_queue, cx).row_span(FILE_LIST_ROW_SPAN)),
        )
        .child(
            panel("SETTINGS")
                .col_span(RIGHT_COLUMN_SPAN)
                .items_center()
                .justify_center(),
        )
}

fn logs_view() -> gpui::Div {
    panel("LOGS").size_full().items_center().justify_center()
}

fn file_list_panel(queue: &FileQueue, cx: &mut Context<FrameRoot>) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .overflow_hidden()
        .card_surface()
        .child(file_list_header(queue.batch_selection_state(), cx))
        .child(file_list_body(queue, cx))
}

fn file_list_header(selection: BatchSelectionState, cx: &mut Context<FrameRoot>) -> gpui::Div {
    div()
        .h(px(PANEL_HEADER_HEIGHT))
        .w_full()
        .flex()
        .items_center()
        .px_4()
        .border_b_1()
        .border_color(color(theme::FRAME_GRAY_100))
        .child(
            div()
                .flex_1()
                .grid()
                .grid_cols(12)
                .gap(px(WORKSPACE_GAP))
                .items_center()
                .text_size(px(theme::TEXT_LABEL_SIZE))
                .text_color(color(theme::FRAME_GRAY_600))
                .child(
                    div().col_span(1).flex().items_center().child(
                        checkbox_hit_area(
                            selection.is_checked,
                            selection.is_indeterminate,
                            selection.is_enabled,
                        )
                        .id("file-list-header-checkbox")
                        .when(selection.is_enabled, |this| this.cursor_pointer())
                        .on_click(cx.listener(
                            |root, _: &ClickEvent, _window, cx| {
                                if !root.file_queue.files().is_empty() {
                                    root.file_queue.toggle_all_batch_selection();
                                    cx.notify();
                                }
                            },
                        )),
                    ),
                )
                .child(header_label("NAME", 5, false))
                .child(header_label("SIZE", 2, true))
                .child(header_label("TARGET", 2, true))
                .child(header_label("STATE", 2, true)),
        )
        .child(
            div()
                .ml_4()
                .w(px(FILE_LIST_ACTIONS_WIDTH))
                .text_size(px(theme::TEXT_LABEL_SIZE))
                .text_color(color(theme::FRAME_GRAY_600))
                .text_right()
                .child("ACTIONS"),
        )
}

fn file_list_body(queue: &FileQueue, cx: &mut Context<FrameRoot>) -> impl IntoElement {
    let body = div()
        .id("file-list-body")
        .flex_1()
        .flex()
        .flex_col()
        .overflow_y_scroll();
    if queue.files().is_empty() {
        return body.child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(theme::TEXT_ROW_SIZE))
                .text_color(color(theme::FRAME_GRAY_600))
                .child("Drop files or use Add Source"),
        );
    }

    let mut body = body;
    for file in queue.files() {
        body = body.child(file_list_row(
            file,
            queue.selected_file_id() == Some(file.id.as_str()),
            cx,
        ));
    }
    body
}

fn file_list_row(
    file: &FileItem,
    is_selected: bool,
    cx: &mut Context<FrameRoot>,
) -> impl IntoElement {
    let group_name = format!("file-list-row-{}", file.id);
    let select_id = file.id.clone();

    div()
        .h(px(FILE_ROW_HEIGHT))
        .w_full()
        .id(element_id("file-list-row", &select_id))
        .group(group_name.clone())
        .flex()
        .items_center()
        .px_4()
        .border_b_1()
        .border_color(color(theme::FRAME_GRAY_100))
        .bg(if is_selected {
            color(theme::FRAME_GRAY_100)
        } else {
            color(theme::TRANSPARENT)
        })
        .hover(|style| style.bg(color(theme::FRAME_GRAY_100)).cursor_pointer())
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            if root.file_queue.select_existing_file(&select_id) {
                cx.notify();
            }
        }))
        .child(
            div()
                .flex_1()
                .grid()
                .grid_cols(12)
                .gap(px(WORKSPACE_GAP))
                .items_center()
                .text_size(px(theme::TEXT_ROW_SIZE))
                .child(
                    div()
                        .col_span(1)
                        .flex()
                        .items_center()
                        .child(row_checkbox_control(
                            file.id.clone(),
                            file.is_selected_for_conversion,
                            cx,
                        )),
                )
                .child(row_label(
                    file.name.clone(),
                    5,
                    false,
                    color(theme::FOREGROUND),
                ))
                .child(row_label(
                    format_file_size(file.size_bytes),
                    2,
                    true,
                    color(theme::FRAME_GRAY_600),
                ))
                .child(row_label(
                    file.original_format.clone(),
                    2,
                    true,
                    color(theme::FRAME_GRAY_600),
                ))
                .child(row_label(
                    file.row_state_label(),
                    2,
                    true,
                    state_tone_color(file.row_state_tone()),
                )),
        )
        .child(row_actions_cell(
            file.id.clone(),
            file.row_actions(),
            group_name,
            cx,
        ))
}

fn header_label(label: &'static str, span: u16, align_right: bool) -> gpui::Div {
    let cell = div().col_span(span).truncate();
    let cell = if align_right { cell.text_right() } else { cell };
    cell.child(label)
}

fn row_label(label: String, span: u16, align_right: bool, text_color: Rgba) -> gpui::Div {
    let cell = div()
        .col_span(span)
        .truncate()
        .whitespace_nowrap()
        .text_color(text_color);
    let cell = if align_right { cell.text_right() } else { cell };
    cell.child(label)
}

fn row_actions_cell(
    file_id: String,
    actions: RowActionAvailability,
    group_name: String,
    cx: &mut Context<FrameRoot>,
) -> impl IntoElement {
    let mut cell = div()
        .id(element_id("file-row-actions", &file_id))
        .ml_4()
        .w(px(FILE_LIST_ACTIONS_WIDTH))
        .h_full()
        .flex()
        .items_center()
        .justify_end()
        .gap_1()
        .opacity(0.0)
        .group_hover(group_name, |style| style.opacity(1.0))
        .on_click(cx.listener(|_, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
        }));

    if actions.can_pause {
        let id = file_id.clone();
        cell = cell.child(
            row_action_button("||", true)
                .id(element_id("file-row-action-pause", &id))
                .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                    cx.stop_propagation();
                    if root.file_queue.pause_file(&id) {
                        cx.notify();
                    }
                })),
        );
    }
    if actions.can_resume {
        let id = file_id.clone();
        cell = cell.child(
            row_action_button(">", true)
                .id(element_id("file-row-action-resume", &id))
                .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                    cx.stop_propagation();
                    if root.file_queue.resume_file(&id) {
                        cx.notify();
                    }
                })),
        );
    }

    if actions.can_delete {
        let id = file_id;
        cell.child(
            row_action_button("X", true)
                .id(element_id("file-row-action-delete", &id))
                .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
                    cx.stop_propagation();
                    if root.file_queue.remove_interactive_file(&id).is_some() {
                        cx.notify();
                    }
                })),
        )
    } else {
        cell.child(row_action_button("X", false))
    }
}

fn row_action_button(label: &'static str, enabled: bool) -> gpui::Div {
    div()
        .w(px(FILE_LIST_ACTION_BUTTON_SIZE))
        .h(px(FILE_LIST_ACTION_BUTTON_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_MD))
        .text_size(px(theme::TEXT_LABEL_SIZE))
        .bg(if enabled {
            color(theme::TRANSPARENT)
        } else {
            color(theme::FRAME_GRAY_100)
        })
        .text_color(if enabled {
            color(theme::FOREGROUND)
        } else {
            color(theme::FRAME_GRAY_400)
        })
        .when(enabled, |this| {
            this.hover(|style| style.bg(color(theme::FRAME_GRAY_200)).cursor_pointer())
        })
        .child(label)
}

fn row_checkbox_control(
    file_id: String,
    is_checked: bool,
    cx: &mut Context<FrameRoot>,
) -> impl IntoElement {
    checkbox_hit_area(is_checked, false, true)
        .id(element_id("file-row-checkbox", &file_id))
        .cursor_pointer()
        .on_click(cx.listener(move |root, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            if root.file_queue.toggle_batch_selection(&file_id).is_some() {
                cx.notify();
            }
        }))
}

fn checkbox_hit_area(is_checked: bool, is_indeterminate: bool, is_enabled: bool) -> gpui::Div {
    div()
        .w(px(theme::MIN_HIT_AREA))
        .h(px(FILE_ROW_HEIGHT))
        .flex()
        .items_center()
        .justify_start()
        .child(checkbox_indicator(is_checked, is_indeterminate, is_enabled))
}

fn checkbox_indicator(is_checked: bool, is_indeterminate: bool, is_enabled: bool) -> gpui::Div {
    let is_active = is_checked || is_indeterminate;
    let mut indicator = div()
        .w(px(FILE_LIST_CHECKBOX_SIZE))
        .h(px(FILE_LIST_CHECKBOX_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_XS))
        .border_1()
        .border_color(if is_active || is_enabled {
            color(theme::FRAME_GRAY_600)
        } else {
            color(theme::FRAME_GRAY_200)
        })
        .bg(if is_active {
            color(theme::FRAME_GRAY_600)
        } else {
            color(theme::TRANSPARENT)
        });

    if is_indeterminate {
        indicator = indicator.child(
            div()
                .w(px(6.0))
                .h(px(2.0))
                .rounded(px(theme::RADIUS_XS))
                .bg(color(theme::FOREGROUND)),
        );
    }

    indicator
}

fn state_tone_color(tone: FileStateTone) -> Rgba {
    match tone {
        FileStateTone::Foreground => color(theme::FOREGROUND),
        FileStateTone::Muted => color(theme::FRAME_GRAY_600),
        FileStateTone::Amber => color(theme::FRAME_AMBER),
        FileStateTone::Red => color(theme::FRAME_RED),
    }
}

fn element_id(prefix: &str, id: &str) -> String {
    format!("{prefix}-{id}")
}

fn panel(label: &'static str) -> gpui::Div {
    div()
        .flex()
        .card_surface()
        .text_xs()
        .text_color(color(theme::FRAME_GRAY_600))
        .child(label)
}

trait FrameSurface {
    fn card_surface(self) -> Self;
}

impl FrameSurface for gpui::Div {
    fn card_surface(self) -> Self {
        self.rounded(px(theme::RADIUS_LG))
            .bg(color(theme::FRAME_GRAY_100))
            .shadow(card_surface_shadows())
    }
}

fn card_surface_shadows() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.10),
            offset: point(px(0.0), px(4.0)),
            blur_radius: px(6.0),
            spread_radius: px(-1.0),
            inset: false,
        },
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.10),
            offset: point(px(0.0), px(2.0)),
            blur_radius: px(4.0),
            spread_radius: px(-2.0),
            inset: false,
        },
        BoxShadow {
            color: color(theme::FRAME_GRAY_200).into(),
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(0.0),
            spread_radius: px(0.0),
            inset: true,
        },
        BoxShadow {
            color: color(theme::FRAME_GRAY_100).into(),
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            inset: true,
        },
    ]
}

fn color(token: theme::RgbaToken) -> Rgba {
    Rgba {
        r: token.red,
        g: token.green,
        b: token.blue,
        a: token.alpha,
    }
}

fn init_app(cx: &mut App, name: impl Into<SharedString>) {
    cx.activate(true);
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    cx.set_menus(vec![Menu {
        name: name.into(),
        items: vec![MenuItem::action("Quit", Quit)],
        disabled: false,
    }]);
    cx.on_window_closed(|cx, _| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
}

fn main() {
    gpui_platform::application()
        .with_assets(FrameAssets)
        .run(|cx| {
            let bounds =
                Bounds::centered(None, size(px(WINDOW_MIN_WIDTH), px(WINDOW_MIN_HEIGHT)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Frame".into()),
                        appears_transparent: true,
                        traffic_light_position: None,
                    }),
                    window_min_size: Some(size(px(WINDOW_MIN_WIDTH), px(WINDOW_MIN_HEIGHT))),
                    window_background: WindowBackgroundAppearance::Opaque,
                    window_decorations: Some(WindowDecorations::Client),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| FrameRoot::new()),
            )
            .expect("failed to open Frame GPUI window");

            init_app(cx, "Frame");
        });
}
