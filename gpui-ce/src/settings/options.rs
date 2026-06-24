use frame_core::{capabilities::AvailableEncoders, media_rules};

use super::{
    model::{
        AUDIO_CHANNEL_DEFINITIONS, AUDIO_CODEC_DEFINITIONS, AudioChannelOption, AudioCodecOption,
        AudioTrackOption, ConversionConfig, FPS_OPTIONS, GIF_COLOR_OPTIONS, GIF_DITHER_OPTIONS,
        GIF_FPS_OPTIONS, METADATA_FIELDS, METADATA_MODES, MetadataField, MetadataFieldOption,
        MetadataMode, MetadataModeOption, OPTIONAL_AUDIO_CODEC_DEFINITIONS, OutputContainerOption,
        OutputModeOption, ProcessingMode, RESOLUTION_OPTIONS, SCALING_ALGORITHM_OPTIONS,
        SourceKind, SourceMetadata, VIDEO_CODEC_DEFINITIONS, VIDEO_PIXEL_FORMAT_DEFINITIONS,
        VIDEO_PRESETS, VideoCodecCapability, VideoCodecOption, VideoPixelFormatOption,
        VideoPresetOption,
    },
    rules::*,
    source_info::{audio_track_detail, display_source_value},
};

#[must_use]
pub fn output_processing_mode_options(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    disabled: bool,
) -> [OutputModeOption; 2] {
    let is_source_image = source_kind_for(metadata) == SourceKind::Image;
    [
        output_mode_option(ProcessingMode::Reencode, config, disabled),
        output_mode_option(ProcessingMode::Copy, config, disabled || is_source_image),
    ]
}

#[must_use]
pub fn visible_output_containers(metadata: Option<&SourceMetadata>) -> Vec<String> {
    let is_source_image = source_kind_for(metadata) == SourceKind::Image;

    media_rules::all_containers()
        .iter()
        .filter(|container| {
            if is_source_image {
                is_image_container(container) || is_gif_container(container)
            } else {
                !is_image_container(container)
            }
        })
        .cloned()
        .collect()
}

#[must_use]
pub fn output_container_options(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    disabled: bool,
) -> Vec<OutputContainerOption> {
    visible_output_containers(metadata)
        .into_iter()
        .map(|container| {
            let disabled_reason =
                output_container_disabled_reason(config, metadata, &container, disabled);
            OutputContainerOption {
                is_selected: config.container.eq_ignore_ascii_case(&container),
                is_disabled: disabled_reason.is_some(),
                disabled_reason,
                container,
            }
        })
        .collect()
}

#[must_use]
pub fn audio_codec_options(
    config: &ConversionConfig,
    available_encoders: &AvailableEncoders,
    disabled: bool,
) -> Vec<AudioCodecOption> {
    let encode_controls_disabled = disabled || config.processing_mode == ProcessingMode::Copy;

    AUDIO_CODEC_DEFINITIONS
        .iter()
        .map(|definition| (definition.codec, definition.label))
        .chain(
            OPTIONAL_AUDIO_CODEC_DEFINITIONS
                .iter()
                .filter(|definition| {
                    definition.codec != "libfdk_aac" || available_encoders.libfdk_aac
                })
                .map(|definition| (definition.codec, definition.label)),
        )
        .map(|definition| {
            let is_compatible =
                is_audio_codec_allowed_for_container(&config.container, definition.0);
            AudioCodecOption {
                codec: definition.0,
                label: definition.1,
                is_selected: config.audio_codec.eq_ignore_ascii_case(definition.0),
                is_disabled: encode_controls_disabled || !is_compatible,
                disabled_reason: (!is_compatible).then_some("Incompatible container"),
            }
        })
        .collect()
}

#[must_use]
pub fn audio_channel_options(config: &ConversionConfig, disabled: bool) -> [AudioChannelOption; 3] {
    let disabled = disabled || config.processing_mode == ProcessingMode::Copy;

    AUDIO_CHANNEL_DEFINITIONS.map(|definition| AudioChannelOption {
        id: definition.id,
        label: definition.label,
        is_selected: config.audio_channels.eq_ignore_ascii_case(definition.id),
        is_disabled: disabled,
    })
}

