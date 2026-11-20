//! The look every app on this desktop has: black window, near-black panels,
//! dark grey borders, white text and greys only.

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const PANEL: Color32 = Color32::from_rgb(0x0a, 0x0a, 0x0a);
pub const RAISED: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
pub const BUTTON: Color32 = Color32::from_rgb(0x19, 0x19, 0x19);
pub const HOVER: Color32 = Color32::from_rgb(0x28, 0x28, 0x28);
pub const ACTIVE: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const BORDER: Color32 = Color32::from_rgb(0x1f, 0x1f, 0x1f);
pub const TEXT: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf2);
pub const MUTED: Color32 = Color32::from_rgb(0x8c, 0x8c, 0x8c);
pub const FAINT: Color32 = Color32::from_rgb(0x55, 0x55, 0x55);
/// Selections and the cursor's cell: a light grey, never a colour.
pub const SELECT: Color32 = Color32::from_rgb(0xd0, 0xd0, 0xd0);
pub const SELECT_FILL: Color32 = Color32::from_rgba_premultiplied(0x30, 0x30, 0x30, 0x60);

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.faint_bg_color = PANEL;
    v.extreme_bg_color = BG;
    v.code_bg_color = RAISED;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(10);
    v.override_text_color = Some(TEXT);
    v.hyperlink_color = TEXT;
    v.selection.bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    v.selection.stroke = Stroke::new(1.0, TEXT);
    v.text_cursor.stroke = Stroke::new(2.0, TEXT);
    let w = &mut v.widgets;
    for (state, fill) in [
        (&mut w.noninteractive, PANEL),
        (&mut w.inactive, BUTTON),
        (&mut w.hovered, HOVER),
        (&mut w.active, ACTIVE),
        (&mut w.open, HOVER),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, BORDER);
        state.fg_stroke = Stroke::new(1.0, TEXT);
        state.corner_radius = CornerRadius::same(8);
    }
    w.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
    w.inactive.bg_stroke = Stroke::NONE;
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.interact_size.y = 28.0;
    });
}

/// Luminance, 0 (black) to 1 (white), to tell dark colours from light ones.
pub fn luminance(c: Color32) -> f32 {
    (0.2126 * c.r() as f32 + 0.7152 * c.g() as f32 + 0.0722 * c.b() as f32) / 255.0
}

/// `#rrggbb` (or `rrggbb`) to a colour.
pub fn parse_hex(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    let s = if s.len() == 8 { &s[2..] } else { s }; // ARGB from Office files
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some(Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

pub fn to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// The colours offered for text, fills and shapes: greys, then a row of
/// colours (the document's own content may have colour; the apps don't).
pub const PALETTE: [&str; 20] = [
    "#FFFFFF", "#D9D9D9", "#A6A6A6", "#737373", "#404040", "#262626", "#121212", "#000000", "#7F7F7F",
    "#BFBFBF", "#E06666", "#F6B26B", "#FFD966", "#93C47D", "#76A5AF", "#6FA8DC", "#8E7CC3", "#C27BA0",
    "#CC0000", "#1155CC",
];
