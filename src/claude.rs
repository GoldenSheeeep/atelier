//! The Claude panel: a conversation down the right of the window. Each
//! message runs Claude Code in the background (`claude -p`, your own login,
//! no API key) with none of its usual tools (files, shell, web), only the
//! app's own (mcp.rs), so all it can do is read and change the open
//! document. The conversation carries on from message to message
//! (`--resume`) until "New chat".

use crate::icons::{self, Icon};
use crate::theme;
use eframe::egui::{self, Align, Key, Layout, RichText, Ui, text::LayoutJob, vec2};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

enum Msg {
    User(String),
    Text(String),
    Action(String),
    Error(String),
}

enum Event {
    Session(String),
    Text(String),
    Action(String),
    Done(Option<String>),
}

pub struct Claude {
    app: String,
    pub open: bool,
    messages: Vec<Msg>,
    input: String,
    session: Option<String>,
    child: Arc<Mutex<Option<Child>>>,
    running: bool,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    system_prompt: String,
    socket: PathBuf,
    examples: Vec<String>,
    focus_input: bool,
    scroll_to_end: bool,
}

impl Claude {
    /// `about` tells Claude what the app and its document are; `examples`
    /// are offered as first questions.
    pub fn new(app: &str, socket: PathBuf, about: &str, examples: &[&str]) -> Claude {
        let (tx, rx) = mpsc::channel();
        let system_prompt = format!(
            "You are Claude, working inside {app}, the user's {about} app, beside the document they have open. \
             Read and change that document only through the `doc` tools: they act on the open document, live, \
             and every change you make can be undone by the user with Ctrl+Z. Look before you change things. \
             Each of the user's messages starts with what they are looking at in brackets. \
             Keep answers short and plain: the panel is narrow. Don't describe tool calls the user can see happen; \
             say what you changed in a sentence."
        );
        Claude {
            app: app.to_owned(),
            open: false,
            messages: Vec::new(),
            input: String::new(),
            session: None,
            child: Arc::new(Mutex::new(None)),
            running: false,
            tx,
            rx,
            system_prompt,
            socket,
            examples: examples.iter().map(|s| (*s).to_owned()).collect(),
            focus_input: false,
            scroll_to_end: false,
        }
    }

    /// While Claude is at work the document is its alone, so its changes
    /// and the user's typing don't cross.
    pub fn busy(&self) -> bool {
        self.running
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.focus_input = self.open;
    }

