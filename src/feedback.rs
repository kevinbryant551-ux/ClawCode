//! Turns a check `Outcome` into what Pinch says in the feedback panel.
use crate::checker::{Diag, Outcome};
use crate::friendly::Friendly;
use crate::lesson::Lesson;
use crate::markdown::{CODE, PINCH, inline};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood {
    Happy,
    Thinking,
    Worried,
    Party,
    Sleepy,
}

#[derive(Debug, Clone)]
pub struct Feedback {
    pub mood: Mood,
    pub lines: Vec<Line<'static>>,
}

impl Feedback {
    pub fn say(mood: Mood, text: &str) -> Feedback {
        Feedback {
            mood,
            lines: text
                .lines()
                .map(|l| Line::from(inline(l, Style::new())))
                .collect(),
        }
    }
}

const BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
const DIM: Style = Style::new().fg(Color::DarkGray);
const GOOD: Style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);
const BAD: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);

/// Pinch varies the opener so repeated failures don't feel robotic.
fn opener(attempt: u32, options: &[&'static str]) -> &'static str {
    options[attempt as usize % options.len()]
}

pub fn describe(outcome: &Outcome, lesson: &Lesson, friendly: &Friendly, attempt: u32) -> Feedback {
    let mut l: Vec<Line<'static>> = Vec::new();
    let mood;
    match outcome {
        Outcome::Passed { warnings, stdout } => {
            mood = Mood::Party;
            l.push(Line::from(Span::styled(
                "✔ It works! Every check passed.",
                GOOD,
            )));
            if !lesson.success.is_empty() {
                l.push(Line::from(inline(&lesson.success, Style::new())));
            }
            if !stdout.trim().is_empty() {
                l.push(Line::from(Span::styled("Your program printed:", DIM)));
                for o in stdout.lines().take(10) {
                    l.push(Line::from(Span::styled(format!("  {o}"), CODE)));
                }
            }
            if let Some(w) = warnings.first() {
                l.push(Line::default());
                l.push(Line::from(Span::styled(
                    format!("Side note: the compiler left {} warning(s). Warnings don't stop your code, but they're good advice:", warnings.len()),
                    DIM,
                )));
                l.push(Line::from(Span::styled(
                    format!("  line {}: {}", w.line.unwrap_or(0), w.message),
                    DIM,
                )));
            }
        }
        Outcome::CompileError { diags } => {
            mood = Mood::Thinking;
            l.push(Line::from(vec![
                Span::styled("✘ The compiler stopped. ", BAD),
                Span::raw(opener(
                    attempt,
                    &[
                        "No worries, this is the compiler being a helpful teacher.",
                        "Compile errors are just Rust catching bugs early. Let's read it together.",
                        "Every Rustacean sees these daily. Here's what it means:",
                    ],
                )),
            ]));
            if diags.iter().any(|d| d.in_tests) && !diags.iter().any(|d| !d.in_tests) {
                l.push(Line::from(inline(
                    "My hidden checks couldn't use your code. Did a function, struct, or field get **renamed**, or its parameters changed? Keep the names and types from the starter code.",
                    Style::new(),
                )));
            }
            let total = diags.len();
            for d in diags.iter().take(3) {
                l.push(Line::default());
                push_diag(&mut l, d, friendly);
            }
            if total > 3 {
                l.push(Line::from(Span::styled(
                    format!("…plus {} more. Fix the first one first: later errors are often caused by earlier ones.", total - 3),
                    DIM,
                )));
            }
        }
        Outcome::RuleFailed { why } => {
            mood = Mood::Thinking;
            l.push(Line::from(Span::styled(
                "◐ Your code runs, but this lesson asks for something specific:",
                BOLD,
            )));
            l.push(Line::from(inline(why, Style::new())));
        }
        Outcome::WrongOutput { expected, actual } => {
            mood = Mood::Thinking;
            l.push(Line::from(vec![
                Span::styled("◐ Your program ran, ", BOLD),
                Span::raw("but it printed something different from what I expected."),
            ]));
            let exp: Vec<&str> = expected.lines().collect();
            let act: Vec<&str> = actual.lines().collect();
            if let Some(i) = (0..exp.len().max(act.len())).find(|&i| exp.get(i) != act.get(i)) {
                l.push(Line::from(format!(
                    "The first difference is on output line {}:",
                    i + 1
                )));
                l.push(Line::from(vec![
                    Span::styled("  expected: ", GOOD),
                    Span::styled(show(exp.get(i)), CODE),
                ]));
                l.push(Line::from(vec![
                    Span::styled("  you got:  ", BAD),
                    Span::styled(show(act.get(i)), CODE),
                ]));
                if let (Some(e), Some(a)) = (exp.get(i), act.get(i)) {
                    if e.eq_ignore_ascii_case(a) {
                        l.push(Line::from(Span::styled(
                            "  (So close! Only upper/lower case differs.)",
                            PINCH,
                        )));
                    } else if e.trim() == a.trim() {
                        l.push(Line::from(Span::styled(
                            "  (Only spaces at the start differ.)",
                            PINCH,
                        )));
                    }
                }
            }
            l.push(Line::default());
            l.push(Line::from(Span::styled("Full expected output:", DIM)));
            for e in exp.iter().take(20) {
                l.push(Line::from(Span::styled(format!("  {e}"), DIM)));
            }
            l.push(Line::from(Span::styled("Your output:", DIM)));
            if act.is_empty() {
                l.push(Line::from(Span::styled(
                    "  (nothing was printed — did you call println!?)",
                    DIM,
                )));
            }
            for a in act.iter().take(20) {
                l.push(Line::from(Span::styled(format!("  {a}"), DIM)));
            }
        }
        Outcome::TestsFailed {
            failures,
            passed,
            total,
        } => {
            mood = if *passed > 0 {
                Mood::Thinking
            } else {
                Mood::Worried
            };
            l.push(Line::from(vec![
                Span::styled(format!("◐ {passed} of {total} checks passed. "), BOLD),
                Span::raw(if *passed > 0 {
                    "Getting there!"
                } else {
                    "Let's look at what went wrong."
                }),
            ]));
            if failures
                .iter()
                .any(|f| f.message.contains("not yet implemented"))
            {
                l.push(Line::from(inline(
                    "There's still a `todo!()` in your code. `todo!()` is a placeholder that crashes on purpose; replace it with real code.",
                    PINCH,
                )));
            }
            for f in failures.iter().take(4) {
                l.push(Line::default());
                l.push(Line::from(vec![
                    Span::styled("✘ check: ", BAD),
                    Span::styled(f.name.clone(), BOLD),
                ]));
                for m in f.message.lines().take(6) {
                    l.push(Line::from(inline(&format!("  {m}"), Style::new())));
                }
            }
        }
        Outcome::Panicked { stderr } => {
            mood = Mood::Worried;
            l.push(Line::from(Span::styled(
                "✘ Your program compiled, but crashed while running (a \"panic\").",
                BAD,
            )));
            if stderr.contains("not yet implemented") {
                l.push(Line::from(inline(
                    "It hit a `todo!()`. That's a placeholder that crashes on purpose; replace it with real code.",
                    Style::new(),
                )));
            } else {
                l.push(Line::from(inline(
                    "A **panic** is Rust stopping the program on purpose because something impossible happened, like `unwrap()` on `None` or dividing by zero.",
                    Style::new(),
                )));
            }
            for line in stderr.lines().filter(|l| !l.starts_with("note:")).take(6) {
                l.push(Line::from(Span::styled(format!("  {line}"), DIM)));
            }
        }
        Outcome::TimedOut => {
            mood = Mood::Sleepy;
            l.push(Line::from(Span::styled(
                "✘ Your program ran for too long, so I stopped it.",
                BAD,
            )));
            l.push(Line::from(inline(
                "This usually means an **infinite loop**: a `loop` without a `break`, or a `while` whose condition never becomes false. Check that the loop variable actually changes.",
                Style::new(),
            )));
        }
        Outcome::ToolMissing(msg) => {
            mood = Mood::Worried;
            l.push(Line::from(Span::styled(
                "✘ I couldn't run the Rust compiler.",
                BAD,
            )));
            l.push(Line::from(msg.clone()));
            l.push(Line::from(inline("Install Rust with rustup (see the README), then make sure `rustc --version` works in a terminal.", Style::new())));
        }
    }
    Feedback { mood, lines: l }
}

fn show(s: Option<&&str>) -> String {
    match s {
        Some(s) => format!("{s:?}"),
        None => "(no line here)".into(),
    }
}

fn push_diag(l: &mut Vec<Line<'static>>, d: &Diag, friendly: &Friendly) {
    let place = match d.line {
        Some(n) if !d.in_tests => format!("line {n}"),
        Some(_) => "Pinch's hidden checks".into(),
        None => "somewhere".into(),
    };
    let code = d
        .code
        .as_deref()
        .map(|c| format!(" [{c}]"))
        .unwrap_or_default();
    l.push(Line::from(vec![
        Span::styled(format!("Error at {place}{code}: "), BAD),
        Span::styled(d.message.clone(), BOLD),
    ]));
    if let Some(e) = friendly.lookup(d.code.as_deref(), &d.message) {
        l.push(Line::from(vec![
            Span::styled("(\\/) ", PINCH),
            Span::styled(e.title.clone(), PINCH.add_modifier(Modifier::BOLD)),
        ]));
        l.push(Line::from(inline(
            &format!("  {}", e.explain),
            Style::new(),
        )));
        let mut fix = vec![Span::styled("  Try: ", GOOD)];
        fix.extend(inline(&e.fix, Style::new()));
        l.push(Line::from(fix));
    }
    l.push(Line::from(Span::styled("  What the compiler said:", DIM)));
    for r in d.rendered.lines().take(14) {
        l.push(Line::from(Span::styled(
            format!("  {r}"),
            Style::new().fg(Color::Gray),
        )));
    }
    if let Some(c) = &d.code {
        l.push(Line::from(Span::styled(
            format!("  (Curious? Run `rustc --explain {c}` in a terminal for the full story.)"),
            DIM,
        )));
    }
}

/// Plain text version, for the `check` command-line mode.
pub fn plain(f: &Feedback) -> String {
    f.lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
