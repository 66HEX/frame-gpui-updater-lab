use frame_core::capabilities::{self, AvailableEncoders};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, command};
use tauri_plugin_shell::ShellExt;

fn has_upscale_models(app: &AppHandle) -> bool {
    let Ok(models_path) = app
        .path()
        .resolve("resources/models", BaseDirectory::Resource)
    else {
        return false;
    };

    capabilities::required_upscale_model_files()
        .iter()
        .all(|name| models_path.join(name).is_file())
}

#[command]
pub async fn get_available_encoders(app: AppHandle) -> Result<AvailableEncoders, String> {
    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| e.to_string())?
        .args(capabilities::ffmpeg_encoder_list_args())
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_upscaler_sidecar = app.shell().sidecar("realesrgan-ncnn-vulkan").is_ok();
    let ml_upscale = has_upscaler_sidecar && has_upscale_models(&app);

    Ok(capabilities::parse_available_encoders(&stdout, ml_upscale))
}
