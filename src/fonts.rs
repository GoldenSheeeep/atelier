//! The system's own fonts (through fontconfig), with real bold faces: egui
//! can slant text for italics but can't embolden it, so bold is a family of
//! its own. egui's built-in fonts stay behind them for symbols and emoji.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::process::Command;
use std::sync::Arc;

/// The families documents use, by the name they're picked with.
pub const FAMILIES: [&str; 3] = ["Sans", "Serif", "Mono"];

fn fc_match(pattern: &str) -> Option<Vec<u8>> {
    let out = Command::new("fc-match").args(["-f", "%{file}", pattern]).output().ok()?;
    let path = String::from_utf8(out.stdout).ok()?;
    std::fs::read(path.trim()).ok()
}

/// The egui family for a document font, bold or not.
pub fn family(name: &str, bold: bool) -> FontFamily {
    let base = match name {
        "Serif" => "serif",
        "Mono" => "mono",
        _ => "sans",
    };
    FontFamily::Name(if bold { format!("{base}-bold") } else { base.to_owned() }.into())
}

pub fn install(ctx: &egui::Context) {
    let mut defs = FontDefinitions::default();
    let fallback_prop = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let fallback_mono = defs.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    for (name, pattern, mono) in [
        ("sans", "sans-serif", false),
        ("sans-bold", "sans-serif:bold", false),
        ("serif", "serif", false),
        ("serif-bold", "serif:bold", false),
        ("mono", "monospace", true),
        ("mono-bold", "monospace:bold", true),
    ] {
        let mut chain = Vec::new();
        if let Some(bytes) = fc_match(pattern) {
            defs.font_data.insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
            chain.push(name.to_owned());
        }
        chain.extend(if mono { fallback_mono.clone() } else { fallback_prop.clone() });
        defs.families.insert(FontFamily::Name(name.into()), chain);
    }
    // The apps' own text in the system's sans too.
    if defs.font_data.contains_key("sans") {
        defs.families.entry(FontFamily::Proportional).or_default().insert(0, "sans".to_owned());
    }
    if defs.font_data.contains_key("mono") {
        defs.families.entry(FontFamily::Monospace).or_default().insert(0, "mono".to_owned());
    }
    ctx.set_fonts(defs);
}
