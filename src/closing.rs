//! "Save changes?" when a window with unsaved changes is closed.

use crate::theme;
use eframe::egui::{self, RichText, ViewportCommand};

pub enum Choice {
    /// Save, then call `close_now`.
    Save,
}

#[derive(Default)]
pub struct CloseGuard {
    asking: bool,
    allowed: bool,
}

impl CloseGuard {
    /// Call every frame: holds a close back while there are unsaved
    /// changes, and asks.
    pub fn check(&mut self, ctx: &egui::Context, dirty: bool) {
        if ctx.input(|i| i.viewport().close_requested()) && dirty && !self.allowed {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.asking = true;
        }
    }

    pub fn close_now(&mut self, ctx: &egui::Context) {
        self.allowed = true;
        self.asking = false;
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    /// The question, while asking.
    pub fn dialog(&mut self, ctx: &egui::Context, name: &str) -> Option<Choice> {
        if !self.asking {
            return None;
        }
        let mut choice = None;
        let modal = egui::Modal::new(egui::Id::unique("close_guard"))
            .frame(egui::Frame::new().fill(theme::PANEL).stroke(egui::Stroke::new(1.0, theme::BORDER)).corner_radius(14.0).inner_margin(20.0))
            .show(ctx, |ui| {
                ui.set_width(340.0);
                ui.label(RichText::new(format!("Save changes to “{name}”?")).size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Your changes are lost if you don't save them.").color(theme::MUTED));
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        choice = Some(Choice::Save);
                    }
                    if ui.button("Don't save").clicked() {
                        self.close_now(ui.ctx());
                    }
                    if ui.button("Cancel").clicked() {
                        self.asking = false;
                    }
                });
            });
        if modal.should_close() {
            self.asking = false;
        }
        if choice.is_some() {
            self.asking = false;
        }
        choice
    }
}
