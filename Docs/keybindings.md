# Keybindings

Custom normal-mode keybindings can be added in
`${XDG_CONFIG_HOME:-$HOME/.config}/tuxedo/keybinds.toml`. On native Windows
2026.10.2+, the default is `%APPDATA%\tuxedo\keybinds.toml`; explicit XDG and
existing legacy settings are respected (see [Configuration](configuration.md#configuration)).

The block below lists every rebindable action with the key it ships with —
copy it, then change the keys you care about and delete the rest (anything you
leave out keeps its default). A value is a single key or an array of
alternatives, e.g. `begin_add = ["N", "Ctrl-n"]`.

```toml
[normal]

# Navigation
cursor_down    = ["j", "Down"]
cursor_up      = ["k", "Up"]
cursor_top     = "gg"
cursor_bottom  = "G"
half_page_down = "Ctrl-d"
half_page_up   = "Ctrl-u"

# Editing
begin_add            = "n"
begin_edit           = "e"
begin_edit_insert    = "i"
toggle_complete      = "x"
delete               = "dd"
reschedule           = "r"
cycle_priority       = "p"
begin_prompt_context = "c"
begin_prompt_person  = "m"
copy_line            = "yy"
copy_body            = "yb"
undo                 = "u"
# begin_prompt_project defaults to "+", which can't be written here (the
# parser reads "+" as a modifier separator). Pick another key to move it, e.g.
# begin_prompt_project = "P"

# Filtering, sort, view
begin_search        = "/"
arm_f               = "f"        # leader for the fp / fc / fm / ff / fs chords
pick_project        = "fp"
pick_context        = "fc"
pick_person         = "fm"
pick_saved_filter   = "ff"
save_current_filter = "fs"
cycle_sort          = "S"
toggle_visual       = "v"
toggle_selected     = "space"
go_list             = "l"
toggle_archive_view = "a"
archive_completed   = "A"
toggle_show_done    = "H"
toggle_show_future  = "F"

# Layout & theme
toggle_left_pane  = "["
toggle_right_pane = "]"
open_theme_picker = "T"
cycle_density     = "D"
toggle_line_num   = "L"
# cycle_theme has no default — bind a key to cycle themes without the picker:
# cycle_theme = "Ctrl-t"

# System
open_command_palette = [":", "Ctrl-P"]
open_share           = "s"
open_help            = "?"
open_settings        = ","
escape_stack         = "Esc"
quit                 = "q"
```

Custom bindings are checked before the defaults. The default bindings remain
available unless the same key or two-key chord is bound to another action in
the file. Action names are snake_case, matching the names in the command
palette where possible: `toggle_complete`, `pick_project`,
`open_theme_picker`, and so on. Key names can be single characters, two-key
chords like `ZZ`, modifier forms like `Ctrl-n` / `Alt-x`, named keys like
`Esc`, `Enter`, `Tab`, arrows, `Page-Up`, `Page-Down`, or `F1` through `F24`.

## Navigation

| Key | Action |
| --- | --- |
| `j` / `↓` | next task |
| `k` / `↑` | previous task |
| `gg` | first task |
| `G` | last task |
| `Ctrl-d` / `Ctrl-u` | half-page down / up |

## Editing

| Key | Action |
| --- | --- |
| `n` | add task |
| `e` | edit current task in Normal mode (see [Edit dialog](#edit-dialog)) |
| `i` | edit current task in Insert mode (see [Edit dialog](#edit-dialog)) |
| `x` | toggle complete |
| `dd` | delete task |
| `p` | cycle priority A → B → C → · |
| `c` | add or remove a context |
| `m` | assign or remove a person (**ペルソナ**); type the alias without `::` |
| `+` | add a project |
| `yy` | copy current line to clipboard |
| `yb` | copy current body only (no priority, dates, projects, contexts, people, `key:value`) |
| `>` | indent task (and its subtree) one level — nests it as a subtask |
| `<` | outdent task (and its subtree) one level |
| `u` | undo (50 levels) |

## Edit dialog

**Fork family editing:** `e` / `i` on a parent opens its existing subtree. In the
add/edit dialog, press `Esc` to enter Normal mode, then `o` to insert a child
line. Use `j` / `k` to switch draft lines, `>` / `<` to adjust their indentation,
and `dd` to remove a draft line. `Enter` saves the whole family as one undoable
change. In Normal mode, another `Esc` cancels the dialog.

The edit dialog uses vim-style modal editing. Press `i` to edit the current task
starting in **Insert mode** — start typing immediately. Press `e` to start in
**Normal mode** so you can navigate before changing anything. The add prompt
(`n`) also opens directly in Insert mode.

The modal keys below apply in Normal mode:

| Key | Action |
| --- | --- |
| `h` / `←` | move cursor left |
| `l` / `→` | move cursor right |
| `w` | jump to start of next word |
| `b` | jump to start of previous word |
| `e` | jump to end of current word |
| `x` | delete character under cursor |
| `dw` | delete to start of next word |
| `cw` | delete to start of next word and enter Insert mode |
| `i` | enter Insert mode before cursor |
| `a` | enter Insert mode after cursor |
| `A` | enter Insert mode at end of line |
| `Esc` (in Insert) | return to Normal mode |
| `Esc` (in Normal) | cancel and close |
| `Enter` (in Insert and Normal) | save |

## Filtering, sort, view

| Key | Action |
| --- | --- |
| `/` | search |
| `fp` | filter by project (`j` / `k` cycles, `Esc` clears) |
| `fc` | filter by context (`j` / `k` cycles, `Esc` clears) |
| `fm` | filter by person (**ペルソナ**; `j` / `k` cycles, `Enter` keeps, `Esc` clears only this filter) |
| `ff` | pick a saved search (`j` / `k` cycles, `Enter` keeps, `Esc` reverts) |
| `fs` | save the active `/`-search as a named filter |
| `S` | cycle sort: priority → due → file order → project |
| `v` | enter visual / multi-select; `space` toggles a row |
| `x` / `dd` (in visual) | bulk-complete / bulk-delete the selection |
| `l` | list (default) view |
| `a` | toggle archive view |
| `A` | archive completed tasks → `done.txt` |
| `H` | toggle showing done tasks in the main list |
| `o` | open the current task's existing `note:<path>` in `$VISUAL` / `$EDITOR` |
| `O` | create the current task's note if needed, then open it |

## Project grouping

Press `S` until the header shows `sort:project`. Tasks appear under `+project`
group headings, with untagged families in a final **NO PROJECT** group. A task
with multiple project tags can appear in more than one group; this does not
create duplicate lines in the task file. Parent/subtask families stay together.
Use `fp` to filter by a project instead of grouping the whole list.

## Layout & theme

| Key | Action |
| --- | --- |
| `[` | toggle filter sidebar |
| `]` | toggle detail sidebar |
| `T` | open theme picker |
| `D` | cycle density: compact → comfortable → cozy |
| `L` | toggle line numbers |

## System

| Key | Action |
| --- | --- |
| `:` / `Ctrl-P` | command palette |
| `s` | share capture QR (phone PWA) |
| `?` | help overlay |
| `,` | settings overlay |
| `q` | quit |

Two-key chord prompts (`gg`, `dd`, `yy`, `yb`, `fp`, `fc`, `fm`, `ff`, `fs`) show
a `g…` / `d…` / `y…` / `f…` indicator in the status-bar mode chip while the
leader is armed; the window is 600 ms.

Copy uses the OSC 52 terminal escape, so it works locally and over SSH on
any terminal that supports it (kitty, alacritty, wezterm, iTerm2, foot,
modern xterm; tmux when `set -g set-clipboard on`). Older terminals will
silently ignore the keystroke.

---

[Documentation index](README.md) · [Project overview](../README.md)
