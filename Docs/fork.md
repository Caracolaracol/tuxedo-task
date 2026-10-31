# Fork additions

Compared with the upstream revision this fork started from:

| Addition | What it does |
| --- | --- |
| **Nested subtasks** | Two spaces per level; `>` / `<` move a whole subtree. Completion, deletion, undo, and priority/due sorting keep families together. |
| **Family editing** | Add a parent with child tasks in one dialog, or edit an existing parent's subtree together. |
| **Project grouping** | `S` cycles to project sort; grouping preserves families and supports multiple project tags. |
| **Mouse interaction** | Click task rows or sidebar filters, navigate with the wheel, and drag task text to copy via OSC 52. Disable with `mouse = false`. |
| **Persona assignments** | `::person` records assignees; **ペルソナ** is the third filter section beside projects and contexts. |
| **Remembered filters** | Active project, context, and person filters survive restarts, separately from named saved searches. |
| **Layout refinements** | Optional pane borders, configurable outer margin, revised task rows and headers, and Japanese labels in selected UI sections. |
| **Config reload fix** | Keeps the filesystem watcher alive and avoids repeated reload notices after its channel disconnects. |
| **Own identity** | An original mask mark, Tuxedo Task branding, and update instructions that point to this repository. |

These additions have unit and rendered-snapshot coverage. They do not replace the
features inherited from Tuxedo: its TUI/CLI workflow, natural-language dates,
recurrence, phone capture, custom themes, saved searches, notes, and atomic writes.

---

[Documentation index](README.md) · [Project overview](../README.md)