    fn send(&mut self, text: String, context: String, ctx: &egui::Context) {
        let text = text.trim().to_owned();
        if text.is_empty() || self.running {
            return;
        }
        self.messages.push(Msg::User(text.clone()));
        self.running = true;
        self.scroll_to_end = true;
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from(self.app.to_lowercase()));
        let mcp = json!({ "mcpServers": { "doc": {
            "command": exe, "args": ["--mcp", self.socket],
        } } });
        let claude = std::env::var("OFFICE_CLAUDE").unwrap_or_else(|_| "claude".to_owned());
        let mut cmd = Command::new(claude);
        cmd.current_dir(crate::state_dir())
            .args(["-p", "--output-format", "stream-json", "--verbose"])
            .args(["--tools", "", "--strict-mcp-config"])
            .arg("--mcp-config")
            .arg(mcp.to_string())
            .args(["--allowedTools", "mcp__doc"])
            .arg("--append-system-prompt")
            .arg(&self.system_prompt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(id) = &self.session {
            cmd.args(["--resume", id]);
        }
        let prompt = format!("[{context}]\n\n{text}");
        let tx = self.tx.clone();
        let slot = self.child.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let done = |tx: &Sender<Event>, e: Option<String>| {
                let _ = tx.send(Event::Done(e));
                ctx.request_repaint();
            };
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => return done(&tx, Some(format!("Couldn't start Claude Code: {e}"))),
            };
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(prompt.as_bytes());
            }
            let stdout = child.stdout.take();
            let mut stderr = child.stderr.take();
            *slot.lock().unwrap() = Some(child);
            let mut error = None;
            let mut finished = false;
            if let Some(out) = stdout {
                for line in BufReader::new(out).lines() {
                    let Ok(line) = line else { break };
                    let Ok(v) = serde_json::from_str::<Value>(&line) else { continue };
                    if let Some(id) = v["session_id"].as_str() {
                        let _ = tx.send(Event::Session(id.to_owned()));
                    }
                    match v["type"].as_str() {
                        Some("assistant") => {
                            for part in v["message"]["content"].as_array().into_iter().flatten() {
                                match part["type"].as_str() {
                                    Some("text") => {
                                        let t = part["text"].as_str().unwrap_or_default().trim();
                                        if !t.is_empty() {
                                            let _ = tx.send(Event::Text(t.to_owned()));
                                        }
                                    }
                                    Some("tool_use") => {
                                        let _ = tx.send(Event::Action(describe(
                                            part["name"].as_str().unwrap_or_default(),
                                            &part["input"],
                                        )));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some("result") => {
                            finished = true;
                            if v["is_error"].as_bool() == Some(true) {
                                error = Some(v["result"].as_str().unwrap_or("Claude stopped with an error.").to_owned());
                            }
                        }
                        _ => {}
                    }
                    ctx.request_repaint();
                }
            }
            let status = slot.lock().unwrap().take().map(|mut c| c.wait());
            if !finished && error.is_none() {
                let mut msg = String::new();
                if let Some(e) = stderr.as_mut() {
                    let _ = e.read_to_string(&mut msg);
                }
                let killed = status.and_then(|s| s.ok()).is_some_and(|s| s.code().is_none());
                if !killed {
                    let msg = msg.trim();
                    error = Some(if msg.is_empty() { "Claude stopped without answering.".to_owned() } else { msg.to_owned() });
                }
            }
            done(&tx, error);
        });
    }

    fn stop(&mut self) {
        if let Some(c) = self.child.lock().unwrap().as_mut() {
            let _ = c.kill();
        }
    }

    fn take_events(&mut self) {
        for e in self.rx.try_iter() {
            match e {
                Event::Session(id) => self.session = Some(id),
                Event::Text(t) => self.messages.push(Msg::Text(t)),
                Event::Action(a) => self.messages.push(Msg::Action(a)),
                Event::Done(err) => {
                    self.running = false;
                    if let Some(e) = err {
                        self.messages.push(Msg::Error(e));
                    }
                }
            }
            self.scroll_to_end = true;
        }
    }

    /// The panel, down the right of the window, when open. `context` says
    /// what the user is looking at (the sheet and selection, the slide...).
    pub fn panel(&mut self, ui: &mut Ui, context: impl FnOnce() -> String) {
        self.take_events();
        if !self.open {
            return;
        }
        let ctx = ui.ctx().clone();
        let mut send = None;
        egui::Panel::right("claude")
            .resizable(true)
            .default_size(380.0)
            .min_size(300.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .inner_margin(egui::Margin::symmetric(14, 12)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    icons::paint(ui.painter(), Icon::Sparkle, ui.cursor().min + vec2(8.0, 10.0), theme::TEXT);
                    ui.add_space(22.0);
                    ui.label(RichText::new("Claude").size(15.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if icons::button(ui, Icon::Close, false, "Close (Ctrl+J)").clicked() {
                            self.open = false;
                        }
                        let new = ui.add_enabled(!self.running && !self.messages.is_empty(), egui::Button::new("New chat"));
                        if new.clicked() {
                            self.messages.clear();
                            self.session = None;
                        }
                    });
                });
                ui.add_space(8.0);

                // The message box, at the bottom.
                egui::Panel::bottom("claude_input")
                    .show_separator_line(false)
                    .frame(egui::Frame::new().inner_margin(egui::Margin { top: 10, ..Default::default() }))
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .fill(theme::BG)
                            .stroke(egui::Stroke::new(1.0, theme::BORDER))
                            .corner_radius(14.0)
                            .inner_margin(egui::Margin::symmetric(10, 8))
                            .show(ui, |ui| {
                                let id = ui.scope_id().with("input");
                                if ui.memory(|m| m.has_focus(id))
                                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter))
                                {
                                    send = Some(std::mem::take(&mut self.input));
                                }
                                let hint = format!("Ask Claude about this {}…", self.app_noun());
                                ui.horizontal(|ui| {
                                    let edit = egui::TextEdit::multiline(&mut self.input)
                                        .id(id)
                                        .hint_text(hint)
                                        .frame(egui::Frame::NONE)
                                        .desired_rows(2)
                                        .desired_width(ui.available_width() - 36.0);
                                    let r = ui.add(edit);
                                    if self.focus_input {
                                        r.request_focus();
                                        self.focus_input = false;
                                    }
                                    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                                        if self.running {
                                            if icons::button(ui, Icon::Stop, false, "Stop").clicked() {
                                                self.stop();
                                            }
                                        } else if icons::button(ui, Icon::Send, false, "Send (Enter)").clicked() {
                                            send = Some(std::mem::take(&mut self.input));
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                        ui.label(RichText::new("Enter sends, Shift+Enter starts a new line.").size(11.0).color(theme::FAINT));
                    });

                egui::ScrollArea::vertical().auto_shrink([false; 2]).stick_to_bottom(true).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if self.messages.is_empty() {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!(
                                "Ask about this {0}, or ask for changes: Claude reads and edits it directly, and Ctrl+Z undoes anything it did.",
                                self.app_noun()
                            ))
                            .color(theme::MUTED),
                        );
                        ui.add_space(10.0);
                        for ex in self.examples.clone() {
                            let b = egui::Button::new(RichText::new(&ex).size(13.0)).fill(theme::BUTTON).corner_radius(12.0);
                            if ui.add_enabled(!self.running, b).clicked() {
                                send = Some(ex);
                            }
                        }
                    }
                    for m in &self.messages {
                        match m {
                            Msg::User(t) => {
                                ui.add_space(6.0);
                                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                                    egui::Frame::new()
                                        .fill(theme::BUTTON)
                                        .corner_radius(14.0)
                                        .inner_margin(egui::Margin::symmetric(12, 8))
                                        .show(ui, |ui| {
                                            ui.set_max_width(ui.available_width() * 0.85);
                                            ui.label(t);
                                        });
                                });
                                ui.add_space(4.0);
                            }
                            Msg::Text(t) => markdown(ui, t),
                            Msg::Action(a) => {
                                ui.label(RichText::new(format!("•  {a}")).size(12.0).color(theme::MUTED));
                            }
                            Msg::Error(e) => {
                                ui.label(RichText::new(e).color(egui::Color32::from_rgb(0xe0, 0x66, 0x66)));
                            }
                        }
                    }
                    if self.running {
                        let t = ui.input(|i| i.time);
                        let dots = ".".repeat(1 + (t * 2.0) as usize % 3);
                        ui.label(RichText::new(format!("Working{dots}")).color(theme::MUTED));
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
                    }
                    if self.scroll_to_end {
                        ui.scroll_to_cursor(Some(Align::Max));
                        self.scroll_to_end = false;
                    }
                });
            });
        if let Some(text) = send {
            self.send(text, context(), &ctx);
        }
    }

    fn app_noun(&self) -> &'static str {
        match self.app.as_str() {
            "Sheets" => "spreadsheet",
            "Slides" => "presentation",
            _ => "document",
        }
    }
}

