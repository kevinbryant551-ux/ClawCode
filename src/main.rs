//! ClawCode: learn Rust by playing through small, compiler-checked challenges.
mod anim;
mod app;
mod checker;
mod feedback;
mod friendly;
mod godot_export;
mod lesson;
mod markdown;
mod progress;
mod shop;
mod tap;
mod ui;
mod widgets;

use anyhow::Result;
use std::process::ExitCode;

const USAGE: &str = "\
ClawCode - learn Rust one small, compiler-checked step at a time.

USAGE:
    clawcode                     Start the game (the normal way to play)
    clawcode --unlock-all        Start with every lesson unlocked
    clawcode list                List lessons and your progress
    clawcode check <id> <file>   Check a .rs file against a lesson (for use with your own editor)
    clawcode verify [id]         For lesson authors: solutions must pass, starters must fail
    clawcode export-godot [file] For the Android app: write lessons as JSON (default godot/data/lessons.json)
    clawcode reset               Delete all saved progress
    clawcode --help              Show this text
";

fn main() -> ExitCode {
    match real_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("(\\/) Oops: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<ExitCode> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let lessons = lesson::load_all()?;
    match args.first().map(String::as_str) {
        None | Some("--unlock-all") => {
            let unlock = args.iter().any(|a| a == "--unlock-all")
                || std::env::var_os("CLAWCODE_UNLOCK_ALL").is_some();
            let mut app = app::App::new(lessons, unlock);
            let mut terminal = ratatui::init();
            let result = app.run(&mut terminal);
            ratatui::restore();
            result?;
            println!("(\\/) See you next time! Your progress is saved.");
            Ok(ExitCode::SUCCESS)
        }
        Some("-h" | "--help" | "help") => {
            print!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        Some("list") => {
            let p = progress::Progress::load();
            println!(
                "Level {} ({}), {} XP, streak {} day(s)\n",
                p.level(),
                progress::level_title(p.level()),
                p.xp,
                p.streak
            );
            for l in &lessons {
                let mark = if p.is_done(&l.id) { "✔" } else { " " };
                println!("  [{mark}] {:<18} {}  ({})", l.id, l.title, l.chapter);
            }
            Ok(ExitCode::SUCCESS)
        }
        Some("check") => {
            let (Some(id), Some(file)) = (args.get(1), args.get(2)) else {
                anyhow::bail!("usage: clawcode check <lesson-id> <file.rs>");
            };
            let l = lessons
                .iter()
                .find(|l| &l.id == id)
                .ok_or_else(|| anyhow::anyhow!("no lesson called {id}; try `clawcode list`"))?;
            let code = std::fs::read_to_string(file)?;
            let outcome = checker::check(l, &code);
            let fr = friendly::Friendly::load(lesson::ERROR_CATALOG);
            println!(
                "{}",
                feedback::plain(&feedback::describe(&outcome, l, &fr, 0))
            );
            Ok(if outcome.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
        Some("verify") => Ok(verify(&lessons, args.get(1).map(String::as_str))),
        Some("export-godot") => {
            let out = args
                .get(1)
                .map_or(godot_export::DEFAULT_OUT, String::as_str);
            if let Some(dir) = std::path::Path::new(out).parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(out, godot_export::export(&lessons))?;
            println!("Wrote {} lessons to {out}", lessons.len());
            Ok(ExitCode::SUCCESS)
        }
        Some("reset") => {
            use std::io::Write;
            print!(
                "Delete all ClawCode progress at {}? Type yes: ",
                progress::Progress::path().display()
            );
            std::io::stdout().flush()?;
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            if answer.trim() == "yes" {
                let _ = std::fs::remove_file(progress::Progress::path());
                println!("Progress deleted. Fresh start!");
            } else {
                println!("Nothing deleted.");
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(other) => {
            eprint!("Unknown command `{other}`.\n\n{USAGE}");
            Ok(ExitCode::FAILURE)
        }
    }
}

/// Lesson-author check: every solution passes and every starter fails.
fn verify(lessons: &[lesson::Lesson], only: Option<&str>) -> ExitCode {
    let mut bad = 0;
    let mut orders = std::collections::BTreeSet::new();
    for l in lessons.iter().filter(|l| only.is_none_or(|o| o == l.id)) {
        let sol = checker::check(l, &l.solution);
        let start = checker::check(l, &l.starter);
        let ok = sol.passed() && !start.passed() && orders.insert(l.order) && !l.hints.is_empty();
        println!(
            "{} {:<20} solution: {:<6} starter: {}",
            if ok { "✔" } else { "✘" },
            l.id,
            if sol.passed() { "passes" } else { "FAILS" },
            if start.passed() {
                "PASSES (should fail)"
            } else {
                "fails (good)"
            }
        );
        if !sol.passed() {
            let fr = friendly::Friendly::load(lesson::ERROR_CATALOG);
            println!(
                "{}\n",
                feedback::plain(&feedback::describe(&sol, l, &fr, 0))
            );
        }
        if !ok {
            bad += 1;
        }
    }
    if bad == 0 {
        println!("\nAll lessons OK.");
        ExitCode::SUCCESS
    } else {
        println!("\n{bad} lesson(s) need attention.");
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    /// Every built-in lesson must be solvable, and its starter must not already pass.
    #[test]
    fn all_lessons_verify() {
        let lessons = crate::lesson::load_all().expect("lessons load");
        assert!(lessons.len() >= 10);
        for l in &lessons {
            assert!(
                crate::checker::check(l, &l.solution).passed(),
                "{}: solution should pass",
                l.id
            );
            assert!(
                !crate::checker::check(l, &l.starter).passed(),
                "{}: starter should fail",
                l.id
            );
        }
    }
}
