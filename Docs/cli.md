# Command-line interface

**Optional reference for shell commands and automation.** You do not need these
commands to use the interactive app. With no subcommand, `tuxedo` opens the TUI.

## Help, version and update instructions

These commands inspect the installed application or show instructions; they
**do not install it**:

```sh
tuxedo --version
tuxedo --help
tuxedo update
```

`--version` identifies the fork as **Tuxedo Task**. `--help` lists available
options and commands. `tuxedo update` (or `tuxedotask update`) only prints upgrade
instructions for your install method; it never installs anything automatically.

## Verify a Windows installer

This PowerShell command only checks a download; it does not install anything:

```powershell
Get-FileHash .\tuxedo-task-windows-x86_64-setup.exe -Algorithm SHA256
```

Compare the hash with the adjacent `.sha256` file from the same official release;
uppercase/lowercase hexadecimal letters mean the same hash.
Signing/SmartScreen reputation and a future WinGet submission are separate from
these checks; no WinGet package is currently advertised as published.

## Task commands and automation

The CLI uses `$TODO_FILE`, `$TODO_DIR/todo.txt`, or an existing `./todo.txt` in
that order. With none available, it uses a temporary sample instead of asking.
Set `TODO_FILE` explicitly when scripting your own files; use a copy while
trying commands that modify tasks.

When the first argument is a recognized subcommand, tuxedo runs a one-shot
command instead of launching the TUI. The surface mirrors
[todo.txt-cli](https://github.com/todotxt/todo.txt-cli/wiki/Usage) — same
commands, aliases, task numbering, and output — so it's a drop-in for scripts
and aliases.

Examples for macOS/Linux shells:

```sh
export TODO_FILE="$HOME/Documents/tasks/todo.txt"
tuxedo add "Pay rent +home @bank due:2026-07-01"   # or: tuxedo a "..."
tuxedo ls @bank                                     # filter by context
tuxedo ls +website ::ana                             # combine project and person
tuxedo listpersona                                   # list known assignees
tuxedo do 3                                          # mark task 3 complete
tuxedo pri 3 A                                        # set priority
tuxedo archive                                        # move done tasks to done.txt
tuxedo ls --json | jq .                              # machine-readable output
```

| Command | Aliases | Arguments | Description |
| --- | --- | --- | --- |
| `add` | `a` | `TEXT...` | Add a task (natural-language dates supported, same as the `n` prompt). |
| `append` | `app` | `N TEXT...` | Append text to task `N`. |
| `prepend` | `prep` | `N TEXT...` | Prepend text to task `N`. |
| `replace` | | `N TEXT...` | Replace task `N` entirely. |
| `pri` | `p` | `N PRIORITY` | Set priority `A`–`Z` on task `N`. |
| `depri` | `dp` | `N...` | Remove priority from the given tasks. |
| `do` | `done`, `complete` | `N...` | Mark tasks complete (recurring tasks spawn their next instance). |
| `del` | `rm` | `N [TERM]` | Delete task `N`, or remove just `TERM` from it. Prompts unless `-f`. |
| `archive` | | | Move completed tasks to the done file. |
| `list` | `ls` | `[TERM...]` | List tasks. `TERM` is `+project`, `@context`, `::person`, or free text. |
| `listall` | `lsa` | `[TERM...]` | List the todo file and the done file. |
| `listpri` | `lsp` | `[PRIORITY]` | List prioritized tasks (optionally a single priority). |
| `listproj` | `lsprj` | | List all `+projects`. |
| `listcon` | `lsc` | | List all `@contexts`. |
| `listpersona` | `lsm`, `listpeople` | | List unique `::person` assignments, including completed tasks in the todo file. |

**Task numbers** are 1-based line numbers in the file, exactly as printed by
`list` — stable regardless of how the list is filtered or sorted. `list`
sorts by the full line (case-insensitive) and prints a `TODO: X of Y tasks
shown` footer, matching todo.txt-cli.

**Options:**

- `-f`, `--force` — skip confirmation prompts (e.g. for `del`).
- `--json` — emit machine-readable JSON instead of text. `list`-style commands
  print an array of task objects; mutating commands print a result object.
  Task objects include a `people` string array alongside `projects` and
  `contexts`; this field name is independent of the Persona display label.
  `listpersona --json` returns bare aliases in its `tags` array.
  No prompts or footers are written in this mode.

Global flags may appear before the subcommand (`tuxedo -f del 3`).

**Differences from todo.txt-cli:** `do` marks a task complete but does **not**
auto-archive it — completed tasks stay in the file until you run `archive` (or
press `A` in the TUI), matching tuxedo's interactive model. There is no `-d`
config-file flag; configure paths with the environment variables above.

## Capture from scripts

The sibling `inbox.txt` described under [Phone capture](phone-capture.md#phone-capture) also
accepts file-based producers. The example below assumes the running app opened
`$HOME/Documents/tasks/todo.txt`, so its inbox is in that same directory:

```sh
echo "Refill prescription tomorrow" >> "$HOME/Documents/tasks/inbox.txt"
echo "Call dentist due:2026-06-01" >> "$HOME/Documents/tasks/inbox.txt"
```

Shell aliases, Shortcuts writing to a synced folder, cron jobs and email-to-file
gateways can append lines to that inbox. The running app picks them up on its
next poll. Plain shell appends do not acquire the app's lock; use the capture
server or the same lock when producers must be serialized with the TUI drain.

---

[Documentation index](README.md) · [Project overview](../README.md)
