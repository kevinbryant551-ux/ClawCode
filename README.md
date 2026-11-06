# ClawCode

```
  (\/)  (\/)
  ( o_o )   Learn Rust, one small pinch at a time.
```

ClawCode is a game you play in your **terminal** that teaches you the
**Rust** programming language. Each lesson gives you short reading cards, pop
quizzes, and a code editor right inside the game. When you press **Ctrl+R**,
your code is checked by the *real* Rust compiler (not an imitation), so what
you learn is exactly how Rust behaves on your own computer.

It is built like a game: you earn **XP** and **shells**, climb **11 levels**,
build **quiz combos**, keep **daily streaks**, collect **badges**, fight three
**bosses**, play a **Daily Tide** mini-quiz, and spend shells on hats and
colours for **Pinch the crab**, your guide. When the compiler complains, Pinch
translates the error into plain English and tells you what to try next.

> **On Android?** There is a phone version built with the Godot engine, played
> entirely by tapping (no typing). See [godot/README.md](godot/README.md) for how
> to download the APK from GitHub Actions or build it yourself.

## Contents

1. [Key terms](#key-terms)
2. [Install on Linux](#install-on-linux)
3. [Run the game](#run-the-game)
4. [The course: 20 lessons](#the-course-20-lessons)
5. [How a lesson flows](#how-a-lesson-flows)
6. [Game systems](#game-systems)
7. [Screens and keys](#screens-and-keys)
8. [Other commands](#other-commands)
9. [How checking works (and safety)](#how-checking-works-and-safety)
10. [Writing your own lessons](#writing-your-own-lessons)
11. [Project layout](#project-layout)
12. [Troubleshooting](#troubleshooting)
13. [Next steps](#next-steps)

## Key terms

Skim this now, and come back whenever a word feels fuzzy. Each term is also
explained where it first shows up below.

| Term | Plain-English meaning |
|---|---|
| **Terminal** | A window where you type commands instead of clicking. Also called the console or command line. Look for "Terminal", "Konsole", "GNOME Terminal" or similar in your app menu. |
| **Shell** | Two meanings, so watch the context. In *programming*, the shell is the program inside the terminal that reads your commands (usually `bash`). In the *game*, **shells** (◉) are the currency you earn and spend in the Shell Shop. The README says "game shells" or "◉ shells" when it means the money. |
| **TUI** | "Text User Interface": an app with menus, boxes and colors that runs *inside* a terminal. ClawCode is a TUI. |
| **Rust** | The programming language you are learning. |
| **Compile** | Translate human-readable source code into machine code your computer can run. |
| **Compiler** | The program that compiles. Rust's compiler is called `rustc`. |
| **`rustc`** | The Rust compiler. ClawCode runs it for you each time you check a lesson. |
| **Linker** | The last build step: it glues compiled pieces into one binary. Rust uses your system's C linker, a program called `cc`, for this. |
| **Cargo** | Rust's build tool and package manager. It downloads libraries, calls `rustc` for you, and runs your program. |
| **Crate** | A Rust package: either a program or a library. ClawCode is one crate, and it uses other people's crates (like `ratatui`, which draws the boxes). |
| **`rustup`** | The official installer and updater for Rust. It installs `rustc`, `cargo` and friends. |
| **PATH** | A list of folders your shell searches when you type a command name. If a program's folder isn't on PATH, the shell says "command not found". |
| **TOML** | A simple text format for settings and data (`name = "value"`). `Cargo.toml` and every ClawCode lesson file use it. |
| **`$EDITOR`** | An *environment variable* (a named setting your shell remembers) holding the name of your favorite text editor, such as `nano` or `vim`. |
| **XP** | Experience points, the game's score. It decides your level. |
| **Level** | Your rank, from 1 to 11. Each level has a title (like "Reef Ranger") and an XP threshold. |
| **Pop quiz** | A multiple-choice question shown between cards. |
| **Combo** | Your current run of pop quizzes answered correctly *on the first try*. A wrong first answer resets it to 0. Bigger combos pay more shells. |
| **Boss** | A special lesson that reviews a whole chapter. Its hidden tests are the boss's hit points (HP). |
| **HP** | Hit points: how much life the boss has left. Each hidden test is 1 HP. |
| **Hidden test** | A small Rust check (`#[test]`) the game runs on your code. You see only whether it passed, plus a plain-language message. |
| **Daily Tide** | A once-a-day mini-quiz of 5 questions mixed from lessons you have read. |
| **Streak** | How many days in a row you passed a lesson or finished the Daily Tide. |

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
(change directory) moves your terminal into it. If you don't have `git`,
install it with your package manager (for example `sudo apt install git`), or
download the project as a ZIP and unpack it.

## Run the game

Make your terminal window **at least 80 columns by 24 rows** (about the size of
a default terminal window), then, from inside the project folder:

```sh
cargo run --release
```

The first time, Cargo downloads the crates ClawCode depends on and compiles
them, which takes a minute or so. `--release` asks for an optimized build, so
the game feels snappy. Later runs start instantly.

The game opens on an animated **title screen** with a menu (Continue Quest,
Daily Tide, Shell Shop, Trophy Room, How to Play, Quit) and a one-line summary
of your level, XP, shells, streak and lessons cleared.

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

## The course: 20 lessons

Lessons are grouped into chapters. The three **boss fights** (☠ on the map)
close out a stretch of chapters. Base XP is what the lesson is worth before
bonuses.

| # | Lesson | Chapter | Base XP |
|---|---|---|---|
| 1 | Hello, Tide Pool! | Getting Started | 50 |
| 2 | Keeping Things in Buckets | Getting Started | 60 |
| 3 | Types of Things | The Basics | 70 |
| 4 | Build Your Own Moves | The Basics | 80 |
| 5 | Tide Decisions | Control Flow | 80 |
| 6 | Round and Round | Control Flow | 90 |
| 7 | Who Owns the Shell? | Ownership | 110 |
| 8 | Build a Crab | Building Your Own Types | 90 |
| 9 | Weather or Not | Building Your Own Types | 100 |
| 10 | Maybe, Maybe Not | Handling the Unexpected | 120 |
| 11 | ☠ **Boss: Borrowzilla** | Boss: Borrowzilla | 150 |
| 12 | Bucket of Shells | Collections | 90 |
| 13 | String Theory | Collections | 90 |
| 14 | Shell Tally | Collections | 100 |
| 15 | ☠ **Boss: The Collection Kraken** | Boss: The Collection Kraken | 180 |
| 16 | Shared Superpowers | Superpowers | 110 |
| 17 | Pocket Functions | Superpowers | 100 |
| 18 | The Conveyor Belt | Superpowers | 110 |
| 19 | When Things Go Wrong | Superpowers | 120 |
| 20 | ☠ **The Rust Leviathan** | Final Boss: The Rust Leviathan | 250 |

Together that is 2150 base XP. **Unlocking:** lesson 1 is open from the start;
each later lesson unlocks when you pass the one before it. Passed lessons can
be replayed any time. To skip the ordering and open everything:

```sh
cargo run --release -- --unlock-all
# or, if installed:
clawcode --unlock-all
```

Setting the environment variable `CLAWCODE_UNLOCK_ALL` to anything does the
same thing.

## How a lesson flows

Every lesson has two phases.

**1. Learn phase (reading cards).** The lesson text is split into short
**cards**, one idea each, shown one at a time. Press `→` or `Enter` for the
next card, `←` to go back. Between cards, Pinch may spring a **pop quiz**:

- Pick an answer with `1`-`4`, or move with `↑`/`↓` and press `Enter`.
- A wrong answer is **crossed out**, and you may try again until you find the
  right one. Pinch explains the answer afterwards.
- Only your **first** answer counts for rewards and combos (details in
  [Game systems](#game-systems)).
- `Tab` skips straight to the code. The first time you reach the editor, the
  lesson counts as "read" (which feeds the Daily Tide).

**2. Code phase (the editor).** The lesson text and goal are on the left, your
code editor is on the right, and Pinch's messages are below. Write code and
press `Ctrl+R`. Pinch reports whether it compiled, whether the output or tests
matched, and what to fix. Coming back to a lesson you already read takes you
straight to the editor; press `Ctrl+L` or `F3` there (or `l` on the map) to
reread the cards.

**Boss lessons** are the same, with a twist: the boss's picture and an **HP
bar** appear at the top left. Each hidden test is 1 HP, so every test your code
passes damages the boss, and `Ctrl+R` is labelled **ATTACK**. Pass them all and
the boss is defeated.

The editor helps a little: pressing Enter keeps your current indentation and
adds one more level after an opening `{`, and typing `}` at the start of an
indented line steps back one level. Your unfinished code is saved as a draft,
so you can quit and pick up where you left off.

## Game systems

### XP (experience points)

When you pass a lesson for the **first** time, you earn:

| Rule | Effect |
|---|---|
| Lesson cleared (or "Boss defeated") | + base XP |
| Passed on your very first try | + 50% of base |
| Boss bounty (boss lessons only) | + 50% of base |
| Each tip you revealed | - 10% of base per tip (the first-try bonus still counts) |
| You peeked at the solution | - 50% of base |
| Streak of 2 or more days | + 5 XP per streak day, up to + 25 |
| Minimum | You always earn at least 25% of base |

**Replays give no XP or shells**, though passing any lesson still counts
toward your streak. Example: a 100 XP lesson passed on the first try with no
tips earns 150 XP (plus any streak bonus).

Tips and the solution: each `Ctrl+T` reveals the next tip. The solution (`Ctrl+P`)
becomes available after **3 tries** or after you have seen all the tips.

### Shells (◉)

Shells are the game's money. Clearing a lesson for the first time pays XP
earned divided by 4 (rounded down); defeating a boss pays the same **+25**
extra. Pop quizzes and the Daily Tide pay shells too (below).

### Pop quizzes and combos

Each pop quiz pays out **only once, the first time you ever answer it**, and
only your *first answer* matters:

| First answer | XP | Shells | Combo |
|---|---|---|---|
| Correct | 10 | 3 + your combo (the combo counts at most 10, so up to 13) | +1 |
| Wrong | 0 | 1 | resets to 0 |

Your **combo** is the number of quizzes in a row you got right on the first
try. The toast "COMBO xN!" appears at x2 and above. Your best combo is tracked
in the Trophy Room. Quizzes you have already solved, and retries after a wrong
answer, give nothing more.

### Daily Tide

A once-a-day mini-quiz from the title menu (or `d` on the title and map).

- **5 questions**, drawn from the pop quizzes of lessons you have **read or
  cleared**. You need at least **3** such quizzes available; otherwise Pinch
  asks you to read a lesson first.
- You can play it **once per day** (by your local calendar). It is picked
  from the day's date, so the same day always offers the same set.
- Your **first answer** to each question is the one that counts. A wrong
  answer is crossed out and you keep trying so you see the right one.
- - Rewards: **5 XP** for showing up, **8 XP per correct answer**, **+20 XP** for
  a perfect score, plus the streak bonus. Shells: **2 + 3 per correct answer**,
  **+10** if perfect.
- Finishing it **counts for your streak**.

### Streaks

A streak counts the days *in a row* on which you passed at least one lesson
**or** finished the Daily Tide (by your local calendar day). Doing both in one
day still counts as one day. Skip a whole day and the streak starts again at 1.
Your best streak is remembered.

### Levels

There are 11 levels. You reach a level when your total XP hits its threshold.

| Level | Title | XP needed |
|---|---|---|
| 1 | Hatchling | 0 |
| 2 | Tide-pool Tinkerer | 100 |
| 3 | Shell Shaper | 250 |
| 4 | Reef Ranger | 450 |
| 5 | Borrow Buddy | 700 |
| 6 | Ownership Oracle | 1000 |
| 7 | Trait Tamer | 1400 |
| 8 | Iterator Ace | 1900 |
| 9 | Deep-sea Debugger | 2500 |
| 10 | Leviathan Wrangler | 3200 |
| 11 | Rustacean | 4000 |

### Badges

There are 19 badges. See them (and which you have) in the Trophy Room.

| Badge | How to earn it |
|---|---|
| First Steps | Pass your first lesson |
| Flawless | Pass a lesson on the very first try |
| No Peeking | Pass 3 lessons without any tips |
| Persistent Pincer | Pass a lesson after 5 or more tries |
| On a Roll | Keep a 3-day streak |
| Week Warrior | Keep a 7-day streak |
| Borrow Boss | Conquer the ownership lesson |
| Boss Slayer | Defeat your first boss |
| Monster Hunter | Defeat 3 bosses |
| Quiz Whiz | Answer 10 quizzes right on the first try |
| Combo Crab | Reach a 5x quiz combo |
| Unstoppable | Reach a 10x quiz combo |
| Daily Diver | Finish 3 Daily Tides |
| Perfect Tide | Get every Daily Tide question right |
| Fashionista | Buy something in the Shell Shop |
| Collector | Own 5 shop items |
| Error Veteran | Survive 25 compiler errors (they make you stronger!) |
| Halfway Reef | Pass half of the lessons |
| Rustacean Graduate | Pass every lesson |

### Shell Shop

Spend shells on cosmetics for Pinch. In the shop, move to an item and press
`Enter`: if you don't own it, it is **bought and worn**; if you own it, `Enter`
**wears** it; if you are already wearing it, `Enter` **takes it off**. You can
wear one hat and one shell colour at once. A preview of Pinch is shown in the
"Fitting room".

| Item | Type | Price (◉) |
|---|---|---|
| Classic Red | Colour | free (Pinch's natural shell) |
| Party Hat | Hat | 30 |
| Sunset Orange | Colour | 40 |
| Sailor Cap | Hat | 50 |
| Ocean Teal | Colour | 60 |
| Deep Purple | Colour | 60 |
| Top Hat | Hat | 80 |
| Pirate Hat | Hat | 120 |
| Wizard Hat | Hat | 160 |
| Golden Shell | Colour | 200 |
| Crown | Hat | 250 |

### Trophy Room

Shows all badges (★ earned, ☆ not yet) plus your stats: level, XP, lessons
cleared, bosses defeated, code runs, compiler errors survived, quiz accuracy,
best combo, current and best streak, Daily Tides, shells earned, items owned.

### Where progress is saved

Everything (XP, shells, finished lessons, quiz results, badges, streak,
shop items, stats, drafts) is stored in one small text file:

```
~/.local/share/clawcode/progress.json
```

To keep it somewhere else, set `CLAWCODE_PROGRESS` to a file path:

```sh
CLAWCODE_PROGRESS=/tmp/test-progress.json clawcode
```

To start over, run `clawcode reset` (or `cargo run -- reset`). It shows the
file path and deletes it only if you type `yes`.

## Screens and keys

Press **F1** or **?** nearly anywhere for the in-game help. The F-keys exist
because some terminals swallow certain Ctrl combinations. `Ctrl+C` quits from
any screen (progress is saved).

### Title screen

`↑` / `↓` (or `k` / `j`) move, `Enter` or `Space` chooses (Continue Quest opens
the lesson map), `d` / `s` / `t` jump to Daily Tide / Shell Shop / Trophy Room,
`F1` or `?` is help, `q` or `Esc` quits.

### Lesson map

| Key | What it does |
|---|---|
| `↑` / `↓` (or `k` / `j`) | Move the selection |
| `Enter` or `Space` | Open the selected lesson (a locked one tells you to clear the lesson before it) |
| `l` | Reread the selected lesson's cards |
| `d` / `s` / `t` | Daily Tide / Shell Shop / Trophy Room |
| `F1`, `?` or `h` | Help |
| `Esc` or `q` | Back to the title screen |

### Learn phase (reading cards and quizzes)

| Key | What it does |
|---|---|
| `→`, `Enter`, `Space` or `l` | Next card (on the last card, go to the code) |
| `←`, `Backspace` or `h` | Previous card |
| `↑` / `↓`, `j` / `k`, `PgUp` / `PgDn` | Scroll a long card |
| `1`-`4`, or `↑` `↓` then `Enter` | Answer a pop quiz |
| `Tab` | Skip straight to the code |
| `Esc` | Back to the lesson map |

### Code phase (the editor)

| Key | What it does |
|---|---|
| `Ctrl+R` or `F5` | Run and check your code with the real Rust compiler (**ATTACK** in boss fights) |
| `Ctrl+T` or `F2` | Get a tip (each new tip costs a bit of XP) |
| `Ctrl+P` or `F4` | Show the solution (after 3 tries or all tips) |
| `Ctrl+L` or `F3` | Reread the lesson cards |
| `Ctrl+E` | Open your code in your own editor (`$VISUAL` or `$EDITOR`; falls back to `nano`, then `vi`) |
| `Ctrl+S` | Save (it also saves automatically) |
| `Ctrl+O` or `F8` | Reset the code to the starter version (asks first) |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo in the editor |
| `PgUp` / `PgDn` | Scroll the lesson text |
| `Ctrl+↑↓` / `Alt+↑↓` | Scroll Pinch's messages |
| `F1` or `Ctrl+G` | Help |
| `Esc` | Back to the lesson map (saves first) |
| `Ctrl+C` | Quit (progress is saved) |

### Daily Tide, Shell Shop, Trophy Room, celebration screen

- **Daily Tide:** answer with `1`-`4` (or `↑` `↓` then `Enter`); after a correct
  answer `Enter`, `Space` or `→` moves on; `Esc` leaves to the title screen
  without finishing.
- **Shell Shop:** `↑` / `↓` (or `k` / `j`) choose, `Enter` or `Space` buys,
  wears or takes off, `Esc` or `q` goes back to the title screen.
- **Trophy Room:** `Esc`, `Enter` or `q` goes back to the title screen.
- **After a clear:** `Enter` or `n` opens the next lesson, `m` returns to the
  map with the next lesson selected.

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
| `clawcode export-godot [file]` | For the Android app: converts lessons to `godot/data/lessons.json` (see [godot/README.md](godot/README.md)) |
| `clawcode reset` | Delete all saved progress (asks first) |
| `clawcode --help` | Show the usage text |

Environment variables: `CLAWCODE_PROGRESS` (progress file path),
`CLAWCODE_UNLOCK_ALL` (unlock every lesson), `CLAWCODE_LESSONS_DIR` (load
lessons from a folder; see below).

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
   and runs the tests. Failures are reported in plain language. In a boss
   fight, each passing test knocks 1 HP off the boss.

Limits: your program gets **5 seconds** to run (tests get 10) before it is
stopped, it receives no keyboard input, and ClawCode keeps at most **64 KB**
of its output, so an accidental endless loop of `println!` can't flood your
memory.

**Honest note:** this is *not* a security sandbox. ClawCode compiles and runs
code you typed, as you, on your own machine, exactly like `cargo run` does with
any program. Don't paste in code you don't understand from strangers, and don't
use ClawCode to run untrusted code.

## Writing your own lessons

Each lesson is one TOML file in `lessons/`. The full format is in
[`docs/LESSON_FORMAT.md`](docs/LESSON_FORMAT.md). The short version:

- Write the explanation in the `intro` field and split it into **cards** with
  a line containing only `---` (one idea per card, 3 to 5 cards).
- Add pop quizzes with `[[quiz]]` tables (`after_card`, `question`,
  `choices`, `answer` counting from 0, `explain`). The format guide suggests
  2 or 3 per lesson. These also feed the Daily Tide.
- Make it a boss fight by adding a `[boss]` table (`name`, `art`, `taunt`,
  `defeat`). A boss lesson must use hidden `tests`; each `#[test]` is 1 HP.
- `starter` must fail the check, `solution` must pass it.

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

There is also a rough standalone checker for a single file,
`docs/validate_lesson.sh lessons/03-types.toml` (needs `python3` and `rustc`).

## Project layout

```
clawcode/
├── Cargo.toml              Project description and its dependencies (crates)
├── Cargo.lock              Exact dependency versions, for repeatable builds
├── build.rs                Build script: embeds lessons/ and data/errors.toml into the binary
├── lessons/                The 20 lessons, one TOML file each (01-hello.toml ... 20-boss-leviathan.toml)
├── data/
│   └── errors.toml         Catalog of friendly explanations for compiler errors
├── docs/
│   ├── LESSON_FORMAT.md    How to write a lesson
│   └── validate_lesson.sh  Rough manual lesson validator (needs python3 and rustc)
└── src/
    ├── main.rs             Command-line commands (list, check, verify, reset, ...)
    ├── app.rs              The game's state and brain: screens, key handling, lesson
    │                       flow, quizzes, Daily Tide, shop actions, running checks
    ├── ui.rs               Drawing every screen: title, map, learn, code, boss, shop,
    │                       trophies, overlays and the help popup
    ├── widgets.rs          Shared drawing helpers and the editor's typing assistance
    ├── anim.rs             Animations (confetti, bubbles) and the seeded picker
    │                       used for the Daily Tide
    ├── shop.rs             The Shell Shop items, prices, hat art and shell colours
    ├── checker.rs          Runs rustc, compares output, runs tests, enforces limits
    ├── feedback.rs         Turns check results into Pinch's messages
    ├── friendly.rs         Looks up plain-English explanations for compiler errors
    ├── lesson.rs           Loads and validates lesson files (cards, quizzes, bosses)
    ├── markdown.rs         Renders the lesson text (small Markdown subset)
    └── progress.rs         XP, shells, levels, streaks, badges, quizzes, saving to disk
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
combinations. Use the alternatives: `F5` run, `F2` tip, `F4` solution, `F3`
reread cards, `F8` reset, `F1` help. (`Alt+↑↓` also scrolls Pinch's messages.)

**Things look cramped or cut off.** The window is too small. Enlarge it to at
least 80x24 (or lower the font size).

**The first build is slow.** That's normal: Cargo is downloading and compiling
dependencies. It only happens once.

**The Daily Tide says to read a lesson first.** It needs at least 3 pop quizzes
from lessons you have read or cleared. Read a lesson's cards (or press `Tab`
to the code) and try again. If it says it is done, come back tomorrow.

## Next steps

When you've cleared all twenty lessons, keep going with these (as further
reading, not part of ClawCode):

- [The Rust Programming Language](https://doc.rust-lang.org/book/) ("the Book"), the official free tutorial.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/), learn by reading and tweaking small programs.
- [Rustlings](https://github.com/rust-lang/rustlings), small exercises you fix on your own machine.

Happy pinching! (\/)
