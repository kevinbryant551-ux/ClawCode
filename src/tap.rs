//! Tap puzzles: the button-pushing version of a lesson's coding challenge,
//! used by the Godot (Android) app where typing code on a phone is no fun.
//!
//! A lesson can describe its own puzzles with `[[tap]]` tables (see
//! docs/LESSON_FORMAT.md). If it has none, [`auto_taps`] builds fill-in-the-blank
//! puzzles by comparing the starter code with the solution: the lines the
//! learner had to write become the lines with blanks.
use serde::Deserialize;

/// The marker for a blank in a hand-written `fill` puzzle.
pub const BLANK: &str = "___";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TapKind {
    /// Code with `___` blanks; tap tiles to fill them in order.
    Fill,
    /// Lines of code, shuffled; tap them in the right order.
    Order,
}

/// One tap puzzle as written in a lesson file.
#[derive(Debug, Clone, Deserialize)]
pub struct Tap {
    pub kind: TapKind,
    #[serde(default)]
    pub prompt: String,
    /// For `fill`: code containing `___` blanks.
    /// For `order`: the lines in the correct order (blank lines are ignored).
    pub code: String,
    /// For `fill`: the correct tile for each blank, in order.
    #[serde(default)]
    pub answer: Vec<String>,
    /// Wrong tiles (`fill`) or wrong lines (`order`) mixed in to make it a puzzle.
    #[serde(default)]
    pub decoys: Vec<String>,
    /// Shown after the puzzle is solved.
    #[serde(default)]
    pub explain: String,
}

