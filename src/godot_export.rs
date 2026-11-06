//! `clawcode export-godot`: turns the lessons into one JSON file for the Godot
//! (Android) app in `godot/`. Godot reads JSON natively but not TOML, and it
//! cannot run `rustc` on a phone, so this step also converts the Markdown into
//! Godot BBCode and the coding challenge into tap puzzles.
use crate::lesson::{Lesson, Step};
use crate::tap::{self, Part, TapKind, TokKind};
use serde_json::{Value, json};

pub const DEFAULT_OUT: &str = "godot/data/lessons.json";

// Colours (also used by the Godot theme so everything matches).
const C_HEAD: &str = "#7fd1ff";
const C_TERM: &str = "#e59cff";
const C_CODE: &str = "#ffd36e";
const C_PINCH: &str = "#ff8c7a";
const C_KW: &str = "#c792ea";
const C_STR: &str = "#c3e88d";
const C_NUM: &str = "#f78c6c";
const C_COMMENT: &str = "#7f8ca3";
const C_TYPE: &str = "#ffcb6b";
const C_MACRO: &str = "#82aaff";
const C_CODE_BG: &str = "#1b2333";
/// Roughly how many monospace characters fit across a phone screen in the app.
const PHONE_COLUMNS: usize = 50;

/// Escapes text so Godot's RichTextLabel shows `[` and `]` literally.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '[' => out.push_str("[lb]"),
            ']' => out.push_str("[rb]"),
            _ => out.push(c),
        }
    }
    out
}

fn color(c: &str, s: &str) -> String {
    format!("[color={c}]{s}[/color]")
}

/// Syntax-highlights one fragment of Rust code as BBCode (no [code] wrapper).
pub fn highlight(code: &str) -> String {
    let mut out = String::new();
    for (li, line) in code.split('\n').enumerate() {
        if li > 0 {
            out.push('\n');
        }
        let toks = tap::tokenize(line);
        for (i, t) in toks.iter().enumerate() {
            let e = esc(&t.text);
            let next_is_bang = toks.get(i + 1).is_some_and(|n| n.text == "!");
            out.push_str(&match t.kind {
                TokKind::Comment => color(C_COMMENT, &e),
                TokKind::Str | TokKind::Char => color(C_STR, &e),
                TokKind::Number => color(C_NUM, &e),
                TokKind::Lifetime => color(C_KW, &e),
                TokKind::Ident if tap::KEYWORDS.contains(&t.text.as_str()) => color(C_KW, &e),
                TokKind::Ident if next_is_bang => color(C_MACRO, &e),
                TokKind::Ident if t.text.starts_with(char::is_uppercase) => color(C_TYPE, &e),
                _ => e,
            });
        }
    }
    out
}

/// Handles `inline code` and **bold** inside one line of prose.
pub fn inline(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    let mut bold = false;
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('`')
            && let Some(end) = r.find('`')
        {
            out.push_str(&format!("[code]{}[/code]", color(C_CODE, &esc(&r[..end]))));
            rest = &r[end + 1..];
            continue;
        }
        if let Some(r) = rest.strip_prefix("**") {
            out.push_str(if bold { "[/b]" } else { "[b]" });
            bold = !bold;
            rest = r;
            continue;
        }
        let c = rest.chars().next().unwrap();
        out.push_str(&esc(&c.to_string()));
        rest = &rest[c.len_utf8()..];
    }
    if bold {
        out.push_str("[/b]");
    }
    out
}

/// A shaded block of highlighted code.
fn code_block(lines: &[&str]) -> String {
    let widest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 1;
    // Narrow blocks are padded so the background looks like a box. Wide ones
    // would wrap on a phone, and padded wrapped lines look messy, so skip it.
    let width = if widest <= PHONE_COLUMNS { widest } else { 0 };
    let body: Vec<String> = lines
        .iter()
        .map(|l| {
            let pad = " ".repeat(width.saturating_sub(l.chars().count()));
            format!("[bgcolor={C_CODE_BG}] {}{pad}[/bgcolor]", highlight(l))
        })
        .collect();
    format!("[code]{}[/code]", body.join("\n"))
}

/// Converts the lesson Markdown subset (docs/LESSON_FORMAT.md) into BBCode.
/// Consecutive prose lines are joined into one paragraph so text wraps nicely on a phone.
pub fn markdown(src: &str) -> String {
    let mut blocks: Vec<String> = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let mut code: Option<Vec<&str>> = None;
    let flush = |para: &mut Vec<&str>, blocks: &mut Vec<String>| {
        if !para.is_empty() {
            blocks.push(inline(&para.join(" ")));
            para.clear();
        }
    };
    for raw in src.lines() {
        let line = raw.trim_end();
        if line.trim_start().starts_with("```") {
            if let Some(lines) = code.take() {
                blocks.push(code_block(&lines));
            } else {
                flush(&mut para, &mut blocks);
                code = Some(Vec::new());
            }
            continue;
        }
        if let Some(lines) = code.as_mut() {
            lines.push(line);
            continue;
        }
        if line.trim().is_empty() {
            flush(&mut para, &mut blocks);
        } else if let Some(h) = line.strip_prefix("## ") {
            flush(&mut para, &mut blocks);
            blocks.push(format!(
                "[font_size=30][b]{}[/b][/font_size]",
                color(C_HEAD, &inline(h))
            ));
        } else if let Some(h) = line.strip_prefix("# ") {
            flush(&mut para, &mut blocks);
            blocks.push(format!(
                "[font_size=36][b]{}[/b][/font_size]",
                color(C_HEAD, &inline(h))
            ));
        } else if let Some(tip) = line.strip_prefix("> ") {
            flush(&mut para, &mut blocks);
            blocks.push(color(
                C_PINCH,
                &format!("[b](\\/) Pinch:[/b] [i]{}[/i]", inline(tip)),
            ));
        } else if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            flush(&mut para, &mut blocks);
            blocks.push(format!("   •  {}", inline(item)));
        } else if let Some((term, def)) = line
            .split_once(" :: ")
            .filter(|(t, _)| !t.trim().is_empty() && t.len() < 40)
        {
            flush(&mut para, &mut blocks);
            blocks.push(format!(
                "[b]{}[/b]  —  [i]{}[/i]",
                color(C_TERM, &esc(term.trim().trim_matches('`'))),
                inline(def.trim())
            ));
        } else {
            para.push(line.trim());
        }
    }
    if let Some(lines) = code {
        blocks.push(code_block(&lines));
    }
    flush(&mut para, &mut blocks);
    blocks.join("\n\n")
}

