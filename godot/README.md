# ClawCode for Android (Godot)

This folder is the **phone version** of ClawCode, built with the **Godot 4**
game engine. It teaches the same lessons as the terminal game, but you play
it **by tapping buttons only**. You never type code on a phone keyboard.

The terminal (desktop) game in the repo root still works exactly as before.

## Contents

1. [Key terms](#key-terms)
2. [How the phone version plays](#how-the-phone-version-plays)
3. [Easiest way to get the APK (no setup)](#easiest-way-to-get-the-apk-no-setup)
4. [Run it on your Linux computer](#run-it-on-your-linux-computer)
5. [Build the Android APK yourself](#build-the-android-apk-yourself)
6. [Install the APK on your phone](#install-the-apk-on-your-phone)
7. [Adding tap puzzles to a lesson](#adding-tap-puzzles-to-a-lesson)
8. [How the pieces fit together](#how-the-pieces-fit-together)
9. [Troubleshooting](#troubleshooting)

## Key terms

| Term | Plain-English meaning |
|---|---|
| **Godot** | A free, open-source game engine (a toolkit for making games). You open a project in the **Godot editor**, and Godot can **export** it to Android, Linux, Windows and more. |
| **GDScript** | Godot's own programming language. It looks a lot like Python. All the app's code is in `scripts/*.gd`. |
| **Scene** / **node** | Godot builds everything from **nodes** (a button, a label, a scroll area...) arranged in a tree. A saved tree is a **scene** (`.tscn` file). This app builds most of its nodes in code. |
| **Autoload** | A script Godot loads once at startup and keeps around, reachable from every other script by name. Here, `Game` (`scripts/game.gd`) holds the lessons, your progress and the colour theme. |
| **Export** | Turning a Godot project into a finished app file for one platform. |
| **Export preset** | Saved settings for one export target (here: "Android"), stored in `export_presets.cfg`. |
| **Export templates** | Pre-built copies of the Godot engine for each platform. Godot glues your project onto them when it exports. They are a separate download from the editor. |
| **APK** | "Android Package": the file you install on an Android phone. |
| **AAB** | "Android App Bundle": the format the Google Play Store wants. You only need it if you publish to Play. |
| **Android SDK** | Google's "Software Development Kit": the tools for building and signing Android apps. Godot uses a few of them (`apksigner`, `adb`, the build tools). |
| **JDK** | "Java Development Kit". Android's tools are written in Java, so you need a JDK (version **17**) installed. **OpenJDK** is the free version. |
| **Keystore** | A file that holds a secret **signing key**. Android only installs apps that are **signed**. A **debug keystore** is a throwaway key for testing. |
| **adb** | "Android Debug Bridge": a command that talks to your phone over USB to install and run apps. |
| **Sideloading** | Installing an app from an APK file instead of from the Play Store. |
| **BBCode** | A tiny markup language like `[b]bold[/b]`. Godot's `RichTextLabel` uses it for coloured, formatted text. |
| **JSON** | A plain-text data format Godot can read natively. The lessons are converted from TOML to JSON for the app. |
| **CI** / **GitHub Actions** | "Continuous Integration": GitHub runs scripts on every push. This repo's CI tests the app and builds an APK for you. |
| **Artifact** | A file a CI run saves for you to download, such as the APK. |
| **`user://`** | Godot's private data folder for the app. Your progress is saved there as `progress.json`. On Linux it is `~/.local/share/godot/app_userdata/ClawCode/`. |

## How the phone version plays

Each lesson goes through the same phases as the terminal game, with every
interaction turned into a tap:

1. **Learn cards.** Short reading cards with coloured code examples. Tap **Next**.
2. **Pop quizzes.** Multiple choice. Tap an answer. Right answers on the first
   try build a **combo**, which pays bonus shells.
3. **Tap puzzles** (instead of typing code). There are two kinds:
   - **Fill the blanks**: the code has blanks, and the next one to fill glows
     orange. Tap a **tile** (a small code button like `mut` or `&`) to drop it
     in. Tap a filled blank to take the tile back. Some tiles are **decoys**
     (look-alike wrong answers).
   - **Put the lines in order** (a "Parsons puzzle"): tap lines of code from
     top to bottom to build the program. Watch out for decoy lines.

   Tap **Check** when every slot is full. Wrong slots turn red and you lose a
   **heart** (♥). Run out and Pinch refills them. You lose nothing permanent. **Hint**
   shows the lesson's hints one by one. After 3 misses on one puzzle a
   **Show me the answer** button appears (it halves that lesson's XP).
4. **Boss fights.** Boss lessons show the boss's art and taunt, then an
   **HP bar**. Every correct blank is one hit.
5. **Victory.** Confetti, XP, shells, streaks and level-ups, using the same
   rules as the terminal game.

Back on Android (or **Esc** on a computer) goes back one screen.

### Where the puzzles come from

- **Lessons 1 to 5** have hand-made puzzles (`[[tap]]` tables in their lesson files).
- **Lessons 6 to 20** use **automatic** puzzles. The exporter compares each
  lesson's starter code with its solution, finds the lines you would have had
  to write, and blanks out the most "Rust-y" token on each one (keywords like
  `match` or `mut`, operators like `&` or `?`, values). Decoys come from a list
  of common mix-ups (`&` vs `&mut`, `Some` vs `None`, `=>` vs `->`...). These
  are playable, but hand-made puzzles are better. You can replace them one lesson at a time.

## Easiest way to get the APK (no setup)

Every push to the pull request runs the **Godot (Android)** workflow on GitHub,
which tests the app and builds a **debug APK**:

1. Open the repo on GitHub and click the **Actions** tab.
2. Click the latest **Godot (Android)** run.
3. Scroll to **Artifacts** and download **ClawCode-debug-apk** (a `.zip`).
4. Unzip it to get `ClawCode-debug.apk`, then follow
   [Install the APK on your phone](#install-the-apk-on-your-phone).

## Run it on your Linux computer

You can play and test on your computer before touching a phone.

1. **Download Godot 4.4** (the standard version, not ".NET") from
   <https://godotengine.org/download/linux/>. You get a `.zip` with one program in it:

   ```sh
   cd ~/Downloads
   unzip Godot_v4.4.1-stable_linux.x86_64.zip
   chmod +x Godot_v4.4.1-stable_linux.x86_64      # mark it as runnable
   mkdir -p ~/.local/bin
   mv Godot_v4.4.1-stable_linux.x86_64 ~/.local/bin/godot
   ```

   (`~/.local/bin` is usually on your **PATH**, so you can now type `godot`
   in a terminal. If not, log out and back in, or run it with its full path.)
   Godot is also available on Flathub (`flatpak install flathub org.godotengine.Godot`)
   and in some distro repos, but check the version is **4.4 or newer**.

2. **Open the project**: run `godot`, click **Import**, choose this repo's
   `godot/project.godot`, then **Import & Edit**.

3. Press **F5** (or the ▶ Play button at the top right). The window is phone-shaped.

To run it straight from a terminal without the editor:

```sh
cd ~/path/to/clawcode
godot --path godot
```

## Build the Android APK yourself

You need four things: **JDK 17**, the **Android SDK**, Godot's **export
templates**, and a **debug keystore**. Godot's official guide is
<https://docs.godotengine.org/en/stable/tutorials/export/exporting_for_android.html>.

### 1. Install JDK 17

```sh
# Debian / Ubuntu / Mint / Pop!_OS
sudo apt install openjdk-17-jdk
# Fedora
sudo dnf install java-17-openjdk-devel
# Arch / Manjaro
sudo pacman -S jdk17-openjdk
```

Check with `java -version` (it should say 17). The JDK usually lives in
`/usr/lib/jvm/java-17-openjdk-amd64` (Debian/Ubuntu) or
`/usr/lib/jvm/java-17-openjdk` (Fedora/Arch).

### 2. Install the Android SDK

The simplest route is **Android Studio** (<https://developer.android.com/studio>,
or `flatpak install flathub com.google.AndroidStudio`). Open it once, go to
**More Actions > SDK Manager** and install:

- **SDK Platform**: Android 14 (API 34)
- **SDK Tools**: Android SDK **Build-Tools 34**, **Platform-Tools**, **Command-line Tools (latest)**

The SDK ends up in `~/Android/Sdk`.

(Command-line only? Download "Command line tools only" from the same page,
unzip it into `~/Android/Sdk/cmdline-tools/latest/`, then run
`~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager "platform-tools" "build-tools;34.0.0" "platforms;android-34"`.)

### 3. Make a debug keystore

```sh
keytool -keyalg RSA -genkeypair -alias androiddebugkey -keypass android \
  -keystore ~/debug.keystore -storepass android \
  -dname "CN=Android Debug,O=Android,C=US" -validity 9999 -deststoretype pkcs12
```

(`keytool` comes with the JDK. This makes a throwaway key with the standard
debug user `androiddebugkey` and password `android`. Never use it for a Play Store release.)

### 4. Tell Godot where everything is

In the Godot editor: **Editor > Editor Settings > Export > Android** and set:

| Setting | Value |
|---|---|
| Java SDK Path | `/usr/lib/jvm/java-17-openjdk-amd64` (or yours) |
| Android SDK Path | `/home/YOU/Android/Sdk` |
| Debug Keystore | `/home/YOU/debug.keystore` |
| Debug Keystore User | `androiddebugkey` |
| Debug Keystore Pass | `android` |

### 5. Install export templates

**Editor > Manage Export Templates > Download and Install**. (This is about
1 GB. Godot only needs the Android part, but it downloads them all.)

### 6. Export

**Project > Export...**, pick the **Android** preset (already set up in
`export_presets.cfg`), then **Export Project** and save as
`build/ClawCode-debug.apk`. Leave "Export With Debug" ticked.

Or from a terminal:

```sh
cd ~/path/to/clawcode
godot --headless --path godot --export-debug "Android" "$PWD/godot/build/ClawCode-debug.apk"
```

### What the Android preset sets

- **Package name** `io.github.kevinbryant551.clawcode` (an app's unique ID on a phone).
- **Architectures** `arm64-v8a` (almost every phone since 2017) and `armeabi-v7a` (older phones).
- **Portrait** orientation and the **Compatibility** renderer (OpenGL ES 3), which runs on the widest range of phones.
- **No internet permission**: the app works fully offline.
- `include_filter="data/*.json"` makes sure the lessons file is packed into the APK
  (Godot only packs its own resource types unless told otherwise).

## Install the APK on your phone

**Option A: copy the file over.** Copy `ClawCode-debug.apk` to the phone (USB,
cloud drive, email to yourself), tap it in the phone's **Files** app, and allow
"Install unknown apps" for that app when Android asks.

**Option B: adb over USB.**

1. On the phone, open **Settings > About phone** and tap **Build number** 7 times
   to unlock **Developer options**. Then turn on **Settings > System > Developer options > USB debugging**.
2. Plug in the phone and accept the "Allow USB debugging?" prompt.
3. On Linux:

   ```sh
   sudo apt install adb          # or: sudo dnf install android-tools / sudo pacman -S android-tools
   adb devices                   # your phone should be listed as "device"
   adb install -r godot/build/ClawCode-debug.apk     # -r replaces an older install
   ```

With USB debugging on, the Godot editor also shows a small **Android** button at the top
right that exports and runs the game on your phone in one click (**Remote Debug**).

## Adding tap puzzles to a lesson

Tap puzzles live in the same lesson TOML files as everything else (see
`docs/LESSON_FORMAT.md`, section "Tap puzzles"). After editing a lesson, regenerate
the app's data from the repo root:

```sh
cargo run -- export-godot      # writes godot/data/lessons.json
cargo test                     # checks every puzzle is solvable
```

CI fails if you forget to regenerate `lessons.json`.

## How the pieces fit together

```
lessons/*.toml ──(cargo run -- export-godot)──▶ godot/data/lessons.json ──▶ Godot app
                  src/godot_export.rs: Markdown → BBCode, code colouring
                  src/tap.rs: [[tap]] puzzles, or automatic ones
```

| File | What it does |
|---|---|
| `project.godot` | Project settings: phone-sized portrait window, stretch mode, renderer, the `Game` autoload. |
| `main.tscn`, `scripts/main.gd` | Root node: background, confetti layer, switches between screens, handles Back. |
| `scripts/game.gd` | `Game` autoload: loads lessons, saves progress, XP/level/streak rules, colour theme. |
| `scripts/title_screen.gd` | Title: Pinch, stats, Play, "Unlock all", "Reset progress". |
| `scripts/map_screen.gd` | Lesson list grouped by chapter, with XP bar. |
| `scripts/lesson_screen.gd` | Cards, quizzes, boss intro, tap puzzles, victory. |
| `scripts/puzzle.gd` | Puzzle rules with no UI (place, undo, check, reveal), so they can be unit-tested. |
| `scripts/pinch.gd` | Pinch the crab, drawn with circles and lines, with moods. |
| `scripts/confetti.gd` | Confetti bursts. |
| `scripts/ui.gd` | Helpers for big, touch-friendly buttons, panels and scroll areas. |
| `tests/run_tests.gd` | Headless unit tests: every lesson's puzzles can be solved, decoys are wrong. |
| `tests/selftest.gd` | Headless play-through of all 20 lessons through the real screens. |
| `export_presets.cfg` | The Android export preset. |

Run the tests yourself:

```sh
godot --headless --path godot --import                               # first time only
godot --headless --path godot --script res://tests/run_tests.gd
godot --headless --path godot -- --selftest
```

## Troubleshooting

- **"No export template found"**: do step 5 (export templates), and make sure
  the templates version matches your editor version exactly.
- **"A valid Java SDK path is required"** or **"Invalid Android SDK path"**: re-check step 4.
  The SDK path is the folder that *contains* `platform-tools` and `build-tools`.
- **"Unable to find apksigner"**: install **Build-Tools** in the SDK Manager.
- **Phone says "App not installed"**: an older copy signed with a different
  key is installed. Uninstall it first (`adb uninstall io.github.kevinbryant551.clawcode`).
- **Lessons are missing or old in the app**: run `cargo run -- export-godot` again.
- **Text looks tiny or huge on desktop**: the window is a phone-shaped preview; resize it.
  The layout stretches to fit.
