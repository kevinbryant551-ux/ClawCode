extends Control
## One lesson, in phases:
##   learn      reading cards and pop quizzes (tap a choice)
##   boss       the boss's entrance (boss lessons only)
##   puzzle     tap puzzles: fill the blanks or put lines in order
##   victory    rewards and confetti
## Nothing here needs a keyboard: every action is a button.

const UI := preload("res://scripts/ui.gd")
const Puzzle := preload("res://scripts/puzzle.gd")
const Pinch := preload("res://scripts/pinch.gd")

const MAX_HEARTS := 3

var main
var index: int
var lesson: Dictionary
var steps: Array
var taps: Array

var phase := "learn"
var step_i := 0
var tap_i := 0
var puzzle  # the current Puzzle (see puzzle.gd)
var solved := false
var reveal_offered := false

var hearts := MAX_HEARTS
var mistakes := 0
var round_mistakes := 0
var hints_used := 0
var hint_i := 0
var revealed := false
var combo := 0
var best_combo := 0
var quiz_wrong := false
var boss_hp := 0
var boss_max := 0

# Layout
var title_label: Label
var status_label: Label
var step_bar: ProgressBar
var boss_row: HBoxContainer
var boss_bar: ProgressBar
var scroll: ScrollContainer
var body: VBoxContainer
var footer: VBoxContainer
var pinch
var pinch_says: RichTextLabel

# Puzzle widgets
var code_label: RichTextLabel
var feedback: RichTextLabel
var tile_buttons: Array = []
var placed_box: VBoxContainer
var check_btn: Button


func setup(m, i: int) -> void:
	main = m
	index = i
	lesson = Game.lessons[i]
	steps = lesson["steps"]
	taps = lesson["taps"]
	for t in taps:
		boss_max += int(t["hp"])
	boss_hp = boss_max

	var col := UI.vbox(12)
	var root := UI.margin(col, 24, 24, 24, 20)
	add_child(root)
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	var top := UI.hbox()
	var back := UI.button("‹", Game.PANEL_LIGHT, 40, 80)
	back.custom_minimum_size = Vector2(90, 80)
	back.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	back.pressed.connect(go_back)
	top.add_child(back)
	title_label = UI.label(lesson["title"], 28, Game.TEXT)
	top.add_child(title_label)
	status_label = UI.label("", 26, Game.BAD)
	status_label.size_flags_horizontal = Control.SIZE_SHRINK_END
	status_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	top.add_child(status_label)
	col.add_child(top)

	step_bar = UI.bar(Game.BLUE, 12)
	step_bar.max_value = steps.size() + taps.size()
	col.add_child(step_bar)

	boss_row = UI.hbox()
	var boss_name := UI.label("", 22, Game.BAD)
	boss_name.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	boss_row.add_child(boss_name)
	boss_bar = UI.bar(Game.BAD, 22)
	boss_bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	boss_bar.max_value = maxi(boss_max, 1)
	boss_bar.value = boss_hp
	boss_row.add_child(boss_bar)
	boss_row.visible = false
	if is_boss():
		boss_name.text = lesson["boss"]["name"] + " HP"
	col.add_child(boss_row)

	var sc: Array = UI.scroller(18)
	scroll = sc[0]
	body = sc[1]
	col.add_child(scroll)

	var pinch_row := UI.hbox(8)
	pinch = Pinch.new(Vector2(130, 100))
	pinch_row.add_child(pinch)
	pinch_says = UI.rich("", 24)
	pinch_says.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pinch_row.add_child(pinch_says)
	col.add_child(pinch_row)

	footer = UI.vbox(12)
	col.add_child(footer)

	if Game.is_done(lesson["id"]):
		say("Welcome back! Replays are for practice: no XP this time, but the streak counts.")
	else:
		say("Let's learn [b]%s[/b]!" % ", ".join(PackedStringArray(lesson["concepts"])))
	show_step()


func is_boss() -> bool:
	return lesson["boss"] != null


func say(bb: String, mood: String = "") -> void:
	pinch_says.text = bb
	if mood != "":
		pinch.jump(mood)


func update_status() -> void:
	var s := ""
	if phase == "puzzle":
		s = "♥".repeat(hearts) + "♡".repeat(MAX_HEARTS - hearts)
	elif combo >= 2:
		s = "combo x%d" % combo
	status_label.text = s
	status_label.add_theme_color_override("font_color", Game.BAD if phase == "puzzle" else Game.GOLD)


func scroll_top() -> void:
	scroll.scroll_vertical = 0


