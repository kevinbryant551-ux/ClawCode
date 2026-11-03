//! The interactive terminal app: the map screen and the lesson screen.
use crate::checker;
use crate::feedback::{self, Feedback, Mood};
use crate::friendly::Friendly;
use crate::lesson::Lesson;
use crate::markdown;
use crate::progress::{self, Progress, Reward};
use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::{DefaultTerminal, Frame};
use tui_textarea::{CursorMove, TextArea};

const ACCENT: Color = Color::Cyan;
const CRAB: Color = Color::LightRed;
/// Tries before the "show solution" key works.
const TRIES_BEFORE_SOLUTION: u32 = 3;

pub struct App {
    lessons: Vec<Lesson>,
    progress: Progress,
    friendly: Friendly,
    unlock_all: bool,
    screen: Screen,
    overlay: Option<Overlay>,
    map_state: ListState,
    map_rows: Vec<Option<usize>>, // list row -> lesson index (None = chapter header)
    quit: bool,
    rustc: Option<String>,
}

enum Screen {
    Map,
    Lesson(Box<LessonView>),
}

struct LessonView {
    idx: usize,
    editor: TextArea<'static>,
    intro: Vec<Line<'static>>,
    intro_scroll: u16,
    feedback: Feedback,
    fb_scroll: u16,
}

enum Overlay {
    Help,
    Badges,
    Celebrate { reward: Reward, idx: usize },
    ConfirmReset,
    Solution(String),
}

impl App {
    pub fn new(lessons: Vec<Lesson>, unlock_all: bool) -> App {
        let progress = Progress::load();
        let friendly = Friendly::load(crate::lesson::ERROR_CATALOG);
        let mut app = App {
            lessons,
            progress,
            friendly,
            unlock_all,
            screen: Screen::Map,
            overlay: None,
            map_state: ListState::default(),
            map_rows: Vec::new(),
            quit: false,
            rustc: checker::rustc_version(),
        };
        app.build_map_rows();
        // Start the cursor on the first lesson that isn't finished yet.
        let first_open = (0..app.lessons.len())
            .find(|&i| !app.progress.is_done(&app.lessons[i].id))
            .unwrap_or(0);
        app.select_lesson(first_open);
        app
    }

    fn build_map_rows(&mut self) {
        self.map_rows.clear();
        let mut chapter = "";
        for (i, l) in self.lessons.iter().enumerate() {
            if l.chapter != chapter {
                chapter = &l.chapter;
                self.map_rows.push(None);
            }
            self.map_rows.push(Some(i));
        }
    }

    fn select_lesson(&mut self, idx: usize) {
        let row = self
            .map_rows
            .iter()
            .position(|r| *r == Some(idx))
            .unwrap_or(1);
        self.map_state.select(Some(row));
    }

    fn selected_lesson(&self) -> usize {
        self.map_state
            .selected()
            .and_then(|r| self.map_rows.get(r).copied().flatten())
            .unwrap_or(0)
    }

    fn unlocked(&self, idx: usize) -> bool {
        self.unlock_all
            || idx == 0
            || self.progress.is_done(&self.lessons[idx].id)
            || self.progress.is_done(&self.lessons[idx - 1].id)
    }

    fn save(&mut self) {
        if let Screen::Lesson(v) = &self.screen {
            let code = v.editor.lines().join("\n");
            let id = self.lessons[v.idx].id.clone();
            if code.trim() != self.lessons[v.idx].starter.trim() {
                self.progress.drafts.insert(id, code);
            } else {
                self.progress.drafts.remove(&id);
            }
        }
        let _ = self.progress.save();
    }

