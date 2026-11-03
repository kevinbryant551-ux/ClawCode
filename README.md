# ClawCode

```
  (\/)  (\/)
  ( o_o )   Learn Rust, one small pinch at a time.
```

ClawCode is a game you play in your **terminal** that teaches you the
**Rust** programming language. Each lesson gives you a short explanation, a
goal, and a code editor right inside the game. When you press **Ctrl+R**, your
code is checked by the *real* Rust compiler (not an imitation), so what you
learn is exactly how Rust behaves on your own computer. You earn **XP**, climb
**levels**, keep **daily streaks**, and collect **badges**. When the compiler
complains, **Pinch the crab** (your guide) translates the error into plain
English and tells you what to try next.

There are 10 lessons in 6 chapters:

| Chapter | Lessons |
|---|---|
| Getting Started | 1. Hello, Tide Pool! - 2. Keeping Things in Buckets |
| The Basics | 3. Types of Things - 4. Build Your Own Moves |
| Control Flow | 5. Tide Decisions - 6. Round and Round |
| Ownership | 7. Who Owns the Shell? |
| Building Your Own Types | 8. Build a Crab - 9. Weather or Not |
| Handling the Unexpected | 10. Maybe, Maybe Not |

## Contents

1. [Key terms](#key-terms)
2. [Install on Linux](#install-on-linux)
3. [Run the game](#run-the-game)
4. [How to play](#how-to-play)
5. [XP, levels, badges, streaks](#xp-levels-badges-and-streaks)
6. [Other commands](#other-commands)
7. [How checking works (and safety)](#how-checking-works-and-safety)
8. [Writing your own lessons](#writing-your-own-lessons)
9. [Project layout](#project-layout)
10. [Troubleshooting](#troubleshooting)
11. [Next steps](#next-steps)

## Key terms

Skim this now, and come back whenever a word feels fuzzy. Each term is also
explained where it first shows up below.

| Term | Plain-English meaning |
|---|---|
| **Terminal** | A window where you type commands instead of clicking. Also called the console or command line. Look for "Terminal", "Konsole", "GNOME Terminal" or similar in your app menu. |
| **Shell** | The program inside the terminal that reads your commands (usually `bash`). |
| **TUI** | "Text User Interface": an app with menus, boxes and colors that runs *inside* a terminal. ClawCode is a TUI. |
| **Rust** | The programming language you are learning. |
| **Compile** | Translate human-readable source code into machine code your computer can run. |
| **Compiler** | The program that compiles. Rust's compiler is called `rustc`. |
| **`rustc`** | The Rust compiler. ClawCode runs it for you each time you check a lesson. |
| **Binary** | A finished, runnable program file (what the compiler produces). ClawCode itself becomes a binary named `clawcode`. |
| **Linker** | The last build step: it glues compiled pieces into one binary. Rust uses your system's C linker, a program called `cc`, for this. |
| **Cargo** | Rust's build tool and package manager. It downloads libraries, calls `rustc` for you, and runs your program. |
| **Crate** | A Rust package: either a program or a library. ClawCode is one crate, and it uses other people's crates (like `ratatui`, which draws the boxes). |
| **`rustup`** | The official installer and updater for Rust. It installs `rustc`, `cargo` and friends. |
| **PATH** | A list of folders your shell searches when you type a command name. If a program's folder isn't on PATH, the shell says "command not found". |
| **TOML** | A simple text format for settings and data (`name = "value"`). `Cargo.toml` and every ClawCode lesson file use it. |
| **`$EDITOR`** | An *environment variable* (a named setting your shell remembers) holding the name of your favorite text editor, such as `nano` or `vim`. |
| **XP** | Experience points, the game's score. |
| **Streak** | How many days in a row you passed at least one lesson. |

## Install on Linux

You need three things: Rust (via `rustup`), a C linker, and the ClawCode code.

### 1. Install Rust with rustup

Open a terminal and paste:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

(`curl` downloads the official installer script from the internet and hands it
to `sh`, which runs it. The extra flags insist on a secure HTTPS connection.)
Choose the default option when asked. This installs `rustc`, `cargo` and
`rustup` into a folder called `~/.cargo/bin` (`~` means your home folder).

### 2. Make the new commands available

Your current terminal doesn't know about the new folder yet (it isn't on your
PATH). Fix that for this window:

```sh
source "$HOME/.cargo/env"
```

`source` runs a file's commands inside the current shell. That file adds
`~/.cargo/bin` to PATH. New terminals will do it automatically.

### 3. Check that it worked

```sh
rustc --version
cargo --version
```

Each should print a version number. If you see "command not found", see
[Troubleshooting](#troubleshooting).

### 4. Install a C linker

Before Rust can turn your code into a runnable binary, it calls the linker
`cc`. Most desktop Linux installs don't include it. Pick your distribution:

```sh
# Ubuntu, Debian, Linux Mint
sudo apt install build-essential

# Fedora
sudo dnf install gcc

# Arch
sudo pacman -S base-devel
```

(`sudo` means "run as administrator"; it will ask for your password. Nothing
shows while you type it, which is normal.)

### 5. Get the ClawCode code

```sh
git clone https://github.com/kevinbryant551-ux/ClawCode.git
cd ClawCode
```

`git clone` copies the project into a new folder called `ClawCode`, and `cd`
(change directory) moves your terminal into it. If you don't have `git`, install it with your package manager
(for example `sudo apt install git`), or download the project as a ZIP and unpack it.

## Run the game

Make your terminal window **at least 80 columns by 24 rows** (about the size of
a default terminal window), then, from inside the `clawcode` folder:

```sh
cargo run --release
```

The first time, Cargo downloads the crates ClawCode depends on and compiles
them, which takes a minute or so. `--release` asks for an optimized build, so
the game feels snappy. Later runs start instantly.

### Optional: install it as a command

If you'd rather type just `clawcode` from anywhere:

```sh
cargo install --path .
clawcode
```

`cargo install` builds the project and copies the finished binary into
`~/.cargo/bin`, the same folder that holds `rustc` and `cargo`. Since the
rustup installer already put that folder on your PATH, the shell can find
`clawcode` right away. The lessons are built into the binary, so it keeps
working even if you move or delete the project folder.

## How to play

When the game starts you see the **lesson map**: all lessons grouped by chapter.
Pick one and press Enter. The **lesson screen** has the explanation and goal
on the left and your code editor on the right, with Pinch's messages below.

### Map screen keys

| Key | What it does |
|---|---|
| `↑` / `↓` (or `k` / `j`) | Move the selection |
| `Enter` | Start the selected lesson |
| `b` | Show your badges |
| `F1` or `?` | Help |
| `q` or `Esc` | Quit (progress is saved) |

### Lesson screen keys

Press **F1** at any time to see this list inside the game.

| Key | What it does |
|---|---|
| `Ctrl+R` or `F5` | Run and check your code with the real Rust compiler |
| `Ctrl+T` or `F2` | Get a tip (each new tip costs a bit of XP) |
| `Ctrl+P` or `F4` | Show the solution (after 3 tries or all tips) |
| `Ctrl+E` | Open your code in your own editor (`$VISUAL` or `$EDITOR`; falls back to `nano`, then `vi`) |
| `Ctrl+S` | Save (it also saves automatically) |
| `Ctrl+O` or `F8` | Reset the code to the starter version |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo in the editor |
| `PgUp` / `PgDn` | Scroll the lesson text |
| `Ctrl+↑↓` / `Alt+↑↓` | Scroll Pinch's messages |
| `Esc` | Back to the lesson map |
| `Ctrl+C` | Quit (progress is saved) |

The F-keys exist because some terminals swallow certain Ctrl combinations.

The editor helps a little: pressing Enter keeps your current indentation and
adds one more level after an opening `{`, and typing `}` at the start of an
indented line steps back one level. Your unfinished code is saved as a draft,
so you can quit and pick up where you left off.

### Unlocking lessons

Lesson 1 is open from the start. Each later lesson unlocks when you pass the
one before it. Already-passed lessons can be replayed any time. To skip the
ordering and open everything, start with:

```sh
cargo run --release -- --unlock-all
# or, if installed:
clawcode --unlock-all
```

Setting the environment variable `CLAWCODE_UNLOCK_ALL` to anything does the
same thing.

## XP, levels, badges, and streaks

### How XP is calculated

Each lesson has a **base XP** (50 for lesson 1, up to 120 for lesson 10; 850
in total across all ten). When you pass a lesson for the first time:

| Rule | Effect |
|---|---|
| Passing | + base XP |
| Passed on your very first try | + 50% of base |
| Each tip you revealed | - 10% of base per tip (the first-try bonus still counts) |
| You peeked at the solution | - 50% of base |
| Streak of 2 or more days | + 5 XP per streak day, up to + 25 |
| Minimum | You always earn at least 25% of base |

**Replays give no XP**, though passing any lesson still counts toward your
streak. Example: a 100 XP lesson passed on the first try with no tips earns
150 XP.

### Levels

| Level | Title | XP needed |
|---|---|---|
| 1 | Hatchling | 0 |
| 2 | Tide-pool Tinkerer | 100 |
| 3 | Shell Shaper | 250 |
| 4 | Reef Ranger | 450 |
| 5 | Borrow Buddy | 700 |
| 6 | Ownership Oracle | 1000 |
| 7 | Trait Tamer | 1400 |
| 8 | Deep-sea Debugger | 1900 |
| 9 | Rustacean | 2500 |

The ten lessons add up to 850 base XP (more with bonuses), so the higher
levels are there for future lessons you or others add.

### Badges

| Badge | How to earn it |
|---|---|
| First Steps | Pass your first lesson |
| Flawless | Pass a lesson on the very first try |
| No Peeking | Pass 3 lessons without any hints |
| Persistent Pincer | Pass a lesson after 5 or more tries |
| On a Roll | Keep a 3-day streak |
| Week Warrior | Keep a 7-day streak |
| Borrow Boss | Conquer the ownership lesson |
| Halfway Reef | Pass half of the lessons |
| Rustacean Graduate | Pass every lesson |

### Streaks

A streak counts the days *in a row* on which you passed at least one lesson
(by your local calendar day). Pass two lessons in one day and it still counts
as one day. Skip a whole day and the streak starts again at 1.

### Where progress is saved

Everything (XP, finished lessons, badges, streak, drafts) is stored in one
small text file:

```
~/.local/share/clawcode/progress.json
```

To keep it somewhere else, set `CLAWCODE_PROGRESS` to a file path:

```sh
CLAWCODE_PROGRESS=/tmp/test-progress.json clawcode
```

To start over, run `clawcode reset` (or `cargo run -- reset`). It shows the
file path and deletes it only if you type `yes`.

## Other commands

You can use these with `cargo run --release --` in front (the `--` separates
Cargo's options from ClawCode's), or with plain `clawcode` if installed.

| Command | What it does |
|---|---|
| `clawcode` | Start the game |
| `clawcode --unlock-all` | Start with every lesson unlocked |
| `clawcode list` | Print all lessons, your level, XP and streak; `✔` marks passed lessons |
| `clawcode check <id> <file.rs>` | Check a `.rs` file against a lesson, without the in-game editor |
| `clawcode verify [id]` | For lesson authors: solutions must pass, starters must fail |
| `clawcode reset` | Delete all saved progress (asks first) |
| `clawcode --help` | Show the usage text |

`check` is handy if you prefer your own editor. Find a lesson id with `list`
(for example `01-hello`), write your code in a file, then:

```sh
clawcode check 01-hello my_solution.rs
```

It prints the same kind of feedback as the game and exits with success only if
the lesson passes. (It checks your code but doesn't award XP.)

## How checking works (and safety)

When you press Ctrl+R, ClawCode:

1. Creates a fresh temporary folder and writes your code to `main.rs` there.
2. Compiles it with the real compiler: `rustc --edition 2024 main.rs`. Compiler
   errors are handed to Pinch to explain. (Warnings are allowed.)
3. Applies the lesson's **source rules**, if any, such as "must use `mut`" or
   "must not use `unwrap()`". Comments are ignored, so a rule can't be
   satisfied by writing it in a comment.
4. If the lesson expects printed output, runs your program and compares its
   output with the expected text. Trailing spaces on each line and trailing
   blank lines are ignored; everything else must match.
5. If the lesson has **hidden tests**, it appends a test module (`mod
   clawcode_tests`) to a copy of your code, compiles it with `rustc --test`,
   and runs the tests. Failures are reported in plain language.

Limits: your program gets **5 seconds** to run (tests get 10) before it is
stopped, it receives no keyboard input, and ClawCode keeps at most **64 KB**
of its output, so an accidental endless loop of `println!` can't flood your
memory.

**Honest note:** this is *not* a security sandbox. ClawCode compiles and runs
code you typed, as you, on your own machine, exactly like `cargo run` does with
any program. Don't paste in code you don't understand from strangers, and don't
use ClawCode to run untrusted code.

## Writing your own lessons

Each lesson is one TOML file in `lessons/`. The full format (fields, the
Markdown subset, source rules, hidden tests) is in
[`docs/LESSON_FORMAT.md`](docs/LESSON_FORMAT.md).

Lessons are **embedded into the binary at build time** by `build.rs`, so a
normal build only sees the lessons that existed when it was compiled. While
writing, load them straight from disk instead:

```sh
CLAWCODE_LESSONS_DIR=./lessons cargo run
```

Then check that every solution passes and every starter fails:

```sh
cargo run -- verify
cargo run -- verify 03-types   # just one lesson
```

## Project layout

```
clawcode/
├── Cargo.toml              Project description and its dependencies (crates)
├── Cargo.lock              Exact dependency versions, for repeatable builds
├── build.rs                Build script: embeds lessons/ and data/errors.toml into the binary
├── lessons/                The 10 lessons, one TOML file each (01-hello.toml ...)
├── data/
│   └── errors.toml         Catalog of friendly explanations for compiler errors
├── docs/
│   ├── LESSON_FORMAT.md    How to write a lesson
│   └── validate_lesson.sh  Rough manual lesson validator (needs python3 and rustc)
└── src/
    ├── main.rs             Command-line commands (list, check, verify, reset, ...)
    ├── app.rs              The TUI: screens, keys, editor, drawing
    ├── checker.rs          Runs rustc, compares output, runs tests, enforces limits
    ├── feedback.rs         Turns check results into Pinch's messages
    ├── friendly.rs         Looks up plain-English explanations for compiler errors
    ├── lesson.rs           Loads and validates lesson files
    ├── markdown.rs         Renders the lesson text (small Markdown subset)
    └── progress.rs         XP, levels, streaks, badges, saving to disk
```

## Troubleshooting

**"linker `cc` not found"** (a build error mentioning `cc`, or the game saying
the compiler stopped unexpectedly). You're missing the C linker. Install it
from [step 4](#4-install-a-c-linker), then try again.

**"rustc: command not found" / "cargo: command not found"**, or the game shows
a red warning that it can't find `rustc`. Your shell can't see
`~/.cargo/bin`. Run `source "$HOME/.cargo/env"`, or open a new terminal. If
that doesn't help, check that `~/.cargo/bin` exists, and re-run the rustup
install command.

**Boxes or the crab look garbled, or emoji show as squares.** Use a terminal set
to UTF-8 (nearly all modern ones are) and a modern font that has box-drawing
characters, such as DejaVu Sans Mono, Fira Code or JetBrains Mono.

**A Ctrl+key does nothing.** Some terminals and desktop shortcuts grab certain
combinations. Use the alternatives: `F5` run, `F2` tip, `F4` solution, `F8`
reset, `F1` help. (`Alt+↑↓` also scrolls Pinch's messages.)

**Things look cramped or cut off.** The window is too small. Enlarge it to at
least 80x24 (or lower the font size).

**The first build is slow.** That's normal: Cargo is downloading and compiling
dependencies. It only happens once.

## Next steps

When you've cleared all ten lessons, keep going with these (as further
reading, not part of ClawCode):

- [The Rust Programming Language](https://doc.rust-lang.org/book/) ("the Book"), the official free tutorial.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/), learn by reading and tweaking small programs.
- [Rustlings](https://github.com/rust-lang/rustlings), small exercises you fix on your own machine.

Happy pinching! (\/)
