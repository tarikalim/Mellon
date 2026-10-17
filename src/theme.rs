//! Window colours: Tarık's Zed theme "Islands iTerm", as in Ithildin and Redbook.

pub struct Palette {
    pub bg: u32,      // background
    pub surface: u32, // elevated_surface.background
    pub hover: u32,   // element.hover
    pub selected: u32,
    pub border: u32,
    pub text: u32,
    pub muted: u32,
    pub accent: u32,
    pub focus: u32,
    pub error: u32,
    pub caret: u32,
    pub success: u32,
    pub warning: u32,
}

// ponytail: one built-in theme; Ithildin's Zed theme picker if a second one is ever wanted.
pub const PAL: Palette = Palette {
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

/// The palette as a gpui-component theme, so its own widgets match the window.
pub fn component_theme(p: &Palette) -> serde_json::Value {
    let hex = |c: u32| format!("#{c:06x}");
    serde_json::json!({
        "name": "Mellon",
        "mode": "dark",
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
            "scrollbar.thumb.background": "#ffffff26"
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn component_theme_is_a_valid_gpui_component_theme() {
        let theme = super::component_theme(&super::PAL);
        let config: gpui_kit::component::ThemeConfig =
            serde_json::from_value(theme.clone()).unwrap();
        // serde drops unknown keys silently: every colour given must come back out.
        let read_back = serde_json::to_value(&config.colors).unwrap();
        for (key, value) in theme["colors"].as_object().unwrap() {
            assert_eq!(read_back.get(key), Some(value), "no colour {key:?}");
        }
    }
}
