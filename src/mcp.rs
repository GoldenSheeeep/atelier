//! The MCP server Claude starts (`<app> --mcp <socket>`): it speaks MCP on
//! stdin and stdout, lists the app's tools, and passes each call on to the
//! running app's socket (bridge.rs).

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

pub fn serve(app: &str, tools: Value, socket: &str) {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut conn: Option<(UnixStream, BufReader<UnixStream>)> = None;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let Ok(msg) = serde_json::from_str::<Value>(&line) else { continue };
        let Some(id) = msg.get("id").cloned() else { continue }; // a notification
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let result = match msg["method"].as_str().unwrap_or_default() {
            "initialize" => Ok(json!({
                "protocolVersion": params["protocolVersion"].as_str().unwrap_or("2025-06-18"),
                "capabilities": { "tools": {} },
                "serverInfo": { "name": app, "version": env!("CARGO_PKG_VERSION") },
            })),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tools })),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or_default();
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                let (text, is_error) = match call(&mut conn, socket, name, args) {
                    Ok(t) => (t, false),
                    Err(e) => (e, true),
                };
                Ok(json!({ "content": [{ "type": "text", "text": text }], "isError": is_error }))
            }
            other => Err(format!("unknown method {other}")),
        };
        let reply = match result {
            Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
            Err(e) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": e } }),
        };
        if writeln!(stdout, "{reply}").and_then(|_| stdout.flush()).is_err() {
            break;
        }
    }
}

fn call(
    conn: &mut Option<(UnixStream, BufReader<UnixStream>)>,
    socket: &str,
    tool: &str,
    args: Value,
) -> Result<String, String> {
    if conn.is_none() {
        let s = UnixStream::connect(socket).map_err(|e| format!("the app isn't reachable: {e}"))?;
        let r = BufReader::new(s.try_clone().map_err(|e| e.to_string())?);
        *conn = Some((s, r));
    }
    let (s, r) = conn.as_mut().unwrap();
    let res = (|| {
        writeln!(s, "{}", json!({ "tool": tool, "args": args })).map_err(|e| e.to_string())?;
        let mut line = String::new();
        r.read_line(&mut line).map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        match v.get("ok").and_then(Value::as_str) {
            Some(ok) => Ok(ok.to_owned()),
            None => Err(v["error"].as_str().unwrap_or("failed").to_owned()),
        }
    })();
    if res.is_err() {
        *conn = None;
    }
    res
}

/// A tool's description for `tools/list`, from its parameters: each one a
/// (name, JSON type, description, required) row.
pub fn tool(name: &str, description: &str, params: &[(&str, &str, &str, bool)]) -> Value {
    let mut props = serde_json::Map::new();
    let mut required = Vec::new();
    for (p, ty, desc, req) in params {
        let schema = match *ty {
            "rows" => json!({ "type": "array", "description": desc,
                "items": { "type": "array", "items": { "type": ["string", "number", "boolean", "null"] } } }),
            "object" => json!({ "type": "object", "description": desc }),
            "array" => json!({ "type": "array", "description": desc, "items": { "type": "object" } }),
            t => json!({ "type": t, "description": desc }),
        };
        props.insert((*p).to_owned(), schema);
        if *req {
            required.push(*p);
        }
    }
    json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": props, "required": required },
    })
}
