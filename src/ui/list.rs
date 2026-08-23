use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, GroupKey, ListDueBucket, Mode, View};
use crate::theme::Theme;
use crate::ui::{header, keep_cursor_visible, task_row};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let bordered = app.prefs.borders;

    let filter_label = header::filter_label(&app.filter);
    let props = header::HeaderProps {
        title: Some(&display_path(&app.file_path)),
        count: app.visible_indices().len(),
        sort: app.sort_label(),
        filter: filter_label.as_deref(),
    };

    let body_area = if bordered {
        // Border around the whole pane, with the header row as the border
        // title. Content renders inside; the body area is measured post-border
        // so the scroll and mouse hit-map stay in sync with the screen.
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border).bg(theme.bg))
            .title(header::header_line(theme, props))
            .style(Style::default().bg(theme.bg));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [_spacer, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(inner);
        body
    } else {
        super::fill_bg(frame, area, Style::default().bg(theme.bg));
        let [header_area, _spacer, body] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .areas(area);
        header::render(frame, header_area, theme, props);
        body
    };

    if app.tasks().is_empty() {
        crate::ui::empty::render(frame, body_area, app);
        return;
    }

    let visible = app.visible_indices();
    let groups = app.visible_groups();
    let mut lines: Vec<Line> = Vec::new();
    let mut hit_rows: Vec<Option<usize>> = Vec::new();
    let mut hit_texts: Vec<Option<String>> = Vec::new();
    let mut cursor_line: Option<usize> = None;

    if visible.is_empty() {
        lines.push(Line::from(Span::styled(
            "   no tasks match".to_string(),
            Style::default().fg(theme.dim),
        )));
        hit_rows.push(None);
        hit_texts.push(None);
    } else {
        let blank = super::density_blank_lines(app.prefs.density);
        let counts = group_counts(groups);
        let last = visible.len().saturating_sub(1);
        let mut last_group: Option<&GroupKey> = None;

        for (i, (&abs, gk)) in visible.iter().zip(groups.iter()).enumerate() {
            // Emit a section header on group transitions. `GroupKey::None`
            // means the active sort is `Sort::File`; we never render a header
            // for it, so the layout is identical to the pre-grouping version.
            if !matches!(gk, GroupKey::None) && last_group != Some(gk) {
                if !lines.is_empty() {
                    push_blanks(&mut lines, blank);
                    hit_rows.extend(std::iter::repeat_n(None, blank));
                    hit_texts.extend(std::iter::repeat_n(None, blank));
                }
                lines.push(group_header(theme, gk, counts.lookup(gk)));
                hit_rows.push(None);
                hit_texts.push(None);
                last_group = Some(gk);
            }

            // If a mouse text selection is active, map this row's screen
            // position (using the same scroll the mouse handler reads) to the
            // selected column span so the renderer can highlight it. `lines.len()`
            // is the real line index (headers/blanks included), matching the
            // index `body_rows`/`body_texts` use in the handler. The column
            // span is made relative to the body rect so it aligns with the
            // per-row text offset `task_row` tracks.
            let sel_cols = app.mouse_sel().borrow().and_then(|sel| {
                let scroll = app.scroll_for_view(app.view());
                let screen_row = body_area.y as i64 + lines.len() as i64 - scroll as i64;
                let (r1, r2) = sel.row_span();
                (screen_row >= r1 as i64 && screen_row <= r2 as i64).then_some((
                    sel.anchor_col.min(sel.cur_col).saturating_sub(body_area.x),
                    sel.anchor_col.max(sel.cur_col).saturating_sub(body_area.x),
                ))
            });

            let task = &app.tasks()[abs];
            let opts = task_row::RowOpts {
                idx_label: i,
                cursor: i == app.cursor && app.mode != Mode::Help && app.mode != Mode::Settings,
                multi_mode: app.effective_mode() == Mode::Visual,
                multi_checked: app.selection.is_selected(abs),
                selected: app.selection.is_selected(abs),
                show_line_num: app.prefs.layout.line_num,
                match_term: if app.filter.search.is_empty() {
                    None
                } else {
                    Some(&app.filter.search)
                },
                today: app.today(),
                hidden_keys: &app.prefs.hidden_keys,
                sel_cols,
            };
            if i == app.cursor {
                cursor_line = Some(lines.len());
            }
            let rendered = task_row::build_line(task, opts, theme);
            // Keep the exact visible text (all spans concatenated) so mouse
            // selection copies verbatim what's on screen.
            let text: String = rendered.spans.iter().map(|s| s.content.as_ref()).collect();
            lines.push(rendered);
            hit_rows.push(Some(i));
            hit_texts.push(Some(text));
            if matches!(gk, GroupKey::None) && i != last {
                for _ in 0..blank {
                    lines.push(Line::raw(""));
                }
                hit_rows.extend(std::iter::repeat_n(None, blank));
                hit_texts.extend(std::iter::repeat_n(None, blank));
            }
        }
    }

    let scroll_cell = &app.view_scroll[View::List.idx()];
    let scroll = keep_cursor_visible(
        scroll_cell.get(),
        cursor_line,
        body_area.height,
        lines.len(),
    );
    scroll_cell.set(scroll);

    // Rebuild the mouse hit-map for this frame. The handler maps a click row
    // back to a line index using the same scroll the paragraph renders with.
    let mut mouse = app.mouse_hit.borrow_mut();
    mouse.body_rect = Some((body_area.x, body_area.y, body_area.width, body_area.height));
    mouse.body_rows = hit_rows;
    mouse.body_texts = hit_texts;

    let para = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .scroll((scroll, 0));
    frame.render_widget(para, body_area);
}

