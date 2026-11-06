extends Control
## A burst of falling confetti drawn on top of everything. Call burst().

const COLORS := [Color("#ff7a59"), Color("#ffd36e"), Color("#4cd98a"), Color("#7fd1ff"), Color("#e59cff")]
var bits: Array = []


func _init() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE


func burst(count: int = 90) -> void:
	for i in count:
		bits.append({
			"pos": Vector2(randf_range(0.2, 0.8) * size.x, size.y * 0.35),
			"vel": Vector2(randf_range(-420, 420), randf_range(-1100, -400)),
			"rot": randf() * TAU,
			"spin": randf_range(-8, 8),
			"color": COLORS[i % COLORS.size()],
			"life": randf_range(1.6, 2.6),
		})


func _process(delta: float) -> void:
	if bits.is_empty():
		return
	for b in bits:
		b["vel"] += Vector2(0, 1400) * delta
		b["vel"] *= 0.99
		b["pos"] += b["vel"] * delta
		b["rot"] += b["spin"] * delta
		b["life"] -= delta
	bits = bits.filter(func(b): return b["life"] > 0.0)
	queue_redraw()


func _draw() -> void:
	for b in bits:
		draw_set_transform(b["pos"], b["rot"], Vector2.ONE)
		var c: Color = b["color"]
		c.a = clampf(b["life"], 0.0, 1.0)
		draw_rect(Rect2(-7, -4, 14, 8), c)
	draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
