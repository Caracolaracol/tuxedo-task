use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::theme::Theme;

// Original domino-mask mark for Tuxedo Task. Each design cell renders as two
// blocks to keep square proportions under typical 2:1 terminal cell metrics.
// The open eye shapes inherit the panel background; the mask uses the theme
// accent so it remains legible across built-in and custom palettes.
pub const WIDTH: u16 = 30;
pub const HEIGHT: u16 = 5;

const BLOCK: &str = "██";
const SPACE: &str = "  ";

// 15 columns × 5 rows. 'b' = mask, ' ' = background / eye openings.
const GRID: [&[u8; 15]; 5] = [
    b"bb           bb",
    b"bbbbbbb bbbbbbb",
    b"bbb  bbbbb  bbb",
    b" bb   bbb   bb ",
    b"  bbbbb bbbbb  ",
];

#[cfg(test)]
pub(crate) fn ascii_rows() -> Vec<String> {
    GRID.iter()
        .map(|row| {
            row.iter()
                .map(|c| if *c == b'b' { BLOCK } else { SPACE })
                .collect()
        })
        .collect()
}

pub fn centered_lines(theme: &Theme, inner_width: u16) -> Vec<Line<'static>> {
    let pad_w = inner_width.saturating_sub(WIDTH) / 2;
    let pad = " ".repeat(pad_w as usize);
    let body = Style::default().fg(theme.accent);
    GRID.iter()
        .map(|row| {
            let mut spans: Vec<Span<'static>> = Vec::with_capacity(row.len() + 1);
            spans.push(Span::raw(pad.clone()));
            for &c in row.iter() {
                spans.push(if c == b'b' {
                    Span::styled(BLOCK, body)
                } else {
                    Span::raw(SPACE)
                });
            }
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_shape() {
        let rows = ascii_rows();
        assert_eq!(rows.len(), HEIGHT as usize);
        for row in &rows {
            assert_eq!(row.chars().count(), WIDTH as usize);
        }
        // A symmetric silhouette with two open eyes, not the upstream bowtie.
        for row in GRID {
            assert!(row.iter().eq(row.iter().rev()));
        }
        assert_eq!(GRID[2], b"bbb  bbbbb  bbb");
        assert_eq!(GRID[3], b" bb   bbb   bb ");
    }

    #[test]
    fn mask_is_centered_and_uses_theme_accent() {
        let theme = &crate::theme::BUILT_IN[0];
        let lines = centered_lines(theme, WIDTH + 10);
        assert_eq!(lines.len(), HEIGHT as usize);
        for line in lines {
            assert_eq!(line.spans[0].content, "     ");
            assert_eq!(line.width(), WIDTH as usize + 5);
            for span in &line.spans[1..] {
                if span.content == BLOCK {
                    assert_eq!(span.style.fg, Some(theme.accent));
                }
            }
        }
    }
}