fn display_path(p: &std::path::Path) -> String {
    if let Some(home) = std::env::var_os("HOME")
        && let Ok(rel) = p.strip_prefix(&home)
    {
        return format!("~/{}", rel.display());
    }
    p.display().to_string()
}

/// Tally rows per `GroupKey` so each header can show its count without an
/// extra rescan during the render loop. Keyed by a stable string form so
/// `GroupKey::ListPriority(None)` and `ListPriority(Some('A'))` don't collide.
struct GroupCounts {
    inner: std::collections::HashMap<String, usize>,
}

impl GroupCounts {
    fn lookup(&self, gk: &GroupKey) -> usize {
        self.inner.get(&group_count_key(gk)).copied().unwrap_or(0)
    }
}

fn group_counts(groups: &[GroupKey]) -> GroupCounts {
    let mut inner: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for g in groups {
        if matches!(g, GroupKey::None) {
            continue;
        }
        *inner.entry(group_count_key(g)).or_insert(0) += 1;
    }
    GroupCounts { inner }
}

fn group_count_key(gk: &GroupKey) -> String {
    match gk {
        GroupKey::ListPriority(Some(c)) => format!("p:{c}"),
        GroupKey::ListPriority(None) => "p:_".to_string(),
        GroupKey::ListDue(b) => format!("d:{}", b.label()),
        GroupKey::ListProject(Some(p)) => format!("proj:{p}"),
        GroupKey::ListProject(None) => "proj:_".to_string(),
        // Not produced for List view; encode defensively.
        GroupKey::ArchiveDate(d) => format!("a:{d}"),
        GroupKey::None => String::new(),
    }
}

fn group_header<'a>(theme: &Theme, gk: &GroupKey, count: usize) -> Line<'a> {
    let (label, color) = match gk {
        GroupKey::ListPriority(Some(c)) => (format!("PRIORITY {c}"), theme.priority_color(*c)),
        GroupKey::ListPriority(None) => ("NO PRIORITY".to_string(), theme.dim),
        GroupKey::ListDue(b) => (b.label().to_string(), due_bucket_color(theme, *b)),
        GroupKey::ListProject(Some(p)) => (format!("+{p}"), theme.project),
        GroupKey::ListProject(None) => ("NO PROJECT".to_string(), theme.dim),
        // Defensive fallthrough — not produced under List view.
        GroupKey::ArchiveDate(d) => (d.clone(), theme.accent),
        GroupKey::None => (String::new(), theme.fg),
    };

    let char_count = count.to_string().len() + label.len();
    let divider_length = 80 - char_count;

    Line::from(vec![
        Span::raw(" "),
        Span::styled(format!("({})", count), Style::default().fg(theme.dim)),
        Span::raw("  "),
        Span::styled(
            label,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "─".repeat(divider_length),
            Style::default().fg(theme.border),
        ),
    ])
}

fn due_bucket_color(theme: &Theme, b: ListDueBucket) -> Color {
    match b {
        ListDueBucket::Overdue => theme.overdue,
        ListDueBucket::Today => theme.today,
        ListDueBucket::ThisWeek => theme.accent,
        ListDueBucket::NextWeek => theme.accent,
        ListDueBucket::Later => theme.accent,
        ListDueBucket::NoDue => theme.dim,
    }
}

fn push_blanks(lines: &mut Vec<Line>, n: usize) {
    for _ in 0..n {
        lines.push(Line::raw(" "));
    }
}
