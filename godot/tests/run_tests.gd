extends SceneTree
## Headless unit tests for the lesson data and puzzle logic.
## Run from the repo root:
##   godot --headless --path godot --script res://tests/run_tests.gd

const Puzzle := preload("res://scripts/puzzle.gd")

var failures := 0


func expect(cond: bool, msg: String) -> void:
	if not cond:
		failures += 1
		printerr("FAIL: ", msg)


func _init() -> void:
	var f := FileAccess.open("res://data/lessons.json", FileAccess.READ)
	expect(f != null, "lessons.json opens")
	if f == null:
		quit(1)
		return
	var data = JSON.parse_string(f.get_as_text())
	expect(typeof(data) == TYPE_DICTIONARY, "lessons.json parses")
	var lessons: Array = data["lessons"]
	expect(lessons.size() >= 5, "at least 5 lessons")
	var taps := 0
	for l in lessons:
		var id: String = l["id"]
		for st in l["steps"]:
			if st.has("card"):
				expect(int(st["card"]) < l["cards"].size(), id + ": card index in range")
			else:
				var q: Dictionary = l["quizzes"][int(st["quiz"])]
				expect(int(q["answer"]) < q["choices"].size(), id + ": quiz answer in range")
		expect(not l["taps"].is_empty(), id + ": has tap puzzles")
		for tap in l["taps"]:
			taps += 1
			test_puzzle(id, tap)
	test_order_duplicates()
	print("Checked %d lessons and %d tap puzzles." % [lessons.size(), taps])
	if failures == 0:
		print("ALL TESTS PASSED")
		quit(0)
	else:
		printerr("%d test(s) failed" % failures)
		quit(1)


func test_puzzle(id: String, tap: Dictionary) -> void:
	var p = Puzzle.new(tap, 42)
	expect(not p.is_solved(), id + ": a fresh puzzle is not solved")
	expect(not p.is_full(), id + ": a fresh puzzle is not full")
	# Solve it by tapping the right tiles, like a player would.
	var wanted: Array = []
	if p.kind == "fill":
		wanted = tap["answer"]
		var bb: String = p.fill_bbcode()
		expect(bb.count("[url=") == tap["answer"].size(), id + ": one link per blank")
	else:
		for l in tap["lines"]:
			wanted.append(l["text"])
	for w in wanted:
		var placed := false
		for t in p.tiles.size():
			if not p.is_used(t) and p.tile_text(t) == str(w):
				placed = p.place(t)
				break
		expect(placed, id + ": could place tile " + str(w))
	expect(p.is_full(), id + ": full after placing every answer")
	expect(p.is_solved(), id + ": solved after placing the right tiles")
	# Undo then reveal.
	p.undo()
	expect(not p.is_solved(), id + ": undo makes it unsolved")
	p.reveal()
	expect(p.is_solved(), id + ": reveal solves it")
	# A decoy in the first slot must be wrong.
	if not tap["decoys"].is_empty():
		var q = Puzzle.new(tap, 7)
		for t in q.tiles.size():
			var txt: String = q.tile_text(t)
			var d = tap["decoys"][0]
			var dtext: String = str(d["text"]) if typeof(d) == TYPE_DICTIONARY else str(d)
			if txt == dtext:
				q.place(t)
				break
		expect(q.wrong_slots().has(0), id + ": a decoy is marked wrong")


func test_order_duplicates() -> void:
	var tap := {
		"kind": "order", "prompt": "", "explain": "", "hp": 4, "decoys": [],
		"lines": [{"text": "a {", "bb": ""}, {"text": "}", "bb": ""}, {"text": "b {", "bb": ""}, {"text": "}", "bb": ""}],
	}
	var p = Puzzle.new(tap, 3)
	var order := ["a {", "}", "b {", "}"]
	for w in order:
		for t in p.tiles.size():
			if not p.is_used(t) and p.tile_text(t) == w:
				p.place(t)
				break
	expect(p.is_solved(), "identical lines are interchangeable")
