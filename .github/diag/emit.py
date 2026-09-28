# Temporary (arena branch): surface CI output as annotations, since raw logs are unreachable.
import re, sys
path, title = sys.argv[1], sys.argv[2]
text = open(path, errors="replace").read()
text = re.sub(r"\x1b\[[0-9;]*m", "", text)
lines = text.splitlines()
keep = []
for i, line in enumerate(lines):
    if re.search(r"^(error|warning)|panicked|FAILED|failures:|assert|^---- |left:|right:|stderr|Error|thread '", line):
        keep.extend(lines[max(0, i - 2): i + 14])
body = "\n".join(dict.fromkeys(f"{n}\x00{l}" for n, l in enumerate(keep)).keys())
body = "\n".join(l.split("\x00", 1)[1] for l in body.splitlines()) or text[-6000:]
chunks = [body[i:i + 3800] for i in range(0, len(body), 3800)][:30]
kinds = ["error"] * 10 + ["warning"] * 10 + ["notice"] * 10
for n, (kind, chunk) in enumerate(zip(kinds, chunks)):
    esc = chunk.replace("%", "%25").replace("\r", "").replace("\n", "%0A")
    print(f"::{kind} title={title} {n}::{esc}")
