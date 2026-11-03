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
