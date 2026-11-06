extends Control
## The root of the app. It owns the background, the confetti layer and the
## current screen, and swaps screens (title -> map -> lesson).

const TitleScreen := preload("res://scripts/title_screen.gd")
const MapScreen := preload("res://scripts/map_screen.gd")
const LessonScreen := preload("res://scripts/lesson_screen.gd")
const Confetti := preload("res://scripts/confetti.gd")

var current: Control
var layer: Control
var confetti: Control


func _ready() -> void:
	Game.ensure_loaded()
	theme = Game.theme
	var bg := ColorRect.new()
	bg.color = Game.BG
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(bg)
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	layer = Control.new()
	add_child(layer)
	layer.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	confetti = Confetti.new()
	add_child(confetti)
	confetti.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	if OS.get_cmdline_user_args().has("--selftest"):
		var SelfTest = load("res://tests/selftest.gd")
		SelfTest.new().run(self)
	else:
		show_title()


func _switch(screen: Control) -> void:
	if current:
		layer.remove_child(current)
		current.queue_free()
	current = screen
	layer.add_child(screen)
	screen.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	screen.modulate.a = 0.0
	screen.create_tween().tween_property(screen, "modulate:a", 1.0, 0.18)


func show_title() -> void:
	var s = TitleScreen.new()
	s.setup(self)
	_switch(s)


func show_map() -> void:
	var s = MapScreen.new()
	s.setup(self)
	_switch(s)


func show_lesson(index: int) -> void:
	var s = LessonScreen.new()
	s.setup(self, index)
	_switch(s)


## Android's back button (or Esc on a computer) goes back one screen.
func go_back() -> void:
	if current and current.has_method("go_back"):
		current.go_back()
	else:
		get_tree().quit()


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_GO_BACK_REQUEST:
		go_back()


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_cancel"):
		go_back()
