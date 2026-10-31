# Configuration

On macOS/Linux, persisted to
`${XDG_CONFIG_HOME:-$HOME/.config}/tuxedo/config.toml`. Native Windows from
2026.10.2 uses `%APPDATA%\tuxedo\config.toml` by default. For home-relative
paths, `%USERPROFILE%` supplies the home directory when `HOME` is absent.
An explicit absolute `XDG_CONFIG_HOME` takes priority;
an existing legacy `~/.config/tuxedo` directory is kept in place rather than
silently migrated. Keybindings and user themes share that configuration base.
The update-check cache uses `%LOCALAPPDATA%\tuxedo-task` on native Windows
(or the explicit `XDG_CACHE_HOME`). Task files stay at the paths you choose.

Cycling theme, density, or sort, and toggling sidebars / line-numbers / done-visibility
all update the file. Active project/context/person filters are also persisted. Unknown
keys are ignored.

Fork-specific layout options (unquoted values, as in the existing config):

```toml
mouse = true
borders = true
margin = 0
```

`mouse` and `borders` default to `true`; `margin` defaults to `0`. Set `mouse = false`
for terminal-native text selection. Upstream Tuxedo shares this configuration
location but may not understand fork-only fields.

**Hot-reload.** Edits to `config.toml` are picked up while the TUI is running —
change the theme, density, sort, layout, saved filters, or any other field and
the UI updates within ~200 ms. Parse failures (e.g. a typo mid-edit) leave the
running config intact and flash a warning in the status bar.

Two additional keys, `share_token` and `share_port`, are written by the
[phone capture](phone-capture.md#phone-capture) server on first use. Treat `share_token`
as a secret — anyone who has the value and LAN reach can append to your
inbox. Delete the key from `config.toml` to rotate it on the next `s`
press.

Saved searches (created with `fs`) are written one per line as
`filter.<name> = <query>`, where `<query>` is the `/`-search needle. They
round-trip as plain text, so you can add, rename, or delete them by editing
`config.toml` directly; a repeated `filter.<name>` keeps the last value, and
`<name>` may not contain `=`.

Task-note actions resolve relative `note:<path>` tokens under `notes_dir`.
If `notes_dir` is not set, tuxedo falls back to `$NOTES_DIR` and then
`~/notes`. `O` creates missing notes under `projects/tuxedo-tasks/` using a
small Markdown template and appends the generated `note:<path>` token to the
task; `o` only opens an existing linked note.

```toml
notes_dir = ~/notes
```

## Hiding `key:value` tags

Some `key:value` extensions are for machines, not eyes — e.g. a `uid:` you
sync against. Add a comma-separated `hide_keys` line to `config.toml` and
those keys' tokens are dropped from the task rows (list and archive views):

```toml
hide_keys = uid, sync
```

Matching is case-insensitive. Hiding is purely visual — the tags stay on
disk untouched, still serialize, and still show in the detail pane's **RAW**
section (a deliberate escape hatch). Searches still match hidden text; the
hidden characters just aren't drawn.

---

[Documentation index](README.md) · [Project overview](../README.md)
