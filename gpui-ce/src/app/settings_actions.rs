use super::*;

impl FrameRoot {
    pub(super) fn open_app_settings(&mut self) {
        self.is_settings_open = true;
        self.max_concurrency_draft = self.max_concurrency.to_string();
        self.max_concurrency_error = None;
    }
    pub(super) fn close_app_settings(&mut self) {
        self.is_settings_open = false;
        self.max_concurrency_error = None;
        self.app_settings_value_focus = None;
        if self.active_text_input == Some(FrameTextInputKind::MaxConcurrency) {
            self.stop_text_input_cursor();
        }
    }
    pub(super) fn apply_max_concurrency_draft(&mut self) -> bool {
        let Some(value) = self.parsed_max_concurrency_draft() else {
            self.max_concurrency_error =
                Some("Enter a whole number greater than zero.".to_string());
            return false;
        };

        match self.conversion_processes.update_max_concurrency(value) {
            Ok(()) => {
                self.max_concurrency = value;
                self.max_concurrency_draft = value.to_string();
                self.max_concurrency_error = None;
                true
            }
            Err(error) => {
                self.max_concurrency_error = Some(error.to_string());
                false
            }
        }
    }
    pub(super) fn parsed_max_concurrency_draft(&self) -> Option<usize> {
        let trimmed = self.max_concurrency_draft.trim();
        let value = trimmed.parse::<usize>().ok()?;
        (value > 0).then_some(value)
    }

    pub(super) fn prompt_subtitle_burn_file(&mut self, cx: &mut Context<Self>) {
        if self.file_queue.selected_file_locked() {
            return;
        }

        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select subtitle file".into()),
        });

        cx.spawn(async move |this, cx| {
            let paths = match receiver.await {
                Ok(Ok(Some(paths))) => paths,
                Ok(Ok(None)) | Err(_) => return,
                Ok(Err(error)) => {
                    eprintln!("Failed to open subtitle picker: {error}");
                    return;
                }
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let path = path.to_string_lossy().to_string();

            this.update(cx, |root, cx| {
                if root
                    .update_selected_config(|config| apply_subtitle_burn_path(config, Some(path)))
                {
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn save_preset_from_draft(&mut self) -> bool {
        if self.file_queue.selected_file_locked() {
            return false;
        }
        let name = self.preset_name_draft.trim();
        if name.is_empty() {
            self.preset_notice = Some(PresetNotice {
                text: "Name required".to_string(),
                tone: PresetNoticeTone::Error,
            });
            return false;
        }

        let Some(config) = self.selected_config().cloned() else {
            self.preset_notice = Some(PresetNotice {
                text: "Preset not saved".to_string(),
                tone: PresetNoticeTone::Error,
            });
            return false;
        };

        self.next_custom_preset_sequence += 1;
        let id = format!("custom-preset-{}", self.next_custom_preset_sequence);
        self.presets.push(create_custom_preset(id, name, &config));
        self.preset_name_draft.clear();
        self.preset_notice = Some(PresetNotice {
            text: "Preset saved".to_string(),
            tone: PresetNoticeTone::Success,
        });
        true
    }

    pub(super) fn delete_preset(&mut self, preset_id: &str) -> bool {
        let Some(index) = self
            .presets
            .iter()
            .position(|preset| preset.id == preset_id && !preset.built_in)
        else {
            self.preset_notice = Some(PresetNotice {
                text: "Unable to delete".to_string(),
                tone: PresetNoticeTone::Error,
            });
            return false;
        };

        self.presets.remove(index);
        self.preset_notice = Some(PresetNotice {
            text: "Preset removed".to_string(),
            tone: PresetNoticeTone::Success,
        });
        true
    }

    pub(super) fn apply_preset_to_selected(&mut self, preset_id: &str) -> bool {
        if self.file_queue.selected_file_locked() {
            return false;
        }
        let Some(preset) = self
            .presets
            .iter()
            .find(|preset| preset.id == preset_id)
            .cloned()
        else {
            return false;
        };
        let metadata = self.selected_source_metadata();
        if !crate::settings::preset_is_compatible(&preset, metadata.as_ref()) {
            return false;
        }
        let changed =
            self.update_selected_config(|config| apply_preset(config, &preset, metadata.as_ref()));
        if changed {
            self.preset_notice = Some(PresetNotice {
                text: format!("Applied {}", preset.name),
                tone: PresetNoticeTone::Success,
            });
        }
        changed
    }

    pub(super) fn confirm_apply_preset_to_all(
        &mut self,
        preset_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.file_queue.selected_file_locked() {
            return;
        }
        let Some(preset) = self
            .presets
            .iter()
            .find(|preset| preset.id == preset_id)
            .cloned()
        else {
            return;
        };

        let detail = format!(
            "This will apply \"{}\" to all pending files in the queue. Existing settings will be overwritten.",
            preset.name
        );
        let receiver = window.prompt(
            PromptLevel::Warning,
            "Apply to all?",
            Some(&detail),
            &[PromptButton::ok("Apply"), PromptButton::cancel("Cancel")],
            cx,
        );

        cx.spawn(async move |this, cx| {
            let Ok(answer) = receiver.await else {
                return;
            };
            if answer != 0 {
                return;
            }

            this.update(cx, |root, cx| {
                if root.apply_preset_to_all_pending(&preset) {
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn apply_preset_to_all_pending(&mut self, preset: &PresetDefinition) -> bool {
        let mut changed = false;
        for file in self.file_queue.files_mut() {
            if !file.status.is_actionable_for_conversion() {
                continue;
            }
            let metadata = self.source_metadata.metadata_for(&file.id).cloned();
            if !crate::settings::preset_is_compatible(preset, metadata.as_ref()) {
                continue;
            }
            if apply_preset(&mut file.config, preset, metadata.as_ref()) {
                changed = true;
            }
        }

        if changed {
            self.preset_notice = Some(PresetNotice {
                text: "Applied to all items".to_string(),
                tone: PresetNoticeTone::Success,
            });
        }

        changed
    }
}
