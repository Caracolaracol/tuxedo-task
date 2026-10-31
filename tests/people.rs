//! Persona assignments, using isolated task/config directories only.
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use ratatui::{Terminal, backend::TestBackend};
use tuxedo::action::Action;
use tuxedo::app::{App, Filter, FilterTarget, Mode, Sort, TokenKind, active_token};
use tuxedo::config::Config;
use tuxedo::core::{Store, TagOutcome, UndoOutcome, filter};
use tuxedo::{nl, todo, ui};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "tuxedo-people-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn seed(&self, raw: &str) -> PathBuf {
        let path = self.0.join("todo.txt");
        std::fs::write(&path, raw).unwrap();
        path
    }
    fn app(&self, raw: &str, cfg: Config) -> App {
        let mut app = App::new(self.seed(raw), raw.into(), "2026-05-06".into(), cfg);
        app.config_path = Some(self.0.join("config.toml"));
        app
    }
    fn store(&self, raw: &str) -> Store {
        Store::open_sync(self.seed(raw), raw.into(), "2026-05-06".into())
    }
    fn cli(&self, args: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_tuxedo"))
            .args(args)
            .current_dir(&self.0)
            .env("TODO_FILE", self.0.join("todo.txt"))
            .env("TODO_DIR", &self.0)
            .env("DONE_FILE", self.0.join("done.txt"))
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("XDG_CACHE_HOME", self.0.join("cache"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn people_parse_multiple_unicode_aliases_without_changing_raw_or_other_metadata() {
    let raw = "  (A) 2026-05-01 Ship +release @office ::ana ::José ::山田 ::ana due:2026-05-08 rec:+1w t:-2d";
    let task = todo::parse_line(raw).unwrap();
    assert_eq!(task.people, ["ana", "José", "山田"]);
    assert_eq!(task.projects, ["release"]);
    assert_eq!(task.contexts, ["office"]);
    assert_eq!(task.due.as_deref(), Some("2026-05-08"));
    assert_eq!(task.rec.as_deref(), Some("+1w"));
    assert_eq!(task.threshold.as_deref(), Some("-2d"));
    assert_eq!(task.indent_level, 1);
    assert_eq!(todo::serialize(&[task]), format!("{raw}\n"));
}

#[test]
fn people_do_not_capture_urls_namespaces_ipv6_or_empty_tokens() {
    for literal in [
        "::",
        ":ana",
        ":::ana",
        "due:2026-05-08",
        "https://host/::ana",
        "http://[::1]/",
        "std::path",
        "::std::path",
        "::1",
        "::abcd",
        "::ffff:192.0.2.1",
        "::ana@home",
        "::ana+work",
        "::ana/path",
    ] {
        let raw = format!("Inspect {literal}");
        let task = todo::parse_line(&raw).unwrap();
        assert!(task.people.is_empty(), "{literal} is not an assignment");
        assert_eq!(task.raw, raw);
    }
    assert_eq!(
        todo::body_only("Inspect ::1 ::abcd ::ana +work @home"),
        "Inspect ::1 ::abcd"
    );
}

#[test]
fn people_ignore_quoted_text_and_notes() {
    let task = todo::parse_line("Call note:\"ask ::ana to call ::José tomorrow\" ::maria").unwrap();
    assert_eq!(task.people, ["maria"]);
    assert!(
        todo::parse_line("Say \"hello ::ana there\"")
            .unwrap()
            .people
            .is_empty()
    );
}

#[test]
fn people_mutations_preserve_indent_other_assignees_and_quoted_notes() {
    let mut task = todo::parse_line(
        "    Call ::ana ::ana ::maria note:\"ask ::ana about the release\" +work @phone",
    )
    .unwrap();
    assert!(!task.add_person("ana").unwrap());
    assert!(task.remove_person("ana").unwrap());
    assert_eq!(task.people, ["maria"]);
    assert!(task.raw.contains("note:\"ask ::ana about the release\""));
    assert!(!task.remove_person("ana").unwrap());
    assert!(task.add_person("José").unwrap());
    assert_eq!(task.indent_level, 2);
    assert_eq!(task.projects, ["work"]);
    assert_eq!(task.contexts, ["phone"]);
    assert_eq!(todo::body_only(&task.raw), "Call");
    task.mark_done("2026-05-06").unwrap();
    task.unmark_done().unwrap();
    task.cycle_priority().unwrap();
    assert_eq!(task.people, ["maria", "José"]);
    assert_eq!(task.indent_level, 2);
}

#[test]
fn people_reject_invalid_or_ipv6_ambiguous_aliases_without_mutation() {
    for name in [
        "",
        "two words",
        "a:b",
        "@ana",
        "+ana",
        "ana/path",
        "ana\\path",
        "ana\"",
        "123",
        "ed",
        "abcd",
    ] {
        let mut task = todo::parse_line("Call ::ana").unwrap();
        assert!(task.add_person(name).is_err(), "{name}");
        assert!(task.remove_person(name).is_err(), "{name}");
        assert_eq!(task.raw, "Call ::ana");
    }
    for name in [
        "ana",
        "edgar",
        "abe-team",
        "ana-maria",
        "ana_maria",
        "José",
        "山田",
        "_ana",
        "anna2",
    ] {
        assert!(todo::is_valid_person_name(name), "{name}");
    }
}

#[test]
fn people_store_assignment_is_persisted_undoable_and_reconciles_shared_edits() {
    let fixture = Fixture::new();
    let mut store = fixture.store("Parent ::maria\n  Child +work @office\n");
    assert!(matches!(
        store.toggle_person(1, "ana"),
        TagOutcome::Added { .. }
    ));
    let disk = std::fs::read_to_string(fixture.0.join("todo.txt")).unwrap();
    assert!(disk.contains("  Child +work @office ::ana"));
    assert_eq!(store.tasks()[0].people, ["maria"]);
    assert!(matches!(
        store.toggle_person(1, "ana"),
        TagOutcome::Removed { .. }
    ));
    assert!(store.tasks()[1].people.is_empty());
    assert!(matches!(store.undo(), UndoOutcome::Undone));
    assert_eq!(store.tasks()[1].people, ["ana"]);
    assert_eq!(store.tasks()[1].indent_level, 1);
    std::fs::write(fixture.0.join("todo.txt"), "External change ::maria\n").unwrap();
    assert!(matches!(
        store.toggle_person(1, "ana"),
        TagOutcome::Aborted(_)
    ));
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("todo.txt")).unwrap(),
        "External change ::maria\n"
    );
    assert!(matches!(
        store.toggle_person(99, "ana"),
        TagOutcome::OutOfRange
    ));
    assert!(matches!(
        store.toggle_person(0, "two words"),
        TagOutcome::InvalidName
    ));
}

#[test]
fn people_filter_matches_exact_case_sensitive_aliases_and_combines_components() {
    let tasks = todo::parse_file(
        "Ship +work @office ::ana\nShip +work @office ::anna\nShip +work @office ::Ana\nShip +home @office ::ana\n",
    );
    let mut f = Filter {
        project: Some("work".into()),
        context: Some("office".into()),
        person: Some("ana".into()),
        search: "ship".into(),
    };
    let matched: Vec<_> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| filter::passes_user_filter(t, &f, Some("ship")))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(matched, [0]);
    assert!(f.has_any());
    f.clear();
    assert!(!f.has_any());
    assert!(f.person.is_none());
    f.person = Some("ana".into());
    assert!(f.has_any());
}

