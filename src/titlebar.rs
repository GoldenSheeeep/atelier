//! The title bar every window on this desktop has: 40 px, black, the
//! document's name on the left, then the app's own controls, and the round
//! 24 px minimize / maximize / close buttons on the right. Dragging the bar
//! where there's nothing else moves the window; a double click maximizes.

use crate::theme;
use eframe::egui::{self, Align, Layout, Sense, Stroke, Ui, ViewportCommand, vec2};

pub const HEIGHT: f32 = 40.0;

#[derive(Clone, Copy)]
enum WindowButton {
    Minimize,
    Maximize,
    Close,
}

fn window_button(ui: &mut Ui, kind: WindowButton) {
    let (r, resp) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::click());
    let p = ui.painter();
    let c = r.center();
    p.circle_filled(c, 12.0, if resp.hovered() { theme::HOVER } else { theme::BUTTON });
    let white = Stroke::new(2.0, egui::Color32::WHITE);
    let at = |x: f32, y: f32| egui::pos2(c.x - 12.0 + x, c.y - 12.0 + y);
    match kind {
        WindowButton::Minimize => {
            p.line_segment([at(8.5, 14.5), at(15.5, 14.5)], white);
        }
        WindowButton::Maximize => {
            let sq = egui::Rect::from_min_max(at(8.75, 8.75), at(15.25, 15.25));
            p.rect_stroke(sq, 1.0, white, egui::StrokeKind::Middle);
        }
        WindowButton::Close => {
            p.line_segment([at(8.75, 8.75), at(15.25, 15.25)], white);
            p.line_segment([at(15.25, 8.75), at(8.75, 15.25)], white);
        }
    }
    if resp.clicked() {
        ui.ctx().send_viewport_cmd(match kind {
            WindowButton::Minimize => ViewportCommand::Minimized(true),
            // The desktop decides whether that fills the screen or restores.
            WindowButton::Maximize => ViewportCommand::Maximized(true),
            WindowButton::Close => ViewportCommand::Close,
        });
    }
}

/// The bar, with `left` (the name, menus) after the document name and
/// `right` (the app's buttons) before the window buttons.
pub fn show(
    ui: &mut Ui,
    title: &str,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    egui::Panel::top("title_bar")
        .exact_size(HEIGHT)
        .show_separator_line(false)
        .frame(egui::Frame::new().fill(theme::BG).inner_margin(egui::Margin { left: 16, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let bar = ui.interact(ui.max_rect(), ui.scope_id().with("drag"), Sense::click_and_drag());
            if bar.drag_started() {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }
            if bar.double_clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
            }
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.label(egui::RichText::new(title).size(14.0).color(theme::TEXT));
                ui.add_space(10.0);
                // Menus as plain words, lit only under the pointer.
                ui.scope(|ui| {
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                    left(ui);
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    window_button(ui, WindowButton::Close);
                    window_button(ui, WindowButton::Maximize);
                    window_button(ui, WindowButton::Minimize);
                    ui.add_space(4.0);
                    ui.spacing_mut().item_spacing.x = 6.0;
                    right(ui);
                });
            });
        });
}
