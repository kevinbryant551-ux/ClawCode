//! Drawing every screen. All state lives in `app.rs`; this file only reads it.
use crate::app::{App, BossState, DailyView, LearnView, Overlay, QuizState, Screen, TITLE_MENU};
use crate::feedback::Mood;
use crate::lesson::Step;
use crate::markdown;
use crate::progress::{self, Progress, Reward};
use crate::shop;
use crate::widgets::{ACCENT, keys_line, line_text, popup, rounded, wrapped_height};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Wrap};

const GOLD: Color = Color::Yellow;
const DIM: Style = Style::new().fg(Color::DarkGray);
/// Characters of Pinch's message revealed per animation tick.
const TYPE_SPEED: u64 = 5;

// Block letters for the title logo (5 rows each).
const FONT: &[(char, [&str; 5])] = &[
    ('C', [" ████", "█    ", "█    ", "█    ", " ████"]),
    ('L', ["█    ", "█    ", "█    ", "█    ", "█████"]),
    ('A', [" ███ ", "█   █", "█████", "█   █", "█   █"]),
    ('W', ["█   █", "█   █", "█ █ █", "██ ██", "█   █"]),
    ('O', [" ███ ", "█   █", "█   █", "█   █", " ███ "]),
    ('D', ["████ ", "█   █", "█   █", "█   █", "████ "]),
    ('E', ["█████", "█    ", "████ ", "█    ", "█████"]),
];

const FACTS: &[&str] = &[
    "Rust's unofficial mascot is a crab, and its fans call themselves Rustaceans.",
    "The Rust compiler checks memory safety before your program ever runs.",
    "`cargo` builds your code, runs tests, and downloads libraries, all in one tool.",
    "Rust has no garbage collector: ownership decides when memory is freed.",
    "Compiler errors are free code review. Every error you fix makes you stronger!",
    "`rustc --explain E0382` prints a whole mini-lesson about that error code.",
    "Rust lets you write `1_000_000` so big numbers are easy to read.",
    "A `match` must handle every case, so you can't forget one.",
    "`Option` means there's no null to crash on in safe Rust.",
    "Variables are immutable by default. `mut` is you saying 'this one changes'.",
    "Firefox, Discord, Dropbox and the Linux kernel all use some Rust.",
    "Tiny daily practice beats one huge session. Keep that streak alive!",
];

impl App {
    pub fn draw(&mut self, f: &mut Frame) {
        let area = f.area();
        self.last_area = area;
        f.render_widget(Block::new().style(Style::new()), area);
        if area.width < 70 || area.height < 20 {
            f.render_widget(
                Paragraph::new(
                    "(\\/) Pinch needs a bigger window!\nPlease make your terminal at least 80x24.",
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
                area,
            );
            return;
        }
        match &self.screen {
            Screen::Title => self.draw_title(f, area),
            Screen::Map => self.draw_map(f, area),
            Screen::Learn(v) => self.draw_learn(f, area, v),
            Screen::Code(_) => self.draw_code(f, area),
            Screen::Daily(v) => self.draw_daily(f, area, v),
            Screen::Shop => self.draw_shop(f, area),
            Screen::Trophies => self.draw_trophies(f, area),
        }
        self.confetti.draw(f.buffer_mut(), area);
        match &self.overlay {
            Some(Overlay::Help) => draw_help(f, area, self.rustc.as_deref()),
            Some(Overlay::Celebrate {
                reward,
                next,
                title,
                note,
            }) => self.draw_celebrate(f, area, reward, *next, title, note),
            Some(Overlay::ConfirmReset) => popup(
                f,
                area,
                " Reset? ",
                Text::from(
                    "Replace your code with the original starter code?\n\n  y = yes, reset     any other key = cancel",
                ),
                56,
                7,
            ),
            Some(Overlay::Solution(code)) => {
                let mut t = Text::from(Line::from(Span::styled(
                    "Here's one way to solve it. Press C to copy it into your editor, any other key to close.",
                    Style::new().fg(Color::LightRed),
                )));
                t.lines.push(Line::default());
                for l in code.lines() {
                    t.lines
                        .push(Line::from(Span::styled(l.to_string(), markdown::CODE)));
                }
                let h = (code.lines().count() as u16 + 5).min(area.height - 2);
                popup(f, area, " Solution ", t, 90, h);
            }
            None => {}
        }
        self.draw_toast(f, area);
    }

    fn crab_color(&self) -> Color {
        shop::shell_color(self.progress.color.as_deref())
    }

    fn blink(&self) -> bool {
        self.tick % 70 < 3
    }

    fn pinch(
        &self,
        f: &mut Frame,
        area: Rect,
        mood: Mood,
        lines: Vec<Line<'static>>,
        scroll: u16,
        born: u64,
    ) {
        let lines = if born == 0 {
            lines
        } else {
            reveal(lines, (self.tick.saturating_sub(born)) * TYPE_SPEED)
        };
        draw_pinch(
            f,
            area,
            mood,
            lines,
            scroll,
            self.progress.hat.as_deref(),
            self.crab_color(),
            self.blink(),
        );
    }

    fn header(&self, title: Vec<Span<'static>>) -> Line<'static> {
        let p = &self.progress;
        let lvl = p.level();
        let (into, span) = progress::level_progress(p.xp);
        let bar_len = 10usize;
        let filled = if span == 0 {
            bar_len
        } else {
            (into as usize * bar_len) / span as usize
        };
        let mut spans = vec![Span::styled(
            " 🦀 ClawCode ",
            Style::new().fg(Color::Black).bg(self.crab_color()).bold(),
        )];
        spans.extend(title);
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Lv {lvl} {} ", progress::level_title(lvl)),
            Style::new().fg(GOLD).bold(),
        ));
        spans.push(Span::styled("█".repeat(filled), Style::new().fg(GOLD)));
        spans.push(Span::styled("░".repeat(bar_len - filled), DIM));
        spans.push(Span::raw(format!(" {} XP  ", p.xp)));
        spans.push(Span::styled(
            format!("◉ {} ", p.shells),
            Style::new().fg(Color::LightCyan).bold(),
        ));
        let streak = p.live_streak(self.today);
        spans.push(Span::styled(
            format!(" 🔥{streak}"),
            Style::new().fg(if streak > 0 {
                Color::LightRed
            } else {
                Color::DarkGray
            }),
        ));
        if p.combo >= 2 {
            spans.push(Span::styled(
                format!("  combo x{}", p.combo),
                Style::new().fg(Color::LightMagenta).bold(),
            ));
        }
        Line::from(spans)
    }

