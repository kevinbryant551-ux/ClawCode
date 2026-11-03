# ClawCode lesson format

Each lesson is one TOML file in `lessons/`, named `NN-slug.toml` (for example `03-types.toml`).
Lessons are shown in the order of their `order` field. A lesson unlocks when the one before it is passed.

## Fields

```toml
id = "03-types"            # must equal the file name without .toml
order = 3                  # position in the course (1, 2, 3, ...)
chapter = "The Basics"     # chapters group lessons on the map screen
title = "Types of Things"  # short, fun title
xp = 60                    # XP for passing (50-120; harder lessons give more)
concepts = ["i32", "f64", "bool", "char", "type annotations"]   # keywords taught; shown as tags

# The explanation, written in a small Markdown subset (see below).
intro = '''
...
'''

goal = "One sentence that says exactly what the learner must do."

# Code that opens in the editor. It must NOT pass the check as given.
starter = '''
fn main() {
}
'''

# A known-good answer. It MUST pass the check. Shown only after several failed tries.
solution = '''
fn main() {
}
'''

# 2 to 4 hints, from gentle nudge to nearly the answer.
hints = [
  "Gentle nudge.",
  "More specific.",
  "Almost the answer.",
]

# What Pinch the crab says when the learner passes.
success = "Short celebration that also restates the key idea."

[check]
# Optional: the program's printed output must equal this.
# Comparison ignores trailing spaces on each line and trailing blank lines.
expected_output = '''
Hello, Rust!
'''

# Optional: Rust test functions. They are pasted after the learner's code inside
#   #[cfg(test)] mod clawcode_tests { use super::*; <tests here> }
# so they can call the learner's functions and types directly.
tests = '''
#[test]
fn adds_small_numbers() {
    assert_eq!(add(2, 3), 5, "add(2, 3) should be 5");
}
'''

# Optional source-code rules. `text` is a plain substring searched for in the
# learner's code with comments removed. `why` is shown if the rule fails.
[[check.must_contain]]
text = "mut"
why = "This lesson is about `mut`, so make the variable mutable."

[[check.must_not_contain]]
text = "unwrap()"
why = "Try handling the None case with `match` instead of `unwrap()`."
```

At least one of `expected_output` or `tests` must be present. Both may be present.
Always include a clear message as the third argument of `assert_eq!` / `assert!` in tests; the learner sees it.

## The Markdown subset used in `intro`

- `# Heading` and `## Subheading`
- Blank line between paragraphs
- `inline code` in backticks, **bold** with double asterisks
- `- ` bullet lists
- Fenced code blocks with three backticks (` ```rust ` ... ` ``` `)
- A line starting with `> ` is a tip spoken by Pinch the crab
- `Term :: definition` on its own line renders as a highlighted keyword definition (use for new vocabulary)

## How lessons are checked

The engine writes the learner's code to a temp folder and runs the real compiler:

- Output check: `rustc --edition 2024 main.rs -o main`, then runs `./main` (5 second limit) and compares stdout.
- Tests check: appends the test module, `rustc --edition 2024 --test main.rs`, runs the test binary.

To validate a lesson by hand:

```sh
cargo run -- verify            # checks every lesson: solution passes, starter fails
cargo run -- verify 03-types   # just one lesson
```
