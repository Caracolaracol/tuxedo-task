## Getting started

Once [installation](#installation) is complete, open Tuxedo Task and create your
first task. `tuxedo` and `tuxedotask` open the same app when installed through
Homebrew or a precompiled download.

### Your first task

> **You do not need to create `todo.txt` yourself.** When you open a path that
> does not exist, Tuxedo Task creates the file **and any missing parent folders**
> automatically. If the file already exists, it opens without clearing its
> contents. You need write permission to the chosen location.
>
> Installing the program does not choose or create your personal task file;
> opening it as shown below does.

1. Open the app with the path where you want to keep your tasks. On macOS/Linux:

```sh
tuxedo "$HOME/Documents/tasks/todo.txt"
```

On Windows, in PowerShell:

```powershell
tuxedo "$HOME\Documents\tasks\todo.txt"
```

These examples open or create `Documents/tasks/todo.txt` in your home folder.
There is no need to run `touch` or create an empty file beforehand. Back up
existing task files and configuration before switching from another client.

2. Press `n`, type a task, and press `Enter` to save. If natural-language parsing
   rewrites your draft, review it and press `Enter` again to confirm.
3. Move with `j` / `k`, press `x` to complete a task, or `u` to undo a change.
4. Press `?` for help inside the app and `q` to quit. Changes are saved as you
   make them; there is no separate Save button.

### What if I just run tuxedo?

If a task file is already configured or `todo.txt` exists in your terminal's
current folder, the app opens that file. Otherwise, a welcome screen lets you:

- Press `c` to create `todo.txt` in the **current terminal folder** and start
  with an empty list.
- Press `s` to try a sample in the system's temporary folder instead.

See [which file opens](#which-file-tuxedo-opens) for the full lookup order.

### Where to go next

- [Tasks and subtasks](#tasks-and-subtasks): understand the file format and hierarchy.
- [Keybindings](#keybindings): navigation, family editing, filtering, and shortcuts.
- [Configuration](#configuration): layout, filters, notes, and preferences.
- [Fork additions](#fork-additions): distinguish our additions from inherited features.

> **Your files stay local.** No account or database is required. Natural-language
> parsing is offline. Optional phone capture uses your trusted LAN; the optional
> update check contacts GitHub and can be disabled with `TUXEDO_NO_UPDATE_CHECK=1`.

For optional shell commands and automation, see the
[command-line reference near the end](#command-line-interface).
