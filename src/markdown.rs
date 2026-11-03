//! A tiny renderer for the Markdown subset used in lesson intros.
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub const CODE: Style = Style::new().fg(Color::Yellow);
pub const TERM: Style = Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD);
pub const PINCH: Style = Style::new().fg(Color::LightRed);

pub fn render(src: &str) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let mut in_code = false;
    for raw in src.lines() {
        let line = raw.trim_end();
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            if !in_code {
                out.push(Line::default());
            }
            continue;
        }
        if in_code {
            out.push(Line::from(Span::styled(format!("  │ {line}"), CODE)));
            continue;
        }
        if let Some(h) = line.strip_prefix("## ") {
            out.push(Line::from(Span::styled(
                h.to_string(),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
        } else if let Some(h) = line.strip_prefix("# ") {
            out.push(Line::from(Span::styled(
                h.to_uppercase(),
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));
        } else if let Some(tip) = line.strip_prefix("> ") {
            let mut spans = vec![Span::styled(
                "(\\/) Pinch: ",
                PINCH.add_modifier(Modifier::BOLD),
            )];
            spans.extend(inline(
                tip,
                Style::new()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::ITALIC),
            ));
            out.push(Line::from(spans));
        } else if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            let mut spans = vec![Span::raw("  • ")];
            spans.extend(inline(item, Style::new()));
            out.push(Line::from(spans));
        } else if let Some((term, def)) = line
            .split_once(" :: ")
            .filter(|(t, _)| !t.trim().is_empty() && t.len() < 40)
        {
            let mut spans = vec![
                Span::styled(format!("◆ {}", term.trim().trim_matches('`')), TERM),
                Span::raw(" — "),
            ];
            spans.extend(inline(
                def.trim(),
                Style::new().add_modifier(Modifier::ITALIC),
            ));
            out.push(Line::from(spans));
        } else {
            out.push(Line::from(inline(line, Style::new())));
        }
    }
    out
}

/// Handles `code` and **bold** inside a line.
pub fn inline(text: &str, base: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut buf = String::new();
    let mut chars = text.chars().peekable();
    let flush = |buf: &mut String, spans: &mut Vec<Span<'static>>, style: Style| {
        if !buf.is_empty() {
            spans.push(Span::styled(std::mem::take(buf), style));
        }
    };
    while let Some(c) = chars.next() {
        if c == '`' {
            flush(&mut buf, &mut spans, base);
            let mut code = String::new();
            for c2 in chars.by_ref() {
                if c2 == '`' {
                    break;
                }
                code.push(c2);
            }
            spans.push(Span::styled(code, CODE));
        } else if c == '*' && chars.peek() == Some(&'*') {
            chars.next();
            flush(&mut buf, &mut spans, base);
            let mut bold = String::new();
            while let Some(c2) = chars.next() {
                if c2 == '*' && chars.peek() == Some(&'*') {
                    chars.next();
                    break;
                }
                bold.push(c2);
            }
            spans.push(Span::styled(bold, base.add_modifier(Modifier::BOLD)));
        } else {
            buf.push(c);
        }
    }
    flush(&mut buf, &mut spans, base);
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(l: &Line) -> String {
        l.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn renders_subset() {
        let lines = render(
            "# Title\nuse `let` **now**\n\nmut :: can change; call with `::`\n> tip\n```rust\nfn main() {}\n```",
        );
        assert_eq!(text(&lines[0]), "TITLE");
        assert_eq!(text(&lines[1]), "use let now");
        assert!(text(&lines[3]).starts_with("◆ mut — can change"));
        assert!(text(&lines[4]).contains("Pinch"));
        assert!(text(&lines[5]).contains("fn main"));
    }
}
