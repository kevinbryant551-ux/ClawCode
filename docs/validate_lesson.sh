#!/usr/bin/env bash
# Rough manual validator used while the engine is being built.
# Usage: docs/validate_lesson.sh lessons/03-types.toml
# Requires python3 (tomllib) and rustc.
set -euo pipefail
f="$1"; d="$(mktemp -d)"
python3 - "$f" "$d" <<'PY'
import sys, tomllib, pathlib
f, d = sys.argv[1], pathlib.Path(sys.argv[2])
L = tomllib.load(open(f, "rb"))
c = L.get("check", {})
for which in ("solution", "starter"):
    code = L[which]
    (d / f"{which}_main.rs").write_text(code)
    if c.get("tests"):
        (d / f"{which}_test.rs").write_text(code + "\n#[cfg(test)]\nmod clawcode_tests {\n    use super::*;\n" + c["tests"] + "\n}\n")
(d / "expected.txt").write_text(c.get("expected_output", "") or "")
(d / "has_out").write_text("1" if c.get("expected_output") else "")
(d / "has_tests").write_text("1" if c.get("tests") else "")
print("parsed OK:", L["id"])
PY
cd "$d"
norm() { sed -e 's/[[:space:]]*$//' | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}'; }
for which in solution starter; do
  ok=1
  if rustc --edition 2024 -A dead_code ${which}_main.rs -o ${which}_bin 2>${which}_err.txt; then
    if [ -s has_out ]; then
      timeout 5 ./${which}_bin > out.txt || ok=0
      [ "$(norm < out.txt)" == "$(norm < expected.txt)" ] || ok=0
    fi
    if [ -s has_tests ]; then
      rustc --edition 2024 -A dead_code --test ${which}_test.rs -o ${which}_tbin 2>>${which}_err.txt && timeout 5 ./${which}_tbin >/dev/null 2>&1 || ok=0
    fi
  else ok=0; fi
  echo "$which passes: $ok"
done