#[must_use]
pub fn audio_track_options(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    disabled: bool,
) -> Vec<AudioTrackOption> {
    metadata
        .map(|metadata| {
            metadata
                .audio_tracks
                .iter()
                .map(|track| AudioTrackOption {
                    index: track.index,
                    index_label: format!("#{}", track.index),
                    codec: display_source_value(Some(&track.codec)),
                    detail: audio_track_detail(track),
                    is_selected: config.selected_audio_tracks.contains(&track.index),
                    is_disabled: disabled,
                })
                .collect()
        })
        .unwrap_or_default()
}

#[must_use]
pub fn metadata_mode_options(config: &ConversionConfig, disabled: bool) -> [MetadataModeOption; 3] {
    METADATA_MODES.map(|mode| MetadataModeOption {
        mode,
        label: mode.label(),
        is_selected: config.metadata.mode == mode,
        is_disabled: disabled,
    })
}

#[must_use]
pub fn metadata_field_options(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    disabled: bool,
) -> Vec<MetadataFieldOption> {
    metadata_fields_for_source(metadata)
        .iter()
        .copied()
        .map(|field| MetadataFieldOption {
            field,
            id: field.id(),
            label: field.label(),
            value: metadata_field_value(config, field)
                .unwrap_or_default()
                .to_string(),
            placeholder: metadata_field_placeholder(config, metadata, field),
            is_disabled: disabled,
        })
        .collect()
}

#[must_use]
pub fn metadata_fields_for_source(metadata: Option<&SourceMetadata>) -> Vec<MetadataField> {
    let is_image = source_kind_for(metadata) == SourceKind::Image;
    METADATA_FIELDS
        .iter()
        .copied()
        .filter(|field| !is_image || field.visible_for_image())
        .collect()
}

#[must_use]
pub fn metadata_field_value(config: &ConversionConfig, field: MetadataField) -> Option<&str> {
    match field {
        MetadataField::Title => config.metadata.title.as_deref(),
        MetadataField::Artist => config.metadata.artist.as_deref(),
        MetadataField::Album => config.metadata.album.as_deref(),
        MetadataField::Genre => config.metadata.genre.as_deref(),
        MetadataField::Date => config.metadata.date.as_deref(),
        MetadataField::Comment => config.metadata.comment.as_deref(),
    }
}

#[must_use]
pub fn metadata_field_placeholder(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    field: MetadataField,
) -> String {
    if config.metadata.mode != MetadataMode::Preserve {
        return String::new();
    }

    metadata
        .and_then(|metadata| metadata.tags.as_ref())
        .and_then(|tags| tags.value(field))
        .filter(|value| !value.trim().is_empty())
        .map_or_else(
            || "Leave empty to keep original".to_string(),
            ToString::to_string,
        )
}

#[must_use]
pub fn resolution_options() -> &'static [&'static str] {
    &RESOLUTION_OPTIONS
}

#[must_use]
pub fn scaling_algorithm_options() -> &'static [&'static str] {
    &SCALING_ALGORITHM_OPTIONS
}

#[must_use]
pub fn fps_options(is_gif: bool) -> &'static [&'static str] {
    if is_gif {
        &GIF_FPS_OPTIONS
    } else {
        &FPS_OPTIONS
    }
}

#[must_use]
pub fn gif_color_options() -> &'static [u16] {
    &GIF_COLOR_OPTIONS
}

#[must_use]
pub fn gif_dither_options() -> &'static [&'static str] {
    &GIF_DITHER_OPTIONS
}

#[must_use]
pub fn video_codec_options(
    config: &ConversionConfig,
    available_encoders: &AvailableEncoders,
    disabled: bool,
) -> Vec<VideoCodecOption> {
    VIDEO_CODEC_DEFINITIONS
        .iter()
        .filter(|definition| {
            definition.capability.is_none_or(|capability| {
                video_codec_capability_available(available_encoders, capability)
            })
        })
        .map(|definition| {
            let allowed = is_video_codec_allowed_for_container(&config.container, definition.codec);
            VideoCodecOption {
                codec: definition.codec,
                label: definition.label,
                is_selected: allowed && config.video_codec.eq_ignore_ascii_case(definition.codec),
                is_disabled: disabled || !allowed,
                disabled_reason: (!allowed).then_some("Incompatible container"),
            }
        })
        .collect()
}

