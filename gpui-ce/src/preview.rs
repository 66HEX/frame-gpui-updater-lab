//! Preview geometry helpers for the GPUI rewrite.

use crate::settings::ProcessingMode;
use frame_core::media_rules;

pub const MIN_CROP: f64 = 0.05;
pub const DEFAULT_OVERLAY_WIDTH: f64 = 0.18;
pub const MIN_OVERLAY_WIDTH: f64 = 0.03;
pub const MAX_OVERLAY_WIDTH: f64 = 0.8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MetadataStatus {
    #[default]
    Idle,
    Loading,
    Ready,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceMediaKind {
    Video,
    Audio,
    Image,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PreviewMediaKind {
    #[default]
    Unknown,
    Video,
    Audio,
    Image,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewControlInput<'a> {
    pub metadata_status: MetadataStatus,
    pub source_media_kind: Option<SourceMediaKind>,
    pub controls_disabled: bool,
    pub processing_mode: ProcessingMode,
    pub container: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewControlAvailability {
    pub media_kind: PreviewMediaKind,
    pub hide_visual_controls: bool,
    pub trim_disabled: bool,
    pub overlay_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineDragTarget {
    Start,
    End,
    Scrub,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediaSnapshot {
    pub current_time: f64,
    pub duration: f64,
    pub paused: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlaybackMediaCommand {
    pub seek_to: Option<f64>,
    pub pause: bool,
    pub play: bool,
}

impl PlaybackMediaCommand {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            seek_to: None,
            pause: false,
            play: false,
        }
    }

    #[must_use]
    pub const fn seek(time: f64) -> Self {
        Self {
            seek_to: Some(time),
            pause: false,
            play: false,
        }
    }

    #[must_use]
    pub const fn pause() -> Self {
        Self {
            seek_to: None,
            pause: true,
            play: false,
        }
    }

    #[must_use]
    pub const fn play() -> Self {
        Self {
            seek_to: None,
            pause: false,
            play: true,
        }
    }

    #[must_use]
    pub const fn pause_and_seek(time: f64) -> Self {
        Self {
            seek_to: Some(time),
            pause: true,
            play: false,
        }
    }

    #[must_use]
    pub const fn seek_and_play(time: f64) -> Self {
        Self {
            seek_to: Some(time),
            pause: false,
            play: true,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrimSelection {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineDragUpdate {
    pub command: PlaybackMediaCommand,
    pub trim: Option<TrimSelection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineDragEnd {
    pub command: PlaybackMediaCommand,
    pub trim: Option<TrimSelection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewPlaybackState {
    is_image: bool,
    has_media: bool,
    is_playing: bool,
    current_time: f64,
    duration: f64,
    start_value: f64,
    end_value: f64,
    dragging: Option<TimelineDragTarget>,
    was_playing_before_scrub: bool,
    previous_initial_start: Option<String>,
    previous_initial_end: Option<String>,
}

impl PreviewPlaybackState {
    #[must_use]
    pub const fn new(is_image: bool) -> Self {
        Self {
            is_image,
            has_media: false,
            is_playing: false,
            current_time: 0.0,
            duration: 0.0,
            start_value: 0.0,
            end_value: 0.0,
            dragging: None,
            was_playing_before_scrub: false,
            previous_initial_start: None,
            previous_initial_end: None,
        }
    }

    #[must_use]
    pub const fn is_playing(&self) -> bool {
        self.is_playing
    }

    #[must_use]
    pub const fn current_time(&self) -> f64 {
        self.current_time
    }

    #[must_use]
    pub const fn duration(&self) -> f64 {
        self.duration
    }

    #[must_use]
    pub const fn start_value(&self) -> f64 {
        self.start_value
    }

    #[must_use]
    pub const fn end_value(&self) -> f64 {
        self.end_value
    }

    #[must_use]
    pub const fn dragging(&self) -> Option<TimelineDragTarget> {
        self.dragging
    }

    pub const fn set_is_image(&mut self, is_image: bool) {
        self.is_image = is_image;
    }

    pub fn clear_media(&mut self) {
        self.has_media = false;
        self.is_playing = false;
        self.current_time = 0.0;
        self.duration = 0.0;
        self.start_value = 0.0;
        self.end_value = 0.0;
    }

    pub fn sync_media(&mut self, snapshot: MediaSnapshot) {
        self.has_media = true;
        self.is_playing = !snapshot.paused;
        self.current_time = finite_or_zero(snapshot.current_time);
        self.sync_from_media(snapshot);
    }

    pub fn sync_initial_values(&mut self, initial_start: Option<&str>, initial_end: Option<&str>) {
        if self.previous_initial_start.as_deref() != initial_start {
            self.previous_initial_start = initial_start.map(str::to_string);
            self.start_value = initial_start.map_or(0.0, parse_time_to_seconds);
        }

        if self.previous_initial_end.as_deref() != initial_end {
            self.previous_initial_end = initial_end.map(str::to_string);
            if let Some(initial_end) = initial_end {
                self.end_value = parse_time_to_seconds(initial_end);
            } else if self.duration > 0.0 {
                self.end_value = self.duration;
            }
        } else if initial_end.is_none() && self.duration > 0.0 && self.end_value == 0.0 {
            self.end_value = self.duration;
        }
    }

    pub fn sync_from_media(&mut self, snapshot: MediaSnapshot) {
        if self.is_image {
            return;
        }

        self.duration = finite_or_zero(snapshot.duration);
        self.current_time = finite_or_zero(snapshot.current_time);
        self.end_value = self
            .previous_initial_end
            .as_deref()
            .map_or(self.duration, parse_time_to_seconds);

        if self.start_value > self.duration {
            self.start_value = 0.0;
        }
        if self.end_value > self.duration {
            self.end_value = self.duration;
        }
    }

    pub const fn handle_play(&mut self) {
        self.is_playing = true;
    }

    pub const fn handle_pause(&mut self) {
        self.is_playing = false;
    }

    #[must_use]
    pub fn handle_time_update(&mut self, current_time: f64) -> PlaybackMediaCommand {
        if self.is_image || !self.has_media {
            return PlaybackMediaCommand::none();
        }

        self.current_time = finite_or_zero(current_time);
        if self.dragging.is_some() {
            return PlaybackMediaCommand::none();
        }

        if self.current_time >= self.end_value && self.end_value > self.start_value {
            self.is_playing = false;
            self.current_time = self.start_value;
            return PlaybackMediaCommand::pause_and_seek(self.start_value);
        }

        PlaybackMediaCommand::none()
    }

    #[must_use]
    pub fn toggle_play(&mut self) -> PlaybackMediaCommand {
        if self.is_image || !self.has_media {
            return PlaybackMediaCommand::none();
        }

        if self.is_playing {
            return PlaybackMediaCommand::pause();
        }

        if self.current_time < self.start_value || self.current_time >= self.end_value {
            self.current_time = self.start_value;
            return PlaybackMediaCommand::seek_and_play(self.start_value);
        }

        PlaybackMediaCommand::play()
    }

    #[must_use]
    pub fn commit_trim_values(&self) -> Option<TrimSelection> {
        if self.is_image {
            return None;
        }

        Some(TrimSelection {
            start_time: (self.start_value > 0.0).then(|| format_time(self.start_value)),
            end_time: (self.duration > 0.0 && self.end_value < self.duration)
                .then(|| format_time(self.end_value)),
        })
    }

    #[must_use]
    pub fn set_start_from_input(&mut self, value: f64) -> Option<PlaybackMediaCommand> {
        if value >= 0.0 && value < self.end_value {
            self.start_value = value;
            return Some(PlaybackMediaCommand::seek(value));
        }

        None
    }

    #[must_use]
    pub fn set_end_from_input(&mut self, value: f64) -> Option<PlaybackMediaCommand> {
        if value > self.start_value && value <= self.duration {
            self.end_value = value;
            return Some(PlaybackMediaCommand::seek(value));
        }

        None
    }

    pub fn begin_handle_drag(&mut self, target: TimelineDragTarget) -> bool {
        if self.is_image {
            return false;
        }

        self.dragging = Some(target);
        true
    }

    #[must_use]
    pub fn seek_to_percent(&mut self, percent: f64) -> PlaybackMediaCommand {
        if self.is_image {
            return PlaybackMediaCommand::none();
        }

        let time = self.time_from_slider_percent(percent);
        self.current_time = time;
        self.dragging = Some(TimelineDragTarget::Scrub);
        self.was_playing_before_scrub = self.is_playing;

        if self.is_playing {
            PlaybackMediaCommand::pause_and_seek(time)
        } else {
            PlaybackMediaCommand::seek(time)
        }
    }

    #[must_use]
    pub fn drag_to_percent(&mut self, percent: f64) -> TimelineDragUpdate {
        if self.is_image {
            return TimelineDragUpdate {
                command: PlaybackMediaCommand::none(),
                trim: None,
            };
        }

        let Some(dragging) = self.dragging else {
            return TimelineDragUpdate {
                command: PlaybackMediaCommand::none(),
                trim: None,
            };
        };

        let time = self.time_from_slider_percent(percent);
        match dragging {
            TimelineDragTarget::Scrub => {
                self.current_time = time;
                TimelineDragUpdate {
                    command: PlaybackMediaCommand::seek(time),
                    trim: None,
                }
            }
            TimelineDragTarget::Start => {
                self.start_value = time.min(self.end_value - 1.0);
                TimelineDragUpdate {
                    command: PlaybackMediaCommand::seek(self.start_value),
                    trim: self.commit_trim_values(),
                }
            }
            TimelineDragTarget::End => {
                self.end_value = time.max(self.start_value + 1.0);
                TimelineDragUpdate {
                    command: PlaybackMediaCommand::seek(self.end_value),
                    trim: self.commit_trim_values(),
                }
            }
        }
    }

    #[must_use]
    pub fn end_drag(&mut self) -> TimelineDragEnd {
        let command =
            if self.dragging == Some(TimelineDragTarget::Scrub) && self.was_playing_before_scrub {
                PlaybackMediaCommand::play()
            } else {
                PlaybackMediaCommand::none()
            };
        let trim = if self
            .dragging
            .is_some_and(|target| target != TimelineDragTarget::Scrub)
        {
            self.commit_trim_values()
        } else {
            None
        };

        self.dragging = None;
        self.was_playing_before_scrub = false;

        TimelineDragEnd { command, trim }
    }

    #[must_use]
    pub fn to_timeline_percent(&self, value: f64) -> f64 {
        if !self.duration.is_finite() || self.duration <= 0.0 {
            return 0.0;
        }

        (value / self.duration) * 100.0
    }

    fn time_from_slider_percent(&self, percent: f64) -> f64 {
        clamp(percent, 0.0, 1.0) * self.duration
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CropRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DragHandle {
    Move,
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl From<&str> for PreviewRotation {
    fn from(value: &str) -> Self {
        match value.trim() {
            "90" => Self::Deg90,
            "180" => Self::Deg180,
            "270" => Self::Deg270,
            _ => Self::Deg0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectOption {
    pub id: &'static str,
    pub display: &'static str,
    pub ratio: Option<f64>,
}

pub const ASPECT_OPTIONS: [AspectOption; 5] = [
    AspectOption {
        id: "free",
        display: "Free",
        ratio: None,
    },
    AspectOption {
        id: "1:1",
        display: "1:1",
        ratio: Some(1.0),
    },
    AspectOption {
        id: "4:5",
        display: "4:5",
        ratio: Some(4.0 / 5.0),
    },
    AspectOption {
        id: "16:9",
        display: "16:9",
        ratio: Some(16.0 / 9.0),
    },
    AspectOption {
        id: "9:16",
        display: "9:16",
        ratio: Some(9.0 / 16.0),
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragDelta {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualCropDrag<'a> {
    pub start_rect: CropRect,
    pub handle: DragHandle,
    pub start_point: Point,
    pub current_point: Point,
    pub aspect_id: &'a str,
    pub source_width: f64,
    pub source_height: f64,
    pub is_side_rotation: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayDragHandle {
    Move,
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlaySizeDirection {
    Increase,
    Decrease,
}

impl OverlaySizeDirection {
    const fn step(self) -> f64 {
        match self {
            Self::Increase => 0.025,
            Self::Decrease => -0.025,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayDragPoint {
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewOverlay {
    pub enabled: bool,
    pub path: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub opacity: f64,
    pub anchor: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayModeChange {
    pub changed: bool,
    pub should_deactivate_crop: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreviewOverlayState {
    overlay_mode: bool,
    overlay: Option<PreviewOverlay>,
    drag_origin: Option<OverlayDragOrigin>,
}

#[derive(Clone, Debug, PartialEq)]
struct OverlayDragOrigin {
    handle: OverlayDragHandle,
    start_overlay: PreviewOverlay,
    start_point: OverlayDragPoint,
}

impl PreviewOverlayState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            overlay_mode: false,
            overlay: None,
            drag_origin: None,
        }
    }

    #[must_use]
    pub const fn overlay_mode(&self) -> bool {
        self.overlay_mode
    }

    #[must_use]
    pub const fn overlay(&self) -> Option<&PreviewOverlay> {
        self.overlay.as_ref()
    }

    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.drag_origin.is_some()
    }

    pub fn sync_initial_overlay(&mut self, initial_overlay: Option<&PreviewOverlay>) {
        if self.drag_origin.is_some() {
            return;
        }

        self.overlay = initial_overlay
            .filter(|overlay| overlay.enabled && !overlay.path.is_empty())
            .map(normalize_overlay);

        if self.overlay.is_none() {
            self.overlay_mode = false;
        }
    }

    pub fn set_overlay_from_path(
        &mut self,
        path: impl Into<String>,
        controls_disabled: bool,
    ) -> Option<PreviewOverlay> {
        if controls_disabled {
            return None;
        }

        let overlay = create_default_overlay(path);
        self.overlay_mode = true;
        self.overlay = Some(overlay.clone());
        Some(overlay)
    }

    pub fn toggle_overlay_mode(&mut self, controls_disabled: bool) -> OverlayModeChange {
        if controls_disabled || self.overlay.is_none() {
            return OverlayModeChange {
                changed: false,
                should_deactivate_crop: false,
            };
        }

        let should_deactivate_crop = !self.overlay_mode;
        self.overlay_mode = !self.overlay_mode;
        OverlayModeChange {
            changed: true,
            should_deactivate_crop,
        }
    }

    pub fn set_overlay_mode(&mut self, value: bool, controls_disabled: bool) -> OverlayModeChange {
        if controls_disabled && value {
            return OverlayModeChange {
                changed: false,
                should_deactivate_crop: false,
            };
        }

        let next_mode = value && self.overlay.is_some();
        let changed = self.overlay_mode != next_mode;
        self.overlay_mode = next_mode;
        OverlayModeChange {
            changed,
            should_deactivate_crop: value,
        }
    }

    pub fn begin_overlay_drag(
        &mut self,
        handle: OverlayDragHandle,
        point: OverlayDragPoint,
        controls_disabled: bool,
    ) -> bool {
        let Some(overlay) = &self.overlay else {
            return false;
        };
        if !self.overlay_mode || controls_disabled {
            return false;
        }

        let mut start_overlay = overlay.clone();
        start_overlay.width = point.width.unwrap_or(start_overlay.width);
        self.drag_origin = Some(OverlayDragOrigin {
            handle,
            start_overlay,
            start_point: point,
        });
        true
    }

    pub fn update_overlay_drag(&mut self, point: OverlayDragPoint) -> Option<PreviewOverlay> {
        let Some(drag_origin) = &self.drag_origin else {
            return None;
        };
        self.overlay.as_ref()?;

        let next_overlay = match drag_origin.handle {
            OverlayDragHandle::Move => {
                let height = drag_origin
                    .start_point
                    .height
                    .unwrap_or(drag_origin.start_overlay.width);
                let center = clamp_overlay_center(
                    drag_origin.start_overlay.x + point.x - drag_origin.start_point.x,
                    drag_origin.start_overlay.y + point.y - drag_origin.start_point.y,
                    drag_origin.start_overlay.width,
                    height,
                );
                PreviewOverlay {
                    x: center.x,
                    y: center.y,
                    anchor: "custom".to_string(),
                    ..drag_origin.start_overlay.clone()
                }
            }
            OverlayDragHandle::NorthWest
            | OverlayDragHandle::NorthEast
            | OverlayDragHandle::SouthEast
            | OverlayDragHandle::SouthWest => {
                let (Some(point_width), Some(start_width), Some(start_height)) = (
                    point.width,
                    drag_origin.start_point.width,
                    drag_origin.start_point.height,
                ) else {
                    return None;
                };
                if point_width <= 0.0 || start_width <= 0.0 || start_height <= 0.0 {
                    return None;
                }

                let start_left =
                    drag_origin.start_overlay.x - drag_origin.start_overlay.width / 2.0;
                let start_right =
                    drag_origin.start_overlay.x + drag_origin.start_overlay.width / 2.0;
                let start_top = drag_origin.start_overlay.y - start_height / 2.0;
                let start_bottom = drag_origin.start_overlay.y + start_height / 2.0;
                let anchor_x = if drag_origin.handle.anchors_right_edge() {
                    start_right
                } else {
                    start_left
                };
                let anchor_y = if drag_origin.handle.anchors_bottom_edge() {
                    start_bottom
                } else {
                    start_top
                };
                let aspect = start_height / start_width;
                let raw_width_from_x = (point.x - anchor_x).abs();
                let raw_width_from_y = (point.y - anchor_y).abs() / aspect;
                let width = clamp_overlay_width(raw_width_from_x.max(raw_width_from_y), aspect);
                let height = width * aspect;
                let center = clamp_overlay_center(
                    anchor_x + (drag_origin.handle.direction_x() * width) / 2.0,
                    anchor_y + (drag_origin.handle.direction_y() * height) / 2.0,
                    width,
                    height,
                );

                PreviewOverlay {
                    x: center.x,
                    y: center.y,
                    width,
                    anchor: "custom".to_string(),
                    ..drag_origin.start_overlay.clone()
                }
            }
        };

        self.overlay = Some(next_overlay.clone());
        Some(next_overlay)
    }

    pub fn end_overlay_drag(&mut self) {
        self.drag_origin = None;
    }

    pub fn set_opacity(&mut self, value: f64, controls_disabled: bool) -> Option<PreviewOverlay> {
        if controls_disabled {
            return None;
        }

        let overlay = self.overlay.as_mut()?;
        overlay.opacity = clamp(value, 0.0, 1.0);
        Some(overlay.clone())
    }

    pub fn nudge_size(
        &mut self,
        direction: OverlaySizeDirection,
        height_ratio: Option<f64>,
        controls_disabled: bool,
    ) -> Option<PreviewOverlay> {
        if controls_disabled {
            return None;
        }

        let overlay = self.overlay.as_mut()?;
        let height_ratio_for_width = height_ratio.unwrap_or(1.0);
        let width = clamp_overlay_width(overlay.width + direction.step(), height_ratio_for_width);
        let height = width * height_ratio.unwrap_or(1.0);
        let center = clamp_overlay_center(overlay.x, overlay.y, width, height);
        overlay.x = center.x;
        overlay.y = center.y;
        overlay.width = width;
        overlay.anchor = "custom".to_string();
        Some(overlay.clone())
    }

    pub fn remove_overlay(&mut self, controls_disabled: bool) -> Option<Option<PreviewOverlay>> {
        if controls_disabled {
            return None;
        }

        self.overlay_mode = false;
        self.overlay = None;
        Some(None)
    }

    pub fn destroy(&mut self) {
        self.end_overlay_drag();
    }
}

impl OverlayDragHandle {
    const fn anchors_right_edge(self) -> bool {
        matches!(self, Self::NorthWest | Self::SouthWest)
    }

    const fn anchors_bottom_edge(self) -> bool {
        matches!(self, Self::NorthWest | Self::NorthEast)
    }

    const fn direction_x(self) -> f64 {
        match self {
            Self::NorthWest | Self::SouthWest => -1.0,
            Self::NorthEast | Self::SouthEast | Self::Move => 1.0,
        }
    }

    const fn direction_y(self) -> f64 {
        match self {
            Self::NorthWest | Self::NorthEast => -1.0,
            Self::SouthEast | Self::SouthWest | Self::Move => 1.0,
        }
    }
}

#[must_use]
pub fn parse_time_to_seconds(time: &str) -> f64 {
    let mut parts = time.split(':');
    let (Some(hours), Some(minutes), Some(seconds), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return 0.0;
    };

    let Ok(hours) = hours.parse::<f64>() else {
        return 0.0;
    };
    let Ok(minutes) = minutes.parse::<f64>() else {
        return 0.0;
    };
    let Ok(seconds) = seconds.parse::<f64>() else {
        return 0.0;
    };

    hours * 3600.0 + minutes * 60.0 + seconds
}

#[must_use]
pub fn format_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor();
    let minutes = ((seconds % 3600.0) / 60.0).floor();
    let seconds = seconds % 60.0;

    format!("{hours:02.0}:{minutes:02.0}:{seconds:06.3}")
}

#[must_use]
pub fn clamp_overlay_center(x: f64, y: f64, width: f64, height: f64) -> Point {
    let half_width = (width / 2.0).min(0.5);
    let half_height = (height / 2.0).min(0.5);
    Point {
        x: clamp(x, half_width, 1.0 - half_width),
        y: clamp(y, half_height, 1.0 - half_height),
    }
}

#[must_use]
pub fn max_overlay_width(height_ratio: f64) -> f64 {
    if !height_ratio.is_finite() || height_ratio <= 0.0 {
        return MAX_OVERLAY_WIDTH;
    }

    MAX_OVERLAY_WIDTH.min(1.0 / height_ratio)
}

#[must_use]
pub fn clamp_overlay_width(width: f64, height_ratio: f64) -> f64 {
    let max_width = max_overlay_width(height_ratio);
    let min_width = MIN_OVERLAY_WIDTH.min(max_width);
    clamp(width, min_width, max_width)
}

#[must_use]
pub fn create_default_overlay(path: impl Into<String>) -> PreviewOverlay {
    let width = DEFAULT_OVERLAY_WIDTH;
    let center = clamp_overlay_center(0.5, 0.5, width, width);
    PreviewOverlay {
        enabled: true,
        path: path.into(),
        x: center.x,
        y: center.y,
        width,
        opacity: 1.0,
        anchor: "custom".to_string(),
    }
}

#[must_use]
pub fn normalize_overlay(overlay: &PreviewOverlay) -> PreviewOverlay {
    let width = clamp(overlay.width, MIN_OVERLAY_WIDTH, MAX_OVERLAY_WIDTH);
    let center = clamp_overlay_center(overlay.x, overlay.y, width, width);
    PreviewOverlay {
        enabled: overlay.enabled,
        path: overlay.path.clone(),
        x: center.x,
        y: center.y,
        width,
        opacity: clamp(overlay.opacity, 0.0, 1.0),
        anchor: "custom".to_string(),
    }
}

#[must_use]
pub fn preview_media_kind(
    metadata_status: MetadataStatus,
    source_media_kind: Option<SourceMediaKind>,
) -> PreviewMediaKind {
    if metadata_status != MetadataStatus::Ready {
        return PreviewMediaKind::Unknown;
    }

    match source_media_kind {
        Some(SourceMediaKind::Video) => PreviewMediaKind::Video,
        Some(SourceMediaKind::Audio) => PreviewMediaKind::Audio,
        Some(SourceMediaKind::Image) => PreviewMediaKind::Image,
        None => PreviewMediaKind::Unknown,
    }
}

#[must_use]
pub fn preview_control_availability(input: PreviewControlInput<'_>) -> PreviewControlAvailability {
    let media_kind = preview_media_kind(input.metadata_status, input.source_media_kind);
    let container = input.container.unwrap_or_default();
    let is_audio_only_output = media_rules::is_audio_only_container(container);
    let is_image = media_kind == PreviewMediaKind::Image;

    PreviewControlAvailability {
        media_kind,
        hide_visual_controls: media_kind == PreviewMediaKind::Audio
            || (media_kind == PreviewMediaKind::Video && is_audio_only_output),
        trim_disabled: input.controls_disabled
            || is_image
            || media_kind == PreviewMediaKind::Unknown,
        overlay_available: media_kind == PreviewMediaKind::Video
            && !is_audio_only_output
            && input.processing_mode != ProcessingMode::Copy
            && !media_rules::is_gif_container(container),
    }
}

#[must_use]
pub fn aspect_value(id: &str) -> Option<f64> {
    ASPECT_OPTIONS
        .iter()
        .find(|option| option.id == id)
        .and_then(|option| option.ratio)
}

#[must_use]
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.min(max).max(min)
}

#[must_use]
pub fn clamp_rect(rect: CropRect) -> CropRect {
    let mut x = rect.x;
    let mut y = rect.y;
    let mut width = rect.width;
    let mut height = rect.height;

    if width < MIN_CROP {
        width = MIN_CROP;
    }
    if height < MIN_CROP {
        height = MIN_CROP;
    }
    if x < 0.0 {
        x = 0.0;
    }
    if y < 0.0 {
        y = 0.0;
    }
    if x + width > 1.0 {
        x = 1.0 - width;
    }
    if y + height > 1.0 {
        y = 1.0 - height;
    }

    CropRect {
        x,
        y,
        width,
        height,
    }
}

#[must_use]
pub fn effective_aspect_ratio(
    target_ratio: f64,
    source_width: f64,
    source_height: f64,
    is_side_rotation: bool,
) -> f64 {
    if source_width.abs() <= f64::EPSILON || source_height.abs() <= f64::EPSILON {
        return target_ratio;
    }

    let physical_aspect = source_width / source_height;
    if is_side_rotation {
        1.0 / target_ratio / physical_aspect
    } else {
        target_ratio / physical_aspect
    }
}

#[must_use]
pub fn transform_crop_rect(
    rect: CropRect,
    rotation: PreviewRotation,
    flip_horizontal: bool,
    flip_vertical: bool,
    inverse: bool,
) -> CropRect {
    let center_x = rect.x + rect.width / 2.0 - 0.5;
    let center_y = rect.y + rect.height / 2.0 - 0.5;

    let (center_x, center_y, width, height) = if inverse {
        let (center_x, center_y) = flip_center(center_x, center_y, flip_horizontal, flip_vertical);
        inverse_rotate_center(center_x, center_y, rect.width, rect.height, rotation)
    } else {
        let (center_x, center_y, width, height) =
            rotate_center(center_x, center_y, rect.width, rect.height, rotation);
        let (center_x, center_y) = flip_center(center_x, center_y, flip_horizontal, flip_vertical);
        (center_x, center_y, width, height)
    };

    CropRect {
        x: center_x - width / 2.0 + 0.5,
        y: center_y - height / 2.0 + 0.5,
        width,
        height,
    }
}

#[must_use]
pub fn remap_drag_deltas(
    dx: f64,
    dy: f64,
    rotation: PreviewRotation,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> DragDelta {
    let (mut dx, mut dy) = match rotation {
        PreviewRotation::Deg0 => (dx, dy),
        PreviewRotation::Deg90 => (dy, -dx),
        PreviewRotation::Deg180 => (-dx, -dy),
        PreviewRotation::Deg270 => (-dy, dx),
    };

    if flip_horizontal {
        dx = -dx;
    }
    if flip_vertical {
        dy = -dy;
    }

    DragDelta { dx, dy }
}

#[must_use]
pub fn adjust_rect_to_ratio(
    rect: CropRect,
    ratio: f64,
    source_width: f64,
    source_height: f64,
    is_side_rotation: bool,
) -> CropRect {
    let effective_ratio =
        effective_aspect_ratio(ratio, source_width, source_height, is_side_rotation);
    let mut width = rect.width;
    let mut height = rect.height;

    if width / height > effective_ratio {
        width = height * effective_ratio;
    } else {
        height = width / effective_ratio;
    }

    let center_x = rect.x + rect.width / 2.0;
    let center_y = rect.y + rect.height / 2.0;
    let mut x = center_x - width / 2.0;
    let mut y = center_y - height / 2.0;

    if x < 0.0 {
        x = 0.0;
    }
    if y < 0.0 {
        y = 0.0;
    }
    if x + width > 1.0 {
        x = 1.0 - width;
    }
    if y + height > 1.0 {
        y = 1.0 - height;
    }

    CropRect {
        x,
        y,
        width,
        height,
    }
}

#[must_use]
pub fn enforce_aspect(
    rect: CropRect,
    handle: DragHandle,
    start_rect: CropRect,
    ratio: f64,
    source_width: f64,
    source_height: f64,
    is_side_rotation: bool,
) -> CropRect {
    let effective_ratio =
        effective_aspect_ratio(ratio, source_width, source_height, is_side_rotation);
    let mut width = rect.width;
    let mut height = rect.height;

    if width / height > effective_ratio {
        width = height * effective_ratio;
    } else {
        height = width / effective_ratio;
    }

    let mut next = rect;
    match handle {
        DragHandle::East => {
            next.x = start_rect.x;
            next.width = width;
            let center_y = start_rect.y + start_rect.height / 2.0;
            next.y = center_y - height / 2.0;
            next.height = height;
        }
        DragHandle::West => {
            next.width = width;
            next.x = start_rect.x + start_rect.width - width;
            let center_y = start_rect.y + start_rect.height / 2.0;
            next.y = center_y - height / 2.0;
            next.height = height;
        }
        DragHandle::North => {
            next.height = height;
            next.y = start_rect.y + start_rect.height - height;
            let center_x = start_rect.x + start_rect.width / 2.0;
            next.x = center_x - width / 2.0;
            next.width = width;
        }
        DragHandle::South => {
            next.height = height;
            next.y = start_rect.y;
            let center_x = start_rect.x + start_rect.width / 2.0;
            next.x = center_x - width / 2.0;
            next.width = width;
        }
        DragHandle::NorthEast => {
            next.x = start_rect.x;
            next.y = start_rect.y + start_rect.height - height;
            next.width = width;
            next.height = height;
        }
        DragHandle::NorthWest => {
            next.width = width;
            next.height = height;
            next.x = start_rect.x + start_rect.width - width;
            next.y = start_rect.y + start_rect.height - height;
        }
        DragHandle::SouthEast => {
            next.x = start_rect.x;
            next.y = start_rect.y;
            next.width = width;
            next.height = height;
        }
        DragHandle::SouthWest => {
            next.width = width;
            next.height = height;
            next.x = start_rect.x + start_rect.width - width;
            next.y = start_rect.y;
        }
        DragHandle::Move => {}
    }

    next
}

#[must_use]
pub fn apply_visual_crop_drag(drag: VisualCropDrag<'_>) -> CropRect {
    let dx = drag.current_point.x - drag.start_point.x;
    let dy = drag.current_point.y - drag.start_point.y;

    if drag.handle == DragHandle::Move {
        let x = clamp(drag.start_rect.x + dx, 0.0, 1.0 - drag.start_rect.width);
        let y = clamp(drag.start_rect.y + dy, 0.0, 1.0 - drag.start_rect.height);
        return CropRect {
            x,
            y,
            width: drag.start_rect.width,
            height: drag.start_rect.height,
        };
    }

    let mut left = drag.start_rect.x;
    let mut right = drag.start_rect.x + drag.start_rect.width;
    let mut top = drag.start_rect.y;
    let mut bottom = drag.start_rect.y + drag.start_rect.height;

    if drag.handle.includes_west() {
        left = clamp(drag.start_rect.x + dx, 0.0, right - MIN_CROP);
    }
    if drag.handle.includes_east() {
        right = clamp(
            drag.start_rect.x + drag.start_rect.width + dx,
            left + MIN_CROP,
            1.0,
        );
    }
    if drag.handle.includes_north() {
        top = clamp(drag.start_rect.y + dy, 0.0, bottom - MIN_CROP);
    }
    if drag.handle.includes_south() {
        bottom = clamp(
            drag.start_rect.y + drag.start_rect.height + dy,
            top + MIN_CROP,
            1.0,
        );
    }

    let mut next_rect = CropRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    };

    if let Some(ratio) = aspect_value(drag.aspect_id) {
        next_rect = enforce_aspect(
            next_rect,
            drag.handle,
            drag.start_rect,
            ratio,
            drag.source_width,
            drag.source_height,
            drag.is_side_rotation,
        );
    }

    clamp_rect(next_rect)
}

#[must_use]
pub fn handle_cursor(handle: DragHandle, is_side_rotation: bool) -> &'static str {
    match handle {
        DragHandle::North | DragHandle::South => {
            if is_side_rotation {
                "ew-resize"
            } else {
                "ns-resize"
            }
        }
        DragHandle::East | DragHandle::West => {
            if is_side_rotation {
                "ns-resize"
            } else {
                "ew-resize"
            }
        }
        DragHandle::NorthWest | DragHandle::SouthEast => {
            if is_side_rotation {
                "nesw-resize"
            } else {
                "nwse-resize"
            }
        }
        DragHandle::NorthEast | DragHandle::SouthWest => {
            if is_side_rotation {
                "nwse-resize"
            } else {
                "nesw-resize"
            }
        }
        DragHandle::Move => "default",
    }
}

impl DragHandle {
    const fn includes_west(self) -> bool {
        matches!(self, Self::West | Self::NorthWest | Self::SouthWest)
    }

    const fn includes_east(self) -> bool {
        matches!(self, Self::East | Self::NorthEast | Self::SouthEast)
    }

    const fn includes_north(self) -> bool {
        matches!(self, Self::North | Self::NorthEast | Self::NorthWest)
    }

    const fn includes_south(self) -> bool {
        matches!(self, Self::South | Self::SouthEast | Self::SouthWest)
    }
}

fn rotate_center(
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64,
    rotation: PreviewRotation,
) -> (f64, f64, f64, f64) {
    match rotation {
        PreviewRotation::Deg0 => (center_x, center_y, width, height),
        PreviewRotation::Deg90 => (-center_y, center_x, height, width),
        PreviewRotation::Deg180 => (-center_x, -center_y, width, height),
        PreviewRotation::Deg270 => (center_y, -center_x, height, width),
    }
}

fn inverse_rotate_center(
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64,
    rotation: PreviewRotation,
) -> (f64, f64, f64, f64) {
    match rotation {
        PreviewRotation::Deg0 => (center_x, center_y, width, height),
        PreviewRotation::Deg90 => (center_y, -center_x, height, width),
        PreviewRotation::Deg180 => (-center_x, -center_y, width, height),
        PreviewRotation::Deg270 => (-center_y, center_x, height, width),
    }
}

fn flip_center(
    mut center_x: f64,
    mut center_y: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> (f64, f64) {
    if flip_horizontal {
        center_x = -center_x;
    }
    if flip_vertical {
        center_y = -center_y;
    }

    (center_x, center_y)
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rect_close(actual: CropRect, expected: CropRect) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
        assert_close(actual.width, expected.width);
        assert_close(actual.height, expected.height);
    }

    fn assert_close(actual: f64, expected: f64) {
        const EPSILON: f64 = 0.000_001;
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {actual} to be within {EPSILON} of {expected}"
        );
    }

    mod aspect_value {
        use super::*;

        #[test]
        fn returns_free_as_unconstrained() {
            assert_eq!(aspect_value("free"), None);
        }

        #[test]
        fn returns_original_wide_ratio() {
            assert_close(aspect_value("16:9").unwrap(), 16.0 / 9.0);
        }

        #[test]
        fn returns_none_for_unknown_ratio() {
            assert_eq!(aspect_value("2:1"), None);
        }
    }

    mod transform_crop_rect {
        use super::*;

        #[test]
        fn round_trips_zero_rotation_without_flips() {
            assert_round_trip(PreviewRotation::Deg0, false, false);
        }

        #[test]
        fn round_trips_side_rotation_with_both_flips() {
            assert_round_trip(PreviewRotation::Deg90, true, true);
        }

        #[test]
        fn round_trips_half_rotation_with_horizontal_flip() {
            assert_round_trip(PreviewRotation::Deg180, true, false);
        }

        #[test]
        fn round_trips_reverse_side_rotation_with_vertical_flip() {
            assert_round_trip(PreviewRotation::Deg270, false, true);
        }

        fn assert_round_trip(
            rotation: PreviewRotation,
            flip_horizontal: bool,
            flip_vertical: bool,
        ) {
            let rect = CropRect {
                x: 0.2,
                y: 0.15,
                width: 0.35,
                height: 0.45,
            };

            let transformed = super::super::transform_crop_rect(
                rect,
                rotation,
                flip_horizontal,
                flip_vertical,
                false,
            );
            let round_trip = super::super::transform_crop_rect(
                transformed,
                rotation,
                flip_horizontal,
                flip_vertical,
                true,
            );

            assert_rect_close(round_trip, rect);
        }
    }

    mod remap_drag_deltas {
        use super::*;

        #[test]
        fn leaves_zero_rotation_deltas_unchanged() {
            assert_eq!(
                super::super::remap_drag_deltas(0.2, 0.1, PreviewRotation::Deg0, false, false),
                DragDelta { dx: 0.2, dy: 0.1 }
            );
        }

        #[test]
        fn remaps_clockwise_side_rotation() {
            assert_eq!(
                super::super::remap_drag_deltas(0.2, 0.1, PreviewRotation::Deg90, false, false),
                DragDelta { dx: 0.1, dy: -0.2 }
            );
        }

        #[test]
        fn remaps_half_rotation() {
            assert_eq!(
                super::super::remap_drag_deltas(0.2, 0.1, PreviewRotation::Deg180, false, false),
                DragDelta { dx: -0.2, dy: -0.1 }
            );
        }

        #[test]
        fn remaps_reverse_side_rotation() {
            assert_eq!(
                super::super::remap_drag_deltas(0.2, 0.1, PreviewRotation::Deg270, false, false),
                DragDelta { dx: -0.1, dy: 0.2 }
            );
        }

        #[test]
        fn applies_flips_after_rotation() {
            assert_eq!(
                super::super::remap_drag_deltas(0.2, 0.1, PreviewRotation::Deg90, true, true),
                DragDelta { dx: -0.1, dy: 0.2 }
            );
        }
    }

    mod clamp_rect {
        use super::*;

        #[test]
        fn keeps_crop_inside_normalized_bounds() {
            assert_rect_close(
                super::super::clamp_rect(CropRect {
                    x: -0.2,
                    y: 0.9,
                    width: 0.2,
                    height: 0.3,
                }),
                CropRect {
                    x: 0.0,
                    y: 0.7,
                    width: 0.2,
                    height: 0.3,
                },
            );
        }

        #[test]
        fn enforces_original_minimum_crop_size() {
            assert_rect_close(
                super::super::clamp_rect(CropRect {
                    x: 0.9,
                    y: 0.9,
                    width: 0.01,
                    height: 0.01,
                }),
                CropRect {
                    x: 0.9,
                    y: 0.9,
                    width: 0.05,
                    height: 0.05,
                },
            );
        }
    }

    mod adjust_rect_to_ratio {
        use super::*;

        #[test]
        fn preserves_center_when_possible() {
            let adjusted = super::super::adjust_rect_to_ratio(
                CropRect {
                    x: 0.2,
                    y: 0.2,
                    width: 0.6,
                    height: 0.4,
                },
                1.0,
                1920.0,
                1080.0,
                false,
            );

            assert_close(adjusted.x + adjusted.width / 2.0, 0.5);
            assert_close(adjusted.y + adjusted.height / 2.0, 0.4);
            assert_close(adjusted.width / adjusted.height, 1.0 / (1920.0 / 1080.0));
        }
    }

    mod enforce_aspect {
        use super::*;

        #[test]
        fn anchors_the_dragged_edge() {
            let start_rect = CropRect {
                x: 0.2,
                y: 0.2,
                width: 0.4,
                height: 0.4,
            };
            let next = super::super::enforce_aspect(
                CropRect {
                    x: 0.2,
                    y: 0.2,
                    width: 0.55,
                    height: 0.4,
                },
                DragHandle::East,
                start_rect,
                16.0 / 9.0,
                1920.0,
                1080.0,
                false,
            );

            assert_eq!(next.x, start_rect.x);
            assert_close(
                next.y + next.height / 2.0,
                start_rect.y + start_rect.height / 2.0,
            );
            assert_close(next.width / next.height, 1.0);
        }
    }

    mod apply_visual_crop_drag {
        use super::*;

        #[test]
        fn keeps_drag_deltas_in_visual_space_for_side_rotations() {
            let next = super::super::apply_visual_crop_drag(VisualCropDrag {
                start_rect: CropRect {
                    x: 0.2,
                    y: 0.2,
                    width: 0.4,
                    height: 0.4,
                },
                handle: DragHandle::East,
                start_point: Point { x: 0.6, y: 0.4 },
                current_point: Point { x: 0.7, y: 0.4 },
                aspect_id: "free",
                source_width: 1920.0,
                source_height: 1080.0,
                is_side_rotation: true,
            });

            assert_rect_close(
                next,
                CropRect {
                    x: 0.2,
                    y: 0.2,
                    width: 0.5,
                    height: 0.4,
                },
            );
        }

        #[test]
        fn moves_crop_directly_in_visual_space_for_side_rotations() {
            let next = super::super::apply_visual_crop_drag(VisualCropDrag {
                start_rect: CropRect {
                    x: 0.2,
                    y: 0.2,
                    width: 0.4,
                    height: 0.4,
                },
                handle: DragHandle::Move,
                start_point: Point { x: 0.4, y: 0.4 },
                current_point: Point { x: 0.5, y: 0.45 },
                aspect_id: "free",
                source_width: 1920.0,
                source_height: 1080.0,
                is_side_rotation: true,
            });

            assert_rect_close(
                next,
                CropRect {
                    x: 0.3,
                    y: 0.25,
                    width: 0.4,
                    height: 0.4,
                },
            );
        }

        #[test]
        fn enforces_fixed_aspect_ratios_when_dragging_corner_handles() {
            let next = super::super::apply_visual_crop_drag(VisualCropDrag {
                start_rect: CropRect {
                    x: 0.2,
                    y: 0.2,
                    width: 0.4,
                    height: 0.4,
                },
                handle: DragHandle::SouthEast,
                start_point: Point { x: 0.6, y: 0.6 },
                current_point: Point { x: 0.8, y: 0.6 },
                aspect_id: "16:9",
                source_width: 1920.0,
                source_height: 1080.0,
                is_side_rotation: false,
            });

            assert_close(next.width / next.height, 1.0);
        }
    }

    mod handle_cursor {
        use super::*;

        #[test]
        fn swaps_cardinal_cursors_for_side_rotation() {
            assert_eq!(
                super::super::handle_cursor(DragHandle::North, false),
                "ns-resize"
            );
            assert_eq!(
                super::super::handle_cursor(DragHandle::North, true),
                "ew-resize"
            );
        }

        #[test]
        fn swaps_corner_cursors_for_side_rotation() {
            assert_eq!(
                super::super::handle_cursor(DragHandle::NorthEast, false),
                "nesw-resize"
            );
            assert_eq!(
                super::super::handle_cursor(DragHandle::NorthEast, true),
                "nwse-resize"
            );
        }
    }

    mod preview_rotation {
        use super::*;

        #[test]
        fn parses_original_rotation_strings() {
            assert_eq!(PreviewRotation::from("90"), PreviewRotation::Deg90);
            assert_eq!(PreviewRotation::from("180"), PreviewRotation::Deg180);
            assert_eq!(PreviewRotation::from("270"), PreviewRotation::Deg270);
        }

        #[test]
        fn treats_unknown_rotation_as_zero() {
            assert_eq!(PreviewRotation::from(""), PreviewRotation::Deg0);
            assert_eq!(PreviewRotation::from("45"), PreviewRotation::Deg0);
        }
    }

    mod preview_media_kind {
        use super::*;

        #[test]
        fn stays_unknown_until_metadata_is_ready() {
            assert_eq!(
                super::super::preview_media_kind(
                    MetadataStatus::Loading,
                    Some(SourceMediaKind::Video),
                ),
                PreviewMediaKind::Unknown
            );
        }

        #[test]
        fn uses_ready_metadata_kind() {
            assert_eq!(
                super::super::preview_media_kind(
                    MetadataStatus::Ready,
                    Some(SourceMediaKind::Image)
                ),
                PreviewMediaKind::Image
            );
        }
    }

    mod preview_control_availability {
        use super::*;

        #[test]
        fn disables_trim_for_unknown_metadata() {
            assert!(
                availability(MetadataStatus::Idle, Some(SourceMediaKind::Video), "mp4")
                    .trim_disabled
            );
        }

        #[test]
        fn disables_trim_for_image_sources() {
            assert!(
                availability(MetadataStatus::Ready, Some(SourceMediaKind::Image), "png")
                    .trim_disabled
            );
        }

        #[test]
        fn keeps_visual_controls_hidden_for_audio_sources() {
            assert!(
                availability(MetadataStatus::Ready, Some(SourceMediaKind::Audio), "mp3")
                    .hide_visual_controls
            );
        }

        #[test]
        fn hides_visual_controls_for_video_to_audio_output() {
            assert!(
                availability(MetadataStatus::Ready, Some(SourceMediaKind::Video), "mp3")
                    .hide_visual_controls
            );
        }

        #[test]
        fn enables_overlay_for_reencoded_video_output() {
            assert!(
                availability(MetadataStatus::Ready, Some(SourceMediaKind::Video), "mp4")
                    .overlay_available
            );
        }

        #[test]
        fn disables_overlay_for_copy_mode() {
            let availability = super::super::preview_control_availability(PreviewControlInput {
                metadata_status: MetadataStatus::Ready,
                source_media_kind: Some(SourceMediaKind::Video),
                controls_disabled: false,
                processing_mode: ProcessingMode::Copy,
                container: Some("mp4"),
            });

            assert!(!availability.overlay_available);
        }

        #[test]
        fn disables_overlay_for_gif_output() {
            assert!(
                !availability(MetadataStatus::Ready, Some(SourceMediaKind::Video), "gif")
                    .overlay_available
            );
        }

        #[test]
        fn respects_global_control_disabled_state_for_trim() {
            let availability = super::super::preview_control_availability(PreviewControlInput {
                metadata_status: MetadataStatus::Ready,
                source_media_kind: Some(SourceMediaKind::Video),
                controls_disabled: true,
                processing_mode: ProcessingMode::Reencode,
                container: Some("mp4"),
            });

            assert!(availability.trim_disabled);
        }

        fn availability(
            metadata_status: MetadataStatus,
            source_media_kind: Option<SourceMediaKind>,
            container: &str,
        ) -> PreviewControlAvailability {
            super::super::preview_control_availability(PreviewControlInput {
                metadata_status,
                source_media_kind,
                controls_disabled: false,
                processing_mode: ProcessingMode::Reencode,
                container: Some(container),
            })
        }
    }

    mod time_formatting {
        use super::*;

        #[test]
        fn parse_time_to_seconds_accepts_original_hh_mm_ss_fraction() {
            assert_close(super::super::parse_time_to_seconds("01:02:03.250"), 3723.25);
        }

        #[test]
        fn parse_time_to_seconds_returns_zero_for_partial_timecodes() {
            assert_eq!(super::super::parse_time_to_seconds("02:03"), 0.0);
        }

        #[test]
        fn format_time_matches_original_trim_precision() {
            assert_eq!(super::super::format_time(3723.25), "01:02:03.250");
        }

        #[test]
        fn format_time_pads_single_digit_seconds_like_svelte() {
            assert_eq!(super::super::format_time(61.2), "00:01:01.200");
        }
    }

    mod preview_playback_state {
        use super::*;

        #[test]
        fn sync_initial_values_reads_start_and_end_timecodes() {
            let mut playback = playback_with_media(120.0);

            playback.sync_initial_values(Some("00:00:05.000"), Some("00:01:00.500"));

            assert_close(playback.start_value(), 5.0);
            assert_close(playback.end_value(), 60.5);
        }

        #[test]
        fn sync_initial_values_uses_duration_when_end_is_missing() {
            let mut playback = playback_with_media(120.0);

            playback.sync_initial_values(Some("00:00:05.000"), None);

            assert_close(playback.end_value(), 120.0);
        }

        #[test]
        fn sync_from_media_clamps_trim_values_to_duration() {
            let mut playback = PreviewPlaybackState::new(false);
            playback.sync_initial_values(Some("00:05:00.000"), Some("00:10:00.000"));

            playback.sync_media(MediaSnapshot {
                current_time: 12.0,
                duration: 30.0,
                paused: true,
            });

            assert_close(playback.start_value(), 0.0);
            assert_close(playback.end_value(), 30.0);
        }

        #[test]
        fn clear_media_resets_timeline_state_like_detached_media_element() {
            let mut playback = playback_with_media(120.0);
            playback.sync_initial_values(Some("00:00:05.000"), Some("00:00:20.000"));

            playback.clear_media();

            assert_close(playback.duration(), 0.0);
            assert_close(playback.start_value(), 0.0);
            assert_close(playback.end_value(), 0.0);
        }

        #[test]
        fn handle_time_update_loops_back_to_trim_start_at_end() {
            let mut playback = playback_with_media(120.0);
            playback.sync_initial_values(Some("00:00:10.000"), Some("00:00:20.000"));

            let command = playback.handle_time_update(20.0);

            assert_eq!(command, PlaybackMediaCommand::pause_and_seek(10.0));
            assert_close(playback.current_time(), 10.0);
        }

        #[test]
        fn toggle_play_seeks_to_start_when_current_time_is_outside_trim() {
            let mut playback = playback_with_media(120.0);
            playback.sync_initial_values(Some("00:00:10.000"), Some("00:00:20.000"));
            playback.sync_media(MediaSnapshot {
                current_time: 30.0,
                duration: 120.0,
                paused: true,
            });
            playback.handle_pause();

            let command = playback.toggle_play();

            assert_eq!(command, PlaybackMediaCommand::seek_and_play(10.0));
        }

        #[test]
        fn toggle_play_returns_pause_command_when_playing() {
            let mut playback = playback_with_media(120.0);
            playback.handle_play();

            assert_eq!(playback.toggle_play(), PlaybackMediaCommand::pause());
        }

        #[test]
        fn commit_trim_values_omits_zero_start_and_full_duration_end() {
            let playback = playback_with_media(120.0);

            assert_eq!(
                playback.commit_trim_values(),
                Some(TrimSelection {
                    start_time: None,
                    end_time: None,
                })
            );
        }

        #[test]
        fn commit_trim_values_formats_partial_trim_bounds() {
            let mut playback = playback_with_media(120.0);
            playback.sync_initial_values(Some("00:00:05.000"), Some("00:00:30.250"));

            assert_eq!(
                playback.commit_trim_values(),
                Some(TrimSelection {
                    start_time: Some("00:00:05.000".to_string()),
                    end_time: Some("00:00:30.250".to_string()),
                })
            );
        }

        #[test]
        fn image_playback_ignores_trim_commit_and_play_toggle() {
            let mut playback = PreviewPlaybackState::new(true);
            playback.sync_media(MediaSnapshot {
                current_time: 5.0,
                duration: 120.0,
                paused: true,
            });

            assert_eq!(playback.commit_trim_values(), None);
            assert_eq!(playback.toggle_play(), PlaybackMediaCommand::none());
        }

        #[test]
        fn set_start_from_input_rejects_values_after_end() {
            let mut playback = playback_with_media(120.0);

            assert_eq!(playback.set_start_from_input(121.0), None);
        }

        #[test]
        fn set_end_from_input_accepts_values_within_duration() {
            let mut playback = playback_with_media(120.0);
            playback.sync_initial_values(Some("00:00:05.000"), None);

            assert_eq!(
                playback.set_end_from_input(60.0),
                Some(PlaybackMediaCommand::seek(60.0))
            );
            assert_close(playback.end_value(), 60.0);
        }

        #[test]
        fn begin_handle_drag_ignores_image_sources() {
            let mut playback = PreviewPlaybackState::new(true);

            assert!(!playback.begin_handle_drag(TimelineDragTarget::Start));
        }

        #[test]
        fn drag_start_handle_uses_one_second_gap_before_end() {
            let mut playback = playback_with_media(120.0);
            playback.sync_initial_values(None, Some("00:00:20.000"));
            assert!(playback.begin_handle_drag(TimelineDragTarget::Start));

            let update = playback.drag_to_percent(0.5);

            assert_eq!(update.command, PlaybackMediaCommand::seek(19.0));
            assert_close(playback.start_value(), 19.0);
            assert!(update.trim.is_some());
        }

        #[test]
        fn drag_end_handle_uses_one_second_gap_after_start() {
            let mut playback = playback_with_media(120.0);
            playback.sync_initial_values(Some("00:00:30.000"), None);
            assert!(playback.begin_handle_drag(TimelineDragTarget::End));

            let update = playback.drag_to_percent(0.1);

            assert_eq!(update.command, PlaybackMediaCommand::seek(31.0));
            assert_close(playback.end_value(), 31.0);
        }

        #[test]
        fn seek_to_percent_pauses_active_scrub_and_remembers_play_state() {
            let mut playback = playback_with_media(120.0);
            playback.handle_play();

            let command = playback.seek_to_percent(0.25);

            assert_eq!(command, PlaybackMediaCommand::pause_and_seek(30.0));
            assert_eq!(playback.dragging(), Some(TimelineDragTarget::Scrub));
        }

        #[test]
        fn end_drag_resumes_scrub_when_it_started_while_playing() {
            let mut playback = playback_with_media(120.0);
            playback.handle_play();
            let _ = playback.seek_to_percent(0.25);

            let end = playback.end_drag();

            assert_eq!(end.command, PlaybackMediaCommand::play());
            assert_eq!(end.trim, None);
            assert_eq!(playback.dragging(), None);
        }

        #[test]
        fn end_drag_commits_trim_when_handle_was_dragged() {
            let mut playback = playback_with_media(120.0);
            playback.begin_handle_drag(TimelineDragTarget::End);
            let _ = playback.drag_to_percent(0.5);

            let end = playback.end_drag();

            assert_eq!(end.command, PlaybackMediaCommand::none());
            assert_eq!(
                end.trim,
                Some(TrimSelection {
                    start_time: None,
                    end_time: Some("00:01:00.000".to_string()),
                })
            );
        }

        #[test]
        fn timeline_percent_returns_zero_without_positive_duration() {
            let playback = PreviewPlaybackState::new(false);

            assert_eq!(playback.to_timeline_percent(15.0), 0.0);
        }

        #[test]
        fn timeline_percent_matches_original_value_over_duration_formula() {
            let playback = playback_with_media(120.0);

            assert_close(playback.to_timeline_percent(30.0), 25.0);
        }

        fn playback_with_media(duration: f64) -> PreviewPlaybackState {
            let mut playback = PreviewPlaybackState::new(false);
            playback.sync_media(MediaSnapshot {
                current_time: 0.0,
                duration,
                paused: true,
            });
            playback
        }
    }

    mod preview_overlay_state {
        use super::*;

        #[test]
        fn clamp_overlay_center_keeps_overlay_inside_bounds() {
            let center = super::super::clamp_overlay_center(0.98, 0.01, 0.4, 0.2);

            assert_close(center.x, 0.8);
            assert_close(center.y, 0.1);
        }

        #[test]
        fn max_overlay_width_accounts_for_tall_overlay_aspect() {
            assert_close(super::super::max_overlay_width(2.0), 0.5);
        }

        #[test]
        fn clamp_overlay_width_uses_original_minimum_width() {
            assert_close(super::super::clamp_overlay_width(0.01, 1.0), 0.03);
        }

        #[test]
        fn create_default_overlay_matches_original_centered_overlay() {
            let overlay = super::super::create_default_overlay("/tmp/logo.png");

            assert_eq!(overlay.path, "/tmp/logo.png");
            assert_close(overlay.x, 0.5);
            assert_close(overlay.y, 0.5);
            assert_close(overlay.width, DEFAULT_OVERLAY_WIDTH);
            assert_close(overlay.opacity, 1.0);
        }

        #[test]
        fn normalize_overlay_clamps_position_size_and_opacity() {
            let overlay = super::super::normalize_overlay(&PreviewOverlay {
                enabled: true,
                path: "/tmp/logo.png".to_string(),
                x: 0.99,
                y: -0.1,
                width: 2.0,
                opacity: 2.0,
                anchor: "ignored".to_string(),
            });

            assert_close(overlay.x, 0.6);
            assert_close(overlay.y, 0.4);
            assert_close(overlay.width, MAX_OVERLAY_WIDTH);
            assert_close(overlay.opacity, 1.0);
            assert_eq!(overlay.anchor, "custom");
        }

        #[test]
        fn sync_initial_overlay_discards_disabled_or_empty_overlay() {
            let mut state = PreviewOverlayState::new();
            state.set_overlay_from_path("/tmp/logo.png", false);

            state.sync_initial_overlay(Some(&PreviewOverlay {
                enabled: false,
                path: "/tmp/logo.png".to_string(),
                x: 0.5,
                y: 0.5,
                width: 0.2,
                opacity: 1.0,
                anchor: "custom".to_string(),
            }));

            assert_eq!(state.overlay(), None);
            assert!(!state.overlay_mode());
        }

        #[test]
        fn sync_initial_overlay_ignores_external_changes_while_dragging() {
            let mut state = state_with_overlay();
            assert!(state.begin_overlay_drag(
                OverlayDragHandle::Move,
                OverlayDragPoint {
                    x: 0.5,
                    y: 0.5,
                    width: Some(0.18),
                    height: Some(0.18),
                },
                false,
            ));

            state.sync_initial_overlay(None);

            assert!(state.overlay().is_some());
            assert!(state.is_dragging());
        }

        #[test]
        fn set_overlay_from_path_is_blocked_when_controls_are_disabled() {
            let mut state = PreviewOverlayState::new();

            assert_eq!(state.set_overlay_from_path("/tmp/logo.png", true), None);
            assert_eq!(state.overlay(), None);
        }

        #[test]
        fn set_overlay_from_path_persists_overlay_and_enters_overlay_mode() {
            let mut state = PreviewOverlayState::new();

            let overlay = state.set_overlay_from_path("/tmp/logo.png", false).unwrap();

            assert_eq!(overlay.path, "/tmp/logo.png");
            assert!(state.overlay_mode());
            assert!(state.overlay().is_some());
        }

        #[test]
        fn toggle_overlay_mode_requests_crop_deactivation_when_enabling() {
            let mut state = state_with_overlay();
            state.set_overlay_mode(false, false);

            let change = state.toggle_overlay_mode(false);

            assert!(change.changed);
            assert!(change.should_deactivate_crop);
            assert!(state.overlay_mode());
        }

        #[test]
        fn toggle_overlay_mode_does_nothing_without_overlay() {
            let mut state = PreviewOverlayState::new();

            let change = state.toggle_overlay_mode(false);

            assert!(!change.changed);
            assert!(!change.should_deactivate_crop);
        }

        #[test]
        fn set_overlay_mode_refuses_enabling_when_controls_are_disabled() {
            let mut state = state_with_overlay();
            state.set_overlay_mode(false, false);

            let change = state.set_overlay_mode(true, true);

            assert!(!change.changed);
            assert!(!state.overlay_mode());
        }

        #[test]
        fn begin_overlay_drag_requires_overlay_mode() {
            let mut state = state_with_overlay();
            state.set_overlay_mode(false, false);

            assert!(!state.begin_overlay_drag(
                OverlayDragHandle::Move,
                OverlayDragPoint {
                    x: 0.5,
                    y: 0.5,
                    width: Some(0.18),
                    height: Some(0.18),
                },
                false,
            ));
        }

        #[test]
        fn move_drag_updates_center_in_normalized_overlay_space() {
            let mut state = state_with_overlay();
            state.begin_overlay_drag(
                OverlayDragHandle::Move,
                OverlayDragPoint {
                    x: 0.5,
                    y: 0.5,
                    width: Some(0.2),
                    height: Some(0.1),
                },
                false,
            );

            let overlay = state
                .update_overlay_drag(OverlayDragPoint {
                    x: 0.6,
                    y: 0.45,
                    width: Some(0.2),
                    height: Some(0.1),
                })
                .unwrap();

            assert_close(overlay.x, 0.6);
            assert_close(overlay.y, 0.45);
        }

        #[test]
        fn resize_drag_preserves_overlay_aspect_from_pointer_metrics() {
            let mut state = state_with_overlay();
            state.sync_initial_overlay(Some(&PreviewOverlay {
                enabled: true,
                path: "/tmp/logo.png".to_string(),
                x: 0.5,
                y: 0.5,
                width: 0.2,
                opacity: 1.0,
                anchor: "custom".to_string(),
            }));
            state.set_overlay_mode(true, false);
            state.begin_overlay_drag(
                OverlayDragHandle::SouthEast,
                OverlayDragPoint {
                    x: 0.6,
                    y: 0.55,
                    width: Some(0.2),
                    height: Some(0.1),
                },
                false,
            );

            let overlay = state
                .update_overlay_drag(OverlayDragPoint {
                    x: 0.8,
                    y: 0.7,
                    width: Some(0.2),
                    height: Some(0.1),
                })
                .unwrap();

            assert_close(overlay.width, 0.5);
            assert_close(overlay.x, 0.65);
            assert_close(overlay.y, 0.575);
        }

        #[test]
        fn resize_drag_requires_pointer_dimensions() {
            let mut state = state_with_overlay();
            state.begin_overlay_drag(
                OverlayDragHandle::SouthEast,
                OverlayDragPoint {
                    x: 0.6,
                    y: 0.6,
                    width: None,
                    height: Some(0.1),
                },
                false,
            );

            assert_eq!(
                state.update_overlay_drag(OverlayDragPoint {
                    x: 0.8,
                    y: 0.7,
                    width: None,
                    height: Some(0.1),
                }),
                None
            );
        }

        #[test]
        fn end_overlay_drag_clears_dragging_state() {
            let mut state = state_with_overlay();
            state.begin_overlay_drag(
                OverlayDragHandle::Move,
                OverlayDragPoint {
                    x: 0.5,
                    y: 0.5,
                    width: Some(0.18),
                    height: Some(0.18),
                },
                false,
            );

            state.end_overlay_drag();

            assert!(!state.is_dragging());
        }

        #[test]
        fn set_opacity_clamps_to_original_zero_one_range() {
            let mut state = state_with_overlay();

            let overlay = state.set_opacity(2.0, false).unwrap();

            assert_close(overlay.opacity, 1.0);
        }

        #[test]
        fn nudge_size_uses_original_step_and_recenters_inside_bounds() {
            let mut state = PreviewOverlayState::new();
            state.sync_initial_overlay(Some(&PreviewOverlay {
                enabled: true,
                path: "/tmp/logo.png".to_string(),
                x: 0.99,
                y: 0.99,
                width: 0.79,
                opacity: 1.0,
                anchor: "custom".to_string(),
            }));

            let overlay = state
                .nudge_size(OverlaySizeDirection::Increase, Some(1.0), false)
                .unwrap();

            assert_close(overlay.width, 0.8);
            assert_close(overlay.x, 0.6);
            assert_close(overlay.y, 0.6);
        }

        #[test]
        fn nudge_size_respects_height_ratio_max_width() {
            let mut state = state_with_overlay();

            let overlay = state
                .nudge_size(OverlaySizeDirection::Increase, Some(4.0), false)
                .unwrap();

            assert_close(overlay.width, 0.205);
        }

        #[test]
        fn remove_overlay_clears_overlay_and_mode() {
            let mut state = state_with_overlay();

            assert_eq!(state.remove_overlay(false), Some(None));
            assert_eq!(state.overlay(), None);
            assert!(!state.overlay_mode());
        }

        #[test]
        fn remove_overlay_is_blocked_when_controls_are_disabled() {
            let mut state = state_with_overlay();

            assert_eq!(state.remove_overlay(true), None);
            assert!(state.overlay().is_some());
        }

        fn state_with_overlay() -> PreviewOverlayState {
            let mut state = PreviewOverlayState::new();
            state.set_overlay_from_path("/tmp/logo.png", false);
            state
        }
    }
}
