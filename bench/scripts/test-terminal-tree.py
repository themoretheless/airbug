#!/usr/bin/env python3
"""Exercise terminal-width detection through a real PTY and an external Cargo harness."""
import errno
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-terminal-tree-") as directory:
    root = Path(directory)
    (root / "src").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="terminal-tree"
version="0.1.0"
edition="2021"
[workspace]
[dependencies]
airbug-bench={{path={json.dumps(str(ROOT / 'bench'))}}}
''')
    (root / "src/main.rs").write_text('''fn main() -> airbug_bench::Result<()> {
    let mut suite = airbug_bench::Suite::new("terminal");
    let mut name = (0..64).map(|i| format!("level-{i:02}")).collect::<Vec<_>>().join("/");
    name.push_str("/длинное имя 🦀");
    suite.bench_custom(&name, |n| std::time::Duration::from_nanos(n * 3));
    suite.main()
}
''')
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target/parity-external"),
               AIRBUG_DASHBOARD="0", NO_COLOR="1")
    built = subprocess.run(["cargo", "build", "--release", "--offline", "--message-format=json"],
                           cwd=root, env=env, capture_output=True, text=True, check=True)
    exe = next(x["executable"] for line in built.stdout.splitlines()
               if (x := json.loads(line)).get("executable"))
    args = [exe, "--output-format", "tree", "--no-history", "--color", "never"]
    for mode in [["--list"], ["--quiet", "--samples", "2", "--iterations", "3", "--warmup-ms", "0", "--resamples", "16"]]:
        pipe = subprocess.run(args + mode, cwd=root, env=env, capture_output=True,
                              text=True, timeout=120)
        assert pipe.returncode == 0, pipe.stdout + pipe.stderr
        pipe = pipe.stdout
        assert "Deep tree:" not in pipe and "    " * 65 + "└── длинное имя 🦀" in pipe, pipe
        for width in (40, 80, 120):
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, width, 0, 0))
            process = subprocess.Popen(args + mode, cwd=root, env=env, stdin=slave, stdout=slave, stderr=slave)
            os.close(slave)
            chunks = []
            deadline = time.monotonic() + 120
            try:
                while True:
                    if time.monotonic() >= deadline:
                        process.kill()
                        raise TimeoutError("terminal harness did not exit")
                    if not select.select([master], [], [], 1)[0]:
                        continue
                    try:
                        chunk = os.read(master, 65536)
                    except OSError as error:
                        if error.errno == errno.EIO:
                            break
                        raise
                    if not chunk:
                        break
                    chunks.append(chunk)
                assert process.wait(timeout=5) == 0
            finally:
                os.close(master)
                if process.poll() is None:
                    process.kill()
                    process.wait()
            output = b"".join(chunks).decode().replace("\r\n", "\n")
            assert "Deep tree: [N] denotes nesting depth." in output, output
            leaf = next(line for line in output.splitlines() if "длинное имя 🦀" in line)
            assert "[65] └── длинное имя 🦀" in leaf, leaf
            assert len(leaf) - len(leaf.lstrip()) == (width // 3 // 4) * 4, leaf
            if mode != ["--list"]:
                assert "3 ns/op" in output, output
    print("PTY tree: listing/results at 40, 80, 120 columns; full pipe identity retained")
