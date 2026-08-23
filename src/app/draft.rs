use super::App;
use super::draft_overlay::DraftOverlay;

/// Sub-mode for the edit/add dialog, mirroring vim's modal editing model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogInputMode {
    #[default]
    Insert,
    Normal,
}

/// Byte offset within a draft `String`. Construction enforces UTF-8 char-boundary
/// landing, so cursor positions can't be left mid-codepoint by direct assignment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DraftCursor(usize);

impl DraftCursor {
    /// Clamp to the nearest char boundary at or before `byte` (or to `s.len()`).
    pub fn clamped(s: &str, byte: usize) -> Self {
        let mut b = byte.min(s.len());
        while b > 0 && !s.is_char_boundary(b) {
            b -= 1;
        }
        Self(b)
    }

    pub fn at_end(s: &str) -> Self {
        Self(s.len())
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn byte(self) -> usize {
        self.0
    }
}

/// One line of the add-task dialog's family buffer. The add modal can draft a
/// parent task plus any number of indented children; the first line is the
/// parent, subsequent lines are subtasks nested by `indent_level`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DraftLine {
    pub text: String,
    pub indent_level: u8,
}

#[derive(Debug, Clone)]
pub struct DraftState {
    /// Lines of the draft. Always at least one entry; the add dialog grows
    /// this with `add_child_line`, while edit/search/prompt modes keep a
    /// single line. The *active* line is the one being edited.
    lines: Vec<DraftLine>,
    /// Index of the line being edited.
    active: usize,
    /// Per-line cursor, kept in lockstep with `lines`.
    cursors: Vec<DraftCursor>,
    autocomplete_selected: usize,
    autocomplete_suppressed: bool,
    /// Open metadata picker (slash menu, calendar, recurrence builder,
    /// priority chooser). At most one at a time. `None` is the default — the
    /// user is just editing text.
    overlay: Option<DraftOverlay>,
    input_mode: DialogInputMode,
}

impl Default for DraftState {
    fn default() -> Self {
        Self {
            lines: vec![DraftLine::default()],
            active: 0,
            cursors: vec![DraftCursor::zero()],
            autocomplete_selected: 0,
            autocomplete_suppressed: false,
            overlay: None,
            input_mode: DialogInputMode::Insert,
        }
    }
}

impl DraftState {
    pub fn text(&self) -> &str {
        &self.lines[self.active].text
    }

    pub fn cursor(&self) -> usize {
        self.cursors[self.active].byte()
    }

    pub fn autocomplete_index(&self) -> usize {
        self.autocomplete_selected
    }

    pub fn autocomplete_suppressed(&self) -> bool {
        self.autocomplete_suppressed
    }

    pub fn overlay(&self) -> Option<&DraftOverlay> {
        self.overlay.as_ref()
    }

    pub fn overlay_mut(&mut self) -> Option<&mut DraftOverlay> {
        self.overlay.as_mut()
    }

    pub fn set_overlay(&mut self, overlay: Option<DraftOverlay>) {
        self.overlay = overlay;
    }

    pub fn input_mode(&self) -> DialogInputMode {
        self.input_mode
    }

    pub fn set_input_mode(&mut self, mode: DialogInputMode) {
        self.input_mode = mode;
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.lines.push(DraftLine::default());
        self.active = 0;
        self.cursors.clear();
        self.cursors.push(DraftCursor::zero());
        self.reset_autocomplete();
        self.overlay = None;
        self.input_mode = DialogInputMode::Insert;
    }

    /// Replace the text and park the cursor at the end. Used when entering
    /// edit mode or otherwise seeding the input from existing text. The input
    /// sub-mode is the caller's choice — `App::draft_set` (`e`) lands in
    /// Normal mode, `App::draft_set_insert` (`i`) lands in Insert mode.
    pub fn set(&mut self, s: String, mode: DialogInputMode) {
        self.lines.clear();
        self.lines.push(DraftLine {
            text: s,
            indent_level: 0,
        });
        self.active = 0;
        self.cursors.clear();
        self.cursors.push(DraftCursor::at_end(&self.lines[0].text));
        self.reset_autocomplete();
        self.overlay = None;
        self.input_mode = mode;
    }

