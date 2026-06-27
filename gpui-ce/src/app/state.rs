use super::*;

impl FrameRoot {
    pub fn new() -> Self {
        let mut root = Self {
            active_view: active_view_from_env_value(
                std::env::var("FRAME_GPUI_INITIAL_VIEW").ok().as_deref(),
            ),
            file_queue: FileQueue::new(),
            conversion_events: ConversionEventState::new(),
            logs_scroll_handle: UniformListScrollHandle::new(),
            last_log_scroll_target: None,
            is_processing: false,
            settings_ui: SettingsUiState::default(),
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            text_input_ui: FrameTextInputUiState::default(),
            source_metadata: SourceMetadataStore::default(),
            conversion_processes: ConversionProcessController::default(),
            available_encoders: AvailableEncoders::default(),
            subtitle_font_families: frame_core::fonts::list_system_font_families(),
            presets: default_presets(),
            subtitle_ui: SubtitleUiState::default(),
            preview_ui: PreviewUiState::default(),
            native_titlebar_controls_hidden: false,
            next_file_sequence: 0,
        };

        root.apply_visual_fixture(visual_fixture_from_env_value(
            std::env::var("FRAME_GPUI_VISUAL_FIXTURE").ok().as_deref(),
        ));
        root
    }
    pub(super) fn app_state(&self) -> FrameAppState {
        FrameAppState::from_file_queue(self.active_view, self.is_processing, &self.file_queue)
    }
    pub(super) fn selected_config(&self) -> Option<&ConversionConfig> {
        self.file_queue.selected_file().map(|file| &file.config)
    }
    pub(super) fn update_selected_config(
        &mut self,
        update: impl FnOnce(&mut ConversionConfig) -> bool,
    ) -> bool {
        self.file_queue
            .selected_file_mut()
            .is_some_and(|file| update(&mut file.config))
    }
    pub(super) fn normalize_selected_config(&mut self, metadata: Option<&SourceMetadata>) -> bool {
        self.update_selected_config(|config| normalize_output_config(config, metadata))
    }
}

impl Default for FrameRoot {
    fn default() -> Self {
        Self::new()
    }
}