    // ---------------- title ----------------

    fn draw_title(&self, f: &mut Frame, area: Rect) {
        self.bubbles.draw(f.buffer_mut(), area);
        let mut lines: Vec<Line> = Vec::new();
        let palette = [
            Color::LightRed,
            Color::Rgb(255, 140, 60),
            GOLD,
            Color::LightCyan,
            Color::LightMagenta,
        ];
        for row in 0..5 {
            let mut spans = Vec::new();
            for (ci, ch) in "CLAWCODE".chars().enumerate() {
                let glyph = FONT
                    .iter()
                    .find(|(c, _)| *c == ch)
                    .map(|(_, g)| g[row])
                    .unwrap_or("     ");
                // A colour wave rolls across the letters; each letter bobs on its own beat.
                let wave = ((self.tick / 3) as usize + ci) % palette.len();
                spans.push(Span::styled(
                    format!("{glyph} "),
                    Style::new().fg(palette[wave]).bold(),
                ));
            }
            lines.push(Line::from(spans).centered());
        }
        lines.push(Line::default());
        lines.push(
            Line::from(Span::styled(
                "~ learn Rust, one pinch at a time ~",
                Style::new().fg(ACCENT).italic(),
            ))
            .centered(),
        );
        lines.push(Line::default());
        let mood = if self.tick % 160 < 80 {
            Mood::Happy
        } else {
            Mood::Party
        };
        for l in pinch_art(mood, self.progress.hat.as_deref(), self.blink()) {
            lines.push(
                Line::from(Span::styled(l, Style::new().fg(self.crab_color()).bold())).centered(),
            );
        }
        lines.push(Line::default());
        let daily_label = if self.daily_done_today() {
            "Daily Tide  ✔ done today"
        } else {
            "Daily Tide  ~ new! ~"
        };
        for (i, item) in TITLE_MENU.iter().enumerate() {
            let label = if i == 1 { daily_label } else { item };
            let sel = i == self.title_sel;
            let pointer = if sel {
                if self.tick % 10 < 5 { "▶ " } else { "▷ " }
            } else {
                "  "
            };
            let style = if sel {
                Style::new().fg(Color::Black).bg(GOLD).bold()
            } else {
                Style::new().fg(Color::Gray)
            };
            lines.push(
                Line::from(vec![
                    Span::styled(pointer, Style::new().fg(GOLD)),
                    Span::styled(format!(" {label:<24}"), style),
                ])
                .centered(),
            );
        }
        lines.push(Line::default());
        let p = &self.progress;
        lines.push(
            Line::from(Span::styled(
                format!(
                    "Lv {} {}  ·  {} XP  ·  ◉ {} shells  ·  🔥 {} day streak  ·  {}/{} lessons",
                    p.level(),
                    progress::level_title(p.level()),
                    p.xp,
                    p.shells,
                    p.live_streak(self.today),
                    self.done_count(),
                    self.lessons.len()
                ),
                Style::new().fg(Color::Gray),
            ))
            .centered(),
        );
        lines.push(Line::default());
        let fact = FACTS[(self.tick / 140) as usize % FACTS.len()];
        let mut fact_spans = vec![Span::styled(
            "Did you know? ",
            Style::new().fg(Color::LightRed).bold(),
        )];
        fact_spans.extend(markdown::inline(
            fact,
            Style::new().fg(Color::Gray).italic(),
        ));
        lines.push(Line::from(fact_spans).centered());

        let h = lines.len() as u16;
        let top = area.y + area.height.saturating_sub(h + 1) / 2;
        let r = Rect {
            x: area.x,
            y: top,
            width: area.width,
            height: h.min(area.height),
        };
        f.render_widget(Paragraph::new(lines), r);
        let foot = Rect {
            y: area.bottom() - 1,
            height: 1,
            ..area
        };
        f.render_widget(
            keys_line(&[
                ("↑↓", "choose"),
                ("Enter", "go"),
                ("d", "daily"),
                ("s", "shop"),
                ("t", "trophies"),
                ("q", "quit"),
            ]),
            foot,
        );
    }

    // ---------------- map ----------------

    fn draw_map(&mut self, f: &mut Frame, area: Rect) {
        let [top, body, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .areas(area);
        f.render_widget(
            Paragraph::new(self.header(vec![Span::styled(
                "  World Map",
                Style::new().fg(ACCENT).bold(),
            )])),
            top,
        );
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(46), Constraint::Percentage(54)])
                .areas(body);

