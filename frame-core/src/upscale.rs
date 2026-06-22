use std::path::Path;

use crate::args::add_metadata_flags;
use crate::codec::{
    add_audio_codec_args, add_fps_args, add_subtitle_codec_args, add_video_codec_args,
};
use crate::error::ConversionError;
use crate::filters::build_audio_filters;
use crate::media_rules::{
    container_supports_audio, container_supports_subtitles, is_image_container,
};
use crate::types::{ConversionConfig, MetadataMode};

#[must_use]
pub fn build_upscale_encode_args(
    output_frames_dir: &Path,
    source_file_path: &str,
    output_path: &str,
    source_fps: f64,
    config: &ConversionConfig,
    source_pixel_format: Option<String>,
) -> Vec<String> {
    let is_image_output = is_image_container(&config.container);
    let supports_audio = container_supports_audio(&config.container);
    let supports_subtitles = container_supports_subtitles(&config.container);
    let has_burn_subtitles = config
        .subtitle_burn_path
        .as_ref()
        .is_some_and(|path| !path.trim().is_empty());

    let mut enc_args = vec![
        "-framerate".to_string(),
        source_fps.to_string(),
        "-start_number".to_string(),
        "1".to_string(),
        "-i".to_string(),
        output_frames_dir
            .join("frame_%08d.png")
            .to_string_lossy()
            .to_string(),
    ];

    if let Some(start) = &config.start_time
        && !start.is_empty()
    {
        enc_args.push("-ss".to_string());
        enc_args.push(start.clone());
    }

    enc_args.push("-i".to_string());
    enc_args.push(source_file_path.to_string());

    match config.metadata.mode {
        MetadataMode::Clean => {
            enc_args.push("-map_metadata".to_string());
            enc_args.push("-1".to_string());
        }
        MetadataMode::Replace => {
            enc_args.push("-map_metadata".to_string());
            enc_args.push("-1".to_string());
            add_metadata_flags(&mut enc_args, &config.metadata);
        }
        MetadataMode::Preserve => {
            enc_args.push("-map_metadata".to_string());
            enc_args.push("1".to_string());
            add_metadata_flags(&mut enc_args, &config.metadata);
        }
    }

    enc_args.push("-map".to_string());
    enc_args.push("0:v:0".to_string());

    if supports_audio {
        if config.selected_audio_tracks.is_empty() {
            enc_args.push("-map".to_string());
            enc_args.push("1:a?".to_string());
        } else {
            for track_index in &config.selected_audio_tracks {
                enc_args.push("-map".to_string());
                enc_args.push(format!("1:{track_index}"));
            }
        }
    }

    if supports_subtitles {
        if !config.selected_subtitle_tracks.is_empty() {
            for track_index in &config.selected_subtitle_tracks {
                enc_args.push("-map".to_string());
                enc_args.push(format!("1:{track_index}"));
            }
        } else if !has_burn_subtitles {
            enc_args.push("-map".to_string());
            enc_args.push("1:s?".to_string());
        }
    }

    add_video_codec_args(&mut enc_args, config);

    if supports_audio {
        add_audio_codec_args(&mut enc_args, config);

        let audio_filters = build_audio_filters(config);
        if !audio_filters.is_empty() {
            enc_args.push("-af".to_string());
            enc_args.push(audio_filters.join(","));
        }
    }

    if supports_subtitles && (!config.selected_subtitle_tracks.is_empty() || !has_burn_subtitles) {
        add_subtitle_codec_args(&mut enc_args, config);
    }

    if is_image_output {
        let configured_pixel_format = config.pixel_format.trim();
        if !configured_pixel_format.is_empty() && configured_pixel_format != "auto" {
            enc_args.push("-pix_fmt".to_string());
            enc_args.push(configured_pixel_format.to_string());
        }

        enc_args.push("-frames:v".to_string());
        enc_args.push("1".to_string());
        enc_args.push("-update".to_string());
        enc_args.push("1".to_string());
    } else {
        add_fps_args(&mut enc_args, config);

        enc_args.push("-pix_fmt".to_string());
        let configured_pixel_format = config.pixel_format.trim();
        if !configured_pixel_format.is_empty() && configured_pixel_format != "auto" {
            enc_args.push(configured_pixel_format.to_string());
        } else if let Some(pixel_format) = source_pixel_format {
            let normalized = pixel_format.trim().to_string();
            if normalized.contains("10") || normalized.contains("12") {
                enc_args.push(normalized);
            } else {
                enc_args.push("yuv420p".to_string());
            }
        } else {
            enc_args.push("yuv420p".to_string());
        }

        enc_args.push("-shortest".to_string());
    }
    enc_args.push("-y".to_string());
    enc_args.push(output_path.to_string());

    enc_args
}

pub fn resolve_upscale_mode(mode: &str) -> Result<(&'static str, &'static str), ConversionError> {
    match mode {
        "esrgan-2x" => Ok(("2", "realesr-animevideov3-x2")),
        "esrgan-4x" => Ok(("4", "realesr-animevideov3-x4")),
        _ => Err(ConversionError::InvalidInput(format!(
            "Invalid upscale mode: {mode}"
        ))),
    }
}

#[must_use]
pub fn compute_upscale_threads(source_width: u32, source_height: u32, scale: u32) -> String {
    let output_pixels = (u64::from(source_width) * u64::from(scale))
        * (u64::from(source_height) * u64::from(scale));

    let proc = if output_pixels > 8_294_400 {
        1
    } else if output_pixels > 2_073_600 {
        2
    } else {
        4
    };

    let cpus = std::thread::available_parallelism()
        .map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX))
        .unwrap_or(4);
    let io = cpus.div_ceil(2).clamp(1, 4);

    format!("{io}:{proc}:{io}")
}

#[must_use]
pub fn ceil_to_u32_saturating(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    if value >= f64::from(u32::MAX) {
        return u32::MAX;
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "value is finite, non-negative and bounded to u32 range"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "negative values are returned early before the cast"
    )]
    let converted = value.ceil() as u32;
    converted
}

#[must_use]
pub fn usize_to_u32_saturating(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_proc(threads_str: &str) -> u32 {
        threads_str
            .split(':')
            .nth(1)
            .expect("upscale thread format should contain proc slot")
            .parse::<u32>()
            .expect("proc slot should be a u32")
    }

    #[test]
    fn resolve_upscale_mode_accepts_real_esrgan_2x() {
        assert_eq!(
            resolve_upscale_mode("esrgan-2x").expect("2x mode should resolve"),
            ("2", "realesr-animevideov3-x2")
        );
    }

    #[test]
    fn resolve_upscale_mode_rejects_unknown_modes() {
        assert!(matches!(
            resolve_upscale_mode("nearest-8x"),
            Err(ConversionError::InvalidInput(_))
        ));
    }

    #[test]
    fn compute_upscale_threads_limits_gpu_proc_for_large_outputs() {
        let result = compute_upscale_threads(1920, 1080, 4);

        assert_eq!(extract_proc(&result), 1);
    }

    #[test]
    fn ceil_to_u32_saturating_clamps_invalid_and_large_values() {
        assert_eq!(ceil_to_u32_saturating(f64::NAN), 0);
        assert_eq!(ceil_to_u32_saturating(-1.0), 0);
        assert_eq!(ceil_to_u32_saturating(f64::from(u32::MAX) + 1.0), u32::MAX);
        assert_eq!(ceil_to_u32_saturating(12.2), 13);
    }

    #[test]
    fn usize_to_u32_saturating_clamps_values_above_u32() {
        assert_eq!(usize_to_u32_saturating(usize::MAX), u32::MAX);
    }
}
