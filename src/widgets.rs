//! Small pieces the three apps' toolbars share.

use crate::icons::{self, Icon};
use crate::theme;
use eframe::egui::{self, Color32, Ui, vec2};

/// A toolbar row: 44 px, near-black, a line under it.
pub fn toolbar(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Panel::top("toolbar")
        .exact_size(44.0)
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL)
                .inner_margin(egui::Margin::symmetric(12, 0))
                .stroke(egui::Stroke::new(1.0, theme::BORDER)),
        )
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                add(ui);
            });
        });
}

/// A thin vertical line between groups of toolbar buttons.
pub fn separator(ui: &mut Ui) {
    ui.add_space(6.0);
    let (r, _) = ui.allocate_exact_size(vec2(1.0, 20.0), egui::Sense::hover());
    ui.painter().rect_filled(r, 0.0, theme::BORDER);
    ui.add_space(6.0);
}

/// A colour button (text colour, fill...) with the current colour as a bar
/// under its icon; picking one returns it, `None` meaning "automatic".
pub fn color_button(ui: &mut Ui, icon: Icon, current: Option<Color32>, tip: &str) -> Option<Option<Color32>> {
    let resp = icons::button(ui, icon, false, tip);
    let bar = egui::Rect::from_center_size(resp.rect.center_bottom() - vec2(0.0, 5.0), vec2(14.0, 3.0));
    ui.painter().rect_filled(bar, 1.0, current.unwrap_or(theme::FAINT));
    let mut picked = None;
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(206.0);
        if ui.button("Automatic").clicked() {
            picked = Some(None);
        }
        ui.add_space(4.0);
        egui::Grid::new("palette").spacing(vec2(4.0, 4.0)).show(ui, |ui| {
            for (i, hex) in theme::PALETTE.iter().enumerate() {
                let c = theme::parse_hex(hex).unwrap();
                let (r, swatch) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::click());
                ui.painter().rect_filled(r, 4.0, c);
                if swatch.hovered() || current == Some(c) {
                    ui.painter().rect_stroke(r.expand(1.5), 5.0, egui::Stroke::new(1.5, theme::TEXT), egui::StrokeKind::Middle);
                }
                if swatch.clicked() {
                    picked = Some(Some(c));
                }
                if i % 10 == 9 {
                    ui.end_row();
                }
            }
        });
    });
    if picked.is_some() {
        egui::Popup::close_all(ui.ctx());
    }
    picked
}

/// A small dropdown of text choices; returns the index picked.
pub fn choice(ui: &mut Ui, id: &str, current: &str, width: f32, options: &[&str]) -> Option<usize> {
    let mut picked = None;
    egui::ComboBox::from_id_salt(id).selected_text(current).width(width).show_ui(ui, |ui| {
        for (i, o) in options.iter().enumerate() {
            if ui.selectable_label(*o == current, *o).clicked() {
                picked = Some(i);
            }
        }
    });
    picked
}

/// A pill-shaped "Claude" button for the title bar, lit while its panel is open.
pub fn claude_toggle(ui: &mut Ui, open: bool) -> bool {
    let galley = ui.painter().layout_no_wrap("Claude".to_owned(), egui::FontId::proportional(13.0), theme::TEXT);
    let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 44.0, 28.0), egui::Sense::click());
    let fill = if open { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::BUTTON };
    let p = ui.painter();
    p.rect_filled(rect, 14.0, fill);
    icons::paint(p, Icon::Sparkle, rect.left_center() + vec2(18.0, 0.0), theme::TEXT);
    p.galley(egui::pos2(rect.left() + 32.0, rect.center().y - galley.size().y / 2.0), galley, theme::TEXT);
    resp.on_hover_text("Ask Claude about this document (Ctrl+J)").clicked()
}
