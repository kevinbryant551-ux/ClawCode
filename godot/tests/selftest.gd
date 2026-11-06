extends Node
## End-to-end smoke test: plays every lesson through the real screens by
## "pressing" buttons in code. Started by main.gd when run with:
##   godot --headless --path godot -- --selftest
## Uses a separate save file so your real progress is never touched.

var failures := 0


func fail(msg: String) -> void:
	failures += 1
	printerr("SELFTEST FAIL: ", msg)


func press(b) -> void:
	if b is Button:
		b.emit_signal("pressed")


func run(main) -> void:
	var tree: SceneTree = main.get_tree()
	print("SELFTEST: starting")
	Game.save_path = "user://selftest_progress.json"
	Game.reset_progress()
	Game.progress["unlock_all"] = true
	main.show_title()
	await tree.process_frame
	main.show_map()
	await tree.process_frame
	for i in Game.lessons.size():
		await play_lesson(main, tree, i)
	if int(Game.progress["xp"]) <= 0:
		fail("no XP was earned")
	main.show_map()
	await tree.process_frame
	Game.reset_progress()
	if failures == 0:
		print("SELFTEST PASSED: played %d lessons" % Game.lessons.size())
		tree.quit(0)
	else:
		printerr("SELFTEST: %d failure(s)" % failures)
		tree.quit(1)


func play_lesson(main, tree: SceneTree, i: int) -> void:
	main.show_lesson(i)
	await tree.process_frame
	var s = main.current
	var id: String = s.lesson["id"]
	var guard := 0
	while s.phase != "victory":
		guard += 1
		if guard > 300:
			fail(id + ": stuck in phase " + s.phase)
			return
		await tree.process_frame
		var foot: Array = s.footer.get_children()
		if s.phase == "learn":
			var st: Dictionary = s.steps[s.step_i]
			if st.has("quiz"):
				var q: Dictionary = s.lesson["quizzes"][int(st["quiz"])]
				var answer := int(q["answer"])
				# Tap a wrong choice first (on odd lessons), then the right one.
				if i % 2 == 1:
					press(foot[(answer + 1) % q["choices"].size()])
				press(foot[answer])
				await tree.process_frame
				press(s.footer.get_children()[-1])  # Next
			else:
				press(foot[0])  # Next
		elif s.phase == "boss":
			press(foot[0])  # Fight!
		elif s.phase == "puzzle":
			if s.solved:
				press(foot[-1])  # Next / Finish
				continue
			var p = s.puzzle
			if i % 2 == 0:
				# Tap the right tiles by text, like a player.
				var wanted: Array = []
				if p.kind == "fill":
					wanted = p.data["answer"]
				else:
					for l in p.data["lines"]:
						wanted.append(l["text"])
				for w in wanted:
					for t in p.tiles.size():
						if not p.is_used(t) and p.tile_text(t) == str(w):
							s._on_tile(t)
							break
			else:
				# Make a mistake first, then use "Show me the answer".
				for t in p.tiles.size():
					s._on_tile(t)
				if not p.is_solved():
					s._on_check()
				s._on_hint()
				s._on_reveal()
			if not p.is_solved():
				fail(id + ": puzzle %d not solved" % s.tap_i)
				return
			s._on_check()
			if not s.solved:
				fail(id + ": check did not accept the right answer")
				return
	if not Game.is_done(id):
		fail(id + ": not marked done after victory")
	print("  played ", id)
