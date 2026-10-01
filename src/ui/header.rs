use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::Filter;
use crate::theme::Theme;

/// Build the human-readable filter chip shown in the header. `+project` /
/// `@context` / `/search` precedence matches what the sidebar shows. Returns
/// `None` when no filter is active.
pub fn filter_label(filter: &Filter) -> Option<String> {
    if let Some(p) = &filter.project {
        Some(format!("+{p}"))
    } else if let Some(c) = &filter.context {
        Some(format!("@{c}"))
    } else if !filter.search.is_empty() {
        Some(format!("/{}", filter.search))
    } else {
        None
    }
}

/// Inputs for the top-of-screen header bar. Grouped into a struct so call
/// sites pass labelled fields instead of positional `&str` args (which were
/// trivially swappable — `title` and `file` have the same type).
pub struct HeaderProps<'a> {
    pub title: Option<&'a str>,
    // pub file: &'a str,
    pub count: usize,
    pub sort: &'a str,
    pub filter: Option<&'a str>,
}

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, props: HeaderProps<'_>) {
    let line = header_line(theme, props);
    let para = Paragraph::new(line).style(Style::default().bg(theme.bg));
    frame.render_widget(para, area);
}

/// Build the header line for a pane. Reused as the border title of the
/// central list/archive pane when borders are enabled.
pub fn header_line<'a>(theme: &Theme, props: HeaderProps<'_>) -> Line<'a> {
    let mut spans: Vec<Span> = vec![
        Span::styled(
            "Tuxedo Task",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];
    if let Some(t) = props.title {
        spans.push(Span::styled(
            t.to_string(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ));
    }
    spans.extend([
        Span::styled("  •  ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("{} tasks", props.count),
            Style::default().fg(theme.dim),
        ),
        Span::styled("  •  ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("sort:{}", props.sort),
            Style::default().fg(theme.accent),
        ),
    ]);
    if let Some(f) = props.filter {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            format!("filter:{}", f),
            Style::default().fg(theme.context),
        ));
    }
    Line::from(spans).style(Style::default().bg(theme.bg))
}