#[test]
fn people_family_filter_includes_assigned_child_and_keeps_entire_family() {
    let fixture = Fixture::new();
    let mut app = fixture.app(
        "Launch +work @office\n  Copy ::ana\n  Design ::maria\nOther ::ana +home\n",
        Config::default(),
    );
    app.prefs.sort = Sort::File;
    app.set_project_filter(Some("work".into()));
    app.set_context_filter(Some("office".into()));
    app.set_person_filter(Some("ana".into()));
    assert_eq!(app.visible_indices(), [0, 1, 2]);
    app.set_person_filter(Some("maria".into()));
    assert_eq!(app.visible_indices(), [0, 1, 2]);
    app.set_person_filter(Some("nobody".into()));
    assert!(app.visible_indices().is_empty());
}

#[test]
fn people_family_filter_does_not_activate_from_hidden_done_or_future_children() {
    let tasks = todo::parse_file(
        "Root\n  x 2026-05-01 Child ::ana\n  Later ::maria t:2026-06-01\nNext ::jose\n",
    );
    for (person, show_done, show_future, expected) in [
        ("ana", false, false, false),
        ("ana", true, false, true),
        ("maria", false, false, false),
        ("maria", false, true, true),
        ("jose", true, true, false),
    ] {
        let f = Filter {
            person: Some(person.into()),
            ..Filter::default()
        };
        assert_eq!(
            filter::list_predicate_family(
                &tasks,
                0,
                show_done,
                show_future,
                "2026-05-06",
                &f,
                None
            ),
            expected,
            "{person}"
        );
    }
}

