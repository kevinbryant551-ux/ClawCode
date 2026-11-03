//! Compiles and runs the learner's code with the real `rustc`, then decides
//! whether the lesson's check passed.
//!
//! Safety model: this runs code the learner typed on their own machine, much like
//! `cargo run` would. We still keep it contained: a fresh temp folder per attempt,
//! no stdin, a wall-clock time limit (the process is killed when it runs over),
//! and a cap on how much output we keep.
use crate::lesson::{Check, Lesson};
use serde::Deserialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

pub const EDITION: &str = "2024";
const RUN_LIMIT: Duration = Duration::from_secs(5);
const COMPILE_LIMIT: Duration = Duration::from_secs(60);
const OUTPUT_CAP: usize = 64 * 1024;

/// One compiler message (error or warning).
#[derive(Debug, Clone)]
pub struct Diag {
    pub is_error: bool,
    pub code: Option<String>,
    pub message: String,
    /// 1-based line in the learner's file, if known.
    pub line: Option<usize>,
    /// True when the problem is inside the hidden test code rather than the learner's code.
    pub in_tests: bool,
    /// rustc's own full explanation, with the source snippet and arrows.
    pub rendered: String,
}

#[derive(Debug, Clone)]
pub struct TestFailure {
    pub name: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Passed {
        stdout: String,
        warnings: Vec<Diag>,
    },
    CompileError {
        diags: Vec<Diag>,
    },
    RuleFailed {
        why: String,
    },
    WrongOutput {
        expected: String,
        actual: String,
    },
    TestsFailed {
        failures: Vec<TestFailure>,
        passed: usize,
        total: usize,
    },
    Panicked {
        stderr: String,
    },
    TimedOut,
    ToolMissing(String),
}

impl Outcome {
    pub fn passed(&self) -> bool {
        matches!(self, Outcome::Passed { .. })
    }
}

/// Runs the complete check for `code` against `lesson`.
pub fn check(lesson: &Lesson, code: &str) -> Outcome {
    let dir = match WorkDir::new() {
        Ok(d) => d,
        Err(e) => return Outcome::ToolMissing(format!("could not create a temp folder: {e}")),
    };
    check_in(&lesson.check, code, &dir.0)
}

fn check_in(check: &Check, code: &str, dir: &Path) -> Outcome {
    let user_lines = code.lines().count();

    // 1. Does the learner's program compile on its own?
    let src = dir.join("main.rs");
    if let Err(e) = std::fs::write(&src, code) {
        return Outcome::ToolMissing(format!("could not write code to disk: {e}"));
    }
    let bin = dir.join("main");
    let compiled = match compile(&src, &bin, false, user_lines) {
        Ok(c) => c,
        Err(o) => return o,
    };
    if compiled.iter().any(|d| d.is_error) {
        return Outcome::CompileError {
            diags: compiled.into_iter().filter(|d| d.is_error).collect(),
        };
    }
    let warnings = compiled;

    // 2. Source rules (e.g. "use `mut`").
    let stripped = strip_comments(code);
    for rule in &check.must_contain {
        if !stripped.contains(&rule.text) {
            return Outcome::RuleFailed {
                why: rule.why.clone(),
            };
        }
    }
    for rule in &check.must_not_contain {
        if stripped.contains(&rule.text) {
            return Outcome::RuleFailed {
                why: rule.why.clone(),
            };
        }
    }

    // 3. Run the program and compare its output.
    let mut stdout = String::new();
    if let Some(expected) = check
        .expected_output
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        let run = run_limited(Command::new(&bin).current_dir(dir), RUN_LIMIT);
        match run {
            Err(o) => return o,
            Ok(r) if r.timed_out => return Outcome::TimedOut,
            Ok(r) if !r.success => return Outcome::Panicked { stderr: r.stderr },
            Ok(r) => {
                if normalize(&r.stdout) != normalize(expected) {
                    return Outcome::WrongOutput {
                        expected: normalize(expected),
                        actual: normalize(&r.stdout),
                    };
                }
                stdout = r.stdout;
            }
        }
    }

    // 4. Hidden tests.
    if let Some(tests) = check.tests.as_deref().filter(|s| !s.trim().is_empty()) {
        let tsrc = dir.join("main_test.rs");
        let full = format!(
            "{code}\n#[cfg(test)]\nmod clawcode_tests {{\n    use super::*;\n{tests}\n}}\n"
        );
        if let Err(e) = std::fs::write(&tsrc, full) {
            return Outcome::ToolMissing(format!("could not write tests to disk: {e}"));
        }
        let tbin = dir.join("main_test");
        match compile(&tsrc, &tbin, true, user_lines) {
            Err(o) => return o,
            Ok(d) if d.iter().any(|d| d.is_error) => {
                return Outcome::CompileError {
                    diags: d.into_iter().filter(|d| d.is_error).collect(),
                };
            }
            Ok(_) => {}
        }
        let run = run_limited(
            Command::new(&tbin)
                .args(["--test-threads=1", "--color", "never"])
                .current_dir(dir),
            RUN_LIMIT * 2,
        );
        match run {
            Err(o) => return o,
            Ok(r) if r.timed_out => return Outcome::TimedOut,
            Ok(r) => {
                let (passed, total, failures) = parse_test_output(&r.stdout);
                if !r.success || !failures.is_empty() {
                    let failures = if failures.is_empty() {
                        vec![TestFailure {
                            name: "tests".into(),
                            message: tail(&format!("{}{}", r.stdout, r.stderr), 30),
                        }]
                    } else {
                        failures
                    };
                    return Outcome::TestsFailed {
                        failures,
                        passed,
                        total,
                    };
                }
            }
        }
    }

    Outcome::Passed { stdout, warnings }
}

