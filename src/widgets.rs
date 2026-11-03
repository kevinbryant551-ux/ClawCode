//! Shared drawing helpers and the code editor's typing assistance.
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use tui_textarea::{CursorMove, TextArea};

pub const ACCENT: Color = Color::Cyan;

pub fn make_editor(code: &str) -> TextArea<'static> {
    let mut ed = TextArea::new(code.lines().map(String::from).collect());
    ed.set_tab_length(4);
    ed.set_hard_tab_indent(false);
    ed.set_line_number_style(Style::new().fg(Color::DarkGray));
    ed.set_cursor_line_style(Style::new());
    ed.set_block(rounded(" Your code (main.rs) ".into()).border_style(Style::new().fg(ACCENT)));
    ed
}

/// Editor typing with a little help: Enter keeps the indentation (and adds a
/// level after `{`), and typing `}` on an empty line steps back a level.
pub fn editor_input(ed: &mut TextArea<'static>, key: KeyEvent) {
    let plain = !key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    match key.code {
        KeyCode::Enter if plain => {
            let (row, col) = ed.cursor();
            let line = ed.lines()[row].clone();
            let before: String = line.chars().take(col).collect();
            let indent: String = line.chars().take_while(|c| *c == ' ').collect();
            let extra = if before.trim_end().ends_with('{')
                || before.trim_end().ends_with('(')
                || before.trim_end().ends_with('[')
            {
                "    "
            } else {
                ""
            };
            let after_starts_close = line
                .chars()
                .skip(col)
                .find(|c| !c.is_whitespace())
                .is_some_and(|c| matches!(c, '}' | ')' | ']'));
            ed.insert_newline();
            ed.insert_str(format!("{indent}{extra}"));
            if !extra.is_empty() && after_starts_close {
                // `{|}` + Enter → put the closing brace on its own line.
                ed.insert_newline();
                ed.insert_str(&indent);
                ed.move_cursor(CursorMove::Up);
                ed.move_cursor(CursorMove::End);
            }
            // Drop spaces that were between the cursor and the closing brace.
        }
        KeyCode::Char(c @ ('}' | ')' | ']')) if plain => {
            let (row, col) = ed.cursor();
            let before: String = ed.lines()[row].chars().take(col).collect();
            if col >= 4 && before.chars().all(|c| c == ' ') {
                for _ in 0..4 {
                    ed.delete_char();
                }
            }
            ed.insert_char(c);
        }
        _ => {
            ed.input(key);
        }
    }
}

pub fn which(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(cmd).is_file()))
}

pub fn rounded(title: String) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
}

pub fn keys_line(keys: &[(&str, &str)]) -> Paragraph<'static> {
    let mut spans = Vec::new();
    for (k, d) in keys {
        spans.push(Span::styled(
            format!(" {k} "),
            Style::new().fg(Color::Black).bg(Color::Gray),
        ));
        spans.push(Span::styled(
            format!(" {d}  "),
            Style::new().fg(Color::Gray),
        ));
    }
    Paragraph::new(Line::from(spans))
}

pub fn line_text(l: &Line) -> String {
    l.spans.iter().map(|s| s.content.as_ref()).collect()
}

/// Rough number of rows a line takes when wrapped to `width` (word wrap makes
/// it a bit taller than this, so we add a small margin for long lines).
pub fn wrapped_height(text: &str, width: u16) -> u16 {
    use unicode_width::UnicodeWidthStr;
    let w = width.max(1) as usize;
    let len = UnicodeWidthStr::width(text);
    if len == 0 {
        1
    } else {
        let rows = len.div_ceil(w);
        (rows + rows / 6) as u16
    }
}

pub fn popup(f: &mut Frame, area: Rect, title: &str, text: Text<'static>, w: u16, h: u16) {
    let w = w.min(area.width.saturating_sub(2));
    let h = h.min(area.height.saturating_sub(2));
    let r = Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    };
    f.render_widget(Clear, r);
    f.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: false }).block(
            rounded(title.to_string())
                .border_style(Style::new().fg(Color::Yellow))
                .border_type(BorderType::Double),
        ),
        r,
    );
}
