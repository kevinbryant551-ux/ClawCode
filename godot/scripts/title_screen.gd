extends Control
## Title screen: Pinch, the game name, your stats and a big Play button.

const UI := preload("res://scripts/ui.gd")
const Pinch := preload("res://scripts/pinch.gd")

var main
var reset_armed := false
var reset_btn: Button
var unlock_btn: Button


func setup(m) -> void:
	main = m
	var col := UI.vbox(22)
	add_child(UI.margin(col, 40, 60, 40, 40))
	get_child(0).set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	var spacer := Control.new()
	spacer.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(spacer)
	var pinch = Pinch.new(Vector2(360, 280))
	pinch.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	col.add_child(pinch)
	col.add_child(UI.label("ClawCode", 76, Game.ACCENT, true))
	col.add_child(UI.label("Learn Rust one pinch at a time.\nTap, snap and build real Rust code.", 28, Game.MUTED, true))

	var p: Dictionary = Game.progress
	var stats := "Level %d  ·  %s\n%d XP   ·   %d shells   ·   streak %d" % [
		Game.level(), Game.level_title(), int(p["xp"]), int(p["shells"]), int(p["streak"])]
	col.add_child(UI.label(stats, 26, Game.GOLD, true))

	var spacer2 := Control.new()
	spacer2.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(spacer2)

	var done := 0
	for l in Game.lessons:
		if Game.is_done(l["id"]):
			done += 1
	var play := UI.button("Play" if done == 0 else "Continue  (%d/%d)" % [done, Game.lessons.size()], Game.ACCENT, 40, 120)
	play.pressed.connect(func(): main.show_map())
	col.add_child(play)

	var row := UI.hbox()
	unlock_btn = UI.button("", Game.PANEL_LIGHT, 22, 72)
	unlock_btn.pressed.connect(_toggle_unlock)
	row.add_child(unlock_btn)
	reset_btn = UI.button("Reset progress", Game.PANEL_LIGHT, 22, 72)
	reset_btn.pressed.connect(_reset)
	row.add_child(reset_btn)
	col.add_child(row)
	_update_unlock_text()


func _update_unlock_text() -> void:
	unlock_btn.text = "Unlock all: ON" if Game.progress.get("unlock_all", false) else "Unlock all: off"


func _toggle_unlock() -> void:
	Game.progress["unlock_all"] = not Game.progress.get("unlock_all", false)
	Game.save_progress()
	_update_unlock_text()


func _reset() -> void:
	if not reset_armed:
		reset_armed = true
		reset_btn.text = "Tap again to erase"
		UI.style_button(reset_btn, Game.BAD)
		return
	Game.reset_progress()
	main.show_title()


func go_back() -> void:
	get_tree().quit()