/// Trailing spaces on each line and trailing blank lines don't count.
pub fn normalize(s: &str) -> String {
    let s = s.replace("\r\n", "\n");
    let lines: Vec<&str> = s.lines().map(|l| l.trim_end()).collect();
    let mut out = lines.join("\n");
    while out.ends_with('\n') {
        out.pop();
    }
    out.trim_start_matches('\n').to_string()
}

fn tail(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

// ---------- compiling ----------

#[derive(Deserialize)]
struct JsonDiag {
    #[serde(rename = "$message_type", default)]
    message_type: String,
    message: String,
    code: Option<JsonCode>,
    level: String,
    #[serde(default)]
    spans: Vec<JsonSpan>,
    rendered: Option<String>,
}
#[derive(Deserialize)]
struct JsonCode {
    code: String,
}
#[derive(Deserialize)]
struct JsonSpan {
    line_start: usize,
    is_primary: bool,
}

fn compile(src: &Path, out: &Path, test: bool, user_lines: usize) -> Result<Vec<Diag>, Outcome> {
    let mut cmd = Command::new(rustc());
    cmd.arg("--edition")
        .arg(EDITION)
        .arg("--error-format=json")
        .arg("-A")
        .arg("dead_code")
        .arg("-C")
        .arg("debuginfo=0")
        .arg("-o")
        .arg(out)
        .arg(src)
        .current_dir(src.parent().unwrap());
    if test {
        cmd.arg("--test");
    }
    let r = run_limited(&mut cmd, COMPILE_LIMIT)?;
    if r.timed_out {
        return Err(Outcome::ToolMissing(
            "the compiler took too long (over a minute)".into(),
        ));
    }
    let dir_prefix = format!("{}/", src.parent().unwrap().display());
    let mut diags = Vec::new();
    for line in r.stderr.lines() {
        let Ok(d) = serde_json::from_str::<JsonDiag>(line) else {
            continue;
        };
        if !d.message_type.is_empty() && d.message_type != "diagnostic" {
            continue;
        }
        let is_error = d.level == "error";
        if !is_error && d.level != "warning" {
            continue;
        }
        // Summary lines like "aborting due to 2 previous errors" carry no span.
        if d.spans.is_empty() && d.message.starts_with("aborting due to") {
            continue;
        }
        if !is_error && d.spans.is_empty() {
            continue; // "1 warning emitted"
        }
        let line = d
            .spans
            .iter()
            .find(|s| s.is_primary)
            .or(d.spans.first())
            .map(|s| s.line_start);
        diags.push(Diag {
            is_error,
            code: d.code.map(|c| c.code),
            message: d.message,
            line,
            in_tests: line.is_some_and(|l| l > user_lines),
            rendered: d
                .rendered
                .unwrap_or_default()
                .replace(&dir_prefix, "")
                .replace("main_test.rs", "main.rs")
                .trim_end()
                .to_string(),
        });
    }
    if !r.success && !diags.iter().any(|d| d.is_error) {
        // Something odd happened (e.g. linker missing). Show the raw text.
        diags.push(Diag {
            is_error: true,
            code: None,
            message: "the compiler stopped unexpectedly".into(),
            line: None,
            in_tests: false,
            rendered: tail(&r.stderr, 20),
        });
    }
    Ok(diags)
}

pub fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Returns the rustc version string, or None if rustc can't be run.
pub fn rustc_version() -> Option<String> {
    let out = Command::new(rustc()).arg("--version").output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ---------- running with limits ----------

struct RunResult {
    stdout: String,
    stderr: String,
    success: bool,
    timed_out: bool,
}

fn run_limited(cmd: &mut Command, limit: Duration) -> Result<RunResult, Outcome> {
    let mut child = cmd
        .env("RUST_BACKTRACE", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            Outcome::ToolMissing(format!(
                "could not start `{}`: {e}. Is Rust installed? (see the README: rustup)",
                cmd.get_program().to_string_lossy()
            ))
        })?;
    let out_reader = spawn_reader(child.stdout.take().unwrap());
    let err_reader = spawn_reader(child.stderr.take().unwrap());
    let start = Instant::now();
    let (success, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.success(), false),
            Ok(None) if start.elapsed() > limit => {
                let _ = child.kill();
                let _ = child.wait();
                break (false, true);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(15)),
            Err(_) => break (false, false),
        }
    };
    Ok(RunResult {
        stdout: out_reader.join().unwrap_or_default(),
        stderr: err_reader.join().unwrap_or_default(),
        success,
        timed_out,
    })
}

