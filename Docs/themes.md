# Themes

Press `T` to choose between Muted Slate, Dawn, Nord, Matrix, and Terminal. Terminal respects your terminal palette. Custom themes can be added too.

## Custom themes

Beyond the built-ins, tuxedo loads any `*.toml` file you drop in
`${XDG_CONFIG_HOME:-$HOME/.config}/tuxedo/themes/` (native Windows 2026.10.2+:
`%APPDATA%\tuxedo\themes`, respecting XDG/legacy settings as described under
[Configuration](configuration.md#configuration)). Each one joins the `T`
picker in sorted filename order. Ready-made themes live in
[`Docs/themes/`](themes) — copy one into your platform's theme directory
and press `T`. macOS/Linux example:

```sh
mkdir -p ~/.config/tuxedo/themes
curl -o ~/.config/tuxedo/themes/gruvbox-dark-soft.toml \
  https://raw.githubusercontent.com/Caracolaracol/tuxedo-task/main/Docs/themes/gruvbox-dark-soft.toml
```

<details>
<summary>Theme file format and field reference</summary>

A theme file is one `key = value` per line. `name` is the label shown in the
picker; every other field is a color value. All fields are required: a file
missing one, carrying an unparseable color, or whose `name` collides with
another theme is skipped with a warning at startup.

**Color values** accept two forms:

- `#rrggbb` — a solid hex color (case-insensitive).
- `reset` or `transparent` — inherits the terminal emulator's own background
  color. Useful for `bg`, `panel`, and `statusbar` when you want your
  terminal's opacity, blur, or wallpaper to show through while keeping a
  custom text palette. Both keywords are case-insensitive and behave
  identically (same effect as the built-in **Terminal** theme).

| Field | Colors |
| --- | --- |
| `name` | label shown in the `T` picker (the only non-color field) |
| `bg` | window background |
| `panel` | filter and detail panel background |
| `border` | panel and modal borders |
| `fg` | primary text |
| `dim` | secondary / muted text |
| `accent` | logo, headings, hints, `::person` assignments, and selection markers |
| `cursor` | current row, and the highlighted row in the `T` picker |
| `selection` | set to the same value as `selected` |
| `statusbar` | status bar background |
| `status_fg` | status bar text |
| `mode_fg` / `mode_bg` | mode chip text / background |
| `pri_a` `pri_b` `pri_c` `pri_d` | priorities A through D |
| `pri_other` | priorities E through Z |
| `project` | `+project` tags |
| `context` | `@context` tags |
| `due` | `due:` date |
| `overdue` | past-due date |
| `today` | date due today |
| `done` | completed tasks |
| `selected` | selected-row background (visual mode) and the active filter |
| `matched` | search-match highlight |

</details>

---

[Documentation index](README.md) · [Project overview](../README.md)
