use super::*;

pub(super) const SETTINGS_SHEET_MOTION_DURATION: Duration = Duration::from_millis(200);
pub(super) const SUBTITLE_POPOVER_MOTION_DURATION: Duration = Duration::from_millis(140);
pub(super) const MOTION_DONE_EPSILON: f32 = 0.001;

const SETTINGS_SHEET_SLIDE_DISTANCE: f32 = 24.0;
const SETTINGS_SHEET_EDGE_INSET: f32 = 8.0;
const SUBTITLE_POPOVER_SLIDE_DISTANCE: f32 = 4.0;

pub(super) fn motion_target(is_open: bool) -> f32 {
    if is_open { 1.0 } else { 0.0 }
}

pub(super) fn motion_is_hidden(progress: f32) -> bool {
    progress <= MOTION_DONE_EPSILON
}

pub(super) fn settings_sheet_slide_offset(progress: f32) -> f32 {
    (1.0 - progress.clamp(0.0, 1.0)) * SETTINGS_SHEET_SLIDE_DISTANCE
}

pub(super) fn settings_sheet_right_inset(progress: f32) -> f32 {
    SETTINGS_SHEET_EDGE_INSET - settings_sheet_slide_offset(progress)
}

pub(super) fn subtitle_popover_slide_offset(progress: f32) -> f32 {
    (1.0 - progress.clamp(0.0, 1.0)) * SUBTITLE_POPOVER_SLIDE_DISTANCE
}