/// Reads a pipe to the end but keeps at most OUTPUT_CAP bytes (an endless
/// `loop { println!() }` must not eat all memory).
fn spawn_reader<R: Read + Send + 'static>(mut r: R) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut buf = [0u8; 8192];
        while let Ok(n) = r.read(&mut buf) {
            if n == 0 {
                break;
            }
            if kept.len() < OUTPUT_CAP {
                let take = n.min(OUTPUT_CAP - kept.len());
                kept.extend_from_slice(&buf[..take]);
            }
        }
        String::from_utf8_lossy(&kept).into_owned()
    })
}

// ---------- test output ----------

/// Parses libtest's plain output into (passed, total, failures).
pub fn parse_test_output(out: &str) -> (usize, usize, Vec<TestFailure>) {
    let mut passed = 0;
    let mut failed_names = Vec::new();
    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("test ") {
            if let Some(name) = rest.strip_suffix(" ... ok") {
                let _ = name;
                passed += 1;
            } else if let Some(name) = rest.strip_suffix(" ... FAILED") {
                failed_names.push(name.to_string());
            }
        }
    }
    let mut failures = Vec::new();
    for name in &failed_names {
        let header = format!("---- {name} stdout ----");
        let msg = out
            .split_once(&header)
            .map(|(_, after)| {
                let block: Vec<&str> = after
                    .lines()
                    .skip_while(|l| l.trim().is_empty())
                    .take_while(|l| {
                        !l.starts_with("---- ")
                            && !l.starts_with("note: run with")
                            && !l.starts_with("failures:")
                            && !l.starts_with("stack backtrace:")
                    })
                    .collect();
                // Drop "thread '...' panicked at main_test.rs:12:5:" — the learner can't see that file.
                block
                    .into_iter()
                    .filter(|l| !l.starts_with("thread '"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();
        let short = name
            .trim_start_matches("clawcode_tests::")
            .replace('_', " ");
        failures.push(TestFailure {
            name: short,
            message: msg,
        });
    }
    (passed, passed + failed_names.len(), failures)
}

// ---------- comment stripping for rules ----------

/// Removes `//` and `/* */` comments, leaving string and char literals alone.
pub fn strip_comments(code: &str) -> String {
    let c: Vec<char> = code.chars().collect();
    let mut out = String::with_capacity(code.len());
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            '/' if c.get(i + 1) == Some(&'/') => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            }
            '/' if c.get(i + 1) == Some(&'*') => {
                let mut depth = 1;
                i += 2;
                while i < c.len() && depth > 0 {
                    if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                        depth += 1;
                        i += 2;
                    } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                out.push(' ');
            }
            '"' => {
                out.push('"');
                i += 1;
                while i < c.len() && c[i] != '"' {
                    if c[i] == '\\' && i + 1 < c.len() {
                        out.push(c[i]);
                        i += 1;
                    }
                    out.push(c[i]);
                    i += 1;
                }
                if i < c.len() {
                    out.push('"');
                    i += 1;
                }
            }
            ch => {
                out.push(ch);
                i += 1;
            }
        }
    }
    out
}

