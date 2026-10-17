//! Window colours. The default is Tarık's Zed theme "Islands iTerm", as in Ithildin and Redbook;
//! other themes are read from Zed's theme files, each colour from the Zed key next to it.

use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde_json::Value;

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: u32,       // background
    pub surface: u32,  // elevated_surface.background
    pub hover: u32,    // element.hover
    pub selected: u32, // element.selected
    pub border: u32,   // border
    pub text: u32,     // text
    pub muted: u32,    // text.muted
    pub accent: u32,   // text.accent, else players[0].cursor
    pub focus: u32,    // border.focused
    pub error: u32,    // error
    pub caret: u32,    // players[0].cursor
    pub success: u32,  // success
    pub warning: u32,  // warning
}

pub const DEFAULT_THEME: &str = "Islands iTerm";

/// ~/.config/zed/themes/islands-iterm.json
const ISLANDS: Palette = Palette {
    dark: true,
    bg: 0x0f141a,
    surface: 0x171c23,
    hover: 0x1e232b,
    selected: 0x2e436e,
    border: 0x22272e,
    text: 0xdfe1e5,
    muted: 0x868a91,
    accent: 0x548af7,
    focus: 0x3574f0,
    error: 0xfa6675,
    caret: 0xced0d6,
    success: 0x73bd79,
    warning: 0xf2c55c,
};

static CURRENT: RwLock<Palette> = RwLock::new(ISLANDS);

/// The palette the window draws with now.
pub fn pal() -> Palette {
    *CURRENT.read().unwrap_or_else(|e| e.into_inner())
}

pub fn set(p: Palette) {
    *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = p;
}

/// Islands iTerm first, then every theme in Zed's user and extension theme files, by name.
pub fn available() -> Vec<(String, Palette)> {
    let mut themes = vec![(DEFAULT_THEME.to_string(), ISLANDS)];
    let (config, data) = zed_dirs();
    let mut files: Vec<PathBuf> = json_files(&config.join("themes"));
    if let Ok(exts) = std::fs::read_dir(data.join("extensions/installed")) {
        for ext in exts.flatten() {
            files.extend(json_files(&ext.path().join("themes")));
        }
    }
    for file in files {
        let Some(family) = std::fs::read(&file)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        else {
            continue;
        };
        for theme in family["themes"].as_array().into_iter().flatten() {
            if let Some(name) = theme["name"].as_str()
                && !themes.iter().any(|(n, _)| n == name)
            {
                themes.push((name.to_string(), from_zed(theme)));
            }
        }
    }
    themes
}

/// Zed's config and data folders: ~/.config/zed and its data dir on macOS and Linux,
/// %APPDATA%\Zed and %LOCALAPPDATA%\Zed on Windows.
fn zed_dirs() -> (PathBuf, PathBuf) {
    let home = crate::config::home();
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    if cfg!(windows) {
        let config = env("APPDATA").unwrap_or_default().join("Zed");
        (config, env("LOCALAPPDATA").unwrap_or_default().join("Zed"))
    } else if cfg!(target_os = "macos") {
        (
            home.join(".config/zed"),
            home.join("Library/Application Support/Zed"),
        )
    } else {
        let data = env("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share"));
        (home.join(".config/zed"), data.join("zed"))
    }
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    found.sort();
    found
}

/// A Zed theme as a palette; a key the theme leaves out falls back to Islands iTerm's.
pub fn from_zed(theme: &Value) -> Palette {
    let style = &theme["style"];
    let dark = theme["appearance"].as_str() != Some("light");
    let bg = color(&style["background"], None).unwrap_or(if dark { ISLANDS.bg } else { 0xffffff });
    let c = |key: &str| color(&style[key], Some(bg));
    let cursor = color(&style["players"][0]["cursor"], Some(bg));
    Palette {
        dark,
        bg,
        surface: c("elevated_surface.background").unwrap_or(ISLANDS.surface),
        hover: c("element.hover").unwrap_or(ISLANDS.hover),
        selected: c("element.selected").unwrap_or(ISLANDS.selected),
        border: c("border").unwrap_or(ISLANDS.border),
        text: c("text").unwrap_or(ISLANDS.text),
        muted: c("text.muted").unwrap_or(ISLANDS.muted),
        accent: c("text.accent").or(cursor).unwrap_or(ISLANDS.accent),
        focus: c("border.focused").unwrap_or(ISLANDS.focus),
        error: c("error").unwrap_or(ISLANDS.error),
        caret: cursor.unwrap_or(ISLANDS.caret),
        success: c("success").unwrap_or(ISLANDS.success),
        warning: c("warning").unwrap_or(ISLANDS.warning),
    }
}

