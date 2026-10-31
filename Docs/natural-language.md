# Natural-language add

Press `n` to open the add prompt. Type the task in plain English. When the
buffer contains recognized phrases (dates, weekdays, recurrence, project /
context names, priority), pressing Enter rewrites the draft into canonical
todo.txt — review or tweak it, then Enter again to save.

| What you type | What lands in the draft |
| --- | --- |
| `Pay rent monthly on the first of the month, show the todo 3 days before the due date. It's part of project home and context bank` | `Pay rent +home @bank due:2026-06-01 rec:+1m t:-3d` |
| `Buy milk tomorrow` | `Buy milk due:2026-05-12` |
| `Call mom every week starting Friday for project family` | `Call mom +family due:2026-05-15 rec:+1w` |
| `Submit timesheet every other friday show 1 day before` | `Submit timesheet due:2026-05-15 rec:+2w t:-1d` |
| `Daily standup high priority` | `(A) standup rec:+1d` |
| `Annual review April 15 +work @office` | `Annual review +work @office due:2027-04-15` |

Recognized vocabulary:

- **Dates** — `today`, `tonight`, `tomorrow`, `yesterday`, weekdays (`monday` / `mon` …), months (`april 15`, `15th of april`), `in 3 days`, `the first of the month`, ISO `2026-05-15`.
- **Recurrence** — `daily`, `weekly`, `biweekly`, `monthly`, `yearly`, `annually`, `every monday`, `every 2 weeks`, `every other friday`, `every business day`.
- **Threshold** — `show 3 days before due`, `2 weeks before due`.
- **Projects / contexts** — prose form `project home` and `context bank`, or the standard `+home` / `@bank` sigils.
- **Priority** — `high priority` → A, `medium priority` → B, `low priority` → C, or `priority A`.

Parsing is rule-based and runs locally — no network calls, no API key. If
the buffer already contains a `due:`, `rec:`, or `t:` token, tuxedo assumes
you've typed canonical form and saves it directly on the first Enter.

---

[Documentation index](README.md) · [Project overview](../README.md)