fn parts_json(parts: &[Part]) -> Value {
    Value::Array(
        parts
            .iter()
            .map(|p| match p {
                Part::Text(t) => json!({ "t": highlight(t) }),
                Part::Blank(i) => json!({ "b": i }),
            })
            .collect(),
    )
}

/// The tap puzzles for a lesson: hand-written `[[tap]]` tables, or automatic ones.
pub fn taps_json(l: &Lesson) -> (Vec<Value>, bool) {
    if !l.tap.is_empty() {
        let taps = l
            .tap
            .iter()
            .map(|t| {
                let prompt = if t.prompt.is_empty() {
                    &l.goal
                } else {
                    &t.prompt
                };
                match t.kind {
                    TapKind::Fill => json!({
                        "kind": "fill",
                        "prompt": inline(prompt),
                        "parts": parts_json(&tap::parts_from_marked(&t.code)),
                        "answer": t.answer,
                        "decoys": t.decoys,
                        "explain": inline(&t.explain),
                        "hp": t.answer.len(),
                    }),
                    TapKind::Order => {
                        let line = |s: &String| json!({ "text": s, "bb": highlight(s) });
                        json!({
                            "kind": "order",
                            "prompt": inline(prompt),
                            "lines": t.order_lines().iter().map(line).collect::<Vec<_>>(),
                            "decoys": t.decoys.iter().map(line).collect::<Vec<_>>(),
                            "explain": inline(&t.explain),
                            "hp": t.order_lines().len(),
                        })
                    }
                }
            })
            .collect();
        return (taps, false);
    }
    let taps = tap::auto_taps(&l.starter, &l.solution, &l.goal, l.boss.is_some())
        .into_iter()
        .map(|p| {
            json!({
                "kind": "fill",
                "prompt": inline(&p.prompt),
                "parts": parts_json(&p.parts),
                "answer": p.answer,
                "decoys": p.decoys,
                "explain": inline(&p.explain),
                "hp": p.answer.len(),
            })
        })
        .collect();
    (taps, true)
}

pub fn lesson_json(l: &Lesson) -> Value {
    let cards: Vec<String> = l.cards().iter().map(|c| markdown(c)).collect();
    let steps: Vec<Value> = l
        .steps()
        .into_iter()
        .map(|s| match s {
            Step::Card(i) => json!({ "card": i }),
            Step::Quiz(i) => json!({ "quiz": i }),
        })
        .collect();
    let quizzes: Vec<Value> = l
        .quiz
        .iter()
        .map(|q| {
            json!({
                "question": markdown(&q.question),
                "choices": q.choices,
                "answer": q.answer,
                "explain": inline(&q.explain),
            })
        })
        .collect();
    let (taps, auto) = taps_json(l);
    json!({
        "id": l.id,
        "order": l.order,
        "chapter": l.chapter,
        "title": l.title,
        "xp": l.xp,
        "concepts": l.concepts,
        "cards": cards,
        "steps": steps,
        "quizzes": quizzes,
        "goal": inline(&l.goal),
        "hints": l.hints.iter().map(|h| inline(h)).collect::<Vec<_>>(),
        "success": inline(&l.success),
        "solution": highlight(l.solution.trim_matches('\n')),
        "boss": l.boss.as_ref().map(|b| json!({
            "name": b.name,
            "art": b.art.trim_matches('\n'),
            "taunt": b.taunt,
            "defeat": b.defeat,
        })),
        "taps": taps,
        "taps_auto": auto,
    })
}

pub fn export(lessons: &[Lesson]) -> String {
    let doc = json!({
        "format": 1,
        "generated_by": "clawcode export-godot (do not edit by hand; edit lessons/*.toml)",
        "lessons": lessons.iter().map(lesson_json).collect::<Vec<_>>(),
    });
    let mut s = serde_json::to_string_pretty(&doc).unwrap();
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_brackets() {
        assert_eq!(esc("a[0]"), "a[lb]0[rb]");
        assert!(highlight("let v = [1];").contains("[lb]"));
    }

    #[test]
    fn markdown_basics() {
        let bb = markdown("# Hi\n\nSome `code` and **bold**\ntext.\n\n> tip");
        assert!(bb.contains("[b]bold[/b]"));
        assert!(bb.contains("Some [code]"));
        assert!(bb.contains("and [b]bold[/b] text."));
        assert!(bb.contains("Pinch:"));
    }

    #[test]
    fn every_lesson_has_playable_taps() {
        for l in crate::lesson::load_all().unwrap() {
            let (taps, _) = taps_json(&l);
            assert!(!taps.is_empty(), "{} has no tap puzzles", l.id);
            for t in &taps {
                if t["kind"] == "fill" {
                    let blanks = t["parts"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|p| p.get("b").is_some())
                        .count();
                    assert_eq!(blanks, t["answer"].as_array().unwrap().len(), "{}", l.id);
                    assert!(blanks > 0, "{}", l.id);
                }
            }
        }
    }
}
