use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, GroupKey, Mode, View};
use crate::ui::{header, keep_cursor_visible, task_row};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let bordered = app.prefs.borders;

    let props = header::HeaderProps {
        title: Some("done.txt"),
        count: app.archive().len(),
        sort: "completion-date",
        filter: None,
    };

    let body_area = if bordered {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border).bg(theme.bg))
            .title(header::header_line(theme, props))
            .style(Style::default().bg(theme.bg));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [_sp, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(inner);
        body
    } else {
        super::fill_bg(frame, area, Style::default().bg(theme.bg));
        let [header_area, _sp, body] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .areas(area);
        header::render(frame, header_area, theme, props);
        body
    };

    let visible = app.visible_indices();
    let groups = app.visible_groups();
    let blank = super::density_blank_lines(app.prefs.density);
    let cursor_active = app.mode != Mode::Help && app.mode != Mode::Settings;

    if visible.is_empty() {
        let para = Paragraph::new(vec![Line::from(Span::styled(
            "   no completed tasks yet".to_string(),
            Style::default().fg(theme.dim),
        ))])
        .style(Style::default().bg(theme.bg).fg(theme.fg));
        frame.render_widget(para, body_area);
        return;
    }

    // Pre-count rows per date so each header can show "{date}  N completed".
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for g in groups {
        if let GroupKey::ArchiveDate(d) = g {
            *counts.entry(d.as_str()).or_insert(0) += 1;
        }
    }

    let mut lines: Vec<Line> = Vec::new();
    let mut hit_rows: Vec<Option<usize>> = Vec::new();
    let mut last_date: Option<&str> = None;
    let mut cursor_line: Option<usize> = None;

    for (i, (&abs, gk)) in visible.iter().zip(groups.iter()).enumerate() {
        let date = match gk {
            GroupKey::ArchiveDate(d) => d.as_str(),
            _ => continue,
        };
        if last_date != Some(date) {
            if !lines.is_empty() {
                for _ in 0..blank {
                    lines.push(Line::raw(" "));
                }
                hit_rows.extend(std::iter::repeat_n(None, blank));
            }
            let count = *counts.get(date).unwrap_or(&0);
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    date.to_string(),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{} completed", count),
                    Style::default().fg(theme.dim),
                ),
            ]));
            hit_rows.push(None);
            last_date = Some(date);
        }

        let task = &app.archive().tasks()[abs];
        let opts = task_row::RowOpts {
            idx_label: i,
            cursor: i == app.cursor && cursor_active,
            multi_mode: false,
            multi_checked: false,
            selected: false,
            show_line_num: app.prefs.layout.line_num,
            match_term: None,
            today: app.today(),
            hidden_keys: &app.prefs.hidden_keys,
            sel_cols: None,
        };
        if i == app.cursor {
            cursor_line = Some(lines.len());
        }
        lines.push(task_row::build_line(task, opts, theme));
        hit_rows.push(Some(i));
    }

    let scroll_cell = &app.view_scroll[View::Archive.idx()];
    let scroll = keep_cursor_visible(
        scroll_cell.get(),
        cursor_line,
        body_area.height,
        lines.len(),
        app.cursor == 0,
    );
    scroll_cell.set(scroll);

    let mut mouse = app.mouse_hit.borrow_mut();
    mouse.body_rect = Some((body_area.x, body_area.y, body_area.width, body_area.height));
    mouse.body_rows = hit_rows;

    let para = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .scroll((scroll, 0));
    frame.render_widget(para, body_area);
}