impl Tap {
    pub fn order_lines(&self) -> Vec<String> {
        self.code
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim_end().to_string())
            .collect()
    }

    /// Checks the puzzle is solvable. Returns a message if it is not.
    pub fn validate(&self) -> Result<(), String> {
        match self.kind {
            TapKind::Fill => {
                let blanks = self.code.matches(BLANK).count();
                if blanks == 0 {
                    return Err("a fill puzzle needs at least one ___ blank".into());
                }
                if blanks != self.answer.len() {
                    return Err(format!(
                        "has {blanks} blanks but {} answers",
                        self.answer.len()
                    ));
                }
                if let Some(d) = self.decoys.iter().find(|d| self.answer.contains(d)) {
                    return Err(format!("decoy {d:?} is also a correct answer"));
                }
            }
            TapKind::Order => {
                let lines = self.order_lines();
                if lines.len() < 2 {
                    return Err("an order puzzle needs at least two lines".into());
                }
                if let Some(d) = self.decoys.iter().find(|d| lines.contains(d)) {
                    return Err(format!("decoy line {d:?} is also a correct line"));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tokenizer (also used by the syntax highlighter in godot_export.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokKind {
    Space,
    Comment,
    Str,
    Char,
    Lifetime,
    Number,
    Ident,
    Punct,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tok {
    pub kind: TokKind,
    pub text: String,
}

const MULTI_PUNCT: &[&str] = &[
    "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "..",
];

/// Splits one line of Rust into tokens. Joining every token's text gives the line back.
pub fn tokenize(line: &str) -> Vec<Tok> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let push = |out: &mut Vec<Tok>, kind, s: &[char]| {
        out.push(Tok {
            kind,
            text: s.iter().collect(),
        })
    };
    while i < chars.len() {
        let c = chars[i];
        let start = i;
        if c.is_whitespace() {
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            push(&mut out, TokKind::Space, &chars[start..i]);
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            push(&mut out, TokKind::Comment, &chars[start..]);
            i = chars.len();
        } else if c == '"' {
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i = (i + 1).min(chars.len());
            push(&mut out, TokKind::Str, &chars[start..i]);
        } else if c == '\'' {
            // 'a' or '\n' is a char; 'static is a lifetime.
            if chars.get(i + 1) == Some(&'\\') {
                i += 2;
                while i < chars.len() && chars[i] != '\'' {
                    i += 1;
                }
                i = (i + 1).min(chars.len());
                push(&mut out, TokKind::Char, &chars[start..i]);
            } else if chars.get(i + 2) == Some(&'\'') {
                i += 3;
                push(&mut out, TokKind::Char, &chars[start..i]);
            } else {
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                push(&mut out, TokKind::Lifetime, &chars[start..i]);
            }
        } else if c.is_ascii_digit() {
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric()
                    || chars[i] == '_'
                    || (chars[i] == '.' && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit())))
            {
                i += 1;
            }
            push(&mut out, TokKind::Number, &chars[start..i]);
        } else if c.is_alphabetic() || c == '_' {
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            push(&mut out, TokKind::Ident, &chars[start..i]);
        } else {
            let rest: String = chars[i..].iter().take(3).collect();
            let len = MULTI_PUNCT
                .iter()
                .find(|p| rest.starts_with(*p))
                .map_or(1, |p| p.len());
            i += len;
            push(&mut out, TokKind::Punct, &chars[start..i]);
        }
    }
    out
}

pub const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "false", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
    "Self", "static", "struct", "super", "trait", "true", "type", "use", "where", "while",
];

// ---------------------------------------------------------------------------
// Automatic puzzles
// ---------------------------------------------------------------------------

/// One piece of a fill puzzle's code: plain text or a numbered blank.
#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    Text(String),
    Blank(usize),
}

/// A fill puzzle ready for the app: code pieces, answers and decoys.
#[derive(Debug, Clone)]
pub struct FillPuzzle {
    pub prompt: String,
    pub parts: Vec<Part>,
    pub answer: Vec<String>,
    pub decoys: Vec<String>,
    pub explain: String,
}

/// Splits hand-written `___` code into parts.
pub fn parts_from_marked(code: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    for (i, piece) in code.trim_matches('\n').split(BLANK).enumerate() {
        if i > 0 {
            parts.push(Part::Blank(i - 1));
        }
        if !piece.is_empty() {
            parts.push(Part::Text(piece.to_string()));
        }
    }
    parts
}

/// Tokens worth blanking out: the "Rust-y" bits a learner should recognise.
fn blank_score(t: &Tok) -> u32 {
    match t.kind {
        TokKind::Ident if t.text == "todo" || t.text == "_" => 0,
        TokKind::Ident if t.text == "let" => 3,
        TokKind::Ident if KEYWORDS.contains(&t.text.as_str()) => 5,
        TokKind::Ident if ["Some", "None", "Ok", "Err"].contains(&t.text.as_str()) => 5,
        TokKind::Ident => 3,
        TokKind::Punct
            if [
                "&", "?", "=>", "->", "..", "..=", "==", "!=", "+=", "-=", "*", "|",
            ]
            .contains(&t.text.as_str()) =>
        {
            4
        }
        TokKind::Punct if t.text == "::" => 2,
        TokKind::Number | TokKind::Str | TokKind::Char => 4,
        TokKind::Lifetime => 2,
        _ => 0,
    }
}

/// Look-alike tiles for a correct answer, so decoys are plausible mistakes.
fn confusables(answer: &str) -> Vec<String> {
    let pairs: &[(&str, &[&str])] = &[
        ("&", &["&mut", "*"]),
        ("&mut", &["&", "mut"]),
        ("mut", &["const", "&"]),
        ("let", &["var", "mut"]),
        ("Some", &["None", "Ok"]),
        ("None", &["Some", "null"]),
        ("Ok", &["Err", "Some"]),
        ("Err", &["Ok", "None"]),
        ("==", &["=", "!="]),
        ("!=", &["==", "="]),
        ("=>", &["->", "="]),
        ("->", &["=>", ":"]),
        ("::", &[".", ":"]),
        ("..", &["..=", ","]),
        ("..=", &["..", "="]),
        ("+=", &["=", "-="]),
        ("-=", &["+=", "="]),
        ("?", &["!", ";"]),
        ("for", &["while", "loop"]),
        ("in", &["of", "="]),
        ("match", &["if", "switch"]),
        ("if", &["match", "while"]),
        ("else", &["elif", "then"]),
        ("fn", &["func", "def"]),
        ("impl", &["trait", "struct"]),
        ("trait", &["impl", "struct"]),
        ("struct", &["enum", "class"]),
        ("enum", &["struct", "match"]),
        ("true", &["false"]),
        ("false", &["true"]),
        ("String", &["&str", "str"]),
        ("str", &["String", "char"]),
        ("i32", &["u32", "f64"]),
        ("u32", &["i32", "usize"]),
        ("f64", &["i32", "u32"]),
        ("usize", &["u32", "i32"]),
        ("char", &["String", "u8"]),
        ("bool", &["i32", "char"]),
        ("Vec", &["Array", "HashMap"]),
        ("push", &["pop", "insert"]),
        ("iter", &["into_iter", "next"]),
        ("map", &["filter", "for_each"]),
        ("filter", &["map", "find"]),
        ("unwrap", &["expect", "unwrap_or"]),
        ("return", &["break", "yield"]),
        ("self", &["Self", "this"]),
        ("Self", &["self", "Type"]),
        ("println", &["print", "format"]),
        ("format", &["println", "write"]),
        ("to_string", &["as_str", "clone"]),
        ("clone", &["copy", "to_string"]),
        ("len", &["size", "count"]),
    ];
    if let Some((_, alts)) = pairs.iter().find(|(a, _)| *a == answer) {
        return alts.iter().map(|s| s.to_string()).collect();
    }
    if let Ok(n) = answer.parse::<i64>() {
        return vec![(n + 1).to_string(), (n - 1).max(0).to_string()];
    }
    Vec::new()
}

/// Longest common subsequence alignment of two line lists, compared trimmed.
/// Returns, for each line of `b`, whether it also appears (aligned) in `a`,
/// and a merged list that keeps comment-only lines from `a` that `b` dropped.
fn align(a: &[&str], b: &[&str]) -> Vec<(String, bool)> {
    let (n, m) = (a.len(), b.len());
    let mut dp = vec![vec![0u16; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i].trim() == b[j].trim() {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < n || j < m {
        if i < n && j < m && a[i].trim() == b[j].trim() {
            out.push((b[j].to_string(), false));
            i += 1;
            j += 1;
        } else if j < m && (i == n || dp[i][j + 1] >= dp[i + 1][j]) {
            out.push((b[j].to_string(), true)); // new in the solution
            j += 1;
        } else {
            // Only in the starter: keep its comments, they explain the task.
            let t = a[i].trim();
            if t.starts_with("//")
                && !t.contains("todo")
                && !t.to_lowercase().contains("replace each")
            {
                out.push((a[i].to_string(), false));
            }
            i += 1;
        }
    }
    out
}

/// Builds fill puzzles from the difference between starter and solution.
/// Normal lessons get one puzzle with up to 6 blanks; boss lessons get up to
/// 3 puzzles of up to 5 blanks (each blank is one boss hit point).
pub fn auto_taps(starter: &str, solution: &str, goal: &str, boss: bool) -> Vec<FillPuzzle> {
    let a: Vec<&str> = starter.trim_matches('\n').lines().collect();
    let b: Vec<&str> = solution.trim_matches('\n').lines().collect();
    let merged = align(&a, &b);
    // Starter-only tokens make good decoys (they are what the learner started with).
    let starter_tokens: Vec<String> = a
        .iter()
        .filter(|l| !b.iter().any(|s| s.trim() == l.trim()))
        .flat_map(|l| tokenize(l))
        .filter(|t| blank_score(t) > 0)
        .map(|t| t.text)
        .collect();
    let changed: Vec<usize> = merged
        .iter()
        .enumerate()
        .filter(|(_, (l, new))| {
            *new && !l.trim().is_empty() && best_token(l, &starter_tokens, &[]).is_some()
        })
        .map(|(i, _)| i)
        .collect();
    if changed.is_empty() {
        return Vec::new();
    }
    let (rounds, per_round) = if boss { (3, 5) } else { (1, 6) };
    let rounds = rounds.min(changed.len());
    let group = changed.len().div_ceil(rounds);

    let mut puzzles = Vec::new();
    for chunk in changed.chunks(group) {
        // Spread the blanks evenly over this chunk's changed lines.
        let picks: Vec<usize> = if chunk.len() <= per_round {
            chunk.to_vec()
        } else {
            (0..per_round)
                .map(|k| chunk[k * chunk.len() / per_round])
                .collect()
        };
        // Show a window of code around the blanks, with a little context.
        let first = chunk[0].saturating_sub(3);
        let last = (chunk[chunk.len() - 1] + 3).min(merged.len() - 1);
        let mut text = String::new();
        let mut parts = Vec::new();
        let mut answer = Vec::new();
        if first > 0 {
            text.push_str("// ...\n");
        }
        for (idx, (line, _)) in merged.iter().enumerate().take(last + 1).skip(first) {
            if picks.contains(&idx) {
                let toks = tokenize(line);
                let bi = best_token(line, &starter_tokens, &answer).unwrap();
                for (ti, t) in toks.iter().enumerate() {
                    if ti == bi {
                        if !text.is_empty() {
                            parts.push(Part::Text(std::mem::take(&mut text)));
                        }
                        parts.push(Part::Blank(answer.len()));
                        answer.push(t.text.clone());
                    } else {
                        text.push_str(&t.text);
                    }
                }
            } else {
                text.push_str(line);
            }
            text.push('\n');
        }
        if last + 1 < merged.len() {
            text.push_str("// ...\n");
        }
        let text = text.trim_end_matches('\n').to_string();
        if !text.is_empty() {
            parts.push(Part::Text(text));
        }

        let mut decoys: Vec<String> = Vec::new();
        let add = |d: String, decoys: &mut Vec<String>| {
            if !d.is_empty() && !answer.contains(&d) && !decoys.contains(&d) && d != "todo" {
                decoys.push(d);
            }
        };
        for ans in &answer {
            for c in confusables(ans) {
                add(c, &mut decoys);
            }
        }
        for t in &starter_tokens {
            add(t.clone(), &mut decoys);
        }
        for d in ["&", "mut", "let", "fn", "match", "Some", "?", "self"] {
            add(d.to_string(), &mut decoys);
        }
        decoys.truncate(answer.len().clamp(3, 5));

        puzzles.push(FillPuzzle {
            prompt: goal.to_string(),
            parts,
            answer,
            decoys,
            explain: String::new(),
        });
    }
    puzzles
}

/// Index of the most interesting token on a line, if any.
/// Tokens the learner had to add (not in the starter's removed lines) score higher.
/// Tokens already used as answers score lower, so a puzzle has variety.
fn best_token(line: &str, removed: &[String], used: &[String]) -> Option<usize> {
    let toks = tokenize(line);
    let mut best: Option<(usize, u32)> = None;
    for (i, t) in toks.iter().enumerate() {
        let mut s = blank_score(t);
        if s > 0 && !removed.contains(&t.text) {
            s += 3;
        }
        if s > 1 && used.contains(&t.text) {
            s -= 1 + 2 * used.iter().filter(|u| **u == t.text).count().min(3) as u32;
            s = s.max(1);
        }
        if s > 0 && best.is_none_or(|(_, bs)| s > bs) {
            best = Some((i, s));
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_round_trips() {
        let line = r#"    let s: &'static str = "hi \"you\""; // done 'x' 1..=3"#;
        let joined: String = tokenize(line).into_iter().map(|t| t.text).collect();
        assert_eq!(joined, line);
        let toks = tokenize("x..=3 'a' 2.5 a::b");
        let texts: Vec<_> = toks.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            ["x", "..=", "3", " ", "'a'", " ", "2.5", " ", "a", "::", "b"]
        );
    }

    #[test]
    fn marked_parts() {
        let p = parts_from_marked("let ___ x = ___;");
        assert_eq!(
            p,
            vec![
                Part::Text("let ".into()),
                Part::Blank(0),
                Part::Text(" x = ".into()),
                Part::Blank(1),
                Part::Text(";".into()),
            ]
        );
    }

    #[test]
    fn auto_puzzle_blanks_the_new_code() {
        let starter = "fn main() {\n    let lives = 3;\n    lives = 2;\n}\n";
        let solution = "fn main() {\n    let mut lives = 3;\n    lives = 2;\n}\n";
        let p = auto_taps(starter, solution, "goal", false);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].answer, vec!["mut"]);
        assert!(!p[0].decoys.is_empty());
        assert!(p[0].parts.contains(&Part::Blank(0)));
    }
}
