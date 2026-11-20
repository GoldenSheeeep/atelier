//! Toolbar icons, drawn as white lines like the launcher icons: no icon
//! font needed, and they stay sharp at any scale.

use crate::theme;
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Undo,
    Redo,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    Bullets,
    Numbers,
    TextColor,
    Fill,
    Plus,
    Minus,
    Trash,
    Up,
    Down,
    Image,
    Rect,
    Ellipse,
    Line,
    TextBox,
    Play,
    Sparkle,
    Send,
    Stop,
    Close,
    Duplicate,
    Quote,
    Search,
    Wrap,
}

/// Paints an icon in a 16 px box centred on `c`.
pub fn paint(p: &Painter, icon: Icon, c: Pos2, color: Color32) {
    let s = Stroke::new(1.6, color);
    let at = |x: f32, y: f32| pos2(c.x - 8.0 + x, c.y - 8.0 + y);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([at(a.0, a.1), at(b.0, b.1)], s);
    };
    let lines = |pts: &[(f32, f32)]| {
        p.add(egui::Shape::line(pts.iter().map(|&(x, y)| at(x, y)).collect(), s));
    };
    match icon {
        Icon::Undo => {
            lines(&[(5.0, 3.5), (2.0, 6.5), (5.0, 9.5)]);
            lines(&[(2.5, 6.5), (10.0, 6.5), (12.5, 7.5), (14.0, 10.0), (12.5, 12.5), (10.0, 13.5), (6.0, 13.5)]);
        }
        Icon::Redo => {
            lines(&[(11.0, 3.5), (14.0, 6.5), (11.0, 9.5)]);
            lines(&[(13.5, 6.5), (6.0, 6.5), (3.5, 7.5), (2.0, 10.0), (3.5, 12.5), (6.0, 13.5), (10.0, 13.5)]);
        }
        Icon::AlignLeft | Icon::AlignCenter | Icon::AlignRight | Icon::AlignJustify => {
            for (i, long) in [true, false, true, false].iter().enumerate() {
                let y = 3.5 + i as f32 * 3.0;
                let w = if *long || icon == Icon::AlignJustify { 12.0 } else { 8.0 };
                let x = match icon {
                    Icon::AlignLeft | Icon::AlignJustify => 2.0,
                    Icon::AlignCenter => 8.0 - w / 2.0,
                    _ => 14.0 - w,
                };
                line((x, y), (x + w, y));
            }
        }
        Icon::Bullets | Icon::Numbers => {
            for i in 0..3 {
                let y = 4.0 + i as f32 * 4.0;
                if icon == Icon::Bullets {
                    p.circle_filled(at(3.0, y), 1.3, color);
                } else {
                    line((2.5, y - 1.5), (3.5, y - 1.5));
                    line((3.5, y - 1.5), (3.5, y + 1.5));
                }
                line((6.5, y), (14.0, y));
            }
        }
        Icon::TextColor => {
            lines(&[(4.0, 11.0), (8.0, 2.5), (12.0, 11.0)]);
            line((5.5, 8.0), (10.5, 8.0));
        }
        Icon::Fill => {
            lines(&[(3.0, 7.0), (8.0, 2.0), (13.0, 7.0), (8.0, 12.0), (3.0, 7.0)]);
            p.circle_filled(at(14.0, 11.0), 1.4, color);
        }
        Icon::Plus => {
            line((8.0, 3.0), (8.0, 13.0));
            line((3.0, 8.0), (13.0, 8.0));
        }
        Icon::Minus => line((3.0, 8.0), (13.0, 8.0)),
        Icon::Trash => {
            line((3.0, 4.5), (13.0, 4.5));
            line((6.5, 4.5), (6.5, 2.5));
            line((6.5, 2.5), (9.5, 2.5));
            line((9.5, 2.5), (9.5, 4.5));
            lines(&[(4.5, 4.5), (5.2, 14.0), (10.8, 14.0), (11.5, 4.5)]);
        }
        Icon::Up => lines(&[(3.5, 10.5), (8.0, 5.5), (12.5, 10.5)]),
        Icon::Down => lines(&[(3.5, 5.5), (8.0, 10.5), (12.5, 5.5)]),
        Icon::Image => {
            p.rect_stroke(Rect::from_min_max(at(2.0, 3.0), at(14.0, 13.0)), 2.0, s, egui::StrokeKind::Middle);
            lines(&[(3.0, 12.0), (7.0, 8.0), (10.0, 11.0), (11.5, 9.5), (13.5, 11.5)]);
            p.circle_stroke(at(10.5, 6.0), 1.2, s);
        }
        Icon::Rect => {
            p.rect_stroke(Rect::from_min_max(at(2.5, 4.0), at(13.5, 12.0)), 1.5, s, egui::StrokeKind::Middle);
        }
        Icon::Ellipse => {
            p.add(egui::Shape::ellipse_stroke(at(8.0, 8.0), vec2(6.0, 4.5), s));
        }
        Icon::Line => line((3.0, 13.0), (13.0, 3.0)),
        Icon::TextBox => {
            line((3.5, 3.5), (12.5, 3.5));
            line((8.0, 3.5), (8.0, 13.0));
            line((6.0, 13.0), (10.0, 13.0));
        }
        Icon::Play => {
            p.add(egui::Shape::convex_polygon(vec![at(4.5, 3.0), at(13.0, 8.0), at(4.5, 13.0)], color, Stroke::NONE));
        }
        Icon::Sparkle => {
            let star = |cx: f32, cy: f32, r: f32| {
                let pts = [
                    (cx, cy - r),
                    (cx + r * 0.28, cy - r * 0.28),
                    (cx + r, cy),
                    (cx + r * 0.28, cy + r * 0.28),
                    (cx, cy + r),
                    (cx - r * 0.28, cy + r * 0.28),
                    (cx - r, cy),
                    (cx - r * 0.28, cy - r * 0.28),
                ];
                p.add(egui::Shape::convex_polygon(pts.iter().map(|&(x, y)| at(x, y)).collect(), color, Stroke::NONE));
            };
            star(7.0, 9.0, 6.0);
            star(13.0, 3.5, 2.6);
        }
        Icon::Send => {
            line((8.0, 13.0), (8.0, 3.5));
            lines(&[(3.5, 8.0), (8.0, 3.5), (12.5, 8.0)]);
        }
        Icon::Stop => {
            p.rect_filled(Rect::from_min_max(at(4.0, 4.0), at(12.0, 12.0)), 1.5, color);
        }
        Icon::Close => {
            line((4.0, 4.0), (12.0, 12.0));
            line((12.0, 4.0), (4.0, 12.0));
        }
        Icon::Duplicate => {
            p.rect_stroke(Rect::from_min_max(at(5.5, 5.5), at(14.0, 14.0)), 1.5, s, egui::StrokeKind::Middle);
            lines(&[(10.5, 3.0), (10.5, 2.0), (2.0, 2.0), (2.0, 10.5), (3.0, 10.5)]);
        }
        Icon::Quote => {
            line((4.0, 3.0), (4.0, 13.0));
            line((7.0, 5.0), (13.0, 5.0));
            line((7.0, 8.0), (13.0, 8.0));
            line((7.0, 11.0), (11.0, 11.0));
        }
        Icon::Search => {
            p.circle_stroke(at(7.0, 7.0), 4.5, s);
            line((10.3, 10.3), (14.0, 14.0));
        }
        Icon::Wrap => {
            line((2.0, 4.0), (14.0, 4.0));
            lines(&[(2.0, 8.0), (12.0, 8.0), (13.5, 9.5), (12.0, 11.0), (8.0, 11.0)]);
            lines(&[(9.5, 9.5), (8.0, 11.0), (9.5, 12.5)]);
            line((2.0, 12.0), (5.0, 12.0));
        }
    }
}

/// A square toolbar button with an icon: dark when hovered, lighter when on.
pub fn button(ui: &mut Ui, icon: Icon, on: bool, tip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
    paint_button_bg(ui, rect, &resp, on);
    let color = if ui.is_enabled() { theme::TEXT } else { theme::FAINT };
    paint(ui.painter(), icon, rect.center(), color);
    resp.on_hover_text(tip)
}

/// A square toolbar button with a few letters instead (B, I, U...).
pub fn text_button(ui: &mut Ui, label: egui::RichText, on: bool, tip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
    paint_button_bg(ui, rect, &resp, on);
    let color = if ui.is_enabled() { theme::TEXT } else { theme::FAINT };
    let galley = egui::WidgetText::from(label.color(color)).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Button,
    );
    let pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(pos, galley, color);
    resp.on_hover_text(tip)
}

fn paint_button_bg(ui: &Ui, rect: Rect, resp: &Response, on: bool) {
    let fill = if on {
        theme::ACTIVE
    } else if resp.hovered() && ui.is_enabled() {
        theme::HOVER
    } else {
        return;
    };
    ui.painter().rect_filled(rect, 7.0, fill);
}
