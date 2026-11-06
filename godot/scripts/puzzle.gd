extends RefCounted
## Pure puzzle logic (no UI), so it can be unit-tested headless.
## Two kinds of tap puzzle, both built from a "tap" entry in lessons.json:
##   fill:  code with numbered blanks; tap tiles to fill the blanks in order.
##   order: shuffled lines of code; tap them in the right order.

var kind: String
var data: Dictionary
## fill: tile texts (answers + decoys, shuffled). order: piece dictionaries {text, bb}.
var tiles: Array = []
## fill: for each blank, the tile index placed there or -1. order: tile indexes in tapped order.
var placed: Array = []
var history: Array = []


func _init(tap: Dictionary, rng_seed: int = -1) -> void:
	data = tap
	kind = tap["kind"]
	var rng := RandomNumberGenerator.new()
	if rng_seed >= 0:
		rng.seed = rng_seed
	else:
		rng.randomize()
	if kind == "fill":
		for a in tap["answer"]:
			tiles.append(str(a))
		for d in tap["decoys"]:
			tiles.append(str(d))
		for i in blank_count():
			placed.append(-1)
	else:
		for l in tap["lines"]:
			tiles.append(l)
		for d in tap["decoys"]:
			tiles.append(d)
	_shuffle(tiles, rng)


static func _shuffle(arr: Array, rng: RandomNumberGenerator) -> void:
	for i in range(arr.size() - 1, 0, -1):
		var j := rng.randi_range(0, i)
		var tmp = arr[i]
		arr[i] = arr[j]
		arr[j] = tmp


func blank_count() -> int:
	return data["answer"].size() if kind == "fill" else data["lines"].size()


func is_used(tile: int) -> bool:
	return placed.has(tile)


## The blank the next tapped tile goes into (fill), or -1 when all are full.
func next_blank() -> int:
	return placed.find(-1) if kind == "fill" else (placed.size() if placed.size() < blank_count() else -1)


## Puts a tile in the next empty slot. Returns false if it can't be placed.
func place(tile: int) -> bool:
	if tile < 0 or tile >= tiles.size() or is_used(tile):
		return false
	if kind == "fill":
		var b := next_blank()
		if b < 0:
			return false
		placed[b] = tile
		history.append(b)
	else:
		if placed.size() >= blank_count():
			return false
		placed.append(tile)
		history.append(placed.size() - 1)
	return true


## Fill: empties one blank. Order: removes the line at that position.
func remove(slot: int) -> void:
	if kind == "fill":
		if slot >= 0 and slot < placed.size():
			placed[slot] = -1
			history.erase(slot)
	elif slot >= 0 and slot < placed.size():
		placed.remove_at(slot)
		history.clear()


func undo() -> void:
	if kind == "fill":
		while not history.is_empty():
			var b: int = history.pop_back()
			if placed[b] != -1:
				placed[b] = -1
				return
	elif not placed.is_empty():
		placed.pop_back()


func is_full() -> bool:
	return next_blank() == -1


func tile_text(tile: int) -> String:
	if tile < 0:
		return ""
	return tiles[tile] if kind == "fill" else str(tiles[tile]["text"])


## Returns the slots that are wrong (empty slots count as wrong). Empty array = solved.
## Order puzzles compare line text, so two identical lines (like two "}") are interchangeable.
func wrong_slots() -> Array:
	var wrong: Array = []
	if kind == "fill":
		for i in placed.size():
			if placed[i] < 0 or tile_text(placed[i]) != str(data["answer"][i]):
				wrong.append(i)
	else:
		var lines: Array = data["lines"]
		for i in lines.size():
			if i >= placed.size() or tile_text(placed[i]) != str(lines[i]["text"]):
				wrong.append(i)
	return wrong


func is_solved() -> bool:
	return wrong_slots().is_empty()


## Fills in the correct answer (used by "Show me").
func reveal() -> void:
	placed.clear()
	history.clear()
	if kind == "fill":
		var used := {}
		for a in data["answer"]:
			for t in tiles.size():
				if tiles[t] == str(a) and not used.has(t):
					used[t] = true
					placed.append(t)
					break
	else:
		var used := {}
		for l in data["lines"]:
			for t in tiles.size():
				if str(tiles[t]["text"]) == str(l["text"]) and not used.has(t):
					used[t] = true
					placed.append(t)
					break


## Escapes [ and ] so RichTextLabel shows them as text.
static func esc(s: String) -> String:
	return s.replace("[", "\u0001").replace("]", "[rb]").replace("\u0001", "[lb]")


## BBCode for a fill puzzle's code, with each blank as a tappable [url] link.
func fill_bbcode(wrong: Array = []) -> String:
	var out := "[code]"
	var nb := next_blank()
	for p in data["parts"]:
		if p.has("t"):
			out += str(p["t"])
		else:
			var b := int(p["b"])
			var tile: int = placed[b]
			var bg := "#2d3a58"
			var fg := "#ffffff"
			var label := "  ?  "
			if tile >= 0:
				label = " " + esc(tile_text(tile)) + " "
				bg = "#3b4f7a"
			if b == nb:
				bg = "#7a4a2a"
				label = " ___ "
			if wrong.has(b):
				bg = "#7a2a35"
			out += "[url=%d][bgcolor=%s][color=%s]%s[/color][/bgcolor][/url]" % [b, bg, fg, label]
	return out + "[/code]"