    /// Seed the whole draft from a pre-built line list (parent plus its
    /// subtasks) and open the dialog. Used by the edit path so editing a task
    /// shows its entire subtree; the active line starts at the parent.
    pub fn set_family(&mut self, lines: Vec<DraftLine>, mode: DialogInputMode) {
        self.lines = if lines.is_empty() {
            vec![DraftLine::default()]
        } else {
            lines
        };
        self.active = 0;
        self.cursors = self
            .lines
            .iter()
            .map(|l| DraftCursor::at_end(&l.text))
            .collect();
        self.reset_autocomplete();
        self.overlay = None;
        self.input_mode = mode;
    }

    pub fn insert_char(&mut self, c: char) {
        let text = &mut self.lines[self.active].text;
        let pos = self.cursors[self.active].byte();
        text.insert(pos, c);
        self.cursors[self.active] = DraftCursor(pos + c.len_utf8());
        self.reset_autocomplete();
    }

    pub fn backspace(&mut self) {
        let pos = self.cursors[self.active].byte();
        if pos == 0 {
            return;
        }
        let text = &mut self.lines[self.active].text;
        let prev = prev_char_boundary(text, pos);
        text.drain(prev..pos);
        self.cursors[self.active] = DraftCursor(prev);
        self.reset_autocomplete();
    }

    pub fn delete_forward(&mut self) {
        let pos = self.cursors[self.active].byte();
        if pos >= self.lines[self.active].text.len() {
            return;
        }
        let next = next_char_boundary(&self.lines[self.active].text, pos);
        self.lines[self.active].text.drain(pos..next);
        self.reset_autocomplete();
    }

    pub fn move_left(&mut self) {
        let pos = self.cursors[self.active].byte();
        if pos == 0 {
            return;
        }
        self.cursors[self.active] =
            DraftCursor(prev_char_boundary(&self.lines[self.active].text, pos));
    }

    pub fn move_right(&mut self) {
        let pos = self.cursors[self.active].byte();
        if pos >= self.lines[self.active].text.len() {
            return;
        }
        self.cursors[self.active] =
            DraftCursor(next_char_boundary(&self.lines[self.active].text, pos));
    }

    pub fn move_home(&mut self) {
        self.cursors[self.active] = DraftCursor::zero();
    }

    pub fn move_end(&mut self) {
        self.cursors[self.active] = DraftCursor::at_end(&self.lines[self.active].text);
    }

    /// Move to the start of the next word (`w`).
    pub fn move_word_forward(&mut self) {
        let s = &self.lines[self.active].text;
        let mut pos = self.cursors[self.active].byte();
        while pos < s.len() && !s.as_bytes()[pos].is_ascii_whitespace() {
            pos = next_char_boundary(s, pos);
        }
        while pos < s.len() && s.as_bytes()[pos].is_ascii_whitespace() {
            pos = next_char_boundary(s, pos);
        }
        self.cursors[self.active] = DraftCursor(pos);
    }

    /// Move to the start of the current or previous word (`b`).
    pub fn move_word_backward(&mut self) {
        let s = &self.lines[self.active].text;
        let pos = self.cursors[self.active].byte();
        if pos == 0 {
            return;
        }
        let mut p = prev_char_boundary(s, pos);
        while p > 0 && s.as_bytes()[p].is_ascii_whitespace() {
            p = prev_char_boundary(s, p);
        }
        while p > 0 && !s.as_bytes()[prev_char_boundary(s, p)].is_ascii_whitespace() {
            p = prev_char_boundary(s, p);
        }
        self.cursors[self.active] = DraftCursor(p);
    }

    /// Delete from the cursor to the start of the next word (`dw`/`cw`).
    pub fn delete_word_forward(&mut self) {
        let start = self.cursors[self.active].byte();
        let s = &self.lines[self.active].text;
        let mut end = start;
        while end < s.len() && !s.as_bytes()[end].is_ascii_whitespace() {
            end = next_char_boundary(s, end);
        }
        while end < s.len() && s.as_bytes()[end].is_ascii_whitespace() {
            end = next_char_boundary(s, end);
        }
        if end > start {
            self.lines[self.active].text.drain(start..end);
            self.reset_autocomplete();
        }
    }

    /// Delete from the start of the previous word to the cursor (`Ctrl+W`).
    /// Whitespace-delimited, mirroring `move_word_backward` and matching
    /// readline's unix-word-rubout.
    pub fn delete_word_backward(&mut self) {
        let end = self.cursors[self.active].byte();
        if end == 0 {
            return;
        }
        let s = &self.lines[self.active].text;
        let mut start = prev_char_boundary(s, end);
        // Skip whitespace immediately before the cursor, then the word itself.
        while start > 0 && s.as_bytes()[start].is_ascii_whitespace() {
            start = prev_char_boundary(s, start);
        }
        while start > 0 && !s.as_bytes()[prev_char_boundary(s, start)].is_ascii_whitespace() {
            start = prev_char_boundary(s, start);
        }
        self.lines[self.active].text.drain(start..end);
        self.cursors[self.active] = DraftCursor(start);
        self.reset_autocomplete();
    }