#[must_use]
pub fn video_pixel_format_options(config: &ConversionConfig) -> Vec<VideoPixelFormatOption> {
    VIDEO_PIXEL_FORMAT_DEFINITIONS
        .iter()
        .map(|definition| {
            let allowed = is_video_pixel_format_allowed_for_container(
                &config.container,
                &config.video_codec,
                definition.id,
            );
            VideoPixelFormatOption {
                id: definition.id,
                label: definition.label,
                is_selected: allowed && config.pixel_format.eq_ignore_ascii_case(definition.id),
                is_disabled: !allowed,
                caption: if definition.id == "auto" {
                    "Encoder default"
                } else if allowed {
                    definition.id
                } else {
                    "Incompatible codec"
                },
            }
        })
        .collect()
}

#[must_use]
pub fn video_preset_options(config: &ConversionConfig, disabled: bool) -> Vec<VideoPresetOption> {
    VIDEO_PRESETS
        .iter()
        .map(|preset| {
            let allowed = is_video_preset_allowed(&config.video_codec, preset);
            VideoPresetOption {
                preset,
                label: video_preset_label(preset),
                caption: if allowed {
                    video_preset_caption(preset)
                } else {
                    "Incompatible preset"
                },
                is_selected: allowed && config.preset == *preset,
                is_disabled: disabled || !allowed,
            }
        })
        .collect()
}

#[must_use]
pub fn is_container_compatible_for_stream_copy(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    container: &str,
) -> bool {
    if config.processing_mode != ProcessingMode::Copy {
        return true;
    }
    if source_kind_for(metadata) == SourceKind::Image {
        return false;
    }
    if is_image_container(container) || is_gif_container(container) {
        return false;
    }

    let Some(metadata) = metadata else {
        return true;
    };

    let selected_audio_codecs = selected_audio_codecs(config, metadata);
    if is_audio_only_container(container) {
        return !selected_audio_codecs.is_empty()
            && selected_audio_codecs
                .iter()
                .all(|codec| is_audio_stream_codec_allowed_for_container(container, codec));
    }

    let Some(video_codec) = metadata.video_codec.as_deref() else {
        return false;
    };
    if !is_video_stream_codec_allowed_for_container(container, video_codec) {
        return false;
    }

    let audio_codecs_allowed = selected_audio_codecs
        .iter()
        .all(|codec| is_audio_stream_codec_allowed_for_container(container, codec));
    let subtitle_codecs_allowed = selected_subtitle_codecs(config, metadata)
        .iter()
        .all(|codec| is_subtitle_codec_allowed_for_container(container, codec));

    audio_codecs_allowed && subtitle_codecs_allowed
}

fn output_mode_option(
    mode: ProcessingMode,
    config: &ConversionConfig,
    is_disabled: bool,
) -> OutputModeOption {
    OutputModeOption {
        mode,
        label: mode.label(),
        hint: mode.hint(),
        is_selected: config.processing_mode == mode,
        is_disabled,
    }
}

fn output_container_disabled_reason(
    config: &ConversionConfig,
    metadata: Option<&SourceMetadata>,
    container: &str,
    disabled: bool,
) -> Option<&'static str> {
    let source_kind = source_kind_for(metadata);

    if disabled {
        return Some("Locked");
    }
    if source_kind == SourceKind::Audio && !is_audio_only_container(container) {
        return Some("Video container unavailable for audio sources");
    }
    if source_kind == SourceKind::Image && is_audio_only_container(container) {
        return Some("Audio container unavailable for image sources");
    }
    if !is_container_compatible_for_stream_copy(config, metadata, container) {
        return Some("Incompatible container");
    }

    None
}

fn selected_audio_codecs<'a>(
    config: &ConversionConfig,
    metadata: &'a SourceMetadata,
) -> Vec<&'a str> {
    if metadata.audio_tracks.is_empty() {
        return Vec::new();
    }
    if config.selected_audio_tracks.is_empty() {
        return metadata
            .audio_tracks
            .iter()
            .map(|track| track.codec.as_str())
            .collect();
    }

    metadata
        .audio_tracks
        .iter()
        .filter(|track| config.selected_audio_tracks.contains(&track.index))
        .map(|track| track.codec.as_str())
        .collect()
}