        let mut items = Vec::new();
        for (row, entry) in self.map_rows.iter().enumerate() {
            match entry {
                None => {
                    let first = self.map_rows[row + 1..]
                        .iter()
                        .flatten()
                        .next()
                        .copied()
                        .unwrap_or(0);
                    let chapter = &self.lessons[first].chapter;
                    let in_ch: Vec<&crate::lesson::Lesson> = self
                        .lessons
                        .iter()
                        .filter(|l| &l.chapter == chapter)
                        .collect();
                    let done = in_ch
                        .iter()
                        .filter(|l| self.progress.is_done(&l.id))
                        .count();
                    let star = if done == in_ch.len() { " ★" } else { "" };
                    let style = if chapter.starts_with("Boss") || chapter.starts_with("Final") {
                        Style::new().fg(Color::LightRed).bold()
                    } else {
                        Style::new().fg(ACCENT).bold()
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(format!("── {chapter} "), style),
                        Span::styled(format!("{done}/{}{star}", in_ch.len()), DIM),
                    ])));
                }
                Some(i) => {
                    let l = &self.lessons[*i];
                    let boss = l.boss.is_some();
                    let (mark, style) = if self.progress.is_done(&l.id) {
                        ("✔", Style::new().fg(Color::Green))
                    } else if self.unlocked(*i) {
                        (
                            if boss { "☠" } else { "▶" },
                            Style::new()
                                .fg(if boss { Color::LightRed } else { GOLD })
                                .bold(),
                        )
                    } else {
                        (if boss { "☠" } else { "○" }, DIM)
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(format!("  {mark} "), style),
                        Span::styled(format!("{:>2}. {}", l.order, l.title), style),
                    ])));
                }
            }
        }
        let done = self.done_count();
        let list = List::new(items)
            .block(rounded(format!(
                " Lessons ({done}/{}) ",
                self.lessons.len()
            )))
            .highlight_style(
                Style::new()
                    .bg(Color::Rgb(40, 50, 70))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("›");
        f.render_stateful_widget(list, left, &mut self.map_state);

        let [pinch_area, detail_area] =
            Layout::vertical([Constraint::Length(8), Constraint::Min(5)]).areas(right);
        let idx = self.selected_lesson();
        let l = &self.lessons[idx];
        let left_n = self.lessons.len() - done;
        let greet = if done == self.lessons.len() {
            "You beat the whole course, Leviathan and all! Replay anything, do the Daily Tide, or show off in the shop.".to_string()
        } else if done == 0 {
            "Hi, I'm Pinch! Each lesson is a few short cards, a quick quiz, then real code. Pick lesson 1 and press Enter!".to_string()
        } else if let Some(b) = l
            .boss
            .as_ref()
            .filter(|_| self.unlocked(idx) && !self.progress.is_done(&l.id))
        {
            format!(
                "Careful... **{}** lurks here. Each hidden test you pass is a hit!",
                b.name
            )
        } else {
            format!(
                "{left_n} lesson{} to go. Don't forget today's **Daily Tide** (d)!",
                if left_n == 1 { "" } else { "s" }
            )
        };
        self.pinch(
            f,
            pinch_area,
            Mood::Happy,
            vec![Line::from(markdown::inline(&greet, Style::new()))],
            0,
            0,
        );

        let mut d: Vec<Line> = vec![
            Line::from(Span::styled(
                l.title.clone(),
                Style::new()
                    .fg(if l.boss.is_some() {
                        Color::LightRed
                    } else {
                        ACCENT
                    })
                    .bold(),
            )),
            Line::from(Span::styled(
                format!(
                    "Chapter: {}  ·  {} XP  ·  {} cards  ·  {} quizzes",
                    l.chapter,
                    l.xp,
                    l.cards().len(),
                    l.quiz.len()
                ),
                DIM,
            )),
            Line::default(),
        ];
        if let Some(b) = &l.boss {
            for a in b.art.trim_matches('\n').lines() {
                d.push(Line::from(Span::styled(
                    a.to_string(),
                    Style::new().fg(Color::LightRed),
                )));
            }
            d.push(Line::from(Span::styled(
                format!("HP {}", "♥".repeat(l.test_count())),
                Style::new().fg(Color::Red),
            )));
            d.push(Line::default());
        }
        let mut tags = vec![Span::raw("You'll learn: ")];
        for c in &l.concepts {
            tags.push(Span::styled(
                format!(" {c} "),
                Style::new().fg(Color::Black).bg(GOLD),
            ));
            tags.push(Span::raw(" "));
        }
        d.push(Line::from(tags));
        d.push(Line::default());
        d.push(Line::from(markdown::inline(
            &format!("Goal: {}", l.goal),
            Style::new(),
        )));
        d.push(Line::default());
        if let Some(c) = self.progress.completed.get(&l.id) {
            d.push(Line::from(Span::styled(
                format!(
                    "✔ Cleared! Earned {} XP in {} tr{}.",
                    c.xp_earned,
                    c.attempts,
                    if c.attempts == 1 { "y" } else { "ies" }
                ),
                Style::new().fg(Color::Green),
            )));
            d.push(Line::from("Enter replays it · l rereads the cards"));
        } else if self.unlocked(idx) {
            d.push(Line::from(Span::styled(
                if l.boss.is_some() {
                    "Press Enter to FIGHT!"
                } else {
                    "Press Enter to start!"
                },
                Style::new().fg(GOLD).bold(),
            )));
        } else {
            d.push(Line::from(Span::styled(
                "○ Locked: clear the lesson before this one to open it.",
                DIM,
            )));
        }
        f.render_widget(
            Paragraph::new(d)
                .wrap(Wrap { trim: false })
                .block(rounded(" Lesson ".into())),
            detail_area,
        );
        f.render_widget(
            keys_line(&[
                ("↑↓", "choose"),
                ("Enter", "start"),
                ("l", "reread"),
                ("d", "daily"),
                ("s", "shop"),
                ("t", "trophies"),
                ("Esc", "title"),
            ]),
            foot,
        );
    }

    // ---------------- learn ----------------

    fn draw_learn(&self, f: &mut Frame, area: Rect, v: &LearnView) {
        let l = &self.lessons[v.idx];
        let [top, prog, body, pinch, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(8),
            Constraint::Length(1),
        ])
        .areas(area);
        f.render_widget(
            Paragraph::new(self.header(vec![Span::styled(
                format!("  {} › {}", l.chapter, l.title),
                Style::new().fg(ACCENT).bold(),
            )])),
            top,
        );
        // Step dots: ● read, ◆ quiz, the current one highlighted.
        let mut dots = vec![Span::raw(" ")];
        for (i, s) in v.steps.iter().enumerate() {
            let (sym, base) = match s {
                Step::Card(_) => ("●", Color::Cyan),
                Step::Quiz(_) => ("◆", Color::LightMagenta),
            };
            let style = if i == v.pos {
                Style::new().fg(GOLD).bold()
            } else if i < v.pos {
                Style::new().fg(base)
            } else {
                DIM
            };
            dots.push(Span::styled(format!("{sym} "), style));
        }
        dots.push(Span::styled("  → code", DIM));
        f.render_widget(
            Paragraph::new(Line::from(dots)).alignment(Alignment::Center),
            prog,
        );

        match (v.steps.get(v.pos), &v.quiz) {
            (Some(Step::Quiz(_)), Some(q)) => draw_quiz(f, body, q, " ◆ Pop Quiz ", self.tick),
            (Some(Step::Card(c)), _) => {
                let cards = l.cards();
                let total = cards.len();
                let lines = markdown::render(cards[*c].trim_end());
                let card_area = fit_box(body, &lines, 1);
                f.render_widget(
                    Paragraph::new(lines)
                        .wrap(Wrap { trim: false })
                        .scroll((v.scroll, 0))
                        .block(
                            rounded(format!(" Card {} of {total} ", c + 1))
                                .border_style(Style::new().fg(ACCENT))
                                .title_bottom(
                                    Line::from(" → next  ← back  Tab: skip to code ")
                                        .right_aligned()
                                        .fg(Color::DarkGray),
                                )
                                .padding(ratatui::widgets::Padding::horizontal(1)),
                        ),
                    card_area,
                );
            }
            _ => {}
        }
        self.pinch(f, pinch, v.pinch.mood, v.pinch.lines.clone(), 0, v.born);
        f.render_widget(
            keys_line(&[
                ("→/Enter", "next"),
                ("←", "back"),
                ("1-4", "answer"),
                ("Tab", "skip to code"),
                ("Esc", "map"),
            ]),
            foot,
        );
    }

    // ---------------- code ----------------

    fn draw_code(&mut self, f: &mut Frame, area: Rect) {
        let Screen::Code(v) = &self.screen else {
            return;
        };
        let l = &self.lessons[v.idx];
        let header = self.header(vec![Span::styled(
            format!(
                "  {} › {}/{} {}",
                l.chapter,
                l.order,
                self.lessons.len(),
                l.title
            ),
            Style::new().fg(ACCENT).bold(),
        )]);
        let fb_height = (area.height / 3).clamp(8, 14);
        let [top, body, pinch, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(fb_height),
            Constraint::Length(1),
        ])
        .areas(area);
        f.render_widget(Paragraph::new(header), top);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(44), Constraint::Percentage(56)])
                .areas(body);

        // Top-left: the boss (if any) or the goal.
        let goal_text = format!("🎯 {}", l.goal);
        let goal_h =
            (wrapped_height(&goal_text, left.width.saturating_sub(2)) + 2).min(left.height / 2);
        let boss_h = l
            .boss
            .as_ref()
            .map_or(0, |b| b.art.trim_matches('\n').lines().count() as u16 + 4);
        let [boss_area, goal_area, intro_area] = Layout::vertical([
            Constraint::Length(boss_h.min(left.height / 2)),
            Constraint::Length(goal_h),
            Constraint::Min(3),
        ])
        .areas(left);
        if let (Some(b), Some(state)) = (&l.boss, &v.boss) {
            draw_boss(f, boss_area, b, state, self.tick);
        }
        f.render_widget(
            Paragraph::new(Line::from(markdown::inline(
                &goal_text,
                Style::new().bold(),
            )))
            .wrap(Wrap { trim: false })
            .block(rounded(" Your goal ".into()).border_style(Style::new().fg(GOLD))),
            goal_area,
        );

        let inner_w = intro_area.width.saturating_sub(2);
        let total: u16 = v
            .intro
            .iter()
            .map(|l| wrapped_height(&line_text(l), inner_w))
            .sum();
        let max_scroll = total.saturating_sub(intro_area.height.saturating_sub(2));
        let intro_scroll = v.intro_scroll.min(max_scroll);
        let more = if intro_scroll < max_scroll {
            " PgDn ↓ more "
        } else {
            ""
        };
        f.render_widget(
            Paragraph::new(v.intro.clone())
                .wrap(Wrap { trim: false })
                .scroll((intro_scroll, 0))
                .block(
                    rounded(" Lesson notes (Ctrl+L: cards) ".into())
                        .title_bottom(Line::from(more).right_aligned().fg(Color::DarkGray)),
                ),
            intro_area,
        );
        f.render_widget(&v.editor, right);

        let mut lines = v.feedback.lines.clone();
        if self.rustc.is_none() {
            lines.insert(0, Line::from(Span::styled("⚠ I can't find `rustc`! Install Rust with rustup (see README) so I can check your code.", Style::new().fg(Color::Red).bold())));
        }
        let inner_w = pinch.width.saturating_sub(18);
        let total: u16 = lines
            .iter()
            .map(|l| wrapped_height(&line_text(l), inner_w))
            .sum();
        let max_fb = total.saturating_sub(pinch.height.saturating_sub(2));
        let fb_scroll = v.fb_scroll.min(max_fb);
        let (mood, born) = (v.feedback.mood, v.born);
        self.pinch(f, pinch, mood, lines, fb_scroll, born);
        if max_fb > fb_scroll {
            let hint = Rect {
                x: pinch.x + pinch.width.saturating_sub(26),
                y: pinch.y + pinch.height - 1,
                width: 24,
                height: 1,
            };
            f.render_widget(
                Paragraph::new(" Ctrl+↓ / Alt+↓ more ").fg(Color::DarkGray),
                hint,
            );
        }
        let attack = if l.boss.is_some() { "ATTACK" } else { "run" };
        f.render_widget(
            keys_line(&[
                ("Ctrl+R", attack),
                ("Ctrl+T", "tip"),
                ("Ctrl+P", "solution"),
                ("Ctrl+L", "cards"),
                ("Ctrl+E", "$EDITOR"),
                ("Esc", "map"),
                ("F1", "help"),
            ]),
            foot,
        );
        // Keep scroll positions clamped for the next key press.
        if let Screen::Code(v) = &mut self.screen {
            v.intro_scroll = intro_scroll;
            v.fb_scroll = fb_scroll;
        }
    }

    // ---------------- daily ----------------

    fn draw_daily(&self, f: &mut Frame, area: Rect, v: &DailyView) {
        let [top, wave, body, pinch, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(6),
            Constraint::Length(8),
            Constraint::Length(1),
        ])
        .areas(area);
        f.render_widget(
            Paragraph::new(self.header(vec![Span::styled(
                "  🌊 Daily Tide",
                Style::new().fg(ACCENT).bold(),
            )])),
            top,
        );
        // A rolling wave whose height shows progress.
        let pct = v.pos as f32 / v.qs.len() as f32;
        let mut row = String::new();
        for x in 0..wave.width {
            let t = (x as f32 * 0.35 + self.tick as f32 * 0.25).sin();
            row.push(if t > 0.3 {
                '~'
            } else if t > -0.3 {
                '≈'
            } else {
                '-'
            });
        }
        let filled = (wave.width as f32 * pct) as usize;
        let (a, b) = row.split_at(row.char_indices().nth(filled).map_or(row.len(), |(i, _)| i));
        f.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(a.to_string(), Style::new().fg(Color::LightCyan)),
                    Span::styled(b.to_string(), Style::new().fg(Color::Rgb(40, 60, 90))),
                ]),
                Line::from(Span::styled(
                    format!(
                        "Question {} of {}  ·  {} right so far",
                        v.pos + 1,
                        v.qs.len(),
                        v.right
                    ),
                    DIM,
                ))
                .centered(),
            ]),
            wave,
        );
        draw_quiz(f, body, &v.quiz, " 🌊 Daily Tide ", self.tick);
        self.pinch(f, pinch, v.pinch.mood, v.pinch.lines.clone(), 0, v.born);
        f.render_widget(
            keys_line(&[
                ("↑↓", "choose"),
                ("1-4", "answer"),
                ("Enter", "confirm / next"),
                ("Esc", "leave"),
            ]),
            foot,
        );
    }

    // ---------------- shop ----------------

    fn draw_shop(&self, f: &mut Frame, area: Rect) {
        let [top, body, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .areas(area);
        f.render_widget(
            Paragraph::new(self.header(vec![Span::styled(
                "  ◉ Shell Shop",
                Style::new().fg(ACCENT).bold(),
            )])),
            top,
        );
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                .areas(body);
        let p = &self.progress;
        let mut items = Vec::new();
        for (i, it) in shop::ITEMS.iter().enumerate() {
            let owned = p.owned.contains(it.id) || it.price == 0;
            let equipped = p.hat.as_deref() == Some(it.id)
                || p.color.as_deref() == Some(it.id)
                || (it.id == "color-red" && p.color.is_none());
            let kind = match it.kind {
                shop::Kind::Hat => "hat  ",
                shop::Kind::Color => "shell",
            };
            let status = if equipped {
                Span::styled(
                    " WEARING ",
                    Style::new().fg(Color::Black).bg(Color::Green).bold(),
                )
            } else if owned {
                Span::styled(" owned ", Style::new().fg(Color::Green))
            } else if p.shells >= it.price {
                Span::styled(
                    format!(" ◉ {} ", it.price),
                    Style::new().fg(Color::LightCyan).bold(),
                )
            } else {
                Span::styled(format!(" ◉ {} ", it.price), DIM)
            };
            let sel = i == self.shop_sel;
            items.push(ListItem::new(Line::from(vec![
                Span::styled(if sel { "› " } else { "  " }, Style::new().fg(GOLD)),
                Span::styled(format!("{kind} "), DIM),
                Span::styled(
                    format!("{:<16}", it.name),
                    if sel {
                        Style::new().fg(GOLD).bold()
                    } else {
                        Style::new()
                    },
                ),
                status,
            ])));
        }
        f.render_widget(
            List::new(items).block(rounded(format!(
                " Items  ·  you have ◉ {} shells ",
                p.shells
            ))),
            left,
        );

        // Preview: Pinch wearing the selected item.
        let it = &shop::ITEMS[self.shop_sel];
        let (hat, color) = match it.kind {
            shop::Kind::Hat => (Some(it.id), shop::shell_color(p.color.as_deref())),
            shop::Kind::Color => (p.hat.as_deref(), shop::shell_color(Some(it.id))),
        };
        let mut lines = vec![Line::default()];
        for l in pinch_art(Mood::Party, hat, self.blink()) {
            lines.push(Line::from(Span::styled(l, Style::new().fg(color).bold())).centered());
        }
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(it.name, Style::new().fg(GOLD).bold())).centered());
        lines.push(Line::from(Span::styled(it.desc, Style::new().italic())).centered());
        lines.push(Line::default());
        lines.push(Line::from(Span::styled("Enter: buy / wear / take off", DIM)).centered());
        lines.push(Line::default());
        lines.push(
            Line::from(Span::styled(
                "Earn shells by clearing lessons, beating bosses,",
                DIM,
            ))
            .centered(),
        );
        lines.push(Line::from(Span::styled("quiz combos and the Daily Tide.", DIM)).centered());
        f.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(rounded(" Fitting room ".into())),
            right,
        );
        f.render_widget(
            keys_line(&[("↑↓", "browse"), ("Enter", "buy / wear"), ("Esc", "back")]),
            foot,
        );
    }

    // ---------------- trophies ----------------

    fn draw_trophies(&self, f: &mut Frame, area: Rect) {
        let [top, body, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .areas(area);
        f.render_widget(
            Paragraph::new(self.header(vec![Span::styled(
                "  ★ Trophy Room",
                Style::new().fg(ACCENT).bold(),
            )])),
            top,
        );
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)])
                .areas(body);
        let p = &self.progress;
        let mut t = Vec::new();
        for b in progress::BADGES {
            let have = p.badges.contains(b.id);
            t.push(Line::from(vec![
                Span::styled(
                    if have { " ★ " } else { " ☆ " },
                    Style::new().fg(if have { GOLD } else { Color::DarkGray }),
                ),
                Span::styled(
                    format!("{:<20}", b.name),
                    if have { Style::new().bold() } else { DIM },
                ),
                Span::styled(b.desc, DIM),
            ]));
        }
        f.render_widget(
            Paragraph::new(t)
                .wrap(Wrap { trim: false })
                .block(rounded(format!(
                    " Badges {}/{} ",
                    p.badges.len(),
                    progress::BADGES.len()
                ))),
            left,
        );
        let s = &p.stats;
        let accuracy = (s.quizzes_right * 100)
            .checked_div(s.quizzes_answered)
            .unwrap_or(0);
        let rows: Vec<(&str, String)> = vec![
            (
                "Level",
                format!("{} {}", p.level(), progress::level_title(p.level())),
            ),
            ("Total XP", p.xp.to_string()),
            (
                "Lessons cleared",
                format!("{}/{}", self.done_count(), self.lessons.len()),
            ),
            ("Bosses defeated", s.bosses_defeated.to_string()),
            ("Code runs", s.runs.to_string()),
            ("Compiler errors survived", s.compile_errors.to_string()),
            (
                "Quizzes right first try",
                format!("{}/{} ({accuracy}%)", s.quizzes_right, s.quizzes_answered),
            ),
            ("Best quiz combo", format!("x{}", p.best_combo)),
            (
                "Current streak",
                format!("{} days", p.live_streak(self.today)),
            ),
            ("Best streak", format!("{} days", p.best_streak)),
            ("Daily Tides", s.dailies.to_string()),
            ("Shells earned", s.shells_earned.to_string()),
            ("Shop items owned", p.owned.len().to_string()),
        ];
        let mut lines: Vec<Line> = pinch_art(Mood::Party, p.hat.as_deref(), self.blink())
            .into_iter()
            .map(|l| {
                Line::from(Span::styled(l, Style::new().fg(self.crab_color()).bold())).centered()
            })
            .collect();
        lines.push(Line::default());
        for (k, v) in rows {
            lines.push(Line::from(vec![
                Span::styled(format!(" {k:<26}"), DIM),
                Span::styled(v, Style::new().fg(GOLD).bold()),
            ]));
        }
        f.render_widget(
            Paragraph::new(lines).block(rounded(" Your stats ".into())),
            right,
        );
        f.render_widget(keys_line(&[("Esc", "back")]), foot);
    }

    // ---------------- overlays ----------------

    fn draw_celebrate(
        &self,
        f: &mut Frame,
        area: Rect,
        r: &Reward,
        next: Option<usize>,
        title: &str,
        note: &str,
    ) {
        let mut t = Text::default();
        let gold = Style::new().fg(GOLD).bold();
        let sparkle = ["\\o/", "\\o/", " o ", "/o\\"][(self.tick / 4) as usize % 4];
        t.lines.push(
            Line::from(Span::styled(
                format!("{sparkle}   {title}   {sparkle}"),
                gold,
            ))
            .centered(),
        );
        if !note.is_empty() {
            t.lines.push(
                Line::from(Span::styled(
                    note.to_string(),
                    Style::new().fg(Color::LightRed).italic(),
                ))
                .centered(),
            );
        }
        t.lines.push(Line::default());
        let mood = if self.tick % 8 < 4 {
            Mood::Party
        } else {
            Mood::Happy
        };
        for l in pinch_art(mood, self.progress.hat.as_deref(), false) {
            t.lines
                .push(Line::from(Span::styled(l, Style::new().fg(self.crab_color()))).centered());
        }
        t.lines.push(Line::default());
        if r.first_time {
            for (name, v) in &r.breakdown {
                let s = if *v >= 0 {
                    format!("+{v}")
                } else {
                    v.to_string()
                };
                t.lines.push(Line::from(vec![
                    Span::raw(format!("  {name:<26}")),
                    Span::styled(
                        format!("{s:>5} XP"),
                        if *v >= 0 {
                            Style::new().fg(Color::Green)
                        } else {
                            Style::new().fg(Color::Red)
                        },
                    ),
                ]));
            }
            t.lines.push(Line::from(vec![
                Span::styled(format!("  {:<26}", "Total"), Style::new().bold()),
                Span::styled(format!("{:>5} XP", format!("+{}", r.xp)), gold),
                Span::styled(
                    format!("   +{} shells", r.shells),
                    Style::new().fg(Color::LightCyan).bold(),
                ),
            ]));
        } else {
            t.lines.push(Line::from(
                "  You'd already cleared this one. Nice practice! (no extra XP)",
            ));
        }
        if let Some(lvl) = r.leveled_up {
            t.lines.push(Line::default());
            t.lines.push(Line::from(Span::styled(
                format!(
                    "  ⬆ LEVEL UP! You are now level {lvl}: {}",
                    progress::level_title(lvl)
                ),
                gold,
            )));
        }
        for b in &r.new_badges {
            t.lines.push(Line::from(vec![
                Span::styled("  ★ New badge: ", Style::new().fg(Color::Magenta).bold()),
                Span::styled(b.name, Style::new().bold()),
                Span::raw(format!(" ({})", b.desc)),
            ]));
        }
        if r.streak > 1 {
            t.lines.push(Line::from(Span::styled(
                format!("  🔥 Streak: {} days in a row!", r.streak),
                Style::new().fg(Color::LightRed),
            )));
        }
        t.lines.push(Line::default());
        let footer = match next {
            Some(i) if i < self.lessons.len() => format!(
                "Enter: next ({})  ·  m: map  ·  Esc: stay",
                self.lessons[i].title
            ),
            Some(_) => "That was the last lesson! Enter: back to the map".to_string(),
            None => "Press any key".to_string(),
        };
        t.lines
            .push(Line::from(Span::styled(footer, Style::new().fg(ACCENT))).centered());
        let h = t.lines.len() as u16 + 2;
        popup(f, area, " 🎉 ", t, 84, h);
    }

    fn draw_toast(&self, f: &mut Frame, area: Rect) {
        let Some((lines, _)) = &self.toast else {
            return;
        };
        let w = lines
            .iter()
            .map(|l| line_text(l).chars().count() as u16)
            .max()
            .unwrap_or(10)
            + 4;
        let h = lines.len() as u16 + 2;
        let r = Rect {
            x: area.right().saturating_sub(w + 1),
            y: area.y + 1,
            width: w.min(area.width),
            height: h,
        };
        f.render_widget(Clear, r);
        f.render_widget(
            Paragraph::new(lines.clone()).block(
                rounded(String::new())
                    .border_style(Style::new().fg(GOLD))
                    .border_type(BorderType::Thick),
            ),
            r,
        );
    }
}

