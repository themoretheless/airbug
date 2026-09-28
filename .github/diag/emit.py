# Temporary (arena branch): surface CI output as annotations, since raw logs are unreachable.
import re, sys
path, title = sys.argv[1], sys.argv[2]
text = open(path, errors="replace").read()
text = re.sub(r"\x1b\[[0-9;]*m", "", text)
lines = [
    l for l in text.splitlines()
    if not re.search(r"\.\.\. ok$|^\s*(Compiling|Checking|Documenting|Downloaded|Fresh) ", l)
]
picked = set()
for i, line in enumerate(lines):
    if re.search(r"^(error|warning)|panicked|FAILED|^failures:|^---- |test result|Running |skipping|^test ", line):
        picked.update(range(max(0, i - 2), min(len(lines), i + 14)))
body = "\n".join(lines[i] for i in sorted(picked)) or text[-6000:]
chunks = [body[i:i + 3800] for i in range(0, len(body), 3800)][:30]
kinds = ["error"] * 10 + ["warning"] * 10 + ["notice"] * 10
for n, (kind, chunk) in enumerate(zip(kinds, chunks)):
    esc = chunk.replace("%", "%25").replace("\r", "").replace("\n", "%0A")
    print(f"::{kind} title={title} {n}::{esc}")