fn selected_subtitle_codecs<'a>(
    config: &ConversionConfig,
    metadata: &'a SourceMetadata,
) -> Vec<&'a str> {
    if metadata.subtitle_tracks.is_empty() {
        return Vec::new();
    }
    if config.selected_subtitle_tracks.is_empty() {
        return metadata
            .subtitle_tracks
            .iter()
            .map(|track| track.codec.as_str())
            .collect();
    }

    metadata
        .subtitle_tracks
        .iter()
        .filter(|track| config.selected_subtitle_tracks.contains(&track.index))
        .map(|track| track.codec.as_str())
        .collect()
}

fn video_codec_capability_available(
    available_encoders: &AvailableEncoders,
    capability: VideoCodecCapability,
) -> bool {
    match capability {
        VideoCodecCapability::H264Videotoolbox => available_encoders.h264_videotoolbox,
        VideoCodecCapability::H264Nvenc => available_encoders.h264_nvenc,
        VideoCodecCapability::HevcVideotoolbox => available_encoders.hevc_videotoolbox,
        VideoCodecCapability::HevcNvenc => available_encoders.hevc_nvenc,
        VideoCodecCapability::Av1Nvenc => available_encoders.av1_nvenc,
    }
}

#[must_use]
pub fn is_nvenc_video_codec(codec: &str) -> bool {
    matches!(codec, "h264_nvenc" | "hevc_nvenc" | "av1_nvenc")
}

#[must_use]
pub fn is_videotoolbox_video_codec(codec: &str) -> bool {
    matches!(codec, "h264_videotoolbox" | "hevc_videotoolbox")
}

#[must_use]
pub fn is_hardware_video_codec(codec: &str) -> bool {
    is_nvenc_video_codec(codec) || is_videotoolbox_video_codec(codec)
}

#[must_use]
pub fn is_video_preset_allowed(codec: &str, preset: &str) -> bool {
    if is_videotoolbox_video_codec(codec) {
        return true;
    }
    if is_nvenc_video_codec(codec) {
        return matches!(preset, "fast" | "medium" | "slow");
    }

    VIDEO_PRESETS.contains(&preset)
}

#[must_use]
pub fn first_allowed_video_preset(codec: &str) -> &'static str {
    VIDEO_PRESETS
        .iter()
        .copied()
        .find(|preset| is_video_preset_allowed(codec, preset))
        .unwrap_or("medium")
}

#[must_use]
pub fn first_allowed_video_codec(
    container: &str,
    available_encoders: Option<&AvailableEncoders>,
) -> String {
    let candidates = media_rules::video_codec_fallback_order().iter();
    let first = candidates
        .filter(|codec| {
            available_encoders.is_none_or(|encoders| {
                VIDEO_CODEC_DEFINITIONS
                    .iter()
                    .find(|definition| definition.codec == codec.as_str())
                    .and_then(|definition| definition.capability)
                    .is_none_or(|capability| video_codec_capability_available(encoders, capability))
            })
        })
        .find(|codec| is_video_codec_allowed_for_container(container, codec))
        .cloned();

    first.unwrap_or_else(|| "libx264".to_string())
}

#[must_use]
pub fn first_allowed_video_pixel_format(container: &str, encoder: &str) -> &'static str {
    VIDEO_PIXEL_FORMAT_DEFINITIONS
        .iter()
        .find(|definition| {
            is_video_pixel_format_allowed_for_container(container, encoder, definition.id)
        })
        .map_or("auto", |definition| definition.id)
}

#[must_use]
pub fn video_preset_label(preset: &str) -> &'static str {
    match preset {
        "ultrafast" => "Ultrafast",
        "superfast" => "Superfast",
        "veryfast" => "Very Fast",
        "faster" => "Faster",
        "fast" => "Fast",
        "medium" => "Medium",
        "slow" => "Slow",
        "slower" => "Slower",
        "veryslow" => "Very Slow",
        _ => "Medium",
    }
}

#[must_use]
pub fn video_preset_caption(preset: &str) -> &'static str {
    match preset {
        "ultrafast" => "Fastest encode, largest file",
        "superfast" => "Very fast encode",
        "veryfast" => "Fast encode",
        "faster" => "Faster than default",
        "fast" => "Fast with reasonable compression",
        "medium" => "Balanced default",
        "slow" => "Smaller file, slower encode",
        "slower" => "High compression",
        "veryslow" => "Smallest file, slowest encode",
        _ => "Balanced default",
    }
}