// ---------------- free drawing helpers ----------------

/// A box `w` wide, as tall as `lines` need (plus `extra`), centred inside `area`.
fn fit_box(area: Rect, lines: &[Line], extra: u16) -> Rect {
    let w = area.width.min(96);
    let inner = w.saturating_sub(4);
    let h: u16 = lines
        .iter()
        .map(|l| wrapped_height(&line_text(l), inner))
        .sum::<u16>()
        + 2
        + extra;
    let h = h.clamp(8, area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 3,
        width: w,
        height: h,
    }
}

/// Shows only the first `n` characters of `lines` (Pinch's typewriter effect).
fn reveal(lines: Vec<Line<'static>>, mut n: u64) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    for line in lines {
        if n == 0 {
            break;
        }
        let mut spans = Vec::new();
        for s in line.spans {
            let len = s.content.chars().count() as u64;
            if n >= len {
                n -= len;
                spans.push(s);
            } else {
                let part: String = s.content.chars().take(n as usize).collect();
                spans.push(Span::styled(part, s.style));
                n = 0;
                break;
            }
        }
        out.push(Line::from(spans));
        n = n.saturating_sub(1);
    }
    out
}

pub fn pinch_art(mood: Mood, hat: Option<&str>, blink: bool) -> Vec<String> {
    let eyes = if blink {
        "-   -"
    } else {
        match mood {
            Mood::Happy => "^   ^",
            Mood::Thinking => "o   O",
            Mood::Worried => "°   °",
            Mood::Party => "★   ★",
            Mood::Sleepy => "-   -",
        }
    };
    let mouth = match mood {
        Mood::Happy | Mood::Party => "\\___/",
        Mood::Thinking => " ~~~ ",
        Mood::Worried => " ___ ",
        Mood::Sleepy => " zZz ",
    };
    let mut v: Vec<String> = shop::hat_art(hat).iter().map(|s| s.to_string()).collect();
    v.extend([
        r" (\/)   (\/) ".to_string(),
        format!("  \\ {eyes} /  "),
        format!("  ( {mouth} )  "),
        r"  /|\   /|\  ".to_string(),
    ]);
    v
}

