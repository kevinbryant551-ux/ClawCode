//! Lesson data: the TOML schema described in docs/LESSON_FORMAT.md.
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::Path;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}
pub use embedded::ERROR_CATALOG;

#[derive(Debug, Clone, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub order: u32,
    pub chapter: String,
    pub title: String,
    pub xp: u32,
    #[serde(default)]
    pub concepts: Vec<String>,
    pub intro: String,
    pub goal: String,
    pub starter: String,
    pub solution: String,
    #[serde(default)]
    pub hints: Vec<String>,
    #[serde(default)]
    pub success: String,
    pub check: Check,
    #[serde(default)]
    pub quiz: Vec<Quiz>,
    pub boss: Option<Boss>,
}

/// A multiple-choice question shown between reading cards.
#[derive(Debug, Clone, Deserialize)]
pub struct Quiz {
    /// 1-based card number this quiz follows. Defaults to the last card.
    pub after_card: Option<usize>,
    pub question: String,
    pub choices: Vec<String>,
    pub answer: usize,
    #[serde(default)]
    pub explain: String,
}

/// Turns a lesson into a boss fight: each hidden test is one hit point.
#[derive(Debug, Clone, Deserialize)]
pub struct Boss {
    pub name: String,
    pub art: String,
    #[serde(default)]
    pub taunt: String,
    #[serde(default)]
    pub defeat: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    Card(usize),
    Quiz(usize),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Check {
    pub expected_output: Option<String>,
    pub tests: Option<String>,
    #[serde(default)]
    pub must_contain: Vec<Rule>,
    #[serde(default)]
    pub must_not_contain: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub text: String,
    pub why: String,
}

impl Lesson {
    /// The intro split into bite-sized cards (separated by `---` lines).
    pub fn cards(&self) -> Vec<String> {
        let mut cards = vec![String::new()];
        for line in self.intro.lines() {
            if line.trim() == "---" {
                cards.push(String::new());
            } else {
                let c = cards.last_mut().unwrap();
                c.push_str(line);
                c.push('\n');
            }
        }
        cards.retain(|c| !c.trim().is_empty());
        if cards.is_empty() {
            cards.push(self.intro.clone());
        }
        cards
    }

    /// The intro with card breaks removed, for the side panel while coding.
    pub fn full_text(&self) -> String {
        self.cards().join("\n")
    }

    /// Number of hidden tests (a boss's hit points).
    pub fn test_count(&self) -> usize {
        self.check
            .tests
            .as_deref()
            .map_or(0, |t| t.matches("#[test]").count())
    }

    /// The order in which reading cards and quizzes are shown.
    pub fn steps(&self) -> Vec<Step> {
        let cards = self.cards().len();
        let mut steps = Vec::new();
        for c in 0..cards {
            steps.push(Step::Card(c));
            for (qi, q) in self.quiz.iter().enumerate() {
                if q.after_card.unwrap_or(cards).clamp(1, cards) == c + 1 {
                    steps.push(Step::Quiz(qi));
                }
            }
        }
        steps
    }

    pub fn parse(file_name: &str, src: &str) -> Result<Lesson> {
        let lesson: Lesson = toml::from_str(src)
            .with_context(|| format!("could not read lesson file {file_name}"))?;
        let stem = file_name.trim_end_matches(".toml");
        if lesson.id != stem {
            bail!(
                "{file_name}: id is {:?} but should match the file name {:?}",
                lesson.id,
                stem
            );
        }
        let has_output = lesson
            .check
            .expected_output
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty());
        let has_tests = lesson
            .check
            .tests
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty());
        if !has_output && !has_tests {
            bail!("{file_name}: [check] needs expected_output or tests");
        }
        for (i, q) in lesson.quiz.iter().enumerate() {
            if !(2..=6).contains(&q.choices.len()) || q.answer >= q.choices.len() {
                bail!(
                    "{file_name}: quiz {} needs 2-6 choices and a valid answer index",
                    i + 1
                );
            }
        }
        if lesson.boss.is_some() && lesson.test_count() == 0 {
            bail!("{file_name}: boss lessons need tests (each test is one hit point)");
        }
        Ok(lesson)
    }
}

/// Loads lessons. If `CLAWCODE_LESSONS_DIR` is set, reads that folder from disk
/// (handy while writing new lessons); otherwise uses the copies built into the binary.
pub fn load_all() -> Result<Vec<Lesson>> {
    let mut lessons = Vec::new();
    if let Ok(dir) = std::env::var("CLAWCODE_LESSONS_DIR") {
        let mut paths: Vec<_> = std::fs::read_dir(Path::new(&dir))
            .with_context(|| format!("could not open lessons folder {dir}"))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        paths.sort();
        for p in paths {
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            let src = std::fs::read_to_string(&p)?;
            lessons.push(Lesson::parse(&name, &src)?);
        }
    } else {
        for (name, src) in embedded::LESSON_FILES {
            lessons.push(Lesson::parse(name, src)?);
        }
    }
    lessons.sort_by_key(|l| l.order);
    if lessons.is_empty() {
        bail!("no lessons found");
    }
    Ok(lessons)
}