# ------------------------------------------------------------------ learn

func show_step() -> void:
	UI.clear(body)
	UI.clear(footer)
	scroll_top()
	update_status()
	if step_i >= steps.size():
		start_challenge()
		return
	step_bar.value = step_i
	var st: Dictionary = steps[step_i]
	if st.has("card"):
		var card_i := int(st["card"])
		body.add_child(UI.panel(UI.rich(lesson["cards"][card_i])))
		var next := UI.button("Next  ›", Game.ACCENT)
		next.pressed.connect(next_step)
		footer.add_child(next)
	else:
		show_quiz(lesson["quizzes"][int(st["quiz"])])


func next_step() -> void:
	step_i += 1
	show_step()


func show_quiz(q: Dictionary) -> void:
	quiz_wrong = false
	body.add_child(UI.label("POP QUIZ", 24, Game.GOLD))
	body.add_child(UI.panel(UI.rich(q["question"])))
	var buttons: Array = []
	for ci in q["choices"].size():
		var b := UI.button(str(q["choices"][ci]), Game.PANEL_LIGHT, 26, 90)
		b.pressed.connect(_on_choice.bind(q, ci, buttons))
		buttons.append(b)
		footer.add_child(b)
	say("Tap the answer you think is right.")


func _on_choice(q: Dictionary, ci: int, buttons: Array) -> void:
	var b: Button = buttons[ci]
	if ci == int(q["answer"]):
		UI.style_button(b, Game.GOOD.darkened(0.35))
		for other in buttons:
			other.disabled = true
		b.disabled = false
		b.mouse_filter = Control.MOUSE_FILTER_IGNORE
		if not quiz_wrong:
			combo += 1
			best_combo = maxi(best_combo, combo)
			say("Correct!" + (" Combo x%d!" % combo if combo >= 2 else ""), "happy")
		else:
			say("That's it!", "happy")
		if str(q["explain"]) != "":
			body.add_child(UI.panel(UI.rich(q["explain"], 26), Color("#173a2b")))
		var next := UI.button("Next  ›", Game.ACCENT)
		next.pressed.connect(next_step)
		footer.add_child(next)
		update_status()
		_scroll_bottom.call_deferred()
	else:
		quiz_wrong = true
		combo = 0
		UI.style_button(b, Game.BAD.darkened(0.3))
		b.disabled = true
		UI.shake(b)
		say("Not that one. Try another!", "sad")
		update_status()


func _scroll_bottom() -> void:
	await get_tree().process_frame
	scroll.scroll_vertical = int(scroll.get_v_scroll_bar().max_value)


# ------------------------------------------------------------------ challenge

func start_challenge() -> void:
	if is_boss() and phase != "boss":
		phase = "boss"
		show_boss_intro()
		return
	phase = "puzzle"
	tap_i = 0
	boss_row.visible = is_boss()
	if taps.is_empty():
		victory()
		return
	show_puzzle()


func show_boss_intro() -> void:
	UI.clear(body)
	UI.clear(footer)
	var b: Dictionary = lesson["boss"]
	body.add_child(UI.label("A wild %s appears!" % b["name"], 36, Game.BAD, true))
	var art := UI.label(b["art"], 26, Game.BAD)
	art.autowrap_mode = TextServer.AUTOWRAP_OFF
	art.add_theme_font_override("font", Game.mono_font)
	art.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	body.add_child(art)
	body.add_child(UI.panel(UI.rich("[i]\"%s\"[/i]" % esc(b["taunt"]))))
	body.add_child(UI.label("Every blank you fill correctly is one hit. Its HP: %d." % boss_max, 24, Game.MUTED))
	say("Eek! We can take it. Remember what we learned!", "wow")
	var fight := UI.button("Fight!", Game.BAD, 40, 110)
	fight.pressed.connect(start_challenge)
	footer.add_child(fight)


static func esc(s: String) -> String:
	return Puzzle.esc(s)


