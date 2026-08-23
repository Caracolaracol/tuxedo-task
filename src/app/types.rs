use std::fmt;
use std::str::FromStr;
use std::time::Duration;

pub const LEADER_WINDOW: Duration = Duration::from_millis(600);
pub const FLASH_TTL: Duration = Duration::from_millis(1400);
pub const UNDO_LIMIT: usize = 50;
pub const AUTOCOMPLETE_CAP: usize = 8;

/// Outcome of `add_from_draft`. The Enter handler in `main.rs` uses this to
/// decide whether to exit Insert mode: `Parsed` means the NL pre-pass
/// rewrote the buffer but did not save, so the user should stay in Insert
/// to review/edit before pressing Enter a second time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddOutcome {
    Saved,
    Parsed,
    Empty,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Search,
    Visual,
    Help,
    Settings,
    PromptProject,    // text input → add project on current task
    PromptContext,    // text input → add/remove context on current task
    PickProject,      // j/k cycles through projects to filter by
    PickContext,      // j/k cycles through contexts to filter by
    PickSavedFilter,  // j/k cycles through saved searches to apply
    PromptSaveFilter, // text input → name the current search and save it
    CommandPalette,
    /// QR + URL overlay for the in-TUI capture server. Any key
    /// dismisses; press `s` again to re-open without rebinding (the
    /// server stays running once started).
    Share,
    /// Theme picker dialog — j/k to preview themes, Enter to accept,
    /// Esc to revert.
    PickTheme,
    /// First-run welcome prompt, shown when `tuxedo` is launched with no
    /// target and no `./todo.txt` exists. `c` creates `./todo.txt`, `s`
    /// opens the bundled sample, `q`/`Esc` quits without creating anything.
    Welcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Archive,
}

impl View {
    /// Stable slot index for keying per-view state arrays. Don't reorder the
    /// `View` variants without updating this together.
    pub fn idx(self) -> usize {
        match self {
            View::List => 0,
            View::Archive => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Priority,
    Due,
    File,
    /// Group tasks under their projects (`+name` headers). A task with several
    /// projects appears once under each; tasks with none land in a final
    /// NO PROJECT group.
    Project,
}

impl Sort {
    pub fn as_str(self) -> &'static str {
        match self {
            Sort::Priority => "priority",
            Sort::Due => "due",
            Sort::File => "file",
            Sort::Project => "project",
        }
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Sort {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "priority" => Ok(Sort::Priority),
            "due" => Ok(Sort::Due),
            "file" => Ok(Sort::File),
            "project" => Ok(Sort::Project),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Compact,
    Comfortable,
    Cozy,
}

impl Density {
    pub fn as_str(self) -> &'static str {
        match self {
            Density::Compact => "compact",
            Density::Comfortable => "comfortable",
            Density::Cozy => "cozy",
        }
    }
}

impl fmt::Display for Density {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Density {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compact" => Ok(Density::Compact),
            "comfortable" => Ok(Density::Comfortable),
            "cozy" => Ok(Density::Cozy),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub project: Option<String>,
    pub context: Option<String>,
    pub search: String,
}

impl Filter {
    /// True when at least one of project / context / search is non-empty.
    pub fn has_any(&self) -> bool {
        self.project.is_some() || self.context.is_some() || !self.search.is_empty()
    }

    /// Drop every filter component back to its empty state.
    pub fn clear(&mut self) {
        self.project = None;
        self.context = None;
        self.search.clear();
    }
}

/// A user-named saved search. `query` is a `/`-search needle (case-insensitive
/// subsequence match on the task body), recalled via the `ff` picker and
/// persisted as a `filter.<name> = <query>` line in the config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedFilter {
    pub name: String,
    pub query: String,
}

/// What clicking a row in the left filter pane should do. Carries the exact
/// name the filter compares against so the mouse handler can toggle it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterTarget {
    Project(String),
    Context(String),
    Saved(String),
}

/// Per-frame hit-test state for mouse support, rebuilt by the renderers each
/// time a frame is drawn. Kept in a `RefCell` because rendering only borrows
/// `&App`; the mouse handler reads it on the next input event. Rects are
/// `(x, y, width, height)` in screen coordinates.
#[derive(Debug, Clone, Default)]
pub struct MouseHit {
    /// Screen rect of the list/archive body area.
    pub body_rect: Option<(u16, u16, u16, u16)>,
    /// Parallel to the rendered body rows: `Some(visible index)` for a task
    /// row, `None` for group headers and density blank lines.
    pub body_rows: Vec<Option<usize>>,
    /// Parallel to `body_rows`: the exact visible text of each rendered row
    /// (indent, line number, glyph, priority box, bullet and body all
    /// included). `None` for headers/blanks. The mouse handler copies from
    /// this verbatim so the selection matches exactly what's on screen.
    pub body_texts: Vec<Option<String>>,
    /// Screen rect of the left filter pane.
    pub left_rect: Option<(u16, u16, u16, u16)>,
    /// Parallel to the rendered filter rows: the filter a click should apply,
    /// or `None` for headers/blanks.
    pub left_rows: Vec<Option<FilterTarget>>,
}

/// An in-progress (or just-finished) mouse text selection over the task list.
/// Coordinates are screen-space rows/columns inside the body area; the renderer
/// highlights rows between `anchor` and `cur`, and releasing the button copies
/// the selected body text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseSel {
    pub anchor_row: u16,
    pub anchor_col: u16,
    pub cur_row: u16,
    pub cur_col: u16,
}

impl MouseSel {
    pub fn new(row: u16, col: u16) -> Self {
        Self {
            anchor_row: row,
            anchor_col: col,
            cur_row: row,
            cur_col: col,
        }
    }

    /// Normalized row span `(min, max)`.
    pub fn row_span(&self) -> (u16, u16) {
        (
            self.anchor_row.min(self.cur_row),
            self.anchor_row.max(self.cur_row),
        )
    }
}