#[allow(clippy::too_many_arguments)]
fn draw_pinch(
    f: &mut Frame,
    area: Rect,
    mood: Mood,
    lines: Vec<Line<'static>>,
    scroll: u16,
    hat: Option<&str>,
    color: Color,
    blink: bool,
) {
    let block = rounded(" Pinch ".into()).border_style(Style::new().fg(color));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let [art, text] =
        Layout::horizontal([Constraint::Length(15), Constraint::Min(10)]).areas(inner);
    let art_lines: Vec<Line> = pinch_art(mood, hat, blink)
        .into_iter()
        .map(|l| Line::from(Span::styled(l, Style::new().fg(color).bold())))
        .collect();
    f.render_widget(Paragraph::new(art_lines), art);
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        text,
    );
}

fn draw_quiz(f: &mut Frame, area: Rect, q: &QuizState, title: &str, tick: u64) {
    let mut lines = markdown::render(q.quiz.question.trim());
    lines.push(Line::default());
    for (i, c) in q.quiz.choices.iter().enumerate() {
        let is_sel = i == q.sel;
        let (mark, style) = if q.solved && i == q.quiz.answer {
            ("✔", Style::new().fg(Color::Black).bg(Color::Green).bold())
        } else if q.wrong.contains(&i) {
            ("✘", Style::new().fg(Color::Red).crossed_out())
        } else if is_sel && !q.solved {
            (
                if tick % 10 < 5 { "▶" } else { "▷" },
                Style::new().fg(Color::Black).bg(GOLD).bold(),
            )
        } else {
            (" ", Style::new())
        };
        let mut spans = vec![Span::styled(format!(" {mark} {}. ", i + 1), style)];
        for s in markdown::inline(c, Style::new()) {
            spans.push(Span::styled(s.content, style.patch(s.style)));
        }
        spans.push(Span::styled(" ", style));
        lines.push(Line::from(spans));
        lines.push(Line::default());
    }
    let area = fit_box(area, &lines, 0);
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            rounded(title.to_string())
                .border_style(Style::new().fg(Color::LightMagenta))
                .padding(ratatui::widgets::Padding::horizontal(1)),
        ),
        area,
    );
}