    // ---------------- main loop ----------------

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            terminal.draw(|f| self.draw(f))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.on_key(key, terminal)?;
            }
        }
        self.save();
        Ok(())
    }

    fn on_key(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            self.quit = true;
            return Ok(());
        }
        if let Some(ov) = self.overlay.take() {
            return self.on_overlay_key(ov, key);
        }
        match self.screen {
            Screen::Map => self.on_map_key(key),
            Screen::Lesson(_) => self.on_lesson_key(key, terminal)?,
        }
        Ok(())
    }

    fn on_overlay_key(&mut self, ov: Overlay, key: KeyEvent) -> Result<()> {
        match ov {
            Overlay::Celebrate { idx, .. } => match key.code {
                KeyCode::Enter | KeyCode::Char('n') => {
                    if idx + 1 < self.lessons.len() {
                        self.open_lesson(idx + 1);
                    } else {
                        self.screen = Screen::Map;
                    }
                }
                KeyCode::Char('m') => {
                    self.screen = Screen::Map;
                    self.select_lesson((idx + 1).min(self.lessons.len() - 1));
                }
                _ => {} // Esc or anything else: stay on this lesson
            },
            Overlay::ConfirmReset => {
                if matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y')) {
                    if let Screen::Lesson(v) = &mut self.screen {
                        let starter = self.lessons[v.idx].starter.clone();
                        v.editor = make_editor(&starter);
                        v.feedback =
                            Feedback::say(Mood::Happy, "Fresh start! The starter code is back.");
                    }
                    self.save();
                }
            }
            Overlay::Solution(code) => {
                if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
                    && let Screen::Lesson(v) = &mut self.screen
                {
                    v.editor = make_editor(&code);
                    v.feedback = Feedback::say(
                        Mood::Happy,
                        "I copied the solution into your editor. Read it line by line, then press **Ctrl+R** to run it.\nTip: try changing something small and run again to see what breaks!",
                    );
                }
            }
            Overlay::Help | Overlay::Badges => {}
        }
        Ok(())
    }

    fn on_map_key(&mut self, key: KeyEvent) {
        let rows = self.map_rows.len();
        let mut row = self.map_state.selected().unwrap_or(1);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Up | KeyCode::Char('k') => loop {
                if row == 0 {
                    break;
                }
                row -= 1;
                if self.map_rows[row].is_some() {
                    self.map_state.select(Some(row));
                    break;
                }
            },
            KeyCode::Down | KeyCode::Char('j') => {
                while row + 1 < rows {
                    row += 1;
                    if self.map_rows[row].is_some() {
                        self.map_state.select(Some(row));
                        break;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let idx = self.selected_lesson();
                if self.unlocked(idx) {
                    self.open_lesson(idx);
                }
            }
            KeyCode::Char('b') => self.overlay = Some(Overlay::Badges),
            KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Char('h') => {
                self.overlay = Some(Overlay::Help)
            }
            _ => {}
        }
    }

    fn open_lesson(&mut self, idx: usize) {
        self.save();
        let lesson = &self.lessons[idx];
        let code = self
            .progress
            .drafts
            .get(&lesson.id)
            .cloned()
            .unwrap_or_else(|| lesson.starter.clone());
        let greeting = if self.progress.is_done(&lesson.id) {
            format!(
                "Welcome back to **{}**! You've already cleared this one, so practice freely.",
                lesson.title
            )
        } else if self.progress.drafts.contains_key(&lesson.id) {
            "I kept your code from last time. Pick up where you left off!".to_string()
        } else {
            "Read the lesson on the left, then edit the code on the right.\nWhen you're ready, press **Ctrl+R** to run it. Stuck? **Ctrl+T** for a tip. (**F1** shows all keys.)".to_string()
        };
        let mood = if self.progress.is_done(&lesson.id) {
            Mood::Party
        } else {
            Mood::Happy
        };
        self.screen = Screen::Lesson(Box::new(LessonView {
            idx,
            editor: make_editor(&code),
            intro: markdown::render(&lesson.intro),
            intro_scroll: 0,
            feedback: Feedback::say(mood, &greeting),
            fb_scroll: 0,
        }));
        self.select_lesson(idx);
    }

    fn on_lesson_key(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let Screen::Lesson(v) = &mut self.screen else {
            return Ok(());
        };
        match (key.code, ctrl) {
            (KeyCode::Esc, _) => {
                self.save();
                self.screen = Screen::Map;
            }
            (KeyCode::Char('r'), true) | (KeyCode::F(5), _) => self.run_check(terminal)?,
            (KeyCode::Char('t'), true) | (KeyCode::F(2), _) => self.next_hint(),
            (KeyCode::Char('s'), true) => {
                self.save();
                if let Screen::Lesson(v) = &mut self.screen {
                    v.feedback =
                        Feedback::say(Mood::Happy, "Saved! Your code is safe even if you quit.");
                }
            }
            (KeyCode::Char('e'), true) => self.external_editor(terminal)?,
            (KeyCode::Char('p'), true) | (KeyCode::F(4), _) => self.peek_solution(),
            (KeyCode::Char('o'), true) | (KeyCode::F(8), _) => {
                self.overlay = Some(Overlay::ConfirmReset)
            }
            (KeyCode::F(1), _) | (KeyCode::Char('g'), true) => self.overlay = Some(Overlay::Help),
            (KeyCode::PageDown, _) if !alt => v.intro_scroll = v.intro_scroll.saturating_add(5),
            (KeyCode::PageUp, _) if !alt => v.intro_scroll = v.intro_scroll.saturating_sub(5),
            (KeyCode::Down, true) | (KeyCode::PageDown, _) => {
                v.fb_scroll = v.fb_scroll.saturating_add(4)
            }
            (KeyCode::Up, true) | (KeyCode::PageUp, _) => {
                v.fb_scroll = v.fb_scroll.saturating_sub(4)
            }
            (KeyCode::Down, false) if alt => v.fb_scroll = v.fb_scroll.saturating_add(2),
            (KeyCode::Up, false) if alt => v.fb_scroll = v.fb_scroll.saturating_sub(2),
            _ => editor_input(&mut v.editor, key),
        }
        Ok(())
    }

    fn run_check(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let Screen::Lesson(v) = &mut self.screen else {
            return Ok(());
        };
        let idx = v.idx;
        let code = v.editor.lines().join("\n") + "\n";
        v.feedback = Feedback::say(
            Mood::Thinking,
            "Compiling your code with the real Rust compiler... 🦀",
        );
        v.fb_scroll = 0;
        terminal.draw(|f| self.draw(f))?;

        let lesson = self.lessons[idx].clone();
        let outcome = checker::check(&lesson, &code);
        let already = self.progress.is_done(&lesson.id);
        let attempt = {
            let a = self.progress.attempts.entry(lesson.id.clone()).or_insert(0);
            if !already {
                *a += 1;
            }
            *a
        };
        let fb = feedback::describe(&outcome, &lesson, &self.friendly, attempt);
        if let Screen::Lesson(v) = &mut self.screen {
            v.feedback = fb;
        }
        if outcome.passed() {
            let reward = self.progress.record_pass(
                &lesson.id,
                lesson.xp,
                self.lessons.len(),
                progress::today(),
            );
            // Keep the passing code as the draft so the learner can revisit it.
            self.progress
                .drafts
                .insert(lesson.id.clone(), code.trim_end().to_string());
            self.overlay = Some(Overlay::Celebrate { reward, idx });
        } else if !already
            && attempt == TRIES_BEFORE_SOLUTION
            && let Screen::Lesson(v) = &mut self.screen
        {
            v.feedback.lines.push(Line::default());
            v.feedback.lines.push(Line::from(Span::styled(
                    "(If you're really stuck, Ctrl+P now shows the solution. It costs some XP, and that's okay!)",
                    Style::new().fg(Color::DarkGray),
                )));
        }
        let _ = self.progress.save();
        Ok(())
    }

    fn next_hint(&mut self) {
        let Screen::Lesson(v) = &mut self.screen else {
            return;
        };
        let lesson = &self.lessons[v.idx];
        if lesson.hints.is_empty() {
            v.feedback = Feedback::say(
                Mood::Thinking,
                "This lesson has no hints. Reread the goal at the top left!",
            );
            return;
        }
        let used = self
            .progress
            .hints_used
            .entry(lesson.id.clone())
            .or_insert(0);
        let done = self.progress.completed.contains_key(&lesson.id);
        if (*used as usize) < lesson.hints.len() && !done {
            *used += 1;
        }
        let shown = (*used as usize).clamp(1, lesson.hints.len());
        let mut lines = vec![Line::from(Span::styled(
            format!(
                "Tip {shown} of {} (each new tip costs a little XP):",
                lesson.hints.len()
            ),
            Style::new().bold(),
        ))];
        for (i, h) in lesson
            .hints
            .iter()
            .take(if done { lesson.hints.len() } else { shown })
            .enumerate()
        {
            let mut spans = vec![Span::styled(format!("{}. ", i + 1), Style::new().fg(CRAB))];
            spans.extend(markdown::inline(h, Style::new()));
            lines.push(Line::from(spans));
        }
        if shown == lesson.hints.len() {
            lines.push(Line::from(Span::styled(
                "That was my last tip! Ctrl+P can show the full solution.",
                Style::new().fg(Color::DarkGray),
            )));
        }
        v.feedback = Feedback {
            mood: Mood::Thinking,
            lines,
        };
        v.fb_scroll = 0;
        let _ = self.progress.save();
    }

    fn peek_solution(&mut self) {
        let Screen::Lesson(v) = &mut self.screen else {
            return;
        };
        let lesson = &self.lessons[v.idx];
        let attempts = *self.progress.attempts.get(&lesson.id).unwrap_or(&0);
        let hints = *self.progress.hints_used.get(&lesson.id).unwrap_or(&0) as usize;
        let done = self.progress.is_done(&lesson.id);
        if done || attempts >= TRIES_BEFORE_SOLUTION || hints >= lesson.hints.len() {
            if !done {
                self.progress.solution_seen.insert(lesson.id.clone());
                let _ = self.progress.save();
            }
            self.overlay = Some(Overlay::Solution(lesson.solution.clone()));
        } else {
            let left = TRIES_BEFORE_SOLUTION - attempts;
            v.feedback = Feedback::say(
                Mood::Happy,
                &format!(
                    "Not yet! Give it {left} more {} first (or use all the tips with Ctrl+T). Struggling a little is how it sticks. You've got this!",
                    if left == 1 { "try" } else { "tries" }
                ),
            );
        }
    }

    /// Opens the code in $VISUAL / $EDITOR (falls back to nano, then vi).
    fn external_editor(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let Screen::Lesson(v) = &mut self.screen else {
            return Ok(());
        };
        let path = std::env::temp_dir().join(format!("clawcode-{}.rs", self.lessons[v.idx].id));
        std::fs::write(&path, v.editor.lines().join("\n") + "\n")?;
        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| {
                if which("nano") {
                    "nano".into()
                } else {
                    "vi".into()
                }
            });
        ratatui::restore();
        let mut parts = editor.split_whitespace();
        let status = std::process::Command::new(parts.next().unwrap_or("vi"))
            .args(parts)
            .arg(&path)
            .status();
        *terminal = ratatui::init();
        let msg = match status {
            Ok(_) => {
                let code = std::fs::read_to_string(&path).unwrap_or_default();
                v.editor = make_editor(code.trim_end());
                "Welcome back! I loaded your edits. Press **Ctrl+R** to run.".to_string()
            }
            Err(e) => format!(
                "I couldn't start the editor `{editor}`: {e}. Set the EDITOR variable, e.g. `export EDITOR=nano`."
            ),
        };
        let _ = std::fs::remove_file(&path);
        v.feedback = Feedback::say(Mood::Happy, &msg);
        Ok(())
    }

    // ---------------- drawing ----------------

    fn draw(&mut self, f: &mut Frame) {
        let area = f.area();
        f.render_widget(Block::new().style(Style::new()), area);
        if area.width < 60 || area.height < 18 {
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
            Screen::Map => self.draw_map(f, area),
            Screen::Lesson(_) => self.draw_lesson(f, area),
        }
        match &self.overlay {
            Some(Overlay::Help) => self.draw_help(f, area),
            Some(Overlay::Badges) => self.draw_badges(f, area),
            Some(Overlay::Celebrate { reward, idx }) => self.draw_celebrate(f, area, reward, *idx),
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
                    Style::new().fg(CRAB),
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
    }

    fn header(&self, title: Line<'static>) -> Line<'static> {
        let lvl = self.progress.level();
        let (into, span) = progress::level_progress(self.progress.xp);
        let bar_len = 10usize;
        let filled = if span == 0 {
            bar_len
        } else {
            (into as usize * bar_len) / span as usize
        };
        let mut spans = title.spans;
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            format!("Lv {lvl} {} ", progress::level_title(lvl)),
            Style::new().fg(Color::Yellow).bold(),
        ));
        spans.push(Span::styled(
            "█".repeat(filled),
            Style::new().fg(Color::Yellow),
        ));
        spans.push(Span::styled(
            "░".repeat(bar_len - filled),
            Style::new().fg(Color::DarkGray),
        ));
        spans.push(Span::raw(format!(" {} XP", self.progress.xp)));
        if span > 0 {
            spans.push(Span::styled(
                format!(" ({} to next)", span - into),
                Style::new().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            format!(
                "streak {} day{}",
                self.progress.streak,
                if self.progress.streak == 1 { "" } else { "s" }
            ),
            Style::new().fg(Color::LightRed),
        ));
        Line::from(spans)
    }

    fn draw_map(&mut self, f: &mut Frame, area: Rect) {
        let [top, body, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .areas(area);
        f.render_widget(
            Paragraph::new(self.header(Line::from(Span::styled(
                " 🦀 ClawCode ",
                Style::new().fg(Color::Black).bg(CRAB).bold(),
            )))),
            top,
        );

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .areas(body);

        let mut items = Vec::new();
        for row in &self.map_rows {
            match row {
                None => {
                    let next = self
                        .map_rows
                        .iter()
                        .skip(items.len() + 1)
                        .flatten()
                        .next()
                        .copied()
                        .unwrap_or(0);
                    let chapter = &self.lessons[next].chapter;
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("── {chapter} "),
                        Style::new().fg(ACCENT).bold(),
                    ))));
                }
                Some(i) => {
                    let l = &self.lessons[*i];
                    let (mark, style) = if self.progress.is_done(&l.id) {
                        ("✔", Style::new().fg(Color::Green))
                    } else if self.unlocked(*i) {
                        ("▶", Style::new().fg(Color::Yellow).bold())
                    } else {
                        ("○", Style::new().fg(Color::DarkGray))
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(format!("  {mark} "), style),
                        Span::styled(format!("{:>2}. {}", l.order, l.title), style),
                    ])));
                }
            }
        }
        let done = self
            .lessons
            .iter()
            .filter(|l| self.progress.is_done(&l.id))
            .count();
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

        // Right side: Pinch + details of the selected lesson.
        let [pinch_area, detail_area] =
            Layout::vertical([Constraint::Length(7), Constraint::Min(5)]).areas(right);
        let idx = self.selected_lesson();
        let l = &self.lessons[idx];
        let greet = if done == self.lessons.len() {
            "You finished the whole course! Replay any lesson to practice. Proud of you!"
                .to_string()
        } else if done == 0 {
            "Hi, I'm Pinch! I'll teach you Rust one small step at a time. Pick lesson 1 and press Enter!".to_string()
        } else {
            format!(
                "Welcome back! {} lesson{} left. Let's keep that streak going!",
                self.lessons.len() - done,
                if self.lessons.len() - done == 1 {
                    ""
                } else {
                    "s"
                }
            )
        };
        draw_pinch(
            f,
            pinch_area,
            Mood::Happy,
            vec![Line::from(markdown::inline(&greet, Style::new()))],
            0,
        );

        let mut d: Vec<Line> = vec![
            Line::from(Span::styled(
                l.title.clone(),
                Style::new().fg(ACCENT).bold(),
            )),
            Line::from(Span::styled(
                format!("Chapter: {}   ·   Worth {} XP", l.chapter, l.xp),
                Style::new().fg(Color::DarkGray),
            )),
            Line::default(),
        ];
        let mut tags = vec![Span::raw("You'll learn: ")];
        for c in &l.concepts {
            tags.push(Span::styled(
                format!(" {c} "),
                Style::new().fg(Color::Black).bg(Color::Yellow),
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
            d.push(Line::from("Press Enter to replay it."));
        } else if self.unlocked(idx) {
            d.push(Line::from(Span::styled(
                "Press Enter to start!",
                Style::new().fg(Color::Yellow).bold(),
            )));
        } else {
            d.push(Line::from(Span::styled(
                "○ Locked: finish the lesson before this one to open it.",
                Style::new().fg(Color::DarkGray),
            )));
        }
        d.push(Line::default());
        d.push(Line::from(Span::styled(
            format!(
                "Badges: {}/{}  (press b to view)",
                self.progress.badges.len(),
                progress::BADGES.len()
            ),
            Style::new().fg(Color::Magenta),
        )));
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
                ("b", "badges"),
                ("F1", "help"),
                ("q", "quit"),
            ]),
            foot,
        );
    }

    fn draw_lesson(&mut self, f: &mut Frame, area: Rect) {
        let header = {
            let Screen::Lesson(v) = &self.screen else {
                return;
            };
            let l = &self.lessons[v.idx];
            self.header(Line::from(vec![
                Span::styled(
                    " 🦀 ClawCode ",
                    Style::new().fg(Color::Black).bg(CRAB).bold(),
                ),
                Span::styled(
                    format!(
                        "  {} › {}/{} {}",
                        l.chapter,
                        l.order,
                        self.lessons.len(),
                        l.title
                    ),
                    Style::new().fg(ACCENT).bold(),
                ),
            ]))
        };
        let rustc_missing = self.rustc.is_none();
        let Screen::Lesson(v) = &mut self.screen else {
            return;
        };
        let lesson = &self.lessons[v.idx];

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

        // Goal box sized to its wrapped text.
        let goal_text = format!("🎯 {}", lesson.goal);
        let goal_h =
            (wrapped_height(&goal_text, left.width.saturating_sub(2)) + 2).min(left.height / 2);
        let [goal_area, intro_area] =
            Layout::vertical([Constraint::Length(goal_h), Constraint::Min(3)]).areas(left);
        f.render_widget(
            Paragraph::new(Line::from(markdown::inline(
                &goal_text,
                Style::new().bold(),
            )))
            .wrap(Wrap { trim: false })
            .block(rounded(" Your goal ".into()).border_style(Style::new().fg(Color::Yellow))),
            goal_area,
        );

        // Clamp scrolling so you can't scroll into emptiness.
        let inner_w = intro_area.width.saturating_sub(2);
        let total: u16 = v
            .intro
            .iter()
            .map(|l| wrapped_height(&line_text(l), inner_w))
            .sum();
        let max_scroll = total.saturating_sub(intro_area.height.saturating_sub(2));
        v.intro_scroll = v.intro_scroll.min(max_scroll);
        let more = if v.intro_scroll < max_scroll {
            " PgDn ↓ more "
        } else {
            ""
        };
        f.render_widget(
            Paragraph::new(v.intro.clone())
                .wrap(Wrap { trim: false })
                .scroll((v.intro_scroll, 0))
                .block(
                    rounded(" Lesson ".into())
                        .title_bottom(Line::from(more).right_aligned().fg(Color::DarkGray)),
                ),
            intro_area,
        );

        f.render_widget(&v.editor, right);

        let mut lines = v.feedback.lines.clone();
        if rustc_missing {
            lines.insert(0, Line::from(Span::styled("⚠ I can't find `rustc`! Install Rust with rustup (see README) so I can check your code.", Style::new().fg(Color::Red).bold())));
        }
        let inner_w = pinch.width.saturating_sub(18);
        let total: u16 = lines
            .iter()
            .map(|l| wrapped_height(&line_text(l), inner_w))
            .sum();
        let max_fb = total.saturating_sub(pinch.height.saturating_sub(2));
        v.fb_scroll = v.fb_scroll.min(max_fb);
        let mood = v.feedback.mood;
        let scroll = v.fb_scroll;
        draw_pinch(f, pinch, mood, lines, scroll);
        if max_fb > scroll {
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

        f.render_widget(
            keys_line(&[
                ("Ctrl+R", "run"),
                ("Ctrl+T", "tip"),
                ("Ctrl+P", "solution"),
                ("Ctrl+E", "$EDITOR"),
                ("PgUp/Dn", "scroll"),
                ("Esc", "map"),
                ("F1", "help"),
            ]),
            foot,
        );
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let rows: &[(&str, &str)] = &[
            (
                "Ctrl+R  or F5",
                "Run & check your code with the real Rust compiler",
            ),
            (
                "Ctrl+T  or F2",
                "Get a tip (each new tip costs a bit of XP)",
            ),
            (
                "Ctrl+P  or F4",
                "Show the solution (after 3 tries or all tips)",
            ),
            ("Ctrl+E", "Open your code in your own editor ($EDITOR)"),
            ("Ctrl+S", "Save (it also saves automatically)"),
            ("Ctrl+O  or F8", "Reset the code to the starter version"),
            ("Ctrl+Z / Ctrl+Y", "Undo / redo in the editor"),
            ("PgUp / PgDn", "Scroll the lesson text"),
            ("Ctrl+↑↓ / Alt+↑↓", "Scroll Pinch's messages"),
            ("Esc", "Back to the lesson map"),
            ("Ctrl+C", "Quit (progress is saved)"),
        ];
        let mut t = Text::default();
        for (k, d) in rows {
            t.lines.push(Line::from(vec![
                Span::styled(format!("{k:<20}"), Style::new().fg(Color::Yellow).bold()),
                Span::raw(*d),
            ]));
        }
        t.lines.push(Line::default());
        t.lines.push(Line::from(Span::styled(
            "XP: base XP for each lesson, +50% if you pass on the first try,",
            Style::new().fg(Color::DarkGray),
        )));
        t.lines.push(Line::from(Span::styled(
            "    a small bonus for daily streaks, minus a little for tips.",
            Style::new().fg(Color::DarkGray),
        )));
        t.lines.push(Line::from(Span::styled(
            format!("Progress file: {}", Progress::path().display()),
            Style::new().fg(Color::DarkGray),
        )));
        t.lines.push(Line::from(Span::styled(
            format!("Compiler: {}", self.rustc.as_deref().unwrap_or("NOT FOUND")),
            Style::new().fg(Color::DarkGray),
        )));
        popup(f, area, " Keys (any key closes) ", t, 78, 20);
    }

    fn draw_badges(&self, f: &mut Frame, area: Rect) {
        let mut t = Text::default();
        for b in progress::BADGES {
            let have = self.progress.badges.contains(b.id);
            t.lines.push(Line::from(vec![
                Span::styled(
                    if have { " ★ " } else { " ☆ " },
                    Style::new().fg(if have { Color::Yellow } else { Color::DarkGray }),
                ),
                Span::styled(
                    format!("{:<20}", b.name),
                    if have {
                        Style::new().bold()
                    } else {
                        Style::new().fg(Color::DarkGray)
                    },
                ),
                Span::styled(b.desc, Style::new().fg(Color::DarkGray)),
            ]));
        }
        t.lines.push(Line::default());
        t.lines.push(Line::from(format!(
            " Best streak: {} day(s)",
            self.progress.best_streak
        )));
        popup(
            f,
            area,
            " Badges ",
            t,
            70,
            progress::BADGES.len() as u16 + 5,
        );
    }

    fn draw_celebrate(&self, f: &mut Frame, area: Rect, r: &Reward, idx: usize) {
        let mut t = Text::default();
        let gold = Style::new().fg(Color::Yellow).bold();
        {
            let art = r"   \o/   L E S S O N   C L E A R !   \o/";
            t.lines.push(Line::from(Span::styled(art, gold)).centered());
        }
        t.lines.push(Line::default());
        for l in crab_art(Mood::Party) {
            t.lines
                .push(Line::from(Span::styled(l, Style::new().fg(CRAB))).centered());
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
                format!("  Streak: {} days in a row!", r.streak),
                Style::new().fg(Color::LightRed),
            )));
        }
        t.lines.push(Line::default());
        let next = if idx + 1 < self.lessons.len() {
            format!(
                "Enter: next lesson ({})  ·  m: map  ·  Esc: stay",
                self.lessons[idx + 1].title
            )
        } else {
            "That was the last lesson! Enter = back to the map".to_string()
        };
        t.lines
            .push(Line::from(Span::styled(next, Style::new().fg(ACCENT))).centered());
        let h = t.lines.len() as u16 + 2;
        popup(f, area, " 🎉 ", t, 84, h);
    }
}

