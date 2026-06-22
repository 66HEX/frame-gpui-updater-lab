#[expect(
    unused_imports,
    reason = "Tauri keeps the old conversion::codec path as a migration compatibility re-export"
)]
pub use frame_core::codec::*;