fn draw_boss(f: &mut Frame, area: Rect, b: &crate::lesson::Boss, s: &BossState, tick: u64) {
    let shaking = tick < s.shake_until;
    let dx = if shaking {
        [0u16, 2, 0, 1][(tick % 4) as usize]
    } else {
        0
    };
    let defeated = s.hp == 0 && s.shown_hp < 0.05;
    let color = if defeated {
        Color::DarkGray
    } else if shaking && tick.is_multiple_of(2) {
        Color::White
    } else {
        Color::LightRed
    };
    let mut lines: Vec<Line> = b
        .art
        .trim_matches('\n')
        .lines()
        .map(|l| {
            Line::from(Span::styled(
                format!("{}{l}", " ".repeat(dx as usize)),
                Style::new().fg(color),
            ))
        })
        .collect();
    let bar_w = area.width.saturating_sub(12) as f32;
    let frac = if s.hp_max == 0 {
        0.0
    } else {
        s.shown_hp / s.hp_max as f32
    };
    let filled = (bar_w * frac).round() as usize;
    lines.push(Line::from(vec![
        Span::styled(" HP ", Style::new().fg(Color::White).bold()),
        Span::styled(
            "█".repeat(filled),
            Style::new().fg(if frac > 0.5 {
                Color::Red
            } else {
                Color::LightRed
            }),
        ),
        Span::styled("░".repeat(bar_w as usize - filled.min(bar_w as usize)), DIM),
        Span::styled(format!(" {}/{}", s.hp, s.hp_max), Style::new().bold()),
    ]));
    let title = if defeated {
        format!(" ☠ {} (defeated) ", b.name)
    } else {
        format!(" ☠ BOSS: {} ", b.name)
    };
    f.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Thick)
                .border_style(Style::new().fg(if defeated {
                    Color::DarkGray
                } else {
                    Color::Red
                }))
                .title(title),
        ),
        area,
    );
}

