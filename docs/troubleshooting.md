# Troubleshooting

## The shell cannot find tuxedo

For a Homebrew install, confirm that Homebrew's `bin` directory is on PATH (see
Homebrew's `brew shellenv` setup instructions), then reopen your terminal.
`tuxedotask` is an alternative name for the same app.

For Windows Setup, close and reopen your terminal after installation. The
optional PATH checkbox affects your user PATH, not terminals already running.
The **Tuxedo Task** Start menu shortcut works even when PATH integration is
unchecked. Re-run Setup to change that option; no Rust/Cargo setup is needed.

For optional Cargo installs, executables normally go into `~/.cargo/bin` on
macOS/Linux or `%USERPROFILE%\.cargo\bin` on Windows, unless `CARGO_HOME` or an
install root was customized. Add the relevant `bin` directory to PATH. For
manual downloads, keep the extracted executable and alias together on PATH.

## The wrong application opens

The fork and upstream use the same executable name. Check what the shell runs:

```sh
command -v tuxedo
tuxedo --version
```

On Windows, use `where.exe tuxedo` instead of `command -v`. The version output for
this fork begins with **Tuxedo Task**. A Homebrew copy may take precedence over a
Cargo copy depending on PATH. Our Homebrew formula explicitly conflicts with
upstream's formula because both expose `tuxedo`; it does not remove your Cargo
copy or migrate data. See [installation](installation.md#installation) before replacing an
existing executable.

## Subtasks look flat in another client

The hierarchy is a fork-specific extension encoded as two spaces per level.
Other clients may treat each line as an independent task or strip indentation.
The fork's formatted CLI listing and JSON also do not currently expose hierarchy.
Use the TUI or inspect the task file when you need to see parent/child structure.

## A task seems to be missing

Check the file opened by the app and the `TODO_FILE` / `TODO_DIR` environment
variables. Clear active project, context, Persona, and search filters; remember that
project/context/Persona filters persist across restarts. Check whether done or future
tasks are hidden, and press `a` to inspect the archive. Threshold dates can also
hide tasks until their visibility date.

## Mouse capture interferes with terminal text selection

Set `mouse = false` in the application's `config.toml` for keyboard-only use and
terminal-native selection. With mouse support enabled, dragging task text copies
through OSC 52; terminal clipboard support and permissions still apply.

## A configuration edit does not apply

The app keeps its last valid config and reports a reload failure when strict
parsing fails. Check the status message and restore the last valid syntax. The
config uses the app's line-based format: copy the unquoted values shown in this
guide rather than assuming every TOML construct is supported. Do not share a
config containing your `share_token`.

## Phone capture is unreachable

Use the address from the app's `s` QR overlay, keep the app running, and put the
phone on the same trusted network. Check local firewall restrictions. The
capture endpoint is plain HTTP and should not be exposed to the public internet.
See [phone capture](phone-capture.md#phone-capture) for token rotation and security details.

---

[Documentation index](README.md) · [Project overview](../README.md)
