# Screens

| | |
| --- | --- |
| **Empty state** • original mask mark and quick-start when the file has no tasks | ![empty](screenshots/empty.svg) |
| **Nested subtasks** • parent/child families grouped by project | ![subtasks](screenshots/subtasks.svg) |
| **List** • list of todos, optionally grouped | ![empty](screenshots/list.svg) |
| **Archive** • completed tasks grouped by completion date | ![archive](screenshots/archive.svg) |
| **Filter sidebar active** • `fp` cycles projects with j/k, `fc` cycles contexts; saved searches list under a **SAVED** heading with live match counts | ![filter](screenshots/filter.svg) |
| **Persona filter** • assign tasks with `m`, then filter by person with `fm` | ![Persona filter](screenshots/persona.svg) |
| **Command palette** • `:` or `Ctrl-P` opens a fuzzy palette over every action | ![command palette](screenshots/command-palette.svg) |
| **Help** • `?` opens the full keybindings overlay | ![help](screenshots/help.svg) |

## Built-in themes

| Muted Slate (default) | Dawn |
| --- | --- |
| ![Muted Slate](screenshots/theme-muted-slate.svg) | ![Dawn](screenshots/theme-dawn.svg) |
| **Nord** | **Matrix** |
| ![Nord](screenshots/theme-nord.svg) | ![Matrix](screenshots/theme-matrix.svg) |
| **Terminal** | |
| ![Terminal](screenshots/theme-terminal.svg) | |

<details>
    <summary>How to generate the screenshots and demo</summary>
    <p>The screenshots in the table above are checked-in SVGs. Regenerate them with:</p>
    <pre>mise run screenshots</pre>
    <p>An optional demo recording can be generated with <a href="https://github.com/charmbracelet/vhs">vhs</a> from <code>docs/demo.tape</code> (requires VHS and its dependencies):</p>
    <pre>mise run demo</pre>
</details>

---

[Documentation index](README.md) · [Project overview](../README.md)
