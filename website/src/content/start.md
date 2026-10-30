## Getting started

Tuxedo Task is a terminal task manager and command-line tool for **todo.txt**.
It is an independent community fork of [Tuxedo by Webstone Technologies](https://github.com/webstonehq/tuxedo),
with nested subtasks, family editing, project grouping, mouse interaction, and
remembered filters added on top of the original application.

### Your first task

1. Follow the [installation instructions](#installation) below. The executable is
   still called `tuxedo`; `tuxedo --version` identifies the fork as **Tuxedo Task**.
   Homebrew and precompiled downloads also include `tuxedotask` as an alias. You
   do not need Rust or Cargo for those install methods.
2. Open a task file. On macOS or Linux, for example:

```sh
tuxedo "$HOME/Documents/tasks/todo.txt"
```

On Windows, use a Windows path, for example:

```powershell
tuxedo C:\tasks\todo.txt
```

The selected file and its parent directories are created if missing. Back up any
existing task files and config before switching from another client.

3. Press `n`, type a task, and press `Enter` to save. If natural-language parsing
   rewrites your draft, review it and press `Enter` again to confirm.
4. Move with `j` / `k`, press `x` to complete a task, or `u` to undo a change.
5. Press `?` for help and `q` to quit. Changes are written as you make them; there
   is no separate Save button.

### Where to go next

- [Tasks and subtasks](#tasks-and-subtasks): understand the file format and hierarchy.
- [Keybindings](#keybindings): navigation, family editing, filtering, and shortcuts.
- [Command-line interface](#command-line-interface): add, list, complete, and script tasks.
- [Configuration](#configuration): layout, filters, notes, and preferences.
- [Fork additions](#fork-additions): distinguish our additions from inherited features.

> **Your files stay local.** No account or database is required. Natural-language
> parsing is offline. Optional phone capture uses your trusted LAN; the optional
> update check contacts GitHub and can be disabled with `TUXEDO_NO_UPDATE_CHECK=1`.