    /// Delete from the start of the line to the cursor (`Ctrl+U`).
    pub fn kill_to_start(&mut self) {
        let pos = self.cursors[self.active].byte();
        if pos == 0 {
            return;
        }
        self.lines[self.active].text.drain(0..pos);
        self.cursors[self.active] = DraftCursor::zero();
        self.reset_autocomplete();
    }

    /// Delete from the cursor to the end of the line (`Ctrl+K`). The cursor
    /// stays put.
    pub fn kill_to_end(&mut self) {
        let pos = self.cursors[self.active].byte();
        if pos >= self.lines[self.active].text.len() {
            return;
        }
        self.lines[self.active].text.truncate(pos);
        self.reset_autocomplete();
    }

    /// Move to the end of the current or next word (`e`).
    pub fn move_word_end(&mut self) {
        let s = &self.lines[self.active].text;
        let mut pos = self.cursors[self.active].byte();
        // Step off the current position first
        if pos < s.len() {
            pos = next_char_boundary(s, pos);
        }
        while pos < s.len() && s.as_bytes()[pos].is_ascii_whitespace() {
            pos = next_char_boundary(s, pos);
        }
        while pos < s.len() && {
            let next = next_char_boundary(s, pos);
            next < s.len() && !s.as_bytes()[next].is_ascii_whitespace()
        } {
            pos = next_char_boundary(s, pos);
        }
        self.cursors[self.active] = DraftCursor(pos);
    }

    /// Cycle the selected autocomplete match. `n` is the current match-list length.
    /// No-op when `n == 0`.
    pub fn step_autocomplete(&mut self, n: usize, forward: bool) {
        if n == 0 {
            return;
        }
        let cur = self.autocomplete_selected.min(n - 1);
        self.autocomplete_selected = if forward {
            (cur + 1) % n
        } else {
            (cur + n - 1) % n
        };
    }

    /// Hide the popup until the next text mutation.
    pub fn suppress_autocomplete(&mut self) {
        self.autocomplete_suppressed = true;
    }

    /// Replace the byte range `[start, end)` of the *active* line with `with`,
    /// parking the cursor at `start + with.len()`. Used by `autocomplete_accept`
    /// to swap in a chosen suggestion. Caller guarantees `start` and `end` are
    /// char boundaries.
    pub fn replace_token(&mut self, start: usize, end: usize, with: &str) {
        self.lines[self.active].text.replace_range(start..end, with);
        self.cursors[self.active] = DraftCursor(start + with.len());
        self.autocomplete_selected = 0;
        self.autocomplete_suppressed = false;
    }

    // ---------------------------------------------------------------------
    // Family (multi-line) operations — used by the add-task dialog
    // ---------------------------------------------------------------------

    /// Number of lines in the draft.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Index of the currently-active line.
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// All lines, parent first.
    pub fn lines(&self) -> &[DraftLine] {
        &self.lines
    }

    /// Indent level of the active line.
    pub fn active_indent(&self) -> u8 {
        self.lines[self.active].indent_level
    }

    /// Insert a new, empty line below the active one, indented one level
    /// deeper, and make it the active line. Returns the index of the new line.
    /// Used by the dialog's `o` key to draft a subtask under the current line.
    pub fn add_child_line(&mut self) -> usize {
        let child = DraftLine {
            text: String::new(),
            indent_level: self.lines[self.active].indent_level.saturating_add(1),
        };
        let insert_at = self.active + 1;
        self.lines.insert(insert_at, child);
        self.cursors.insert(insert_at, DraftCursor::zero());
        self.active = insert_at;
        self.reset_autocomplete();
        self.overlay = None;
        self.active
    }

    /// Move the active line up/down (`j`/`k`). No-op at the edges.
    pub fn move_active(&mut self, delta: isize) {
        let len = self.lines.len() as isize;
        let next = self.active as isize + delta;
        if next < 0 || next >= len {
            return;
        }
        self.active = next as usize;
        self.reset_autocomplete();
        self.overlay = None;
    }