#[test]
fn people_counts_unique_assignments_per_open_task() {
    let tasks = todo::parse_file("One ::ana ::ana ::maria\nTwo ::ana\nx 2026-05-01 Done ::maria\n");
    assert_eq!(
        filter::ordered_unique(&tasks, |t| &t.people),
        [("ana".into(), 2), ("maria".into(), 1)]
    );
}

#[test]
fn people_filter_persists_reloads_and_clears_without_losing_other_config() {
    let fixture = Fixture::new();
    let cfg = Config {
        share_port: Some(18080),
        notes_dir: Some("~/notes".into()),
        filters: vec![("calls".into(), "phone".into())],
        ..Config::default()
    };
    cfg.save_to(&fixture.0.join("config.toml")).unwrap();
    let raw = "Call +work @phone ::José\n";
    let mut app = fixture.app(raw, cfg);
    app.set_project_filter(Some("work".into()));
    app.set_context_filter(Some("phone".into()));
    app.set_person_filter(Some("José".into()));
    let saved = Config::load_from(&fixture.0.join("config.toml"));
    assert_eq!(saved.active_person.as_deref(), Some("José"));
    assert_eq!(saved.share_port, Some(18080));
    assert_eq!(saved.filters, [("calls".into(), "phone".into())]);
    assert_eq!(saved.notes_dir.as_deref(), Some("~/notes"));
    let mut reopened = fixture.app(raw, saved);
    assert_eq!(reopened.filter().person.as_deref(), Some("José"));
    assert_eq!(reopened.visible_indices(), [0]);
    reopened.clear_filter();
    assert!(!reopened.filter().has_any());
    assert_eq!(
        Config::load_from(&fixture.0.join("config.toml")).active_person,
        None
    );
}

#[test]
fn people_picker_cycles_accepts_and_cancels_independently() {
    let fixture = Fixture::new();
    let mut app = fixture.app(
        "One +work @office ::ana\nTwo +work @office ::maria\n",
        Config::default(),
    );
    app.set_project_filter(Some("work".into()));
    app.set_context_filter(Some("office".into()));
    app.enter_pick_person();
    assert_eq!(app.mode, Mode::PickPerson);
    assert_eq!(app.filter().person.as_deref(), Some("ana"));
    app.pick_step(true);
    assert_eq!(app.filter().person.as_deref(), Some("maria"));
    app.pick_step(true);
    assert_eq!(app.filter().person.as_deref(), Some("ana"));
    app.pick_step(false);
    app.pick_accept();
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(
        Config::load_from(&fixture.0.join("config.toml"))
            .active_person
            .as_deref(),
        Some("maria")
    );
    app.enter_pick_person();
    app.pick_cancel();
    assert_eq!(app.filter().person, None);
    assert_eq!(app.filter().project.as_deref(), Some("work"));
    assert_eq!(app.filter().context.as_deref(), Some("office"));
}