fn draw_help(f: &mut Frame, area: Rect, rustc: Option<&str>) {
    let rows: &[(&str, &str)] = &[
        ("Reading cards", ""),
        ("→ / Enter, ←", "Next card, previous card"),
        (
            "1-4 or ↑↓ + Enter",
            "Answer a pop quiz (first-try answers build your combo!)",
        ),
        ("Tab", "Skip straight to the code"),
        ("Coding", ""),
        (
            "Ctrl+R  or F5",
            "Run & check your code (in boss fights: ATTACK)",
        ),
        (
            "Ctrl+T  or F2",
            "Get a tip (each new tip costs a bit of XP)",
        ),
        (
            "Ctrl+P  or F4",
            "Show the solution (after 3 tries or all tips)",
        ),
        ("Ctrl+L  or F3", "Reread the lesson cards"),
        ("Ctrl+E", "Open your code in your own editor ($EDITOR)"),
        ("Ctrl+S / Ctrl+O", "Save / reset to the starter code"),
        ("Ctrl+Z / Ctrl+Y", "Undo / redo"),
        (
            "PgUp/PgDn, Alt+↑↓",
            "Scroll lesson notes, scroll Pinch's messages",
        ),
        ("Esc / Ctrl+C", "Back / quit (progress is always saved)"),
    ];
    let mut t = Text::default();
    for (k, d) in rows {
        if d.is_empty() {
            t.lines.push(Line::from(Span::styled(
                *k,
                Style::new().fg(ACCENT).bold().underlined(),
            )));
        } else {
            t.lines.push(Line::from(vec![
                Span::styled(format!("{k:<20}"), Style::new().fg(GOLD).bold()),
                Span::raw(*d),
            ]));
        }
    }
    t.lines.push(Line::default());
    t.lines.push(Line::from(Span::styled(
        "Earn XP and ◉ shells from lessons, bosses, quizzes and the Daily Tide.",
        DIM,
    )));
    t.lines.push(Line::from(Span::styled(
        "Spend shells in the Shell Shop. Keep a daily 🔥 streak for bonus XP.",
        DIM,
    )));
    t.lines.push(Line::from(Span::styled(
        format!("Progress file: {}", Progress::path().display()),
        DIM,
    )));
    t.lines.push(Line::from(Span::styled(
        format!("Compiler: {}", rustc.unwrap_or("NOT FOUND")),
        DIM,
    )));
    popup(f, area, " How to play (any key closes) ", t, 84, 24);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_cuts_text() {
        let lines = vec![Line::from("hello"), Line::from("world")];
        let r = reveal(lines.clone(), 3);
        assert_eq!(r.len(), 1);
        assert_eq!(line_text(&r[0]), "hel");
        assert_eq!(reveal(lines, 100).len(), 2);
    }

    #[test]
    fn hats_fit() {
        for it in shop::ITEMS {
            for l in shop::hat_art(Some(it.id)) {
                assert!(l.chars().count() <= 13, "{} hat line too wide", it.id);
            }
        }
    }
}
