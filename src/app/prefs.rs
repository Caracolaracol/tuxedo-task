use std::io;

use super::types::{Density, Sort};
use crate::app::WeekStart;
use crate::config::Config;
use crate::theme::{self, Theme};

#[derive(Debug, Clone)]
pub struct Layout {
    pub left: bool,
    pub right: bool,
    pub line_num: bool,
    pub status_bar: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            left: true,
            right: true,
            line_num: true,
            status_bar: true,
        }
    }
}

/// User-tunable preferences persisted to `Config`. Cycle/toggle methods return
/// the flash message for the caller to display, sidestepping any `&mut prefs`
/// + `&mut flash_state` borrow tangle on `App`.
#[derive(Debug, Clone)]
pub struct Prefs {
    theme_idx: usize,
    pub density: Density,
    pub sort: Sort,
    pub layout: Layout,
    pub show_done: bool,
    pub show_future: bool,
    /// Mouse support enabled. See `Config::mouse`.
    pub mouse: bool,
    /// Inset of the whole UI from the terminal edges. See `Config::margin`.
    pub margin: u16,
    /// Draw borders around each pane. See `Config::borders`.
    pub borders: bool,
    /// Metadata keys whose `key:value` tokens are hidden from task rows.
    /// Config-only (no in-app toggle); see `Config::hidden_keys`.
    pub hidden_keys: Vec<String>,
    pub week_start: WeekStart,
}

impl Prefs {
    pub fn from_config(cfg: Config) -> Self {
        let theme_idx = cfg
            .theme
            .as_deref()
            .and_then(|name| theme::all().iter().position(|t| t.name == name))
            .unwrap_or(0);
        Self {
            theme_idx,
            density: cfg.density.unwrap_or(Density::Comfortable),
            sort: cfg.sort.unwrap_or(Sort::Priority),
            layout: Layout {
                left: cfg.show_left.unwrap_or(true),
                right: cfg.show_right.unwrap_or(true),
                line_num: cfg.show_line_num.unwrap_or(true),
                status_bar: cfg.show_status_bar.unwrap_or(true),
            },
            show_done: cfg.show_done.unwrap_or(false),
            show_future: cfg.show_future.unwrap_or(false),
            mouse: cfg.mouse.unwrap_or(true),
            margin: cfg.margin.unwrap_or(0),
            borders: cfg.borders.unwrap_or(true),
            hidden_keys: cfg.hidden_keys,
            week_start: cfg.week_start.unwrap_or(WeekStart::Sunday),
        }
    }

    pub fn theme(&self) -> &'static Theme {
        let all = theme::all();
        all[self.theme_idx % all.len()]
    }

    pub fn theme_idx(&self) -> usize {
        self.theme_idx
    }

    /// Jump directly to a specific theme by index. Used by the screenshot
    /// example to render every theme; production code should call
    /// `cycle_theme` instead so the change persists with a flash message.
    pub fn set_theme_idx(&mut self, idx: usize) {
        self.theme_idx = idx % theme::all().len();
    }

    pub fn sort_label(&self) -> &'static str {
        self.sort.as_str()
    }

    pub fn cycle_theme(&mut self) -> String {
        self.theme_idx = (self.theme_idx + 1) % theme::all().len();
        format!("theme: {}", self.theme().name)
    }

    pub fn cycle_density(&mut self) -> String {
        self.density = match self.density {
            Density::Compact => Density::Comfortable,
            Density::Comfortable => Density::Cozy,
            Density::Cozy => Density::Compact,
        };
        format!("density: {}", self.density)
    }

    pub fn cycle_sort(&mut self) -> String {
        self.sort = match self.sort {
            Sort::Priority => Sort::Project,
            Sort::Project => Sort::Due,
            Sort::Due => Sort::File,
            Sort::File => Sort::Priority,
        };
        format!("sort: {}", self.sort)
    }

    pub fn toggle_left(&mut self) {
        self.layout.left = !self.layout.left;
    }

    pub fn toggle_right(&mut self) {
        self.layout.right = !self.layout.right;
    }

    pub fn toggle_line_num(&mut self) {
        self.layout.line_num = !self.layout.line_num;
    }

    pub fn toggle_show_done(&mut self) {
        self.show_done = !self.show_done;
    }

    pub fn toggle_show_future(&mut self) {
        self.show_future = !self.show_future;
    }

    pub fn cycle_week_start(&mut self) -> String {
        self.week_start = match self.week_start {
            WeekStart::Sunday => WeekStart::Monday,
            WeekStart::Monday => WeekStart::Sunday,
        };
        format!("week_start: {}", self.week_start)
    }

    /// Persist to the XDG config path. Returns the IO error so the caller
    /// can flash it (writing to stderr from inside the alt-screen would
    /// corrupt the TUI). Saving is best-effort — callers that don't care
    /// about reporting can `let _ = prefs.save();`.
    ///
    /// Loads the on-disk config first so non-pref fields (like
    /// `share_token` / `share_port`, owned by the capture server) are
    /// preserved across pref toggles.
    pub fn save(&self) -> io::Result<()> {
        let Some(path) = Config::path() else {
            return Ok(());
        };
        self.save_to(&path)
    }

    /// Persist to an explicit config path. `App::save_prefs` routes through
    /// this so tests writing prefs land in their fixture file instead of the
    /// user's real config — `Config::path()` alone would clobber it.
    pub fn save_to(&self, path: &std::path::Path) -> io::Result<()> {
        let mut cfg = Config::load_from(path);
        cfg.theme = Some(self.theme().name.to_string());
        cfg.density = Some(self.density);
        cfg.sort = Some(self.sort);
        cfg.show_left = Some(self.layout.left);
        cfg.show_right = Some(self.layout.right);
        cfg.show_line_num = Some(self.layout.line_num);
        cfg.show_status_bar = Some(self.layout.status_bar);
        cfg.show_done = Some(self.show_done);
        cfg.show_future = Some(self.show_future);
        cfg.mouse = Some(self.mouse);
        cfg.margin = Some(self.margin);
        cfg.borders = Some(self.borders);
        cfg.hidden_keys = self.hidden_keys.clone();
        cfg.save_to(path)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn save_to_writes_only_target_path_and_preserves_other_fields() {
        // Regression: `Prefs::save` used to write through `Config::load` /
        // `Config::save`, which resolve the *real* XDG path — so any test that
        // called `cycle_sort` (which triggers a pref save) clobbered the
        // developer's actual ~/.config/tuxedo/config.toml. `save_to` must load
        // and persist only at the given path, keeping unrelated fields intact.
        let dir = std::env::temp_dir().join(format!(
            "tuxedo-prefs-save-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        // "Nord" is a built-in theme, so it always resolves in test contexts
        // where `theme::init` may not have been called with user themes. The
        // token must be 64 hex chars (the parser rejects shorter ones).
        std::fs::write(
            &path,
            "theme = Nord\nshare_token = aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
        )
        .unwrap();

        let prefs = Prefs::from_config(Config::load_from(&path));
        prefs.save_to(&path).unwrap();

        let saved = std::fs::read_to_string(&path).unwrap();
        // Theme survives the round-trip (theme_idx resolved Nord).
        assert!(saved.contains("theme = Nord"), "theme lost: {saved}");
        // Unrelated field not owned by Prefs is preserved.
        assert!(saved.contains("share_token"), "share_token lost: {saved}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