func show_puzzle() -> void:
	UI.clear(body)
	UI.clear(footer)
	scroll_top()
	solved = false
	reveal_offered = false
	round_mistakes = 0
	tile_buttons.clear()
	var tap: Dictionary = taps[tap_i]
	puzzle = Puzzle.new(tap)
	step_bar.value = steps.size() + tap_i
	update_status()

	var heading := "Puzzle %d of %d" % [tap_i + 1, taps.size()]
	body.add_child(UI.label(heading.to_upper(), 22, Game.GOLD))
	body.add_child(UI.rich(tap["prompt"], 27))

	if puzzle.kind == "fill":
		code_label = UI.rich("", -1)
		code_label.meta_underlined = false
		code_label.meta_clicked.connect(_on_blank_tapped)
		code_label.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
		body.add_child(UI.panel(code_label, Color("#141a28"), 16))
		feedback = UI.rich("", 26)
		body.add_child(feedback)
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 12)
		flow.add_theme_constant_override("v_separation", 12)
		for ti in puzzle.tiles.size():
			var b := UI.code_button(puzzle.tile_text(ti), Game.mono_font, 28)
			UI.style_button(b, Color("#2f3d5e"))
			b.pressed.connect(_on_tile.bind(ti))
			tile_buttons.append(b)
			flow.add_child(b)
		footer.add_child(flow)
		say("Tap a tile to drop it in the orange blank. Tap a filled blank to take it back.")
	else:
		body.add_child(UI.label("YOUR CODE", 20, Game.MUTED))
		placed_box = UI.vbox(6)
		body.add_child(UI.panel(placed_box, Color("#141a28"), 14))
		feedback = UI.rich("", 26)
		body.add_child(feedback)
		body.add_child(UI.label("PIECES  (tap to add)", 20, Game.MUTED))
		var pieces := UI.vbox(8)
		for ti in puzzle.tiles.size():
			var b := UI.code_button(puzzle.tile_text(ti), Game.mono_font, 24, true)
			UI.style_button(b, Color("#2f3d5e"))
			b.pressed.connect(_on_tile.bind(ti))
			tile_buttons.append(b)
			pieces.add_child(b)
		body.add_child(pieces)
		say("Tap the lines in order, top to bottom. Tap a line in your code to take it back.")

	var row := UI.hbox()
	var undo := UI.button("Undo", Game.PANEL_LIGHT, 26, 88)
	undo.pressed.connect(_on_undo)
	row.add_child(undo)
	var hint := UI.button("Hint", Game.PANEL_LIGHT, 26, 88)
	hint.pressed.connect(_on_hint)
	row.add_child(hint)
	check_btn = UI.button("Check", Game.ACCENT, 30, 88)
	check_btn.pressed.connect(_on_check)
	row.add_child(check_btn)
	footer.add_child(row)
	refresh_puzzle()


func refresh_puzzle(wrong: Array = []) -> void:
	if puzzle.kind == "fill":
		code_label.text = puzzle.fill_bbcode(wrong)
	else:
		UI.clear(placed_box)
		if puzzle.placed.is_empty():
			placed_box.add_child(UI.label("(empty: tap pieces below)", 22, Game.MUTED))
		for pos in puzzle.placed.size():
			var b := UI.code_button(puzzle.tile_text(puzzle.placed[pos]), Game.mono_font, 24, true)
			UI.style_button(b, Game.BAD.darkened(0.4) if wrong.has(pos) else Color("#3b4f7a"))
			b.custom_minimum_size = Vector2(0, 64)
			b.pressed.connect(_on_placed_tapped.bind(pos))
			b.disabled = solved
			placed_box.add_child(b)
	for ti in tile_buttons.size():
		var b: Button = tile_buttons[ti]
		b.disabled = solved or puzzle.is_used(ti)
		b.visible = not (puzzle.kind == "order" and puzzle.is_used(ti))
	check_btn.disabled = solved or not puzzle.is_full()


func _on_tile(ti: int) -> void:
	if solved:
		return
	if puzzle.place(ti):
		feedback.text = ""
		refresh_puzzle()


func _on_blank_tapped(meta) -> void:
	if solved:
		return
	puzzle.remove(int(str(meta)))
	refresh_puzzle()


func _on_placed_tapped(pos: int) -> void:
	if solved:
		return
	puzzle.remove(pos)
	refresh_puzzle()


func _on_undo() -> void:
	if solved:
		return
	puzzle.undo()
	refresh_puzzle()


func _on_hint() -> void:
	var hints: Array = lesson["hints"]
	if hints.is_empty():
		say("No hints here. You've got this!")
		return
	hints_used += 1
	say("[b]Hint:[/b] " + str(hints[mini(hint_i, hints.size() - 1)]), "wow")
	hint_i += 1


