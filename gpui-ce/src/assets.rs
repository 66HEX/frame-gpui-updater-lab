//! Bundled assets for the GPUI rewrite.

use std::borrow::Cow;

use gpui::{App, AssetSource, Result, SharedString};

pub const FRAME_FONT_FAMILY: &str = "IoskeleyMono";
pub const FRAME_FONT_PATH: &str = "fonts/IoskeleyMono-Black.woff2";
pub const ICON_FRAME: &str = "icons/frame.svg";
pub const ICON_LAYOUT_LIST: &str = "icons/layout-list.svg";
pub const ICON_TERMINAL: &str = "icons/terminal.svg";
pub const ICON_HARD_DRIVE: &str = "icons/hard-drive.svg";
pub const ICON_FILE_VIDEO: &str = "icons/file-video.svg";
pub const ICON_SETTINGS: &str = "icons/settings.svg";
pub const ICON_PLUS: &str = "icons/plus.svg";
pub const ICON_PLAY: &str = "icons/play.svg";

const FRAME_ICON_SVG: &str = include_str!("../../src/lib/assets/icons/frame.svg");
const FRAME_FONT_BYTES: &[u8] =
    include_bytes!("../../src/lib/assets/fonts/LoskeleyMono/IoskeleyMono-Black.woff2");

const LAYOUT_LIST_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 256 256"><path d="M32,64a8,8,0,0,1,8-8H216a8,8,0,0,1,0,16H40A8,8,0,0,1,32,64Zm104,56H40a8,8,0,0,0,0,16h96a8,8,0,0,0,0-16Zm0,64H40a8,8,0,0,0,0,16h96a8,8,0,0,0,0-16Zm112-24a8,8,0,0,1-3.76,6.78l-64,40A8,8,0,0,1,168,200V120a8,8,0,0,1,12.24-6.78l64,40A8,8,0,0,1,248,160Zm-23.09,0L184,134.43v51.14Z"/></svg>"#;
const TERMINAL_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 256 256"><path d="M117.31,134l-72,64a8,8,0,1,1-10.63-12L100,128,34.69,70A8,8,0,1,1,45.32,58l72,64a8,8,0,0,1,0,12ZM216,184H120a8,8,0,0,0,0,16h96a8,8,0,0,0,0-16Z"/></svg>"#;
const HARD_DRIVE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 256 256"><path d="M208,136H48a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V152A16,16,0,0,0,208,136Zm0,64H48V152H208v48Zm0-160H48A16,16,0,0,0,32,56v48a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V56A16,16,0,0,0,208,40Zm0,64H48V56H208v48ZM192,80a12,12,0,1,1-12-12A12,12,0,0,1,192,80Zm0,96a12,12,0,1,1-12-12A12,12,0,0,1,192,176Z"/></svg>"#;
const FILE_VIDEO_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 256 256"><path d="M213.66,82.34l-56-56A8,8,0,0,0,152,24H56A16,16,0,0,0,40,40v72a8,8,0,0,0,16,0V40h88V88a8,8,0,0,0,8,8h48V216h-8a8,8,0,0,0,0,16h8a16,16,0,0,0,16-16V88A8,8,0,0,0,213.66,82.34ZM160,51.31,188.69,80H160ZM155.88,145a8,8,0,0,0-8.12.22l-19.95,12.46A16,16,0,0,0,112,144H48a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16h64a16,16,0,0,0,15.81-13.68l19.95,12.46A8,8,0,0,0,160,216V152A8,8,0,0,0,155.88,145ZM112,208H48V160h64v48Zm32-6.43-16-10V176.43l16-10Z"/></svg>"#;
const SETTINGS_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 256 256"><path d="M64,105V40a8,8,0,0,0-16,0v65a32,32,0,0,0,0,62v49a8,8,0,0,0,16,0V167a32,32,0,0,0,0-62Zm-8,47a16,16,0,1,1,16-16A16,16,0,0,1,56,152Zm80-95V40a8,8,0,0,0-16,0V57a32,32,0,0,0,0,62v97a8,8,0,0,0,16,0V119a32,32,0,0,0,0-62Zm-8,47a16,16,0,1,1,16-16A16,16,0,0,1,128,104Zm104,64a32.06,32.06,0,0,0-24-31V40a8,8,0,0,0-16,0v97a32,32,0,0,0,0,62v17a8,8,0,0,0,16,0V199A32.06,32.06,0,0,0,232,168Zm-32,16a16,16,0,1,1,16-16A16,16,0,0,1,200,184Z"/></svg>"#;
const PLUS_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 256 256"><path d="M224,128a8,8,0,0,1-8,8H136v80a8,8,0,0,1-16,0V136H40a8,8,0,0,1,0-16h80V40a8,8,0,0,1,16,0v80h80A8,8,0,0,1,224,128Z"/></svg>"#;
const PLAY_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 256 256"><path d="M240,128a15.74,15.74,0,0,1-7.6,13.51L88.32,229.65a16,16,0,0,1-16.2.3A15.86,15.86,0,0,1,64,216.13V39.87a15.86,15.86,0,0,1,8.12-13.82,16,16,0,0,1,16.2.3L232.4,114.49A15.74,15.74,0,0,1,240,128Z"/></svg>"#;

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameAssets;

