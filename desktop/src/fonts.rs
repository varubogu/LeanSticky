use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

pub fn install_cjk_fallback_font(ctx: &Context) {
    let Some(font_path) = find_cjk_font_path() else {
        return;
    };
    let Ok(font_bytes) = fs::read(&font_path) else {
        return;
    };

    let font_name = "leansticky_cjk_fallback".to_owned();
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        font_name.clone(),
        Arc::new(FontData::from_owned(font_bytes)),
    );

    if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
        family.push(font_name.clone());
    }
    if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
        family.push(font_name);
    }

    ctx.set_fonts(fonts);
}

fn find_cjk_font_path() -> Option<PathBuf> {
    font_candidates()
        .iter()
        .map(Path::new)
        .find(|path| path.is_file())
        .map(Path::to_path_buf)
}

#[cfg(target_os = "linux")]
fn font_candidates() -> &'static [&'static str] {
    &[
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansJP-Regular.otf",
        "/usr/share/fonts/truetype/fonts-japanese-gothic.ttf",
        "/usr/share/fonts/truetype/ipafont-gothic/ipag.ttf",
        "/usr/share/fonts/truetype/takao-gothic/TakaoPGothic.ttf",
        "/mnt/c/Windows/Fonts/BIZ-UDGothicR.ttc",
        "/mnt/c/Windows/Fonts/YuGothR.ttc",
        "/mnt/c/Windows/Fonts/YuGothM.ttc",
        "/mnt/c/Windows/Fonts/meiryo.ttc",
        "/mnt/c/Windows/Fonts/msgothic.ttc",
    ]
}

#[cfg(target_os = "windows")]
fn font_candidates() -> &'static [&'static str] {
    &[
        "C:\\Windows\\Fonts\\BIZ-UDGothicR.ttc",
        "C:\\Windows\\Fonts\\YuGothR.ttc",
        "C:\\Windows\\Fonts\\YuGothM.ttc",
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
    ]
}

#[cfg(target_os = "macos")]
fn font_candidates() -> &'static [&'static str] {
    &[
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    ]
}