// ---------- temp folder ----------

struct WorkDir(PathBuf);

impl WorkDir {
    fn new() -> std::io::Result<Self> {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "clawcode-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p)?;
        Ok(WorkDir(p))
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ignores_trailing_space() {
        assert_eq!(normalize("a  \nb\n\n"), "a\nb");
    }

    #[test]
    fn strip_keeps_strings() {
        let s = strip_comments("let x = \"// not a comment\"; // gone\n/* also gone */ let y = 1;");
        assert!(s.contains("// not a comment"));
        assert!(!s.contains("gone"));
        assert!(s.contains("let y"));
    }

    #[test]
    fn parses_libtest_output() {
        let out = "running 2 tests\ntest clawcode_tests::adds ... ok\ntest clawcode_tests::subtracts ... FAILED\n\nfailures:\n\n---- clawcode_tests::subtracts stdout ----\n\nthread 'clawcode_tests::subtracts' panicked at main_test.rs:9:5:\nassertion `left == right` failed: sub(5, 3) should be 2\n  left: 8\n right: 2\nnote: run with `RUST_BACKTRACE=1`\n\nfailures:\n    clawcode_tests::subtracts\n";
        let (p, t, f) = parse_test_output(out);
        assert_eq!((p, t), (1, 2));
        assert_eq!(f[0].name, "subtracts");
        assert!(f[0].message.contains("sub(5, 3) should be 2"));
        assert!(!f[0].message.contains("panicked"));
    }

    fn check_of(out: Option<&str>, tests: Option<&str>) -> Check {
        Check {
            expected_output: out.map(String::from),
            tests: tests.map(String::from),
            ..Default::default()
        }
    }

    fn run(check: &Check, code: &str) -> Outcome {
        let d = WorkDir::new().unwrap();
        check_in(check, code, &d.0)
    }

    #[test]
    fn real_compile_pass_and_fail() {
        let c = check_of(Some("hi\n"), None);
        assert!(run(&c, "fn main() { println!(\"hi\"); }").passed());
        assert!(matches!(
            run(&c, "fn main() { println!(\"ho\"); }"),
            Outcome::WrongOutput { .. }
        ));
        match run(&c, "fn main() { let x = 1; x = 2; println!(\"{x}\"); }") {
            Outcome::CompileError { diags } => assert_eq!(diags[0].code.as_deref(), Some("E0384")),
            o => panic!("expected compile error, got {o:?}"),
        }
    }

    #[test]
    fn infinite_loop_times_out() {
        let c = check_of(Some("x"), None);
        assert!(matches!(
            run(&c, "fn main() { loop { println!(\"spam\"); } }"),
            Outcome::TimedOut
        ));
    }

    #[test]
    fn hidden_tests_run() {
        let c = check_of(
            None,
            Some("#[test]\nfn two() { assert_eq!(double(1), 2, \"double(1) should be 2\"); }"),
        );
        assert!(run(&c, "fn double(x: i32) -> i32 { x * 2 }\nfn main() {}").passed());
        match run(&c, "fn double(x: i32) -> i32 { x * 3 }\nfn main() {}") {
            Outcome::TestsFailed { failures, .. } => {
                assert!(failures[0].message.contains("double(1) should be 2"))
            }
            o => panic!("expected test failure, got {o:?}"),
        }
    }
}
