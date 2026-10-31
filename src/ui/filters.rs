use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, FilterTarget, ordered_unique};
use crate::search::subseq_match_ci;
use crate::theme::Theme;
use crate::todo;

struct Section<'a> {
    title: &'static str,
    sigil: &'static str,
    hint: &'static str,
    keys: &'static str,
    color: Color,
    values: Vec<(String, usize)>,
    active: Option<&'a str>,
    target: fn(String) -> FilterTarget,
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let block = Block::default()
        .borders(if app.prefs.borders {
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
    let content = if app.prefs.borders {
        block.inner(area)
    } else {
        area
    };
    frame.render_widget(block, area);

    let mut sections = vec![
        Section {
            title: " プロジェクト",
            sigil: "+",
            hint: "project",
            keys: "fp",
            color: theme.project,
            values: ordered_unique(app.tasks(), |t| &t.projects),
            active: app.filter.project.as_deref(),
            target: FilterTarget::Project,
        },
        Section {
            title: " コンテキスト",
            sigil: "@",
            hint: "context",
            keys: "fc",
            color: theme.context,
            values: ordered_unique(app.tasks(), |t| &t.contexts),
            active: app.filter.context.as_deref(),
            target: FilterTarget::Context,
        },
        Section {
            title: " ペルソナ",
            sigil: "::",
            hint: "person",
            keys: "fm",
            color: theme.accent,
            values: ordered_unique(app.tasks(), |t| &t.people),
            active: app.filter.person.as_deref(),
            target: FilterTarget::Person,
        },
    ];
    let saved = app.saved_filters();
    if !saved.is_empty() {
        sections.push(Section {
            title: " SAVED",
            sigil: "",
            hint: "",
            keys: "ff",
            color: theme.accent,
            active: saved
                .iter()
                .find(|f| f.query == app.filter.search)
                .map(|f| f.name.as_str()),
            values: saved
                .iter()
                .map(|f| {
                    let count = app
                        .tasks()
                        .iter()
                        .filter(|t| {
                            !t.done
                                && subseq_match_ci(todo::body_after_priority(&t.raw), &f.query)
                                    .is_some()
                        })
                        .count();
                    (f.name.clone(), count)
                })
                .collect(),
            target: FilterTarget::Saved,
        });
    }

    // Share the vertical space instead of letting a long project list push
    // Persona (and saved searches) offscreen. Short sections donate spare rows.
    let budget = (content.height as usize).saturating_sub(1 + 2 * sections.len());
    let mut limits = vec![1; sections.len()];
    let mut remaining = budget.saturating_sub(sections.len());
    while remaining > 0 {
        let before = remaining;
        for (section, limit) in sections.iter().zip(&mut limits) {
            if *limit < section.values.len() && remaining > 0 {
                *limit += 1;
                remaining -= 1;
            }
        }
        if before == remaining {
            break;
        }
    }

    let mut lines = vec![Line::raw(" ")];
    let mut hit_rows = vec![None];
    for (section, limit) in sections.iter().zip(limits) {
        lines.push(Line::from(Span::styled(
            section.title,
            Style::default()
                .fg(section.color)
                .add_modifier(Modifier::BOLD),
        )));
        hit_rows.push(None);
        if section.values.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("   tag with ", Style::default().fg(theme.dim)),
                Span::styled(
                    format!("{}{}", section.sigil, section.hint),
                    Style::default().fg(section.color),
                ),
            ]));
            hit_rows.push(None);
        } else {
            let truncated = section.values.len() > limit;
            let shown = if truncated {
                limit.saturating_sub(1).max(1)
            } else {
                limit
            };
            let active = section
                .values
                .iter()
                .position(|(name, _)| Some(name.as_str()) == section.active);
            // Keep the active item on screen while cycling with the keyboard.
            let start = active.map_or(0, |i| i.saturating_sub(shown - 1));
            for (name, count) in section.values.iter().skip(start).take(shown) {
                lines.push(filter_row(
                    theme,
                    section.sigil,
                    name,
                    *count,
                    Some(name.as_str()) == section.active,
                    section.color,
                ));
                hit_rows.push(Some((section.target)(name.clone())));
            }
            if truncated && limit > 1 {
                lines.push(Line::from(Span::styled(
                    format!("   … {} to browse", section.keys),
                    Style::default().fg(theme.dim),
                )));
                hit_rows.push(None);
            }
        }
        lines.push(Line::raw(" "));
        hit_rows.push(None);
    }
    let mut mouse = app.mouse_hit.borrow_mut();
    mouse.left_rect = Some((content.x, content.y, content.width, content.height));
    mouse.left_rows = hit_rows;
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(theme.bg).fg(theme.fg)),
        content,
    );
}

fn filter_row(
    theme: &Theme,
    sigil: &str,
    name: &str,
    count: usize,
    active: bool,
    color: Color,
) -> Line<'static> {
    let bg = if active { theme.selected } else { theme.bg };
    let prefix = if active { "▸ " } else { "  " };
    let mut padded = format!("{sigil}{name}");
    let width = Span::raw(padded.as_str()).width();
    padded.push_str(&" ".repeat(16usize.saturating_sub(width)));
    Line::from(vec![
        Span::raw(" "),
        Span::styled(prefix, Style::default().fg(theme.accent)),
        Span::styled(padded, Style::default().fg(color)),
        Span::styled(format!("{count:>3}"), Style::default().fg(theme.dim)),
    ])
    .style(Style::default().bg(bg))
}
