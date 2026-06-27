use super::*;

pub(in crate::app) const FRAME_SLIDER_VISUAL_HEIGHT: f32 = 20.0;
pub(in crate::app) const FRAME_SLIDER_TRACK_HEIGHT: f32 = 4.0;
pub(in crate::app) const FRAME_SLIDER_TRACK_TOP: f32 = 8.0;
pub(in crate::app) const FRAME_SLIDER_HANDLE_WIDTH: f32 = 10.0;
pub(in crate::app) const FRAME_SLIDER_HANDLE_HEIGHT: f32 = 14.0;
pub(in crate::app) const FRAME_SLIDER_HANDLE_TOP: f32 = 3.0;

pub(in crate::app) fn frame_slider(
    id: &'static str,
    fraction: f32,
    disabled: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .relative()
        .h(px(FRAME_SLIDER_VISUAL_HEIGHT))
        .w_full()
        .opacity(if disabled { 0.5 } else { 1.0 })
        .when(!disabled, |this| this.cursor_pointer())
        .child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(px(FRAME_SLIDER_TRACK_TOP))
                .h(px(FRAME_SLIDER_TRACK_HEIGHT))
                .rounded(px(FRAME_SLIDER_TRACK_HEIGHT / 2.0))
                .bg(color(theme::FRAME_GRAY_100))
                .shadow(input_highlight_shadows()),
        )
        .child(
            div()
                .absolute()
                .left_0()
                .top(px(FRAME_SLIDER_TRACK_TOP))
                .h(px(FRAME_SLIDER_TRACK_HEIGHT))
                .w(relative(fraction.clamp(0.0, 1.0)))
                .rounded(px(FRAME_SLIDER_TRACK_HEIGHT / 2.0))
                .bg(color(theme::FOREGROUND)),
        )
}

pub(in crate::app) fn frame_slider_handle(
    id: &'static str,
    fraction: f32,
    enabled: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .absolute()
        .left(relative(fraction.clamp(0.0, 1.0)))
        .top(px(FRAME_SLIDER_HANDLE_TOP))
        .ml(px(-(FRAME_SLIDER_HANDLE_WIDTH / 2.0)))
        .w(px(FRAME_SLIDER_HANDLE_WIDTH))
        .h(px(FRAME_SLIDER_HANDLE_HEIGHT))
        .rounded(px(FRAME_SLIDER_HANDLE_WIDTH / 2.0))
        .bg(color(theme::FOREGROUND))
        .shadow(button_highlight_shadows())
        .when(enabled, |this| this.cursor_ew_resize())
}
