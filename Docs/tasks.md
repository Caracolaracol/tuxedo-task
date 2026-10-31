# Tasks and subtasks

Standard [todo.txt](https://github.com/todotxt/todo.txt) lines:

```
(A) 2026-04-28 Call dentist @phone +health due:2026-05-08
```

- `(A)` — priority, A through Z (omit for none)
- `2026-04-28` — creation date in ISO 8601
- `+project` — project tag
- `@context` — context tag
- `::person` — Tuxedo Task's Persona assignment extension (see below)
- `key:value` — extension; `due:YYYY-MM-DD` is recognized for sort and
  due-bucket grouping in the list view. `note:<path>` is recognized by the
  note actions (`o` / `O`): relative paths resolve under `notes_dir`, then
  `$NOTES_DIR`, then `~/notes`. Keys you'd rather not see can be hidden from
  the rows via [`hide_keys`](configuration.md#hiding-keyvalue-tags)
- `rec:[+]N{d,b,w,m,y}` — recurrence; on completion (`x`), tuxedo inserts
  a fresh copy of the task with `due:` advanced by `N` days, business
  days (Mon–Fri), weeks, months, or years. The `+` prefix means
  *strict* recurrence anchored to the previous due date (e.g.
  `rec:+1m` for monthly rent on the 15th); without it, the new due is
  computed from the completion date (e.g. `rec:1w` for "water plants
  one week after I last did").

Completed tasks are prefixed with `x ` and a completion date:

```
x 2026-05-05 2026-05-01 Submit expense report +work
```

## Persona assignments

Use **Persona**, displayed as **ペルソナ** in the app, alongside projects and
contexts to record who is responsible for a task:

```text
Prepare the launch +website @office ::ana
  Write the announcement ::José
  Review the design ::ana ::maria
```

- A task may have zero, one, or several assignees. Each `::alias` is explicit;
  assigning a parent does **not** rewrite its children or replace other people.
- Press `m`, type an alias **without** `::`, and press `Enter` to assign it.
  Repeat the same alias to remove it. `u` undoes the change.
- While adding or editing, type `::` and use `Tab` to complete an existing
  alias. The `m` prompt also autocompletes aliases.
- Press `fm` to filter by person, or click their sidebar row. Project, context,
  person, and search filters combine. `Enter` keeps the filter across restarts;
  `Esc` in the picker clears only the person filter.
- In the TUI, a matching **visible** parent or subtask keeps the whole family
  together. Other family members provide context, not additional assignments.
  Completed/future tasks still follow `H` / `F`. Project, context, and search
  predicates continue to use the family root. CLI listings match each task's
  own explicit assignment instead.
- Counts reflect open tasks explicitly assigned to each alias, counting
  duplicates on the same task only once.

Aliases are case-sensitive single words beginning with a letter or `_`.
Unicode works (`::José`, `::山田`); use `ana-maria` or `ana_maria` instead of
spaces. Aliases cannot contain whitespace, `+`, `@`, `:`, quotes, `/`, or `\`.
`due:`, `rec:`, times, URLs, and embedded namespaces such as `std::path` are
not Persona tokens. Assignments inside quoted text or quoted notes are ignored.
A literal token beginning with `::` and a valid alias is reserved for Persona.

**IPv6 stays literal:** `::1`, `::abcd`, and other valid compressed IPv6
addresses are never treated as assignments. This also makes short hexadecimal
aliases such as `ed` ambiguous; use `_ed` or `edgar` instead.

Persona is plain-text metadata, not an account system: shared lists identify
responsibility without sending invitations, notifications, or enforcing
permissions. Share/sync the file separately. Other todo.txt clients can retain
these tokens as ordinary text; no file migration is required.

## Subtasks (nested tasks)

Subtasks are expressed with two leading spaces per nesting level:

```
Plan the launch
  Write the announcement
    Draft copy
    Get review
  Prepare the demo
```

Indent the task under the cursor with `>` and outdent it with `<` (the whole
subtree moves together). Subtasks render indented in the list, and they stay
attached to their parent across priority/due sorting. Completing or deleting
a parent (`x` / `dd`) completes or deletes its whole subtree; `u` undoes it
all at once.

**Compatibility:** indentation is a fork-specific extension, not part of the
standard todo.txt hierarchy model. Other clients may display these lines as
independent tasks or strip their indentation. The CLI's formatted listing/JSON
does not currently expose the hierarchy; use the TUI or the file itself when you
need to see parent/child structure.

Recurring example:

```
2026-05-09 Pay rent due:2026-05-15 rec:+1m
```

Pressing `x` on the line above marks the original complete *and* inserts
`2026-05-09 Pay rent due:2026-06-15 rec:+1m`. `u` undoes both at once.

---

[Documentation index](README.md) · [Project overview](../README.md)
