# Usage

`tuxedo` opens the interactive terminal interface. **You do not need to create
`todo.txt` by hand.** Choose where to keep your tasks and open that path:

```sh
tuxedo "$HOME/Documents/tasks/todo.txt"
```

On Windows PowerShell:

```powershell
tuxedo "$HOME\Documents\tasks\todo.txt"
```

The app creates the file **and missing parent folders** when the path does not
exist. If it already exists, its contents are kept. You need write permission
to that location. Installing the program alone does not create a personal task
file; launching it with a path does.

To open an already configured/default file or see the first-run welcome screen:

```sh
tuxedo
```

On a true first run, press `c` to create `todo.txt` in your terminal's current
folder, or `s` to try a temporary sample. No manual `touch` or file setup is
needed. You can also open that sample directly with `tuxedo --sample`.

Inside the app, press `n` to add a task, `Enter` to confirm, `?` for help and
`q` to quit. Changes are saved automatically. The optional shell-command and
scripting reference is [near the end](cli.md#command-line-interface).

When a newer release is available, the status bar shows `↑ <version> (tuxedo
update)` next to the version. The check runs in the background, is cached at
`${XDG_CACHE_HOME:-$HOME/.cache}/tuxedo-task/latest_version.json` for 24 h, and
fails silently when offline. Set `TUXEDO_NO_UPDATE_CHECK=1` to disable.

## Which file tuxedo opens

The interactive app chooses a task file in this order:

1. An explicit `FILE` argument (TUI only).
2. `$TODO_FILE`, if set.
3. `$TODO_DIR/todo.txt`, if `$TODO_DIR` is set.
4. `./todo.txt` in the current directory, if it exists.
5. Otherwise the TUI shows a first-run prompt — press `c` to create
   `./todo.txt` here, or `s` to open a sample todo.txt in the system temp
   directory so you can try the app without choosing a permanent path.

The archive file is `$DONE_FILE` if set, otherwise a sibling `done.txt` next
to the todo file. The file (and any missing parent directories) is created on
first use. These variable names are compatible with todo.txt-cli. Export them
from your shell configuration as below; Tuxedo Task reads the environment,
**not the `todo.cfg` file itself**:

```sh
export TODO_DIR="$HOME/Documents/todo"
export TODO_FILE="$TODO_DIR/todo.txt"
export DONE_FILE="$TODO_DIR/done.txt"
```

Edits are persisted on every change via atomic write (write `.tmp`, rename).

If the file changes on disk (another editor, a sync client, a script),
tuxedo notices on the next keypress, or within ~250 ms while idle, and
reloads. The keystroke that triggered the reload is consumed — press it
again to act on the fresh state — and the status bar flashes a notice.

Pressing `A` appends every completed task to a sibling `done.txt` and
removes them from the working file (atomically: `done.txt` is written
before the originals are dropped). `a` toggles the archive view so you
can browse, un-archive, or permanently delete past tasks.

---

[Documentation index](README.md) · [Project overview](../README.md)
