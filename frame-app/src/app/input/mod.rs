use super::primitives::*;
use super::*;

mod actions;
mod element;
mod entity;
mod runtime;
mod text;

pub(super) use element::{FrameTextInputSpec, frame_text_input};
#[cfg(test)]
pub(super) use element::{should_capture_text_input_drag, should_handle_text_input};
pub(super) use runtime::{FrameTextInputKind, FrameTextInputRuntime, FrameTextInputUiState};
