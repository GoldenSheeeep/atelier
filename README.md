# atelier

The shared part of three desktop office apps written in Rust with
[egui](https://github.com/emilk/egui):
[tableur](https://github.com/GoldenSheeeep/tableur) (Sheets),
[plume](https://github.com/GoldenSheeeep/plume) (Docs) and
[diapo](https://github.com/GoldenSheeeep/diapo) (Slides).

- **Look**: black window, near-black panels, white text and greys, a
  40 px title bar with round minimize / maximize / close buttons, toolbar
  icons drawn as lines (no icon font).
- **Fonts**: the system's sans, serif and monospace through fontconfig,
  with real bold faces.
- **Files**: open and save dialogs through the desktop portal.
- **Claude panel** (Ctrl+J in each app): a conversation down the right of
  the window. Each message runs [Claude Code](https://claude.com/claude-code)
  in the background (`claude -p`, your own login) with none of its usual
  tools, only the app's own, so all it can do is read and change the open
  document. Changes show live and undo with Ctrl+Z.
- **MCP bridge**: the app listens on a socket in `$XDG_RUNTIME_DIR`;
  Claude starts the app's own binary as an MCP server (`<app> --mcp
  <socket>`), which lists the app's tools and passes each call on to the
  running window.

## Using it

```toml
[dependencies]
atelier = { git = "https://github.com/GoldenSheeeep/atelier" }
```

```rust
fn main() -> eframe::Result {
    atelier::run("myapp", "My App", tools::list, App::new)
}
```

`run` starts the window, or the MCP bridge when the binary is started
with `--mcp <socket>`. In the app, take Claude's calls from
`bridge.pending()` (in `eframe::App::logic`, so it works while hidden) and
draw `claude.panel(ui, || context)`.

Needs `claude` (Claude Code) on the `PATH` for the panel (or
`OFFICE_CLAUDE` pointing at it), and `fc-match` for fonts.

## License

MIT
