/// Returns a sorted list of font family names available on the system.
#[tauri::command]
pub fn list_system_fonts() -> Vec<String> {
    frame_core::fonts::list_system_font_families()
}