#[test]
fn persona_name_finds_both_assignment_actions_in_command_palette() {
    let fixture = Fixture::new();
    let mut app = fixture.app("Call ::ana\n", Config::default());
    app.command_palette.open(Mode::Normal);
    app.command_palette.refresh("persona");
    assert_eq!(app.command_palette.hits().len(), 2);
    let first = app.command_palette.current_action();
    app.command_palette.step(1);
    let second = app.command_palette.current_action();
    let actions = [first, second];
    assert!(actions.contains(&Some(Action::BeginPromptPerson)));
    assert!(actions.contains(&Some(Action::PickPerson)));
}

#[test]
fn people_empty_picker_stays_normal() {
    let fixture = Fixture::new();
    let mut app = fixture.app("No assignment\n", Config::default());
    app.enter_pick_person();
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.flash_active().unwrap().contains("ペルソナ"));
}

#[test]
fn people_autocomplete_recognizes_two_character_sigil_and_unicode() {
    let t = active_token("Call ::José", "Call ::José".len()).unwrap();
    assert_eq!(t.kind, TokenKind::Person);
    assert_eq!(t.prefix, "José");
    assert_eq!(t.start, 5);
    assert_eq!(active_token("Call ::", 7).unwrap().prefix, "");
    assert!(active_token("Call :", 6).is_none());
    assert!(active_token("std::ana", 8).is_none());
    assert!(active_token("::std::path", 10).is_none());
    let quoted = "Task note:\"call ::ana";
    assert!(active_token(quoted, quoted.len()).is_none());
    assert!(active_token("https://x/::ana", 14).is_none());
}

#[test]
fn people_autocomplete_replaces_whole_token_from_middle_without_eating_sigils() {
    let fixture = Fixture::new();
    let mut app = fixture.app("One +work @office ::José\nTwo ::ana\n", Config::default());
    app.draft_set("Call ::Jox later".into());
    while app.draft.cursor() > "Call ::Jo".len() {
        app.draft_left();
    }
    app.autocomplete_accept();
    assert_eq!(app.draft.text(), "Call ::José later");
    assert_eq!(app.draft.cursor(), "Call ::José".len());
    app.mode = Mode::PromptPerson;
    app.draft_set("jox".into());
    app.draft_left();
    assert_eq!(app.autocomplete_matches(), ["José"]);
    app.autocomplete_accept();
    assert_eq!(app.draft.text(), "José");
    assert_eq!(
        Action::from_keybind_name("prompt_person"),
        Some(Action::BeginPromptPerson)
    );
    assert_eq!(
        Action::from_keybind_name("pick-person"),
        Some(Action::PickPerson)
    );
}

#[test]
fn people_natural_language_preserves_assignees_separately_from_the_body() {
    let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
    let parsed = nl::try_parse("Ship tomorrow +work @office ::José ::ana", today).unwrap();
    assert_eq!(parsed.people, ["José", "ana"]);
    assert_eq!(parsed.body, "Ship");
    let formatted = nl::format_as_todo_txt(&parsed);
    assert_eq!(formatted, "Ship +work @office ::José ::ana due:2026-05-07");
    assert_eq!(
        todo::parse_line(&formatted).unwrap().people,
        ["José", "ana"]
    );
    let quoted = nl::try_parse("Ship tomorrow note:\"ask ::ana soon\"", today).unwrap();
    assert!(quoted.people.is_empty());
    assert!(nl::format_as_todo_txt(&quoted).contains("::ana"));
}

#[test]
fn people_cli_filters_exactly_and_outputs_additive_json_field() {
    let fixture = Fixture::new();
    fixture.seed(
        "One +work @office ::ana ::José\nTwo +home @office ::ana\nThree +work @office ::anna\n",
    );
    let output = fixture.cli(&["list", "+work", "@office", "::ana", "--json"]);
    assert!(output.contains("\"people\":[\"ana\",\"José\"]"), "{output}");
    assert!(!output.contains("Two"));
    assert!(!output.contains("Three"));
    let added = fixture.cli(&["add", "New task ::maria", "--json"]);
    assert!(added.contains("\"people\":[\"maria\"]"), "{added}");
    assert!(fixture.cli(&["ls", "::maria"]).contains("New task"));
}

