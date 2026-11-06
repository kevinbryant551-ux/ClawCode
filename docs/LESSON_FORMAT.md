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

## Cards (bite-sized reading)

Split `intro` into short cards with a line containing only `---`. The game shows
one card at a time in the "Learn" phase, so each card should be one idea
(40-120 words). Example: card 1 explains the concept, card 2 shows code, card 3
gives a tip. 3 to 5 cards per lesson is ideal.

## Quizzes (quick checks between cards)

```toml
[[quiz]]
after_card = 1          # shown after this card (1-based). Must be <= number of cards.
question = '''
What does this print?
```rust
let x = 2 + 3;
println!("{x}");
```
'''
choices = ["2", "5", "x", "It doesn't compile"]   # 2 to 4 choices
answer = 1              # index of the correct choice, counting from 0
explain = "Shown after answering: why the right answer is right."
```

Use 2 or 3 quizzes per lesson. Mix kinds: "what does this print?", "which line
has the bug?", "what does this keyword mean?". Make wrong choices plausible
mistakes a beginner would actually make. Vary which index is correct.

## Boss lessons

A lesson with a `[boss]` table is a boss fight. Each hidden test is one hit
point: the boss's HP bar drops as more tests pass. Boss lessons must use `tests`
(5 to 8 tests), and usually review the whole chapter.

```toml
[boss]
name = "Borrowzilla"
art = '''
(at most 8 lines, at most 30 columns wide, plain ASCII only)
'''
taunt = "What the boss says when the fight starts."
defeat = "What it says when beaten."
```

## Tap puzzles (for the Android app)

The Godot phone app (`godot/`) can't run `rustc`, and typing code on a phone
is no fun, so it replaces the coding challenge with **tap puzzles**. The
terminal game ignores these tables. Add 1 to 3 per lesson:

```toml
[[tap]]
kind = "fill"           # code with ___ blanks; the player taps tiles to fill them in order
prompt = "Unlock the variable so it can change."   # optional, defaults to `goal`
code = '''
let ___ lives = 3;
lives = ___;
'''
answer = ["mut", "2"]               # one per ___, in order
decoys = ["const", "let", "\"2\""]   # plausible wrong tiles (must not be correct answers)
explain = "Shown when solved."

[[tap]]
kind = "order"          # the lines are shuffled; the player taps them top to bottom
code = '''
fn main() {
    println!("Hi!");
}
'''
decoys = ['    print("Hi!")']   # wrong lines mixed in (optional)
explain = "Shown when solved."
```

- Blanks are written `___` (three underscores). The number of blanks must
  equal the number of `answer` entries.
- In `order` puzzles, blank lines are ignored and identical lines (like two `}`)
  are interchangeable.
- In a boss lesson, each correct blank (or line) is one boss hit point.
- Lessons without `[[tap]]` get **automatic** fill puzzles built from the
  difference between `starter` and `solution`. Hand-made ones teach better.

After editing, run `cargo run -- export-godot` to refresh `godot/data/lessons.json`
and `cargo test` to check every puzzle is solvable.
