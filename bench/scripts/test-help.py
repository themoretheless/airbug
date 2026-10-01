#!/usr/bin/env python3
"""Cargo help stays lazy, readable and responsive to terminal width."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-help-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="help-fixture"
version="0.1.0"
edition="2021"
[workspace]
[dependencies]
airbug-bench={{path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="help"
harness=false
''')
    (root / "benches/help.rs").write_text('''fn main() -> airbug_bench::Result<()> {
    let suite = airbug_bench::Suite::new("help");
    if std::env::var_os("MANUAL").is_some() { suite.main() }
    else { suite.main_registered(|_| panic!("help registered benchmarks")) }
}
''')
    env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_") and k != "COLUMNS"}
    env.update(CARGO_TARGET_DIR=str(ROOT / "target/parity-external"), AIRBUG_DASHBOARD="0")
    build = subprocess.run(["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--bench", "help", "--no-run", "--message-format=json"], env=env, capture_output=True, text=True, check=True)
    executable = next(item["executable"] for line in build.stdout.splitlines() if (item := json.loads(line)).get("executable"))
    env.update(AIRBUG_BENCH_SAMPLES="invalid", AIRBUG_BENCH_THREADS="invalid")
    for manual in (False, True):
        active = dict(env, **({"MANUAL": "1"} if manual else {}))
        for flag in ("-h", "--help", "--help-all"):
            for columns in ("40", "80", "invalid", "0"):
                result = subprocess.run([executable, "--bench", flag], cwd=root, env=dict(active, COLUMNS=columns), capture_output=True, text=True, check=True)
                assert "Airbug benchmarks" in result.stdout
                assert "cargo bench -- --list" in result.stdout
                assert all((len(line) <= (40 if columns == "40" else 80) or len(line.split()) == 1) for line in result.stdout.splitlines()), result.stdout
                assert not (root / "target").exists()
    # Exercise OS terminal-size discovery, independent of COLUMNS.
    if os.name == "posix":
        import errno
        import fcntl
        import pty
        import struct
        import termios
        master, slave = pty.openpty()
        try:
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 40, 0, 0))
            process = subprocess.Popen([executable, "--help-all"], cwd=root, env=env, stdout=slave, stderr=slave)
            os.close(slave)
            slave = None
            output = bytearray()
            while True:
                try:
                    block = os.read(master, 4096)
                except OSError as error:
                    if error.errno == errno.EIO:
                        break
                    raise
                if not block:
                    break
                output.extend(block)
            assert process.wait() == 0, output.decode()
            assert all((len(line) <= 40 or len(line.split()) == 1) for line in output.decode().splitlines()), output.decode()
        finally:
            os.close(master)
            if slave is not None:
                os.close(slave)
print("Help: 24 width/entrypoint cases passed; POSIX PTY width checked when available; no registration or artifacts")