#[test]
fn people_cli_lists_unique_names_including_completed_tasks() {
    let fixture = Fixture::new();
    fixture.seed("One ::ana ::ana\nTwo ::José\nx 2026-05-01 Done ::maria\n");
    let output = fixture.cli(&["listpersona"]);
    assert_eq!(output, "::José\n::ana\n::maria\n");
    assert_eq!(fixture.cli(&["listpeople"]), output);
    let json = fixture.cli(&["lsm", "--json"]);
    assert!(json.contains("\"action\":\"listpersona\""));
    assert!(json.contains("\"tags\":[\"José\",\"ana\",\"maria\"]"));
}

#[test]
fn people_recurrence_preserves_assignments_and_undo_restores_original() {
    let fixture = Fixture::new();
    let raw = "Water plants ::ana ::maria due:2026-05-06 rec:+1w\n";
    let mut store = fixture.store(raw);
    store.toggle_complete(0);
    assert_eq!(store.tasks().len(), 2);
    assert!(
        store
            .tasks()
            .iter()
            .all(|task| task.people == ["ana", "maria"])
    );
    assert!(
        store
            .tasks()
            .iter()
            .any(|task| !task.done && task.due.as_deref() == Some("2026-05-13"))
    );
    assert!(matches!(store.undo(), UndoOutcome::Undone));
    assert_eq!(todo::serialize(store.tasks()), raw);
}

#[test]
fn people_screen_snapshots_cover_assignment_prompt_filter_and_short_terminal() {
    let fixture = Fixture::new();
    let raw = "(A) 2026-05-01 Launch +work @office ::maria\n  2026-05-01 Write copy ::ana\n  2026-05-01 Review ::José\n2026-05-01 Call +home @phone ::ana\n";
    let mut app = fixture.app(raw, Config::default());
    app.prefs.density = tuxedo::app::Density::Compact;
    for (name, rows) in [
        ("people_assignments", 30),
        ("people_short_terminal", 18),
        ("people_filter", 30),
        ("people_prompt", 30),
    ] {
        if name == "people_filter" {
            app.set_person_filter(Some("ana".into()));
        }
        if name == "people_prompt" {
            app.mode = Mode::PromptPerson;
            app.draft_set("an".into());
        }
        // Stable display path, while Store and config writes stay isolated.
        app.file_path = PathBuf::from("target/tuxedo-people-snapshot.txt");
        let mut terminal = Terminal::new(TestBackend::new(130, rows)).unwrap();
        terminal.draw(|f| ui::draw(f, &app)).unwrap();
        let buf = terminal.backend().buffer();
        let mut text = String::new();
        for y in 0..rows {
            for x in 0..130 {
                text.push_str(buf[(x, y)].symbol());
            }
            text.push('\n');
        }
        insta::assert_snapshot!(name, text);
    }
}

#[test]
fn people_sidebar_remains_visible_with_many_tags_and_has_correct_mouse_targets() {
    let fixture = Fixture::new();
    let raw: String = (0..40)
        .map(|i| format!("Task +project{i:02} @context{i:02} ::person{i:02}\n"))
        .collect();
    let mut app = fixture.app(&raw, Config::default());
    app.set_person_filter(Some("person39".into()));
    let mut terminal = Terminal::new(TestBackend::new(130, 24)).unwrap();
    terminal.draw(|f| ui::draw(f, &app)).unwrap();
    let buf = terminal.backend().buffer();
    let text: String = buf.content.iter().map(|cell| cell.symbol()).collect();
    // Wide Katakana use continuation cells; compare headings without spaces.
    assert!(text.replace(' ', "").contains("ペルソナ"));
    let hit = app.mouse_hit().borrow();
    let (_, _, _, height) = hit.left_rect.unwrap();
    let visible = &hit.left_rows[..height as usize];
    assert!(visible.contains(&Some(FilterTarget::Person("person39".into()))));
    assert!(
        visible
            .iter()
            .any(|row| matches!(row, Some(FilterTarget::Project(_))))
    );
    assert!(
        visible
            .iter()
            .any(|row| matches!(row, Some(FilterTarget::Context(_))))
    );
}