/// One line about a tool call, for the conversation: "Write cells A1".
fn describe(tool: &str, input: &Value) -> String {
    let name = tool.trim_start_matches("mcp__doc__").replace('_', " ");
    let mut s = name[..1].to_uppercase() + &name[1..];
    for key in ["range", "start", "sheet", "slide", "find", "at", "first"] {
        if let Some(v) = input.get(key) {
            let v = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            s.push_str(&format!(" {v}"));
            break;
        }
    }
    s
}

/// Claude's answers, with the little Markdown it uses: headings, lists,
/// **bold**, `code` and code blocks.
pub fn markdown(ui: &mut Ui, text: &str) {
    let bold = egui::FontId::new(14.0, crate::fonts::family("Sans", true));
    let body = egui::FontId::proportional(14.0);
    let mono = egui::FontId::monospace(13.0);
    let mut in_code = false;
    let mut code = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_code {
                egui::Frame::new().fill(theme::BG).corner_radius(8.0).inner_margin(8.0).show(ui, |ui| {
                    ui.label(RichText::new(code.trim_end()).font(mono.clone()));
                });
                code.clear();
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            code.push_str(line);
            code.push('\n');
            continue;
        }
        let trimmed = line.trim_start();
        let (prefix, rest, font) = if let Some(h) = trimmed.strip_prefix("### ").or(trimmed.strip_prefix("## ")).or(trimmed.strip_prefix("# ")) {
            ("", h, bold.clone())
        } else if let Some(b) = trimmed.strip_prefix("- ").or(trimmed.strip_prefix("* ")) {
            ("•  ", b, body.clone())
        } else {
            ("", trimmed, body.clone())
        };
        if rest.is_empty() {
            ui.add_space(4.0);
            continue;
        }
        let mut job = LayoutJob::default();
        job.wrap.max_width = ui.available_width();
        let fmt = |font: egui::FontId| egui::TextFormat { font_id: font, color: theme::TEXT, ..Default::default() };
        job.append(prefix, 0.0, fmt(body.clone()));
        // **bold** and `code` spans.
        let mut s = rest;
        while !s.is_empty() {
            let next_bold = s.find("**");
            let next_code = s.find('`');
            match (next_bold, next_code) {
                (Some(b), c) if c.is_none_or(|c| b < c) => {
                    job.append(&s[..b], 0.0, fmt(font.clone()));
                    let after = &s[b + 2..];
                    let end = after.find("**").unwrap_or(after.len());
                    job.append(&after[..end], 0.0, fmt(bold.clone()));
                    s = after.get(end + 2..).unwrap_or("");
                }
                (_, Some(c)) => {
                    job.append(&s[..c], 0.0, fmt(font.clone()));
                    let after = &s[c + 1..];
                    let end = after.find('`').unwrap_or(after.len());
                    let mut f = fmt(mono.clone());
                    f.background = theme::BG;
                    job.append(&after[..end], 0.0, f);
                    s = after.get(end + 1..).unwrap_or("");
                }
                _ => {
                    job.append(s, 0.0, fmt(font.clone()));
                    s = "";
                }
            }
        }
        ui.label(job);
    }
}
