use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, FilterTarget, ordered_unique};
use crate::search::subseq_match_ci;
use crate::theme::Theme;
use crate::todo;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let bordered = app.prefs.borders;

    // Optional border around the whole pane. Content renders inside so the
    // border doesn't collide with the rows; the mouse hit-map uses the inner
    // rect for the same reason. The pane is transparent (bg = main bg) so
    // only the border separates it from the rest of the UI.
    let block = Block::default()
        .borders(if bordered {
            Borders::ALL
        } else {
            Borders::NONE
        })
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border).bg(theme.bg))
        .title(Line::from(Span::styled(
            " フィルター ",
            Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().bg(theme.bg));
    let content = if bordered { block.inner(area) } else { area };
    frame.render_widget(block, area);

    let projects = ordered_unique(app.tasks(), |t| &t.projects);
    let contexts = ordered_unique(app.tasks(), |t| &t.contexts);

    let mut lines: Vec<Line> = Vec::new();
    let mut hit_rows: Vec<Option<FilterTarget>> = Vec::new();
    lines.push(line_pad(theme, vec![Span::raw(" ")]));
    hit_rows.push(None);
    lines.push(line_pad(
        theme,
        vec![Span::styled(
            " プロジェクト",
            Style::default()
                .fg(theme.project)
                .add_modifier(Modifier::BOLD),
        )],
    ));
    hit_rows.push(None);
    if projects.is_empty() {
        lines.push(hint_row(theme, "+project", theme.project));
        hit_rows.push(None);
    } else {
        for (name, count) in &projects {
            let active = app.filter.project.as_deref() == Some(name.as_str());
            lines.push(filter_row(theme, "+", name, *count, active, theme.project));
            hit_rows.push(Some(FilterTarget::Project(name.clone())));
        }
    }
    lines.push(line_pad(theme, vec![Span::raw(" ")]));
    hit_rows.push(None);
    lines.push(line_pad(
        theme,
        vec![Span::styled(
            " コンテキスト",
            Style::default()
                .fg(theme.context)
                .add_modifier(Modifier::BOLD),
        )],
    ));
    hit_rows.push(None);
    if contexts.is_empty() {
        lines.push(hint_row(theme, "@context", theme.context));
        hit_rows.push(None);
    } else {
        for (name, count) in &contexts {
            let active = app.filter.context.as_deref() == Some(name.as_str());
            lines.push(filter_row(theme, "@", name, *count, active, theme.context));
            hit_rows.push(Some(FilterTarget::Context(name.clone())));
        }
    }

    let saved = app.saved_filters();
    if !saved.is_empty() {
        lines.push(line_pad(theme, vec![Span::raw(" ")]));
        hit_rows.push(None);
        lines.push(line_pad(
            theme,
            vec![Span::styled(
                " SAVED",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )],
        ));
        hit_rows.push(None);
        for f in saved {
            let active = app.filter().search == f.query;
            let count = app
                .tasks()
                .iter()
                .filter(|t| {
                    !t.done
                        && subseq_match_ci(todo::body_after_priority(&t.raw), &f.query).is_some()
                })
                .count();
            lines.push(filter_row(theme, "", &f.name, count, active, theme.accent));
            hit_rows.push(Some(FilterTarget::Saved(f.name.clone())));
        }
    }

    // Rebuild the mouse hit-map for this frame.
    let mut mouse = app.mouse_hit.borrow_mut();
    mouse.left_rect = Some((content.x, content.y, content.width, content.height));
    mouse.left_rows = hit_rows;

    let para = Paragraph::new(lines).style(Style::default().bg(theme.bg).fg(theme.fg));
    frame.render_widget(para, content);
}

fn filter_row<'a>(
    theme: &Theme,
    sigil: &str,
    name: &'a str,
    count: usize,
    active: bool,
    sigil_color: ratatui::style::Color,
) -> Line<'a> {
    // Inactive rows are transparent (bg = main bg); the active filter keeps a
    // highlight chip so it still reads as selected against the border.
    let bg = if active { theme.selected } else { theme.bg };
    let prefix = if active { "▸ " } else { "  " };
    let mut padded = format!("{}{}", sigil, name);
    if padded.chars().count() < 16 {
        let pad = 16 - padded.chars().count();
        padded.push_str(&" ".repeat(pad));
    }
    Line::from(vec![
        Span::raw(" "),
        Span::styled(prefix.to_string(), Style::default().fg(theme.accent)),
        Span::styled(padded, Style::default().fg(sigil_color)),
        Span::styled(format!("{:>3}", count), Style::default().fg(theme.dim)),
    ])
    .style(Style::default().bg(bg))
}

fn hint_row<'a>(theme: &Theme, token: &'a str, token_color: ratatui::style::Color) -> Line<'a> {
    Line::from(vec![
        Span::raw("   "),
        Span::styled("tag with ", Style::default().fg(theme.dim)),
        Span::styled(token, Style::default().fg(token_color)),
    ])
    .style(Style::default().bg(theme.bg))
}

fn line_pad<'a>(theme: &Theme, spans: Vec<Span<'a>>) -> Line<'a> {
    Line::from(spans).style(Style::default().bg(theme.bg))
}