func _on_check() -> void:
	if solved:
		return
	var wrong: Array = puzzle.wrong_slots()
	if wrong.is_empty():
		_puzzle_solved()
		return
	mistakes += 1
	round_mistakes += 1
	hearts -= 1
	var n := wrong.size()
	var what := "blank" if puzzle.kind == "fill" else "line"
	var msg := "Not quite: %d %s%s in red %s wrong." % [n, what, "s" if n > 1 else "", "are" if n > 1 else "is"]
	if puzzle.kind == "order":
		msg = "Not quite. The first line in red is in the wrong spot (or is a decoy)."
		wrong = [wrong[0]]
	feedback.text = "[color=#ff8c96]%s[/color]" % msg
	if hearts <= 0:
		hearts = MAX_HEARTS
		say("Ouch! Out of hearts... Pinch patches you up. Hearts refilled!", "sad")
	else:
		say("Close! Fix the red parts and check again." + (" Try the Hint button." if round_mistakes >= 2 else ""), "sad")
	UI.shake(check_btn)
	update_status()
	refresh_puzzle(wrong)
	if round_mistakes >= 3 and not reveal_offered:
		reveal_offered = true
		var reveal_btn := UI.button("Show me the answer", Game.PANEL_LIGHT, 24, 80)
		reveal_btn.pressed.connect(_on_reveal)
		footer.add_child(reveal_btn)


func _on_reveal() -> void:
	revealed = true
	puzzle.reveal()
	refresh_puzzle()
	say("Here it is. Read it through, then tap Check.", "wow")


func _puzzle_solved() -> void:
	solved = true
	var tap: Dictionary = taps[tap_i]
	refresh_puzzle()
	if is_boss():
		boss_hp = maxi(0, boss_hp - int(tap["hp"]))
		var tw := create_tween()
		tw.tween_property(boss_bar, "value", float(boss_hp), 0.5)
		say("POW! %s takes %d damage!" % [lesson["boss"]["name"], int(tap["hp"])], "happy")
	else:
		say("Shell yeah! That's right!", "happy")
	main.confetti.burst(30)
	var ok := "[color=#7be3a6][b]Correct![/b][/color]"
	if str(tap["explain"]) != "":
		ok += " " + str(tap["explain"])
	feedback.text = ok
	UI.clear(footer)
	var next := UI.button("Next  ›" if tap_i + 1 < taps.size() else "Finish!", Game.GOOD.darkened(0.25), 32)
	next.pressed.connect(_next_puzzle)
	footer.add_child(next)
	_scroll_bottom.call_deferred()


func _next_puzzle() -> void:
	tap_i += 1
	if tap_i >= taps.size():
		victory()
	else:
		show_puzzle()


# ------------------------------------------------------------------ victory

func victory() -> void:
	phase = "victory"
	UI.clear(body)
	UI.clear(footer)
	scroll_top()
	step_bar.value = step_bar.max_value
	update_status()
	var reward: Dictionary = Game.record_pass(lesson, mistakes, hints_used, revealed, best_combo)
	main.confetti.burst(120)
	var big = Pinch.new(Vector2(320, 240))
	big.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	body.add_child(big)
	big.jump("happy")
	var headline := "Lesson cleared!"
	if is_boss():
		headline = "%s defeated!" % lesson["boss"]["name"]
	body.add_child(UI.label(headline, 44, Game.GOOD, true))
	if is_boss() and str(lesson["boss"]["defeat"]) != "":
		body.add_child(UI.panel(UI.rich("[i]\"%s\"[/i]" % esc(lesson["boss"]["defeat"]))))
	body.add_child(UI.panel(UI.rich(lesson["success"])))
	if reward["first_time"]:
		var lines := ""
		for l in reward["lines"]:
			lines += "%s  [color=#ffd36e]%+d XP[/color]\n" % [l[0], int(l[1])]
		lines += "[b]Total: +%d XP, +%d shells[/b]" % [int(reward["xp"]), int(reward["shells"])]
		body.add_child(UI.panel(UI.rich(lines, 26)))
		if reward["level_up"]:
			body.add_child(UI.label("LEVEL UP! You are now level %d: %s" % [Game.level(), Game.level_title()], 32, Game.GOLD, true))
	else:
		body.add_child(UI.label("Replay complete. Streak: %d day(s)." % int(reward["streak"]), 26, Game.MUTED, true))
	say("You did it! Want to keep going?", "happy")
	if index + 1 < Game.lessons.size():
		var nxt := UI.button("Next lesson  ›", Game.ACCENT, 34, 110)
		nxt.pressed.connect(main.show_lesson.bind(index + 1))
		footer.add_child(nxt)
	var map := UI.button("Back to map", Game.PANEL_LIGHT, 28, 88)
	map.pressed.connect(main.show_map)
	footer.add_child(map)


func go_back() -> void:
	main.show_map()
