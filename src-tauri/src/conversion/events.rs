use frame_core::events::ConversionEvent;
use tauri::{AppHandle, Emitter};

pub fn emit_conversion_event(app: &AppHandle, event: ConversionEvent) {
    let event_name = event.event_name();

    match event {
        ConversionEvent::Started(payload) => {
            let _ = app.emit(event_name, payload);
        }
        ConversionEvent::Progress(payload) => {
            let _ = app.emit(event_name, payload);
        }
        ConversionEvent::Completed(payload) => {
            let _ = app.emit(event_name, payload);
        }
        ConversionEvent::Error(payload) => {
            let _ = app.emit(event_name, payload);
        }
        ConversionEvent::Log(payload) => {
            let _ = app.emit(event_name, payload);
        }
        ConversionEvent::Cancelled(payload) => {
            let _ = app.emit(event_name, payload);
        }
    }
}
