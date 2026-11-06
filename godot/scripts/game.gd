extends Node
## Global game state (an "autoload" singleton, reachable from any script as `Game`).
## Loads the lessons exported by `cargo run -- export-godot`, keeps the player's
## progress in user://progress.json, and builds the shared UI theme.

const LESSONS_PATH := "res://data/lessons.json"
## Where progress is saved. user:// is the app's private data folder (on Linux: ~/.local/share/godot/app_userdata/ClawCode).
var save_path := "user://progress.json"

## XP needed to reach each level (same numbers as the terminal game).
const LEVEL_XP := [0, 100, 250, 450, 700, 1000, 1400, 1900, 2500, 3200, 4000]
const LEVEL_TITLES := [
	"Hatchling", "Tide-pool Tinkerer", "Shell Shaper", "Reef Ranger", "Borrow Buddy",
	"Ownership Oracle", "Trait Tamer", "Iterator Ace", "Deep-sea Debugger",
	"Leviathan Wrangler", "Rustacean",
]

# Colours shared by every screen.
const BG := Color("#10141f")
const PANEL := Color("#1a2133")
const PANEL_LIGHT := Color("#25304a")
const TEXT := Color("#e8ecf4")
const MUTED := Color("#8b95ab")
const ACCENT := Color("#ff7a59")
const GOOD := Color("#4cd98a")
const BAD := Color("#ff5a6a")
const GOLD := Color("#ffd36e")
const BLUE := Color("#7fd1ff")

var lessons: Array = []
var progress: Dictionary = {}
var theme: Theme
var mono_font: Font
var _loaded := false


func _ready() -> void:
	ensure_loaded()


func ensure_loaded() -> void:
	if _loaded:
		return
	_loaded = true
	load_lessons()
	load_progress()
	theme = build_theme()


# ---------------------------------------------------------------- lessons

func load_lessons() -> void:
	var f := FileAccess.open(LESSONS_PATH, FileAccess.READ)
	if f == null:
		push_error("Could not open %s. Run `cargo run -- export-godot` in the repo root." % LESSONS_PATH)
		return
	var data = JSON.parse_string(f.get_as_text())
	if typeof(data) != TYPE_DICTIONARY or not data.has("lessons"):
		push_error("lessons.json is not valid")
		return
	lessons = data["lessons"]
	lessons.sort_custom(func(a, b): return int(a["order"]) < int(b["order"]))


func lesson_index(id: String) -> int:
	for i in lessons.size():
		if lessons[i]["id"] == id:
			return i
	return -1


func is_unlocked(i: int) -> bool:
	if progress.get("unlock_all", false) or i <= 0:
		return true
	return is_done(lessons[i - 1]["id"])


func is_done(id: String) -> bool:
	return progress["done"].has(id)


func next_lesson_index() -> int:
	for i in lessons.size():
		if not is_done(lessons[i]["id"]):
			return i
	return lessons.size() - 1


# ---------------------------------------------------------------- progress

func default_progress() -> Dictionary:
	return {"xp": 0, "shells": 0, "done": {}, "streak": 0, "last_day": -1, "unlock_all": false, "quiz_best": 0}


func load_progress() -> void:
	progress = default_progress()
	if not FileAccess.file_exists(save_path):
		return
	var f := FileAccess.open(save_path, FileAccess.READ)
	if f == null:
		return
	var data = JSON.parse_string(f.get_as_text())
	if typeof(data) == TYPE_DICTIONARY:
		for k in data.keys():
			progress[k] = data[k]
	if typeof(progress["done"]) != TYPE_DICTIONARY:
		progress["done"] = {}


func save_progress() -> void:
	var f := FileAccess.open(save_path, FileAccess.WRITE)
	if f:
		f.store_string(JSON.stringify(progress, "  "))


func reset_progress() -> void:
	progress = default_progress()
	save_progress()


static func today() -> int:
	return int(Time.get_unix_time_from_system() / 86400.0)


func level() -> int:
	return level_for(int(progress["xp"]))


static func level_for(xp: int) -> int:
	var lvl := 1
	for i in LEVEL_XP.size():
		if xp >= LEVEL_XP[i]:
			lvl = i + 1
	return lvl


func level_title(lvl: int = -1) -> String:
	if lvl < 0:
		lvl = level()
	return LEVEL_TITLES[clampi(lvl - 1, 0, LEVEL_TITLES.size() - 1)]


## Returns [xp into this level, xp the level needs] for the XP bar.
func level_progress() -> Array:
	var xp := int(progress["xp"])
	var lvl := level()
	var start: int = LEVEL_XP[lvl - 1]
	if lvl >= LEVEL_XP.size():
		return [1, 1]
	return [xp - start, int(LEVEL_XP[lvl]) - start]


func touch_streak() -> void:
	var d := today()
	var last := int(progress["last_day"])
	if last == d:
		return
	progress["streak"] = int(progress["streak"]) + 1 if last == d - 1 else 1
	progress["last_day"] = d


