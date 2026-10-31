# Phone capture

Press `s` to start a tiny capture server on your machine's LAN address and
display a QR code for it. Scan it from your phone — any modern browser — to
get a minimal PWA you can install to your home screen. Type a task, tap
Add, and within a tick it shows up in your task list.

Captures never touch `todo.txt` directly. They land in a sibling
`inbox.txt`, which tuxedo drains on every external-change poll: each line
is run through the same natural-language pipeline as the `n` add prompt,
given a creation date if missing, and merged into `todo.txt` as a single
undoable batch (`u` rolls back the whole drain at once).

Advanced file-based producers and shell automation are covered in the
[command-line reference](cli.md#capture-from-scripts), near the end.

The server:

- Binds on first `s` press and stays up for the rest of the session.
  Subsequent `s` presses just re-show the QR; any key dismisses the
  overlay.
- Listens on `0.0.0.0:<port>` so phones on the same WiFi can reach it.
  The port is OS-assigned on first use and persisted to `config.toml` so
  phone bookmarks survive across sessions.
- Gates every protected route on a 64-character hex token baked into the
  URL path. The token is generated once, persisted to `config.toml`, and
  compared in constant time.
- Speaks plain HTTP — **trusted networks only.** On a shared or public
  WiFi anyone passive-sniffing can recover the token. To rotate, delete
  `share_token` from `config.toml` and press `s` again.

Drains from tuxedo-managed producers are crash-safe: the capture server
holds the same advisory lock as the TUI's rename-and-merge, and any
staging file left over from an interrupted drain is replayed on the
next session. Plain shell appends are useful for lightweight capture,
but they do not take that lock; use the capture server or the same lock
if a producer must be serialized with the TUI drain.

---

[Documentation index](README.md) · [Project overview](../README.md)
