extends Control
## Pinch the crab, drawn with simple shapes (no image files needed).
## Call jump() to celebrate and set `mood` to "happy", "sad" or "wow".

const RED := Color("#e8553f")
const RED_DARK := Color("#b83a2a")
const EYE := Color("#ffffff")
const INK := Color("#10141f")

var mood := "happy"
var t := 0.0
var bounce := 0.0


func _init(min_size: Vector2 = Vector2(200, 160)) -> void:
	custom_minimum_size = min_size
	mouse_filter = Control.MOUSE_FILTER_IGNORE


func jump(new_mood: String = "happy") -> void:
	mood = new_mood
	bounce = 1.0


func _process(delta: float) -> void:
	t += delta
	bounce = maxf(0.0, bounce - delta * 2.5)
	queue_redraw()


func _draw() -> void:
	var s := minf(size.x / 220.0, size.y / 170.0)
	var hop := sin(bounce * PI) * 26.0 if mood != "sad" else 0.0
	var c := size / 2.0 + Vector2(0, 20 * s + sin(t * 2.0) * 3.0 * s - hop * s)
	draw_set_transform(c, 0.0, Vector2(s, s))

	# Legs: three on each side.
	for side in [-1.0, 1.0]:
		for k in 3:
			var a := Vector2(side * (28 + k * 12), 18)
			var b := Vector2(side * (58 + k * 12), 46 + k * 5 + sin(t * 6.0 + k) * 2.0)
			draw_line(a, b, RED_DARK, 6.0, true)

	# Arms and claws (they wave a little).
	for side in [-1.0, 1.0]:
		var wave := sin(t * 3.0 + side) * (10.0 if mood == "happy" else 3.0)
		var shoulder := Vector2(side * 42, -2)
		var claw := Vector2(side * 82, -48 + wave)
		if mood == "sad":
			claw = Vector2(side * 78, 10)
		draw_line(shoulder, claw, RED, 9.0, true)
		draw_circle(claw, 22.0, RED)
		var open := 8.0 + absf(sin(t * 4.0)) * 6.0
		draw_colored_polygon(PackedVector2Array([
			claw + Vector2(side * 2, 0),
			claw + Vector2(side * 26, -open),
			claw + Vector2(side * 26, open),
		]), INK)

	# Eye stalks and eyes (with a blink every few seconds).
	for side in [-1.0, 1.0]:
		var base := Vector2(side * 14, -12)
		var eye := Vector2(side * 20, -52)
		draw_line(base, eye, RED_DARK, 6.0, true)
		if fmod(t, 3.7) < 0.12:
			draw_line(eye + Vector2(-9, 0), eye + Vector2(9, 0), INK, 4.0, true)
		else:
			draw_circle(eye, 12.0, EYE)
			var look := Vector2(sin(t * 0.7) * 3.0, 2.0 if mood == "sad" else 0.0)
			draw_circle(eye + look, 6.0, INK)

	# Body: a squashed circle.
	draw_set_transform(c + Vector2(0, 12) * s, 0.0, Vector2(s * 1.45, s))
	draw_circle(Vector2.ZERO, 40.0, RED)
	draw_set_transform(c, 0.0, Vector2(s, s))

	# Mouth.
	match mood:
		"sad":
			draw_arc(Vector2(0, 30), 12.0, PI + 0.4, TAU - 0.4, 12, INK, 4.0, true)
		"wow":
			draw_circle(Vector2(0, 18), 8.0, INK)
		_:
			draw_arc(Vector2(0, 10), 14.0, 0.3, PI - 0.3, 12, INK, 4.0, true)
	draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