    /// Delete the active line. The active line moves up one slot (staying at
    /// index 0 when the top line is removed). Deleting the last remaining line
    /// leaves one empty line behind. Returns the new active index.
    pub fn delete_active(&mut self) -> usize {
        if self.lines.len() == 1 {
            self.lines[0].text.clear();
            self.cursors[0] = DraftCursor::zero();
            return 0;
        }
        self.lines.remove(self.active);
        self.cursors.remove(self.active);
        if self.active >= self.lines.len() {
            self.active = self.lines.len() - 1;
        }
        self.cursors[self.active] = DraftCursor::clamped(
            &self.lines[self.active].text,
            self.cursors[self.active].byte(),
        );
        self.reset_autocomplete();
        self.overlay = None;
        self.active
    }

    /// Nudge the active line's indent level by `delta` (`>` / `<`), clamped at
    /// zero. Does not reorder lines — the family stays in the user's order.
    pub fn indent_active(&mut self, delta: i8) {
        let cur = self.lines[self.active].indent_level;
        self.lines[self.active].indent_level = if delta < 0 {
            cur.saturating_sub(delta.unsigned_abs())
        } else {
            cur.saturating_add(delta as u8)
        };
    }

    fn reset_autocomplete(&mut self) {
        self.autocomplete_selected = 0;
        self.autocomplete_suppressed = false;
    }

    #[cfg(test)]
    pub(crate) fn force_cursor(&mut self, byte: usize) {
        let text = &self.lines[self.active].text;
        self.cursors[self.active] = DraftCursor::clamped(text, byte);
    }
}

/// App-level delegators. These keep the existing `app.draft_*()` call surface
/// intact for main.rs key handlers; the actual logic lives on `DraftState`.
impl App {
    pub fn draft_clear(&mut self) {
        self.draft.clear();
    }

    /// Seed the edit dialog and open it in Normal mode (`e`), so vim users can
    /// navigate before changing anything.
    pub fn draft_set(&mut self, s: String) {
        self.draft.set(s, DialogInputMode::Normal);
    }

    /// Seed the edit dialog and open it in Insert mode (`i`), for immediate
    /// typing without the vim modal step.
    pub fn draft_set_insert(&mut self, s: String) {
        self.draft.set(s, DialogInputMode::Insert);
    }

    /// Seed the edit dialog from a whole family of lines (parent + subtasks),
    /// opening in Normal mode. Used when editing a task that has subtasks, so
    /// the whole subtree is visible and editable at once.
    pub fn draft_set_family(&mut self, lines: Vec<DraftLine>) {
        self.draft.set_family(lines, DialogInputMode::Normal);
    }

    /// Like [`Self::draft_set_family`] but opening in Insert mode.
    pub fn draft_set_family_insert(&mut self, lines: Vec<DraftLine>) {
        self.draft.set_family(lines, DialogInputMode::Insert);
    }

    pub fn draft_insert_char(&mut self, c: char) {
        self.draft.insert_char(c);
    }

    pub fn draft_backspace(&mut self) {
        self.draft.backspace();
    }

    pub fn draft_delete_forward(&mut self) {
        self.draft.delete_forward();
    }

    pub fn draft_left(&mut self) {
        self.draft.move_left();
    }

    pub fn draft_right(&mut self) {
        self.draft.move_right();
    }

    pub fn draft_home(&mut self) {
        self.draft.move_home();
    }

    pub fn draft_end(&mut self) {
        self.draft.move_end();
    }

    pub fn draft_word_forward(&mut self) {
        self.draft.move_word_forward();
    }

    pub fn draft_word_backward(&mut self) {
        self.draft.move_word_backward();
    }

    pub fn draft_word_end(&mut self) {
        self.draft.move_word_end();
    }

    pub fn draft_delete_word_forward(&mut self) {
        self.draft.delete_word_forward();
    }

    pub fn draft_delete_word_backward(&mut self) {
        self.draft.delete_word_backward();
    }

    pub fn draft_kill_to_start(&mut self) {
        self.draft.kill_to_start();
    }

    pub fn draft_kill_to_end(&mut self) {
        self.draft.kill_to_end();
    }
}

pub(super) fn prev_char_boundary(s: &str, i: usize) -> usize {
    let mut j = i.saturating_sub(1);
    while j > 0 && !s.is_char_boundary(j) {
        j -= 1;
    }
    j
}

