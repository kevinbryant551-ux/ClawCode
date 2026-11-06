extends RefCounted
## Small helpers that build touch-friendly widgets in code.
## Every screen is built from these, so the look stays consistent.


static func rich(bb: String, font_size: int = -1) -> RichTextLabel:
	var r := RichTextLabel.new()
	r.bbcode_enabled = true
	r.fit_content = true
	r.scroll_active = false
	r.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	r.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	r.mouse_filter = Control.MOUSE_FILTER_PASS
	if font_size > 0:
		for k in ["normal_font_size", "bold_font_size", "italics_font_size", "bold_italics_font_size"]:
			r.add_theme_font_size_override(k, font_size)
	r.text = bb
	return r


static func label(text: String, font_size: int = 28, color: Color = Color("#e8ecf4"), center: bool = false) -> Label:
	var l := Label.new()
	l.text = text
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.add_theme_font_size_override("font_size", font_size)
	l.add_theme_color_override("font_color", color)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if center:
		l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	return l


static func style_button(b: Button, color: Color, border: Color = Color(0, 0, 0, 0)) -> void:
	b.add_theme_stylebox_override("normal", _box(color, border))
	b.add_theme_stylebox_override("hover", _box(color.lightened(0.08), border))
	b.add_theme_stylebox_override("pressed", _box(color.darkened(0.2), border))
	b.add_theme_stylebox_override("disabled", _box(color.darkened(0.45), border.darkened(0.4)))


static func _box(color: Color, border: Color) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = color
	sb.set_corner_radius_all(18)
	sb.set_content_margin_all(14)
	if border.a > 0:
		sb.border_color = border
		sb.set_border_width_all(3)
	return sb


## A big button. Minimum height 88 px (about 9 mm on a phone) so it is easy to hit.
static func button(text: String, color: Color = Color(0, 0, 0, 0), font_size: int = 30, min_h: int = 88) -> Button:
	var b := Button.new()
	b.text = text
	b.custom_minimum_size = Vector2(0, min_h)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	b.add_theme_font_size_override("font_size", font_size)
	# PASS lets a finger drag that starts on a button still scroll the list.
	b.mouse_filter = Control.MOUSE_FILTER_PASS
	if color.a > 0:
		style_button(b, color)
	return b


## A button showing code in the monospace font, left aligned (tiles and lines).
static func code_button(text: String, mono: Font, font_size: int = 26, expand: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	b.custom_minimum_size = Vector2(88, 80)
	b.add_theme_font_override("font", mono)
	b.add_theme_font_size_override("font_size", font_size)
	b.mouse_filter = Control.MOUSE_FILTER_PASS
	if expand:
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
	return b


static func vbox(sep: int = 16) -> VBoxContainer:
	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", sep)
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	return v


static func hbox(sep: int = 12) -> HBoxContainer:
	var h := HBoxContainer.new()
	h.add_theme_constant_override("separation", sep)
	h.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	return h


static func margin(child: Control, l: int, t: int, r: int, b: int) -> MarginContainer:
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", l)
	m.add_theme_constant_override("margin_top", t)
	m.add_theme_constant_override("margin_right", r)
	m.add_theme_constant_override("margin_bottom", b)
	m.add_child(child)
	return m


## A vertical scroll area (drag with a finger). Returns [scroll, content_vbox].
static func scroller(sep: int = 18) -> Array:
	var s := ScrollContainer.new()
	s.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	s.size_flags_vertical = Control.SIZE_EXPAND_FILL
	s.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var v := vbox(sep)
	s.add_child(v)
	return [s, v]


## A rounded panel around a child.
static func panel(child: Control, color: Color = Color("#1a2133"), pad: int = 22) -> PanelContainer:
	var p := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = color
	sb.set_corner_radius_all(22)
	sb.set_content_margin_all(pad)
	p.add_theme_stylebox_override("panel", sb)
	p.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	p.add_child(child)
	return p


static func clear(node: Node) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


static func bar(color: Color, height: int = 18) -> ProgressBar:
	var p := ProgressBar.new()
	p.show_percentage = false
	p.custom_minimum_size = Vector2(0, height)
	p.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var fill := StyleBoxFlat.new()
	fill.bg_color = color
	fill.set_corner_radius_all(height / 2)
	p.add_theme_stylebox_override("fill", fill)
	var bg := StyleBoxFlat.new()
	bg.bg_color = Color("#25304a")
	bg.set_corner_radius_all(height / 2)
	p.add_theme_stylebox_override("background", bg)
	return p


## A quick side-to-side wiggle, used when an answer is wrong.
static func shake(node: Control) -> void:
	var x := node.position.x
	var tw := node.create_tween()
	for d in [-14, 12, -8, 5, 0]:
		tw.tween_property(node, "position:x", x + d, 0.05)