// ---------------- helpers ----------------

fn make_editor(code: &str) -> TextArea<'static> {
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
fn editor_input(ed: &mut TextArea<'static>, key: KeyEvent) {
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

fn which(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(cmd).is_file()))
}

fn rounded(title: String) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
}

fn keys_line(keys: &[(&str, &str)]) -> Paragraph<'static> {
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

fn line_text(l: &Line) -> String {
    l.spans.iter().map(|s| s.content.as_ref()).collect()
}

/// Rough number of rows a line takes when wrapped to `width` (word wrap makes
/// it a bit taller than this, so we add a small margin for long lines).
fn wrapped_height(text: &str, width: u16) -> u16 {
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

fn popup(f: &mut Frame, area: Rect, title: &str, text: Text<'static>, w: u16, h: u16) {
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

pub fn crab_art(mood: Mood) -> [String; 4] {
    let eyes = match mood {
        Mood::Happy => "^   ^",
        Mood::Thinking => "o   O",
        Mood::Worried => "°   °",
        Mood::Party => "★   ★",
        Mood::Sleepy => "-   -",
    };
    let mouth = match mood {
        Mood::Happy | Mood::Party => "\\___/",
        Mood::Thinking => " ~~~ ",
        Mood::Worried => " ___ ",
        Mood::Sleepy => " zZz ",
    };
    [
        r" (\/)   (\/) ".to_string(),
        format!("  \\ {eyes} /  "),
        format!("  ( {mouth} )  "),
        r"  /|\   /|\  ".to_string(),
    ]
}

fn draw_pinch(f: &mut Frame, area: Rect, mood: Mood, lines: Vec<Line<'static>>, scroll: u16) {
    let block = rounded(" Pinch ".into()).border_style(Style::new().fg(CRAB));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let [art, text] =
        Layout::horizontal([Constraint::Length(15), Constraint::Min(10)]).areas(inner);
    let art_lines: Vec<Line> = crab_art(mood)
        .into_iter()
        .map(|l| Line::from(Span::styled(l, Style::new().fg(CRAB).bold())))
        .collect();
    f.render_widget(Paragraph::new(art_lines), art);
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        text,
    );
}
