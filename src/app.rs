//! Game state and input handling. Drawing lives in `ui.rs`.
use crate::anim::{self, Bubbles, Confetti, Rng};
use crate::checker::{self, Outcome};
use crate::feedback::{self, Feedback, Mood};
use crate::friendly::Friendly;
use crate::lesson::{Lesson, Quiz, Step};
use crate::markdown;
use crate::progress::{self, Progress, Reward};
use crate::shop;
use crate::widgets::{editor_input, make_editor, which};
use anyhow::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};
use tui_textarea::TextArea;

/// Tries before the "show solution" key works.
pub const TRIES_BEFORE_SOLUTION: u32 = 3;
/// One animation frame.
const TICK: Duration = Duration::from_millis(50);
/// Questions in a Daily Tide.
pub const DAILY_QUESTIONS: usize = 5;

pub const TITLE_MENU: &[&str] = &[
    "Continue Quest",
    "Daily Tide",
    "Shell Shop",
    "Trophy Room",
    "How to Play",
    "Quit",
];

pub struct App {
    pub lessons: Vec<Lesson>,
    pub progress: Progress,
    pub friendly: Friendly,
    pub unlock_all: bool,
    pub rustc: Option<String>,
    pub screen: Screen,
    pub overlay: Option<Overlay>,
    pub map_state: ListState,
    pub map_rows: Vec<Option<usize>>,
    pub title_sel: usize,
    pub shop_sel: usize,
    pub tick: u64,
    pub quit: bool,
    pub confetti: Confetti,
    pub bubbles: Bubbles,
    pub rng: Rng,
    /// A short notice in the top-right corner, until the given tick.
    pub toast: Option<(Vec<Line<'static>>, u64)>,
    pub today: i64,
    pub last_area: Rect,
}

pub enum Screen {
    Title,
    Map,
    Learn(Box<LearnView>),
    Code(Box<CodeView>),
    Daily(Box<DailyView>),
    Shop,
    Trophies,
}

pub struct QuizState {
    pub quiz: Quiz,
    pub key: String,
    pub sel: usize,
    pub wrong: Vec<usize>,
    pub solved: bool,
    pub first_try: Option<bool>,
}

impl QuizState {
    fn new(quiz: Quiz, key: String) -> QuizState {
        QuizState {
            quiz,
            key,
            sel: 0,
            wrong: Vec::new(),
            solved: false,
            first_try: None,
        }
    }
}

pub struct LearnView {
    pub idx: usize,
    pub steps: Vec<Step>,
    pub pos: usize,
    pub quiz: Option<QuizState>,
    pub scroll: u16,
    pub pinch: Feedback,
    pub born: u64,
}

pub struct BossState {
    pub hp_max: usize,
    pub hp: usize,
    pub shown_hp: f32,
    pub shake_until: u64,
}

pub struct CodeView {
    pub idx: usize,
    pub editor: TextArea<'static>,
    pub intro: Vec<Line<'static>>,
    pub intro_scroll: u16,
    pub feedback: Feedback,
    pub fb_scroll: u16,
    pub born: u64,
    pub boss: Option<BossState>,
}

pub struct DailyView {
    pub qs: Vec<(usize, usize)>,
    pub pos: usize,
    pub quiz: QuizState,
    pub right: u32,
    pub pinch: Feedback,
    pub born: u64,
}

pub enum Overlay {
    Help,
    Celebrate {
        reward: Reward,
        next: Option<usize>,
        title: String,
        note: String,
    },
    ConfirmReset,
    Solution(String),
}

impl App {
    pub fn new(lessons: Vec<Lesson>, unlock_all: bool) -> App {
        let mut app = App {
            lessons,
            progress: Progress::load(),
            friendly: Friendly::load(crate::lesson::ERROR_CATALOG),
            unlock_all,
            rustc: checker::rustc_version(),
            screen: Screen::Title,
            overlay: None,
            map_state: ListState::default(),
            map_rows: Vec::new(),
            title_sel: 0,
            shop_sel: 0,
            tick: 0,
            quit: false,
            confetti: Confetti::default(),
            bubbles: Bubbles::default(),
            rng: Rng::from_time(),
            toast: None,
            today: progress::today(),
            last_area: Rect::default(),
        };
        app.build_map_rows();
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

    pub fn select_lesson(&mut self, idx: usize) {
        let row = self
            .map_rows
            .iter()
            .position(|r| *r == Some(idx))
            .unwrap_or(1);
        self.map_state.select(Some(row));
    }

    pub fn selected_lesson(&self) -> usize {
        self.map_state
            .selected()
            .and_then(|r| self.map_rows.get(r).copied().flatten())
            .unwrap_or(0)
    }

    pub fn unlocked(&self, idx: usize) -> bool {
        self.unlock_all
            || idx == 0
            || self.progress.is_done(&self.lessons[idx].id)
            || self.progress.is_done(&self.lessons[idx - 1].id)
    }

    pub fn done_count(&self) -> usize {
        self.lessons
            .iter()
            .filter(|l| self.progress.is_done(&l.id))
            .count()
    }

    /// Quizzes the learner has seen (from lessons they've read or cleared): the Daily Tide pool.
    pub fn daily_pool(&self) -> Vec<(usize, usize)> {
        let mut pool = Vec::new();
        for (i, l) in self.lessons.iter().enumerate() {
            if self.progress.learned.contains(&l.id) || self.progress.is_done(&l.id) {
                pool.extend((0..l.quiz.len()).map(|q| (i, q)));
            }
        }
        pool
    }

    pub fn daily_done_today(&self) -> bool {
        self.progress.daily_day == Some(self.today)
    }

    fn save(&mut self) {
        if let Screen::Code(v) = &self.screen {
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

    pub fn toast(&mut self, lines: Vec<Line<'static>>) {
        self.toast = Some((lines, self.tick + 70));
    }

    fn toast_reward(&mut self, r: &Reward, extra: Option<String>) {
        let mut lines = Vec::new();
        if let Some(e) = extra {
            lines.push(Line::from(Span::styled(
                e,
                Style::new().fg(Color::LightRed),
            )));
        }
        if r.xp > 0 || r.shells > 0 {
            lines.push(Line::from(vec![
                Span::styled(format!("+{} XP  ", r.xp), Style::new().fg(Color::Yellow)),
                Span::styled(
                    format!("+{} shells", r.shells),
                    Style::new().fg(Color::LightCyan),
                ),
            ]));
        }
        if let Some(l) = r.leveled_up {
            lines.push(Line::from(Span::styled(
                format!("⬆ LEVEL UP! Lv {l} {}", progress::level_title(l)),
                Style::new().fg(Color::Yellow),
            )));
            self.confetti.burst(self.last_area, &mut self.rng, 60);
        }
        for b in &r.new_badges {
            lines.push(Line::from(Span::styled(
                format!("★ Badge: {}", b.name),
                Style::new().fg(Color::Magenta),
            )));
        }
        if !lines.is_empty() {
            self.toast(lines);
        }
    }

    // ---------------- main loop ----------------

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut last = Instant::now();
        while !self.quit {
            terminal.draw(|f| self.draw(f))?;
            let wait = TICK.saturating_sub(last.elapsed());
            if event::poll(wait)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.on_key(key, terminal)?;
            }
            if last.elapsed() >= TICK {
                last = Instant::now();
                self.on_tick();
            }
        }
        self.save();
        Ok(())
    }

    fn on_tick(&mut self) {
        self.tick += 1;
        self.confetti.tick();
        if matches!(self.screen, Screen::Title) {
            let area = self.last_area;
            self.bubbles.tick(area, &mut self.rng);
        }
        if let Screen::Code(v) = &mut self.screen
            && let Some(b) = &mut v.boss
        {
            let target = b.hp as f32;
            if (b.shown_hp - target).abs() > 0.01 {
                b.shown_hp +=
                    (target - b.shown_hp).signum() * 0.08_f32.min((target - b.shown_hp).abs());
            }
        }
        if self
            .toast
            .as_ref()
            .is_some_and(|(_, until)| *until < self.tick)
        {
            self.toast = None;
        }
    }

    fn on_key(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            self.quit = true;
            return Ok(());
        }
        if let Some(ov) = self.overlay.take() {
            self.on_overlay_key(ov, key);
            return Ok(());
        }
        match self.screen {
            Screen::Title => self.on_title_key(key),
            Screen::Map => self.on_map_key(key),
            Screen::Learn(_) => self.on_learn_key(key),
            Screen::Code(_) => self.on_code_key(key, terminal)?,
            Screen::Daily(_) => self.on_daily_key(key),
            Screen::Shop => self.on_shop_key(key),
            Screen::Trophies => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                    self.screen = Screen::Title;
                }
            }
        }
        Ok(())
    }

    fn on_overlay_key(&mut self, ov: Overlay, key: KeyEvent) {
        match ov {
            Overlay::Celebrate { next, .. } => {
                self.confetti.clear();
                match key.code {
                    KeyCode::Enter | KeyCode::Char('n') => match next {
                        Some(i) if i < self.lessons.len() => self.open_lesson(i),
                        _ => self.screen = Screen::Map,
                    },
                    KeyCode::Char('m') => {
                        self.screen = Screen::Map;
                        if let Some(i) = next {
                            self.select_lesson(i.min(self.lessons.len() - 1));
                        }
                    }
                    _ => {}
                }
            }
            Overlay::ConfirmReset => {
                if matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y')) {
                    if let Screen::Code(v) = &mut self.screen {
                        v.editor = make_editor(&self.lessons[v.idx].starter);
                        v.feedback =
                            Feedback::say(Mood::Happy, "Fresh start! The starter code is back.");
                        v.born = self.tick;
                    }
                    self.save();
                }
            }
            Overlay::Solution(code) => {
                if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
                    && let Screen::Code(v) = &mut self.screen
                {
                    v.editor = make_editor(&code);
                    v.feedback = Feedback::say(
                        Mood::Happy,
                        "I copied the solution into your editor. Read it line by line, then press **Ctrl+R** to run it.\nTip: change something small and run again to see what breaks!",
                    );
                    v.born = self.tick;
                }
            }
            Overlay::Help => {}
        }
    }

    // ---------------- title ----------------

    fn on_title_key(&mut self, key: KeyEvent) {
        let n = TITLE_MENU.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.title_sel = (self.title_sel + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.title_sel = (self.title_sel + 1) % n,
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Enter | KeyCode::Char(' ') => match self.title_sel {
                0 => self.screen = Screen::Map,
                1 => self.open_daily(),
                2 => self.screen = Screen::Shop,
                3 => self.screen = Screen::Trophies,
                4 => self.overlay = Some(Overlay::Help),
                _ => self.quit = true,
            },
            KeyCode::Char('d') => self.open_daily(),
            KeyCode::Char('s') => self.screen = Screen::Shop,
            KeyCode::Char('t') => self.screen = Screen::Trophies,
            KeyCode::F(1) | KeyCode::Char('?') => self.overlay = Some(Overlay::Help),
            _ => {}
        }
    }

    // ---------------- map ----------------

    fn on_map_key(&mut self, key: KeyEvent) {
        let rows = self.map_rows.len();
        let mut row = self.map_state.selected().unwrap_or(1);
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Title,
            KeyCode::Up | KeyCode::Char('k') => {
                while row > 0 {
                    row -= 1;
                    if self.map_rows[row].is_some() {
                        self.map_state.select(Some(row));
                        break;
                    }
                }
            }
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
                } else {
                    self.toast(vec![Line::from(
                        "○ Locked! Clear the lesson before it first.",
                    )]);
                }
            }
            KeyCode::Char('l') => {
                let idx = self.selected_lesson();
                if self.unlocked(idx) {
                    self.open_learn(idx);
                }
            }
            KeyCode::Char('d') => self.open_daily(),
            KeyCode::Char('s') => self.screen = Screen::Shop,
            KeyCode::Char('t') => self.screen = Screen::Trophies,
            KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Char('h') => {
                self.overlay = Some(Overlay::Help)
            }
            _ => {}
        }
    }

    /// Opens a lesson: the reading cards first time through, the editor after.
    pub fn open_lesson(&mut self, idx: usize) {
        let l = &self.lessons[idx];
        let has_steps = l.steps().len() > 1 || !l.quiz.is_empty();
        if has_steps && !self.progress.learned.contains(&l.id) && !self.progress.is_done(&l.id) {
            self.open_learn(idx);
        } else {
            self.open_code(idx);
        }
    }

    // ---------------- learn (cards + quizzes) ----------------

    fn open_learn(&mut self, idx: usize) {
        self.save();
        let l = &self.lessons[idx];
        let steps = l.steps();
        let intro = if let Some(b) = &l.boss {
            format!(
                "⚔ **{}** blocks the path! Study up before the fight. (→ next, Tab skips to the battle)",
                b.name
            )
        } else {
            "Read each card, then press **→** or **Enter** for the next one. **Tab** skips straight to the code.".into()
        };
        self.screen = Screen::Learn(Box::new(LearnView {
            idx,
            steps,
            pos: 0,
            quiz: None,
            scroll: 0,
            pinch: Feedback::say(Mood::Happy, &intro),
            born: self.tick,
        }));
        self.select_lesson(idx);
        self.sync_learn_quiz();
    }

    /// When the current step is a quiz, makes sure its state exists.
    fn sync_learn_quiz(&mut self) {
        let Screen::Learn(v) = &mut self.screen else {
            return;
        };
        let lesson = &self.lessons[v.idx];
        match v.steps.get(v.pos) {
            Some(Step::Quiz(qi)) => {
                let key = format!("{}#{qi}", lesson.id);
                let mut q = QuizState::new(lesson.quiz[*qi].clone(), key.clone());
                if self.progress.quizzes_done.contains(&key) {
                    q.solved = true;
                    q.sel = q.quiz.answer;
                    v.pinch = Feedback::say(
                        Mood::Happy,
                        "You've solved this one before. Enter to continue!",
                    );
                } else {
                    let combo = self.progress.combo;
                    let msg = if combo >= 2 {
                        format!(
                            "Pop quiz! You're on a **{combo}x combo**. Keep it going! (↑↓ or 1-4 to pick, Enter to answer)"
                        )
                    } else {
                        "Pop quiz! Pick an answer with **↑↓** or **1-4**, then press **Enter**."
                            .into()
                    };
                    v.pinch = Feedback::say(Mood::Thinking, &msg);
                }
                v.born = self.tick;
                v.quiz = Some(q);
            }
            _ => v.quiz = None,
        }
    }

    fn on_learn_key(&mut self, key: KeyEvent) {
        let Screen::Learn(v) = &mut self.screen else {
            return;
        };
        let idx = v.idx;
        if let Some(q) = &mut v.quiz
            && !q.solved
        {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    q.sel = (q.sel + q.quiz.choices.len() - 1) % q.quiz.choices.len()
                }
                KeyCode::Down | KeyCode::Char('j') => q.sel = (q.sel + 1) % q.quiz.choices.len(),
                KeyCode::Char(c @ '1'..='6') => {
                    let n = c as usize - '1' as usize;
                    if n < q.quiz.choices.len() {
                        q.sel = n;
                        self.answer_learn_quiz();
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => self.answer_learn_quiz(),
                KeyCode::Esc => {
                    self.save();
                    self.screen = Screen::Map;
                }
                KeyCode::Tab => self.finish_learn(idx),
                KeyCode::Left | KeyCode::Backspace => self.learn_move(-1),
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('l') => {
                self.learn_move(1)
            }
            KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => self.learn_move(-1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::PageDown => {
                v.scroll = v.scroll.saturating_add(3)
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::PageUp => {
                v.scroll = v.scroll.saturating_sub(3)
            }
            KeyCode::Tab => self.finish_learn(idx),
            KeyCode::Esc => {
                self.save();
                self.screen = Screen::Map;
            }
            KeyCode::F(1) | KeyCode::Char('?') => self.overlay = Some(Overlay::Help),
            _ => {}
        }
    }

    fn learn_move(&mut self, dir: i32) {
        let Screen::Learn(v) = &mut self.screen else {
            return;
        };
        let idx = v.idx;
        let new = v.pos as i32 + dir;
        if new < 0 {
            return;
        }
        if new as usize >= v.steps.len() {
            self.finish_learn(idx);
            return;
        }
        v.pos = new as usize;
        v.scroll = 0;
        if matches!(v.steps[v.pos], Step::Card(_)) {
            let left = v.steps[v.pos..]
                .iter()
                .filter(|s| matches!(s, Step::Card(_)))
                .count();
            let lines = [
                "Nice! Keep going.",
                "You're doing great. One idea at a time.",
                "Take your time with this one.",
                "Almost at the coding part!",
            ];
            let msg = if left == 1 {
                "Last card! After this, you write real code."
            } else {
                lines[v.pos % lines.len()]
            };
            v.pinch = Feedback::say(Mood::Happy, msg);
            v.born = self.tick;
        }
        self.sync_learn_quiz();
    }

    fn answer_learn_quiz(&mut self) {
        let Screen::Learn(v) = &mut self.screen else {
            return;
        };
        let Some(q) = &mut v.quiz else { return };
        if q.wrong.contains(&q.sel) {
            return;
        }
        let correct = q.sel == q.quiz.answer;
        let first = q.first_try.is_none();
        if first {
            q.first_try = Some(correct);
        }
        let key = q.key.clone();
        if correct {
            q.solved = true;
            let first_ok = q.first_try == Some(true);
            let explain = q.quiz.explain.clone();
            let (_, _, reward) = self.progress.record_quiz(&key, first_ok, true);
            let combo = self.progress.combo;
            let Screen::Learn(v) = &mut self.screen else {
                return;
            };
            let mut text = if first_ok {
                "✔ **Correct!** ".to_string()
            } else {
                "✔ **There it is!** ".to_string()
            };
            text.push_str(&explain);
            text.push_str("\n(Enter to continue)");
            v.pinch = Feedback::say(Mood::Party, &text);
            v.born = self.tick;
            let extra = (first_ok && combo >= 2).then(|| format!("🔥 COMBO x{combo}!"));
            self.toast_reward(&reward, extra);
            let _ = self.progress.save();
        } else {
            q.wrong.push(q.sel);
            if first {
                // A wrong first answer still marks the quiz as attempted for combo purposes.
                self.progress.combo = 0;
            }
            let Screen::Learn(v) = &mut self.screen else {
                return;
            };
            let msgs = [
                "Not quite! Have another look and try again.",
                "Close, but no. Think about what the card said.",
                "Nope, but wrong answers teach too! Try another.",
            ];
            v.pinch = Feedback::say(Mood::Worried, msgs[v.born as usize % msgs.len()]);
            v.born = self.tick;
        }
    }

    fn finish_learn(&mut self, idx: usize) {
        let id = self.lessons[idx].id.clone();
        if self.progress.learned.insert(id) {
            let _ = self.progress.save();
        }
        self.open_code(idx);
    }

    // ---------------- code ----------------

    fn open_code(&mut self, idx: usize) {
        self.save();
        let lesson = &self.lessons[idx];
        let code = self
            .progress
            .drafts
            .get(&lesson.id)
            .cloned()
            .unwrap_or_else(|| lesson.starter.clone());
        let done = self.progress.is_done(&lesson.id);
        let boss = lesson.boss.as_ref().map(|_| {
            let hp = if done { 0 } else { lesson.test_count() };
            BossState {
                hp_max: lesson.test_count(),
                hp,
                shown_hp: hp as f32,
                shake_until: 0,
            }
        });
        let greeting = if done {
            format!(
                "Welcome back to **{}**! You've already cleared this one, so practice freely. (**Ctrl+L** rereads the cards.)",
                lesson.title
            )
        } else if let Some(b) = &lesson.boss {
            format!(
                "**{}**: \"{}\"\nEvery hidden test you pass is a hit. **Ctrl+R** attacks!",
                b.name, b.taunt
            )
        } else if self.progress.drafts.contains_key(&lesson.id) {
            "I kept your code from last time. Pick up where you left off!".to_string()
        } else {
            "Your turn! Edit the code on the right to reach the goal.\nPress **Ctrl+R** to run it. Stuck? **Ctrl+T** for a tip. (**F1** shows all keys.)".to_string()
        };
        let mood = if done {
            Mood::Party
        } else if boss.is_some() {
            Mood::Worried
        } else {
            Mood::Happy
        };
        self.screen = Screen::Code(Box::new(CodeView {
            idx,
            editor: make_editor(&code),
            intro: markdown::render(&lesson.full_text()),
            intro_scroll: 0,
            feedback: Feedback::say(mood, &greeting),
            fb_scroll: 0,
            born: self.tick,
            boss,
        }));
        self.select_lesson(idx);
    }

    fn on_code_key(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let Screen::Code(v) = &mut self.screen else {
            return Ok(());
        };
        v.born = 0; // any key finishes Pinch's typing animation
        match (key.code, ctrl) {
            (KeyCode::Esc, _) => {
                self.save();
                self.screen = Screen::Map;
            }
            (KeyCode::Char('r'), true) | (KeyCode::F(5), _) => self.run_check(terminal)?,
            (KeyCode::Char('t'), true) | (KeyCode::F(2), _) => self.next_hint(),
            (KeyCode::Char('s'), true) => {
                self.save();
                if let Screen::Code(v) = &mut self.screen {
                    v.feedback =
                        Feedback::say(Mood::Happy, "Saved! Your code is safe even if you quit.");
                }
            }
            (KeyCode::Char('l'), true) | (KeyCode::F(3), _) => {
                let idx = v.idx;
                self.save();
                self.open_learn(idx);
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
                v.fb_scroll = v.fb_scroll.saturating_add(3)
            }
            (KeyCode::Up, true) | (KeyCode::PageUp, _) => {
                v.fb_scroll = v.fb_scroll.saturating_sub(3)
            }
            (KeyCode::Down, false) if alt => v.fb_scroll = v.fb_scroll.saturating_add(3),
            (KeyCode::Up, false) if alt => v.fb_scroll = v.fb_scroll.saturating_sub(3),
            _ => editor_input(&mut v.editor, key),
        }
        Ok(())
    }

    fn run_check(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let Screen::Code(v) = &mut self.screen else {
            return Ok(());
        };
        let idx = v.idx;
        let code = v.editor.lines().join("\n") + "\n";
        let boss_name = self.lessons[idx].boss.as_ref().map(|b| b.name.clone());
        let msg = match &boss_name {
            Some(n) => format!("⚔ Attacking {n} with the real Rust compiler..."),
            None => "Compiling your code with the real Rust compiler... 🦀".into(),
        };
        v.feedback = Feedback::say(Mood::Thinking, &msg);
        v.fb_scroll = 0;
        v.born = 0;
        terminal.draw(|f| self.draw(f))?;

        let lesson = self.lessons[idx].clone();
        let outcome = checker::check(&lesson, &code);
        let already = self.progress.is_done(&lesson.id);
        self.progress.stats.runs += 1;
        if matches!(outcome, Outcome::CompileError { .. }) {
            self.progress.stats.compile_errors += 1;
        }
        let attempt = {
            let a = self.progress.attempts.entry(lesson.id.clone()).or_insert(0);
            if !already {
                *a += 1;
            }
            *a
        };
        let mut fb = feedback::describe(&outcome, &lesson, &self.friendly, attempt);

        // Boss fight bookkeeping: tests passed = damage dealt.
        let tick = self.tick;
        if let (Screen::Code(v), Some(name)) = (&mut self.screen, &boss_name)
            && let Some(b) = &mut v.boss
        {
            let before = b.hp;
            let banner = match &outcome {
                Outcome::Passed { .. } => {
                    b.hp = 0;
                    format!("💥 CRITICAL HIT! {name} goes down!")
                }
                Outcome::TestsFailed { passed, total, .. } => {
                    b.hp = total - passed;
                    if b.hp < before {
                        format!(
                            "⚔ You hit {name} for {} damage! {} HP left.",
                            before - b.hp,
                            b.hp
                        )
                    } else {
                        format!(
                            "⚔ {name} shrugs it off: {passed} of {total} hits land. {} HP left.",
                            b.hp
                        )
                    }
                }
                Outcome::CompileError { .. } => {
                    format!("🛡 {name} blocks! Your attack didn't even compile.")
                }
                _ => format!("🛡 {name} dodges! Your program didn't finish properly."),
            };
            if b.hp < before {
                b.shake_until = tick + 12;
            }
            fb.lines.insert(
                0,
                Line::from(Span::styled(
                    banner,
                    Style::new().fg(Color::LightRed).bold(),
                )),
            );
        }
        if let Screen::Code(v) = &mut self.screen {
            v.feedback = fb;
            v.born = self.tick;
        }

        if outcome.passed() {
            let reward = self.progress.record_pass(
                &lesson.id,
                lesson.xp,
                lesson.boss.is_some(),
                self.lessons.len(),
                self.today,
            );
            self.progress
                .drafts
                .insert(lesson.id.clone(), code.trim_end().to_string());
            let (title, note) = match &lesson.boss {
                Some(b) => (
                    format!("B O S S   D E F E A T E D :  {}", b.name.to_uppercase()),
                    format!("\"{}\"", b.defeat),
                ),
                None => ("L E S S O N   C L E A R !".to_string(), String::new()),
            };
            self.confetti.burst(
                self.last_area,
                &mut self.rng,
                if lesson.boss.is_some() { 220 } else { 120 },
            );
            self.overlay = Some(Overlay::Celebrate {
                reward,
                next: Some(idx + 1),
                title,
                note,
            });
        } else if !already
            && attempt == TRIES_BEFORE_SOLUTION
            && let Screen::Code(v) = &mut self.screen
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
        let Screen::Code(v) = &mut self.screen else {
            return;
        };
        let lesson = &self.lessons[v.idx];
        v.born = self.tick;
        v.fb_scroll = 0;
        if lesson.hints.is_empty() {
            v.feedback = Feedback::say(
                Mood::Thinking,
                "This lesson has no tips. Reread the goal at the top left!",
            );
            return;
        }
        let done = self.progress.is_done(&lesson.id);
        let used = self
            .progress
            .hints_used
            .entry(lesson.id.clone())
            .or_insert(0);
        if (*used as usize) < lesson.hints.len() && !done {
            *used += 1;
        }
        let shown = if done {
            lesson.hints.len()
        } else {
            (*used as usize).clamp(1, lesson.hints.len())
        };
        let mut lines = vec![Line::from(Span::styled(
            format!(
                "Tip {shown} of {} (each new tip costs a little XP):",
                lesson.hints.len()
            ),
            Style::new().add_modifier(ratatui::style::Modifier::BOLD),
        ))];
        for (i, h) in lesson.hints.iter().take(shown).enumerate() {
            let mut spans = vec![Span::styled(
                format!("{}. ", i + 1),
                Style::new().fg(Color::LightRed),
            )];
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
        let _ = self.progress.save();
    }

    fn peek_solution(&mut self) {
        let Screen::Code(v) = &mut self.screen else {
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
            v.born = self.tick;
        }
    }

    /// Opens the code in $VISUAL / $EDITOR (falls back to nano, then vi).
    fn external_editor(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let Screen::Code(v) = &mut self.screen else {
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
        v.born = self.tick;
        Ok(())
    }

    // ---------------- daily tide ----------------

    fn open_daily(&mut self) {
        if self.daily_done_today() {
            self.toast(vec![
                Line::from("🌊 Today's Daily Tide is done!"),
                Line::from("The tide comes back tomorrow."),
            ]);
            return;
        }
        let pool = self.daily_pool();
        if pool.len() < 3 {
            self.toast(vec![
                Line::from("🌊 The Daily Tide mixes quizzes from"),
                Line::from("lessons you've read. Read a lesson first!"),
            ]);
            return;
        }
        let picks = anim::pick(pool.len(), DAILY_QUESTIONS, self.today as u64);
        let qs: Vec<(usize, usize)> = picks.into_iter().map(|i| pool[i]).collect();
        let (li, qi) = qs[0];
        let quiz = QuizState::new(self.lessons[li].quiz[qi].clone(), String::new());
        self.screen = Screen::Daily(Box::new(DailyView {
            qs,
            pos: 0,
            quiz,
            right: 0,
            pinch: Feedback::say(
                Mood::Happy,
                "🌊 The Daily Tide is in! Questions from lessons you've seen. Your **first answer** counts. Ready?",
            ),
            born: self.tick,
        }));
    }

    fn on_daily_key(&mut self, key: KeyEvent) {
        let Screen::Daily(v) = &mut self.screen else {
            return;
        };
        let n = v.quiz.quiz.choices.len();
        if key.code == KeyCode::Esc {
            self.screen = Screen::Title;
            return;
        }
        if v.quiz.solved {
            if matches!(
                key.code,
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right
            ) {
                v.pos += 1;
                if v.pos >= v.qs.len() {
                    let (right, total) = (v.right, v.qs.len() as u32);
                    let reward = self.progress.record_daily(right, total, self.today);
                    let _ = self.progress.save();
                    self.confetti.burst(self.last_area, &mut self.rng, 100);
                    self.screen = Screen::Title;
                    self.overlay = Some(Overlay::Celebrate {
                        reward,
                        next: None,
                        title: "D A I L Y   T I D E   C O M P L E T E".into(),
                        note: format!("You got {right} of {total} right. See you tomorrow!"),
                    });
                } else {
                    let (li, qi) = v.qs[v.pos];
                    v.quiz = QuizState::new(self.lessons[li].quiz[qi].clone(), String::new());
                    v.pinch = Feedback::say(
                        Mood::Thinking,
                        &format!(
                            "Question {} of {}. From the lesson **{}**.",
                            v.pos + 1,
                            v.qs.len(),
                            self.lessons[li].title
                        ),
                    );
                    v.born = self.tick;
                }
            }
            return;
        }
        let submit = match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                v.quiz.sel = (v.quiz.sel + n - 1) % n;
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                v.quiz.sel = (v.quiz.sel + 1) % n;
                false
            }
            KeyCode::Char(c @ '1'..='6') if (c as usize - '1' as usize) < n => {
                v.quiz.sel = c as usize - '1' as usize;
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => true,
            _ => false,
        };
        if !submit || v.quiz.wrong.contains(&v.quiz.sel) {
            return;
        }
        let correct = v.quiz.sel == v.quiz.quiz.answer;
        if v.quiz.first_try.is_none() {
            v.quiz.first_try = Some(correct);
            if correct {
                v.right += 1;
            }
        }
        if correct {
            v.quiz.solved = true;
            let head = if v.quiz.first_try == Some(true) {
                "✔ **Correct!** "
            } else {
                "✔ Got it now. "
            };
            v.pinch = Feedback::say(
                Mood::Party,
                &format!("{head}{}\n(Enter to continue)", v.quiz.quiz.explain),
            );
        } else {
            v.quiz.wrong.push(v.quiz.sel);
            v.pinch = Feedback::say(
                Mood::Worried,
                "Not this one. Pick another to see the right answer.",
            );
        }
        v.born = self.tick;
    }

    // ---------------- shop ----------------

    fn on_shop_key(&mut self, key: KeyEvent) {
        let n = shop::ITEMS.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.shop_sel = (self.shop_sel + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.shop_sel = (self.shop_sel + 1) % n,
            KeyCode::Enter | KeyCode::Char(' ') => {
                let item = &shop::ITEMS[self.shop_sel];
                let was_owned = self.progress.owned.contains(item.id) || item.price == 0;
                let (ok, msg, reward) = self.progress.shop_select(item);
                let _ = self.progress.save();
                if ok && !was_owned {
                    self.confetti.burst(self.last_area, &mut self.rng, 50);
                }
                let mut lines = vec![Line::from(Span::styled(
                    msg,
                    Style::new().fg(if ok {
                        Color::LightGreen
                    } else {
                        Color::LightRed
                    }),
                ))];
                for b in &reward.new_badges {
                    lines.push(Line::from(Span::styled(
                        format!("★ Badge: {}", b.name),
                        Style::new().fg(Color::Magenta),
                    )));
                }
                self.toast(lines);
            }
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Title,
            _ => {}
        }
    }
}