fn next_char_boundary(s: &str, i: usize) -> usize {
    let len = s.len();
    let mut j = (i + 1).min(len);
    while j < len && !s.is_char_boundary(j) {
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::DialogInputMode;
    use crate::app::test_support::build_app;

    #[test]
    fn draft_set_opens_in_normal_mode() {
        // `e` seeds the edit dialog in Normal mode for vim-style navigation.
        let mut app = build_app("");
        app.draft_set("hello".into());
        assert_eq!(app.draft.input_mode(), DialogInputMode::Normal);
    }

    #[test]
    fn draft_set_insert_opens_in_insert_mode() {
        // `i` seeds the edit dialog in Insert mode for immediate typing.
        let mut app = build_app("");
        app.draft_set_insert("hello".into());
        assert_eq!(app.draft.input_mode(), DialogInputMode::Insert);
    }

    #[test]
    fn draft_left_right_navigates_within_text() {
        let mut app = build_app("");
        app.draft_set("hello".into());
        assert_eq!(app.draft.cursor(), 5);
        app.draft_left();
        app.draft_left();
        assert_eq!(app.draft.cursor(), 3);
        app.draft_insert_char('X');
        assert_eq!(app.draft.text(), "helXlo");
        assert_eq!(app.draft.cursor(), 4);
        app.draft_right();
        app.draft_right();
        // Already at end; further right is a no-op.
        app.draft_right();
        assert_eq!(app.draft.cursor(), app.draft.text().len());
    }

    #[test]
    fn draft_backspace_deletes_before_cursor() {
        let mut app = build_app("");
        app.draft_set("abc".into());
        app.draft_left();
        // Cursor between 'b' and 'c'; backspace removes 'b'.
        app.draft_backspace();
        assert_eq!(app.draft.text(), "ac");
        assert_eq!(app.draft.cursor(), 1);
    }

    #[test]
    fn draft_delete_forward_removes_char_at_cursor() {
        let mut app = build_app("");
        app.draft_set("abc".into());
        app.draft_home();
        app.draft_delete_forward();
        assert_eq!(app.draft.text(), "bc");
        assert_eq!(app.draft.cursor(), 0);
    }

    #[test]
    fn draft_handles_multibyte_chars_on_char_boundaries() {
        // "café" — 'é' is two bytes (U+00E9 = 0xC3 0xA9).
        let mut app = build_app("");
        app.draft_set("café".into());
        assert_eq!(app.draft.cursor(), 5);
        app.draft_left();
        // Cursor must land on a char boundary (before 'é', at byte 3).
        assert_eq!(app.draft.cursor(), 3);
        app.draft_backspace();
        assert_eq!(app.draft.text(), "caé");
    }

    #[test]
    fn draft_delete_word_backward_removes_prior_word() {
        let mut app = build_app("");
        app.draft_set("hello world".into());
        // Cursor parks at end (11) after `set`.
        app.draft_delete_word_backward();
        assert_eq!(app.draft.text(), "hello ");
        assert_eq!(app.draft.cursor(), 6);
    }

    #[test]
    fn draft_delete_word_backward_eats_trailing_space_then_word() {
        let mut app = build_app("");
        app.draft_set("hello world ".into());
        app.draft_delete_word_backward();
        assert_eq!(app.draft.text(), "hello ");
        assert_eq!(app.draft.cursor(), 6);
    }

    #[test]
    fn draft_delete_word_backward_at_start_is_noop() {
        let mut app = build_app("");
        app.draft_set("hello".into());
        app.draft_home();
        app.draft_delete_word_backward();
        assert_eq!(app.draft.text(), "hello");
        assert_eq!(app.draft.cursor(), 0);
    }

    #[test]
    fn draft_delete_word_backward_respects_multibyte_boundary() {
        // "café com" — 'é' is two bytes; the deleted word must start on a
        // char boundary so the surviving "café " stays valid UTF-8.
        let mut app = build_app("");
        app.draft_set("café com".into());
        app.draft_delete_word_backward();
        assert_eq!(app.draft.text(), "café ");
    }

    #[test]
    fn draft_kill_to_start_removes_text_before_cursor() {
        let mut app = build_app("");
        app.draft_set("hello world".into());
        app.draft.force_cursor(6); // between "hello " and "world"
        app.draft_kill_to_start();
        assert_eq!(app.draft.text(), "world");
        assert_eq!(app.draft.cursor(), 0);
    }

    #[test]
    fn draft_kill_to_start_at_start_is_noop() {
        let mut app = build_app("");
        app.draft_set("hello".into());
        app.draft_home();
        app.draft_kill_to_start();
        assert_eq!(app.draft.text(), "hello");
        assert_eq!(app.draft.cursor(), 0);
    }

    #[test]
    fn draft_kill_to_end_removes_text_after_cursor() {
        let mut app = build_app("");
        app.draft_set("hello world".into());
        app.draft.force_cursor(5); // right after "hello"
        app.draft_kill_to_end();
        assert_eq!(app.draft.text(), "hello");
        assert_eq!(app.draft.cursor(), 5);
    }

    #[test]
    fn draft_kill_to_end_at_end_is_noop() {
        let mut app = build_app("");
        app.draft_set("hello".into());
        // Cursor already at end after `set`.
        app.draft_kill_to_end();
        assert_eq!(app.draft.text(), "hello");
        assert_eq!(app.draft.cursor(), 5);
    }

    // ---- family (multi-line) operations ---------------------------------

    #[test]
    fn add_child_line_inserts_below_active_with_plus_one_indent() {
        let mut app = build_app("");
        app.draft_set("parent".into());
        app.draft.add_child_line();
        assert_eq!(app.draft.line_count(), 2);
        assert_eq!(app.draft.active_index(), 1);
        assert_eq!(app.draft.active_indent(), 1);
        assert_eq!(app.draft.text(), "");
        // Type into the child, then move back up to the parent.
        for c in "child".chars() {
            app.draft.insert_char(c);
        }
        assert_eq!(app.draft.text(), "child");
        app.draft.move_active(-1);
        assert_eq!(app.draft.text(), "parent");
        assert_eq!(app.draft.active_indent(), 0);
    }

    #[test]
    fn add_child_line_nests_under_deeper_active() {
        let mut app = build_app("");
        app.draft_set("a".into());
        app.draft.add_child_line();
        app.draft.add_child_line();
        // Active is now grandchild at indent 2.
        assert_eq!(app.draft.active_index(), 2);
        assert_eq!(app.draft.active_indent(), 2);
    }

    #[test]
    fn move_active_clamps_at_edges() {
        let mut app = build_app("");
        app.draft_set("a".into());
        app.draft.add_child_line();
        app.draft.add_child_line();
        // At bottom (index 2); down is a no-op.
        app.draft.move_active(1);
        assert_eq!(app.draft.active_index(), 2);
        app.draft.move_active(-1);
        assert_eq!(app.draft.active_index(), 1);
        // To top, then up is a no-op.
        app.draft.move_active(-1);
        assert_eq!(app.draft.active_index(), 0);
        app.draft.move_active(-1);
        assert_eq!(app.draft.active_index(), 0);
    }

    #[test]
    fn delete_active_removes_line_and_keeps_others() {
        let mut app = build_app("");
        app.draft_set("parent".into());
        app.draft.add_child_line();
        for c in "child".chars() {
            app.draft.insert_char(c);
        }
        let _ = app.draft.delete_active();
        assert_eq!(app.draft.line_count(), 1);
        assert_eq!(app.draft.text(), "parent");
        assert_eq!(app.draft.active_index(), 0);
    }

    #[test]
    fn delete_last_remaining_line_clears_it() {
        let mut app = build_app("");
        app.draft_set("only".into());
        let idx = app.draft.delete_active();
        assert_eq!(idx, 0);
        assert_eq!(app.draft.line_count(), 1);
        assert_eq!(app.draft.text(), "");
    }

    #[test]
    fn indent_active_clamps_at_zero() {
        let mut app = build_app("");
        app.draft_set("a".into());
        app.draft.indent_active(-3);
        assert_eq!(app.draft.active_indent(), 0);
        app.draft.indent_active(2);
        assert_eq!(app.draft.active_indent(), 2);
        app.draft.indent_active(-1);
        assert_eq!(app.draft.active_indent(), 1);
    }

    #[test]
    fn clear_resets_to_single_line() {
        let mut app = build_app("");
        app.draft_set("a".into());
        app.draft.add_child_line();
        app.draft.add_child_line();
        app.draft_clear();
        assert_eq!(app.draft.line_count(), 1);
        assert_eq!(app.draft.text(), "");
        assert_eq!(app.draft.active_index(), 0);
    }

    #[test]
    fn lines_returns_parent_first() {
        let mut app = build_app("");
        app.draft_set("parent".into());
        app.draft.add_child_line();
        for c in "child".chars() {
            app.draft.insert_char(c);
        }
        let lines = app.draft.lines();
        assert_eq!(lines[0].text, "parent");
        assert_eq!(lines[0].indent_level, 0);
        assert_eq!(lines[1].text, "child");
        assert_eq!(lines[1].indent_level, 1);
    }
}