## Records a passed lesson and returns what was earned. Mirrors the terminal
## game: base XP, +50% for no mistakes, +50% boss bounty, -10% per hint,
## -50% if the answer was revealed, streak bonus, never below 25% of base.
## Replaying a finished lesson earns nothing (but still counts for the streak).
func record_pass(lesson: Dictionary, mistakes: int, hints: int, revealed: bool, quiz_combo: int) -> Dictionary:
	var id: String = lesson["id"]
	var base := int(lesson["xp"])
	var boss: bool = lesson["boss"] != null
	var before := level()
	touch_streak()
	var reward := {"xp": 0, "shells": 0, "first_time": false, "level_up": false, "lines": [], "streak": int(progress["streak"])}
	if is_done(id):
		save_progress()
		return reward
	reward["first_time"] = true
	var lines: Array = [["Boss defeated" if boss else "Lesson cleared", base]]
	if mistakes == 0 and not revealed:
		lines.append(["No-mistake bonus", base / 2])
	if boss:
		lines.append(["Boss bounty", base / 2])
	if hints > 0:
		lines.append(["Hints used (%d)" % hints, -(base / 10) * hints])
	if revealed:
		lines.append(["Peeked at the answer", -(base / 2)])
	if int(progress["streak"]) > 1:
		lines.append(["%d-day streak" % int(progress["streak"]), mini(5 * int(progress["streak"]), 25)])
	var total := 0
	for l in lines:
		total += int(l[1])
	var earned := maxi(total, base / 4)
	var shells := earned / 4 + (25 if boss else 0) + quiz_combo * 2
	progress["xp"] = int(progress["xp"]) + earned
	progress["shells"] = int(progress["shells"]) + shells
	progress["done"][id] = {"xp": earned, "mistakes": mistakes, "day": today()}
	progress["quiz_best"] = maxi(int(progress.get("quiz_best", 0)), quiz_combo)
	reward["xp"] = earned
	reward["shells"] = shells
	reward["lines"] = lines
	reward["level_up"] = level() > before
	save_progress()
	return reward


# ---------------------------------------------------------------- theme

static func box(color: Color, radius: int = 18, margin: int = 14, border: Color = Color(0, 0, 0, 0)) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = color
	sb.set_corner_radius_all(radius)
	sb.set_content_margin_all(margin)
	if border.a > 0:
		sb.border_color = border
		sb.set_border_width_all(3)
	return sb


static func sys_font(names: Array, weight: int = 400, italic: bool = false) -> SystemFont:
	var f := SystemFont.new()
	f.font_names = PackedStringArray(names)
	f.font_weight = weight
	f.font_italic = italic
	return f


func build_theme() -> Theme:
	var t := Theme.new()
	var sans_names := ["Roboto", "Noto Sans", "Cantarell", "DejaVu Sans", "sans-serif"]
	var mono_names := ["Droid Sans Mono", "Roboto Mono", "Noto Sans Mono", "DejaVu Sans Mono", "Liberation Mono", "monospace"]
	t.default_font = sys_font(sans_names)
	t.default_font_size = 28
	mono_font = sys_font(mono_names)

	t.set_font("normal_font", "RichTextLabel", sys_font(sans_names))
	t.set_font("bold_font", "RichTextLabel", sys_font(sans_names, 700))
	t.set_font("italics_font", "RichTextLabel", sys_font(sans_names, 400, true))
	t.set_font("bold_italics_font", "RichTextLabel", sys_font(sans_names, 700, true))
	t.set_font("mono_font", "RichTextLabel", mono_font)
	for k in ["normal_font_size", "bold_font_size", "italics_font_size", "bold_italics_font_size"]:
		t.set_font_size(k, "RichTextLabel", 28)
	t.set_font_size("mono_font_size", "RichTextLabel", 21)
	t.set_color("default_color", "RichTextLabel", TEXT)
	t.set_constant("line_separation", "RichTextLabel", 4)

	t.set_color("font_color", "Label", TEXT)

	t.set_stylebox("normal", "Button", box(PANEL_LIGHT))
	t.set_stylebox("hover", "Button", box(PANEL_LIGHT.lightened(0.08)))
	t.set_stylebox("pressed", "Button", box(PANEL_LIGHT.darkened(0.2)))
	t.set_stylebox("disabled", "Button", box(PANEL.darkened(0.1)))
	t.set_stylebox("focus", "Button", StyleBoxEmpty.new())
	t.set_color("font_color", "Button", TEXT)
	t.set_color("font_hover_color", "Button", TEXT)
	t.set_color("font_pressed_color", "Button", TEXT)
	t.set_color("font_focus_color", "Button", TEXT)
	t.set_color("font_disabled_color", "Button", MUTED.darkened(0.2))
	t.set_font_size("font_size", "Button", 28)

	var bar_bg := box(PANEL_LIGHT, 10, 0)
	var bar_fill := box(GOLD, 10, 0)
	t.set_stylebox("background", "ProgressBar", bar_bg)
	t.set_stylebox("fill", "ProgressBar", bar_fill)
	t.set_font_size("font_size", "ProgressBar", 18)

	t.set_stylebox("panel", "PanelContainer", box(PANEL, 22, 22))
	t.set_constant("separation", "VBoxContainer", 16)
	t.set_constant("separation", "HBoxContainer", 12)
	t.set_constant("h_separation", "HFlowContainer", 12)
	t.set_constant("v_separation", "HFlowContainer", 12)
	return t
