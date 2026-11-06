extends Control
## The lesson map: your level and XP, then every lesson grouped by chapter.
## Finished lessons are green, the next one glows orange, locked ones are grey.

const UI := preload("res://scripts/ui.gd")

var main
var scroll: ScrollContainer


func setup(m) -> void:
	main = m
	var col := UI.vbox(14)
	var root := UI.margin(col, 28, 28, 28, 20)
	add_child(root)
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	var top := UI.hbox()
	var back := UI.button("‹", Game.PANEL_LIGHT, 40, 80)
	back.custom_minimum_size = Vector2(90, 80)
	back.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	back.pressed.connect(go_back)
	top.add_child(back)
	var p: Dictionary = Game.progress
	var info := UI.vbox(2)
	info.add_child(UI.label("Level %d  ·  %s" % [Game.level(), Game.level_title()], 30, Game.TEXT))
	info.add_child(UI.label("%d XP  ·  %d shells  ·  streak %d" % [int(p["xp"]), int(p["shells"]), int(p["streak"])], 22, Game.GOLD))
	top.add_child(info)
	col.add_child(top)

	var lp: Array = Game.level_progress()
	var xp_bar := UI.bar(Game.GOLD, 16)
	xp_bar.max_value = lp[1]
	xp_bar.value = lp[0]
	col.add_child(xp_bar)

	var sc: Array = UI.scroller(14)
	scroll = sc[0]
	var list: VBoxContainer = sc[1]
	col.add_child(scroll)

	var chapter := ""
	var target: Control = null
	var next_i := Game.next_lesson_index()
	for i in Game.lessons.size():
		var l: Dictionary = Game.lessons[i]
		if l["chapter"] != chapter:
			chapter = l["chapter"]
			var h := UI.label(chapter.to_upper(), 24, Game.BLUE)
			list.add_child(UI.margin(h, 4, 18, 0, 0))
		var done := Game.is_done(l["id"])
		var open := Game.is_unlocked(i)
		var boss: bool = l["boss"] != null
		var title := "%d.  %s" % [int(l["order"]), l["title"]]
		if boss:
			title = "%d.  BOSS: %s" % [int(l["order"]), l["boss"]["name"]]
		var sub := ", ".join(PackedStringArray(l["concepts"]))
		var mark := "  [done]" if done else ("" if open else "  [locked]")
		var b := UI.button(title + mark + "\n" + sub, Color(0, 0, 0, 0), 26, 110)
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		var color := Game.PANEL_LIGHT
		if done:
			color = Color("#1f4a36")
		elif boss and open:
			color = Color("#5a2330")
		var border := Game.ACCENT if i == next_i and open and not done else Color(0, 0, 0, 0)
		UI.style_button(b, color, border)
		b.disabled = not open
		b.pressed.connect(main.show_lesson.bind(i))
		list.add_child(b)
		if i == next_i:
			target = b
	if target:
		_scroll_to.call_deferred(target)


func _scroll_to(c: Control) -> void:
	await get_tree().process_frame
	if is_instance_valid(c) and is_instance_valid(scroll):
		scroll.ensure_control_visible(c)


func go_back() -> void:
	main.show_title()
