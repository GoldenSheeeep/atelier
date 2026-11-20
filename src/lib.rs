//! atelier: what Sheets, Docs and Slides share: the black look, the title bar, the
//! toolbar icons, file dialogs and the Claude panel with the tools it uses
//! to read and change the open document.

pub mod bridge;
pub mod claude;
pub mod closing;
pub mod fonts;
pub mod icons;
pub mod mcp;
pub mod theme;
pub mod titlebar;
pub mod widgets;

pub use eframe::egui;

use std::path::PathBuf;

/// Starts an app: the MCP bridge instead when Claude starts the binary as
/// one (`<app> --mcp <socket>`), the window otherwise.
pub fn run<A: eframe::App + 'static>(
    app_id: &str,
    title: &str,
    tools: fn() -> serde_json::Value,
    make: impl FnOnce(&eframe::CreationContext<'_>) -> A + 'static,
) -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--mcp" {
        mcp::serve(app_id, tools(), &args[2]);
        std::process::exit(0);
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(app_id)
            .with_title(title)
            .with_decorations(false)
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([640.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            fonts::install(&cc.egui_ctx);
            Ok(Box::new(make(cc)))
        }),
    )
}

/// The file given on the command line (opening a file from Files), if any.
pub fn file_arg() -> Option<PathBuf> {
    std::env::args().nth(1).filter(|a| !a.starts_with("--")).map(PathBuf::from)
}

/// Where the apps keep their own state (Claude's conversations start here).
pub fn state_dir() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state"));
    let dir = base.join("office");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Open and save dialogs (the system's, Ski, through the desktop portal).
pub mod files {
    use std::path::{Path, PathBuf};

    pub fn open(kind: &str, extensions: &[&str]) -> Option<PathBuf> {
        rfd::FileDialog::new().add_filter(kind, extensions).pick_file()
    }

    pub fn save(kind: &str, extensions: &[&str], suggested: &str, dir: Option<&Path>) -> Option<PathBuf> {
        let mut d = rfd::FileDialog::new().add_filter(kind, extensions).set_file_name(suggested);
        if let Some(dir) = dir {
            d = d.set_directory(dir);
        }
        let mut path = d.save_file()?;
        // A name typed without the extension gets it.
        let has_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.iter().any(|x| x.eq_ignore_ascii_case(e)));
        if !has_ext {
            path.set_extension(extensions[0]);
        }
        Some(path)
    }

    /// A file's name without folder and extension, for the title bar.
    pub fn stem(path: &Path) -> String {
        path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }
}
