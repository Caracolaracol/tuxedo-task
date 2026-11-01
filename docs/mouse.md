# Mouse interaction

Mouse support is enabled by default in the list and archive views:

- Click a task row to move the cursor to it.
- Click a project, context, or person in the filter pane to toggle that filter.
- Long sections share the pane's available height. Use `fp`, `fc`, or `fm` to
  browse all entries; the active entry stays visible.
- Click a named saved search in the sidebar to apply it.
- Use the scroll wheel to move the cursor up or down.
- Drag across task text and release to copy the selection through OSC 52.

Modal dialogs remain keyboard-driven. Copying depends on the terminal's OSC 52
support. Set `mouse = false` in `config.toml` to use the keyboard only and let
your terminal handle text selection normally.

---

[Documentation index](README.md) · [Project overview](../README.md)