/// "#rrggbb" or "#rrggbbaa"; a translucent colour is blended over `over`.
fn color(v: &Value, over: Option<u32>) -> Option<u32> {
    let hex = v.as_str()?.strip_prefix('#')?;
    let n = u32::from_str_radix(hex, 16).ok()?;
    match hex.len() {
        6 => Some(n),
        8 => {
            let (rgb, alpha) = (n >> 8, (n & 0xff) as f32 / 255.0);
            Some(match over {
                Some(base) => blend(base, rgb, alpha),
                None => rgb,
            })
        }
        _ => None,
    }
}

/// `a` moved towards `b` by `f` (0..1), per channel.
fn blend(a: u32, b: u32, f: f32) -> u32 {
    let ch = |shift: u32| {
        let (x, y) = (((a >> shift) & 0xff) as f32, ((b >> shift) & 0xff) as f32);
        ((x + (y - x) * f).round() as u32) << shift
    };
    ch(16) | ch(8) | ch(0)
}

/// Where the chosen theme's name is kept, next to the config.
fn choice_path() -> PathBuf {
    crate::config::config_dir().join("theme")
}

pub fn saved_choice() -> String {
    std::fs::read_to_string(choice_path())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| DEFAULT_THEME.to_string())
}

pub fn save_choice(name: &str) {
    let _ = std::fs::create_dir_all(crate::config::config_dir());
    if let Err(e) = std::fs::write(choice_path(), name) {
        eprintln!("save theme: {e}");
    }
}

/// The palette as a gpui-component theme, so its own widgets match the window.
pub fn component_theme(p: &Palette) -> serde_json::Value {
    let hex = |c: u32| format!("#{c:06x}");
    serde_json::json!({
        "name": "Mellon",
        "mode": if p.dark { "dark" } else { "light" },
        "colors": {
            "background": hex(p.bg),
            "foreground": hex(p.text),
            "border": hex(p.border),
            "input.border": hex(p.border),
            "ring": hex(p.focus),
            "caret": hex(p.caret),
            "muted.background": hex(p.surface),
            "muted.foreground": hex(p.muted),
            "popover.background": hex(p.surface),
            "popover.foreground": hex(p.text),
            "accent.background": hex(p.hover),
            "accent.foreground": hex(p.text),
            "primary.background": hex(p.accent),
            "primary.foreground": hex(p.bg),
            "primary.hover.background": hex(p.focus),
            "primary.active.background": hex(p.focus),
            "secondary.background": hex(p.surface),
            "secondary.foreground": hex(p.text),
            "secondary.hover.background": hex(p.hover),
            "secondary.active.background": hex(p.selected),
            "selection.background": hex(p.selected),
            "list.active.background": hex(p.selected),
            "list.active.border": hex(p.focus),
            "link": hex(p.accent),
            "info.background": hex(p.accent),
            "success.background": hex(p.success),
            "warning.background": hex(p.warning),
            "danger.background": hex(p.error),
            "scrollbar.thumb.background": if p.dark { "#ffffff26" } else { "#00000026" }
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn component_theme_is_a_valid_gpui_component_theme() {
        let theme = super::component_theme(&super::ISLANDS);
        let config: gpui_kit::component::ThemeConfig =
            serde_json::from_value(theme.clone()).unwrap();
        // serde drops unknown keys silently: every colour given must come back out.
        let read_back = serde_json::to_value(&config.colors).unwrap();
        for (key, value) in theme["colors"].as_object().unwrap() {
            assert_eq!(read_back.get(key), Some(value), "no colour {key:?}");
        }
    }

    #[test]
    fn zed_themes_blend_alpha_and_fall_back() {
        let p = super::from_zed(&serde_json::json!({
            "name": "T", "appearance": "light",
            "style": { "background": "#ffffff", "element.hover": "#00000080", "text": "#000000",
                       "players": [{ "cursor": "#3b9ee5ff" }] }
        }));
        assert!(!p.dark);
        assert_eq!(p.hover, 0x7f7f7f, "half-black over white");
        assert_eq!(p.accent, 0x3b9ee5, "no text.accent: the cursor colour");
        assert_eq!(
            p.error,
            super::ISLANDS.error,
            "missing key: Islands iTerm's"
        );
    }
}