impl AssetSource for FrameAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let asset = match path {
            FRAME_FONT_PATH => Cow::Borrowed(FRAME_FONT_BYTES),
            ICON_FRAME => Cow::Borrowed(FRAME_ICON_SVG.as_bytes()),
            ICON_LAYOUT_LIST => Cow::Borrowed(LAYOUT_LIST_SVG.as_bytes()),
            ICON_TERMINAL => Cow::Borrowed(TERMINAL_SVG.as_bytes()),
            ICON_HARD_DRIVE => Cow::Borrowed(HARD_DRIVE_SVG.as_bytes()),
            ICON_FILE_VIDEO => Cow::Borrowed(FILE_VIDEO_SVG.as_bytes()),
            ICON_SETTINGS => Cow::Borrowed(SETTINGS_SVG.as_bytes()),
            ICON_PLUS => Cow::Borrowed(PLUS_SVG.as_bytes()),
            ICON_PLAY => Cow::Borrowed(PLAY_SVG.as_bytes()),
            _ => return Ok(None),
        };

        Ok(Some(asset))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let assets = match path {
            "fonts" => vec![SharedString::from("IoskeleyMono-Black.woff2")],
            "icons" => vec![
                SharedString::from("file-video.svg"),
                SharedString::from("frame.svg"),
                SharedString::from("hard-drive.svg"),
                SharedString::from("layout-list.svg"),
                SharedString::from("play.svg"),
                SharedString::from("plus.svg"),
                SharedString::from("settings.svg"),
                SharedString::from("terminal.svg"),
            ],
            _ => Vec::new(),
        };

        Ok(assets)
    }
}

pub fn load_frame_fonts(cx: &mut App) -> Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(FRAME_FONT_BYTES)])
}

#[cfg(test)]
mod tests {
    use super::*;

    mod frame_assets {
        use super::*;

        #[test]
        fn load_returns_frame_icon_svg() {
            let loaded = FrameAssets
                .load(ICON_FRAME)
                .expect("asset load should not fail");

            assert!(
                loaded
                    .as_deref()
                    .is_some_and(|bytes| bytes.starts_with(b"<svg"))
            );
        }

        #[test]
        fn load_returns_none_for_unknown_asset() {
            let loaded = FrameAssets
                .load("icons/missing.svg")
                .expect("asset load should not fail");

            assert!(loaded.is_none());
        }

        #[test]
        fn list_returns_titlebar_icon_assets() {
            let listed = FrameAssets
                .list("icons")
                .expect("asset list should not fail");

            assert!(listed.iter().any(|name| name.as_ref() == "layout-list.svg"));
        }
    }
}
