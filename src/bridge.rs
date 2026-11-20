//! The app's end of Claude's tools: a socket in $XDG_RUNTIME_DIR that the
//! MCP bridge (mcp.rs, started by Claude) sends tool calls to, one JSON line
//! each. Calls wait here until the app takes them on its own thread, where
//! the document lives, and answers.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

pub struct Request {
    pub tool: String,
    pub args: Value,
    reply: Sender<Result<String, String>>,
}

impl Request {
    pub fn reply(self, result: Result<String, String>) {
        let _ = self.reply.send(result);
    }

    /// A string argument, or an error naming it.
    pub fn str(&self, key: &str) -> Result<&str, String> {
        self.args.get(key).and_then(Value::as_str).ok_or_else(|| format!("missing \"{key}\""))
    }

    pub fn opt_str(&self, key: &str) -> Option<&str> {
        self.args.get(key).and_then(Value::as_str)
    }

    pub fn int(&self, key: &str) -> Result<i64, String> {
        self.args.get(key).and_then(Value::as_i64).ok_or_else(|| format!("missing \"{key}\""))
    }

    pub fn opt_int(&self, key: &str) -> Option<i64> {
        self.args.get(key).and_then(Value::as_i64)
    }
}

pub struct Bridge {
    pub path: PathBuf,
    rx: Receiver<Request>,
}

impl Bridge {
    pub fn start(app: &str, ctx: eframe::egui::Context) -> Bridge {
        let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let path = dir.join(format!("{app}-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let (tx, rx) = mpsc::channel();
        match UnixListener::bind(&path) {
            Ok(listener) => {
                std::thread::spawn(move || {
                    for stream in listener.incoming().flatten() {
                        let tx = tx.clone();
                        let ctx = ctx.clone();
                        std::thread::spawn(move || serve(stream, tx, ctx));
                    }
                });
            }
            Err(e) => eprintln!("{app}: can't listen on {}: {e}", path.display()),
        }
        Bridge { path, rx }
    }

    /// Tool calls waiting to be answered.
    pub fn pending(&self) -> Vec<Request> {
        self.rx.try_iter().collect()
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn serve(stream: UnixStream, tx: Sender<Request>, ctx: eframe::egui::Context) {
    let Ok(mut out) = stream.try_clone() else { return };
    for line in BufReader::new(stream).lines() {
        let Ok(line) = line else { return };
        let msg: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
        let (reply_tx, reply_rx) = mpsc::channel();
        let req = Request {
            tool: msg["tool"].as_str().unwrap_or_default().to_owned(),
            args: msg.get("args").cloned().unwrap_or(json!({})),
            reply: reply_tx,
        };
        if tx.send(req).is_err() {
            return;
        }
        ctx.request_repaint();
        let answer = match reply_rx.recv_timeout(Duration::from_secs(60)) {
            Ok(Ok(text)) => json!({ "ok": text }),
            Ok(Err(e)) => json!({ "error": e }),
            Err(_) => json!({ "error": "the app didn't answer" }),
        };
        if writeln!(out, "{answer}").is_err() {
            return;
        }
    }
}
