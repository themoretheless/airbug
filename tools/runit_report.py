#!/usr/bin/env python3
"""Run native libtest cases and produce a portable RUnit HTML report (Python 3.9+)."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import shutil
import subprocess
import sys
import tempfile
import time
import uuid

SCHEMA = 1
BAD = {"failed", "broken"}
SUMMARY = re.compile(r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", re.M)


def execute(command, cwd, timeout, limit=131072, env=None):
    """Spool output to disk; retain a bounded tail, including libtest's final summary."""
    started = time.monotonic()
    with tempfile.TemporaryFile() as output:
        kwargs = {"start_new_session": True} if os.name != "nt" else {
            "creationflags": subprocess.CREATE_NEW_PROCESS_GROUP}
        try:
            process = subprocess.Popen(command, cwd=cwd, stdout=output,
                                       stderr=subprocess.STDOUT, env=env, **kwargs)
        except OSError as error:
            ended = time.monotonic()
            return {"code": None, "timeout": False, "duration": ended - started,
                    "_started": started, "_ended": ended,
                    "output": str(error), "truncated": False}
        timed_out = False
        try:
            process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            stop(process)
        except BaseException:
            stop(process)
            raise
        size = output.tell()
        output.seek(max(0, size - limit))
        log = output.read().decode("utf-8", errors="replace")
    ended = time.monotonic()
    return {"code": process.returncode, "timeout": timed_out,
            "duration": ended - started, "_started": started, "_ended": ended,
            "output": log, "truncated": size > limit}


def collect_diagnostics(directory):
    """Decode bounded, per-test sidecar events; never infer data from panic logs."""
    import base64
    import difflib

    result = {"steps": [], "attachments": [], "comparisons": [], "diagnosticErrors": []}
    steps = {}
    errors = result["diagnosticErrors"]

    def read_local(name, maximum):
        path = directory / name
        if path.is_symlink() or path.resolve().parent != directory.resolve():
            raise ValueError("Diagnostic file must be inside its test directory")
        with path.open("rb") as stream:
            data = stream.read(maximum + 1)
        if len(data) > maximum:
            raise ValueError("Diagnostic file exceeds its size limit")
        return data

    def string(event, key, maximum=65536):
        value = event.get(key)
        if not isinstance(value, str) or len(value.encode("utf-8")) > maximum:
            raise ValueError("Invalid diagnostic string: " + key)
        return value

    def identifier(value):
        if type(value) is not int or value < 1 or value > 9007199254740991:
            raise ValueError("Invalid step ID")
        return value

    marker = directory / "report-error.txt"
    if marker.exists() or marker.is_symlink():
        try:
            errors.append(read_local("report-error.txt", 8192).decode("utf-8", errors="replace"))
        except (OSError, ValueError) as error:
            errors.append(str(error))
    source = directory / "events.jsonl"
    if not source.exists() and not source.is_symlink():
        return result
    try:
        data = read_local("events.jsonl", 8 * 1024 * 1024).decode("utf-8")
        lines = data.splitlines()
        if len(lines) > 4096:
            raise ValueError("Too many diagnostic events (limit 4096)")
        attached = 0
        for line in lines:
            event = json.loads(line)
            if not isinstance(event, dict):
                raise ValueError("Invalid diagnostic event")
            kind = event.get("type")
            parent = event.get("parent")
            if parent is not None:
                identifier(parent)
                if parent not in steps or steps[parent]["status"] != "running":
                    raise ValueError("Unknown or completed diagnostic parent step")
            if kind == "step_start":
                key = identifier(event.get("id"))
                if key in steps:
                    raise ValueError("Duplicate step ID")
                depth = 0 if parent is None else steps[parent]["depth"] + 1
                if depth >= 64:
                    raise ValueError("Step nesting limit exceeded (64)")
                step = {"id": key, "parent": parent, "name": string(event, "name"),
                        "status": "running", "duration": None, "depth": depth}
                steps[key] = step
                result["steps"].append(step)
            elif kind == "step_end":
                key = identifier(event.get("id"))
                duration = event.get("duration")
                if (key not in steps or steps[key]["status"] != "running" or
                        event.get("status") not in ("passed", "failed") or
                        type(duration) not in (int, float) or not math.isfinite(duration) or duration < 0):
                    raise ValueError("Invalid step completion")
                if any(step["parent"] == key and step["status"] == "running" for step in steps.values()):
                    raise ValueError("A parent step completed before its child")
                steps[key].update(status=event["status"], duration=duration)
            elif kind == "attachment":
                name, media = string(event, "name"), string(event, "mediaType")
                filename = string(event, "file", 128)
                if not re.fullmatch(r"attachment-[0-9]+-[0-9]+\.bin", filename):
                    raise ValueError("Invalid attachment filename")
                if type(event.get("size")) is not int or event["size"] < 0:
                    raise ValueError("Invalid attachment size")
                content = read_local(filename, 1024 * 1024)
                attached += len(content)
                if len(content) != event["size"] or attached > 4 * 1024 * 1024:
                    raise ValueError("Attachment size or total limit mismatch")
                item = {"parent": parent, "name": name, "mediaType": media,
                        "size": len(content), "base64": base64.b64encode(content).decode("ascii")}
                if media.startswith("text/") or media.split(";", 1)[0] in ("application/json", "application/xml"):
                    item["text"] = content[:65536].decode("utf-8", errors="replace")
                    item["previewTruncated"] = len(content) > 65536
                result["attachments"].append(item)
            elif kind == "comparison":
                expected, actual = string(event, "expected"), string(event, "actual")
                if type(event.get("passed")) is not bool or type(event.get("truncated")) is not bool:
                    raise ValueError("Invalid comparison status")
                before, after = expected.splitlines(keepends=True), actual.splitlines(keepends=True)
                # Bound diff generation independently from stored expected/actual text.
                diff = "".join(difflib.unified_diff([line.removesuffix("\n").replace("\r", "\\r") + "\n" for line in before[:2048]], [line.removesuffix("\n").replace("\r", "\\r") + "\n" for line in after[:2048]], fromfile="expected", tofile="actual"))
                if expected.endswith("\n") != actual.endswith("\n"):
                    diff += "\n\\ No final newline in " + ("actual" if expected.endswith("\n") else "expected") + "\n"
                result["comparisons"].append({"parent": parent, "name": string(event, "name"),
                    "expected": expected, "actual": actual, "passed": event["passed"],
                    "truncated": event["truncated"], "diff": diff[:262144],
                    "diffTruncated": len(before) > 2048 or len(after) > 2048 or len(diff) > 262144})
            else:
                raise ValueError("Unknown diagnostic event type")
    except (OSError, ValueError, UnicodeError) as error:
        errors.append("Incomplete structured diagnostics: " + str(error))
    for step in steps.values():
        if step["status"] == "running":
            step["status"] = "broken"
            errors.append("Step did not finish: " + step["name"])
    return result


def execute_case(command, cwd, timeout, environment):
    with tempfile.TemporaryDirectory(prefix="runit-diagnostics-") as directory:
        result = execute(command, cwd, timeout, env=dict(environment, RUNIT_REPORT_DIR=directory))
        result.update(collect_diagnostics(Path(directory)))
        return result


def stop(process):
    if os.name == "nt":
        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
        if process.poll() is None:
            process.kill()
    else:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    process.wait()


def classify(result, single=True):
    if result["timeout"] or result["code"] is None:
        return "broken"
    summaries = SUMMARY.findall(result["output"])
    if not summaries:
        return "broken"
    passed, failed, ignored = map(int, summaries[-1])
    if single and passed + failed + ignored != 1:
        return "broken"
    if result["code"] != 0:
        return "failed" if failed else "broken"
    if failed:
        return "broken"
    if ignored and not passed:
        return "skipped"
    return "passed"


def case(suite, name, result, status=None, kind="test"):
    identity = hashlib.sha256((suite + "\0" + name).encode()).hexdigest()[:24]
    return {"id": identity, "suite": suite, "name": name, "kind": kind,
            "status": status or ("broken" if result.get("diagnosticErrors") and classify(result) == "passed" else classify(result)), **result}


def timeline_event(name, result, origin, suite="Infrastructure"):
    return {"name": name, "suite": suite, "kind": "phase",
            "status": "passed" if result["code"] == 0 and not result["timeout"] else "broken",
            "startOffset": result["_started"] - origin,
            "endOffset": result["_ended"] - origin, "duration": result["duration"]}


def discover(output):
    tests = []
    for line in output.splitlines():
        if line.endswith(": test"):
            tests.append(line[:-6])
        elif line.endswith(": benchmark") or not line.strip():
            continue
        else:
            raise ValueError("Unsupported libtest discovery output: " + line[:200])
    if len(tests) != len(set(tests)):
        raise ValueError("Duplicate native test names")
    return tests


def runtime_environment(package, artifact, messages, target_dir, rust_lib):
    """Restore Cargo's package variables and generated dynamic-library search paths."""
    env = dict(os.environ)
    env.update(CARGO=shutil.which("cargo") or "cargo",
               CARGO_MANIFEST_DIR=str(Path(package["manifest_path"]).parent),
               CARGO_MANIFEST_PATH=package["manifest_path"],
               CARGO_CRATE_NAME=artifact["target"]["name"].replace("-", "_"))
    for field in ("name", "version", "description", "homepage", "repository", "license",
                  "license_file", "rust_version", "readme"):
        env["CARGO_PKG_" + field.upper()] = package.get(field) or ""
    env["CARGO_PKG_AUTHORS"] = ":".join(package.get("authors", []))
    version, _, prerelease = package["version"].partition("-")
    for field, value in zip(("MAJOR", "MINOR", "PATCH"), version.split("+", 1)[0].split(".")):
        env["CARGO_PKG_VERSION_" + field] = value
    env["CARGO_PKG_VERSION_PRE"] = prerelease.split("+", 1)[0]
    executable = Path(artifact["executable"])
    paths = [str(executable.parent), str(executable.parent.parent), rust_lib]
    for message in messages:
        if message.get("reason") == "build-script-executed":
            for raw in message.get("linked_paths", []):
                path = Path(raw.split("=", 1)[-1]).resolve()
                if path == target_dir or target_dir in path.parents:
                    paths.append(str(path))
            if message.get("package_id") == artifact["package_id"]:
                env.update(message.get("env", []))
                env["OUT_DIR"] = message.get("out_dir", "")
        if (message.get("reason") == "compiler-artifact" and message.get("executable")
                and message.get("package_id") == artifact["package_id"]
                and "bin" in message.get("target", {}).get("kind", [])
                and not message.get("profile", {}).get("test")):
            env["CARGO_BIN_EXE_" + message["target"]["name"]] = message["executable"]
    variable = "PATH" if os.name == "nt" else (
        "DYLD_FALLBACK_LIBRARY_PATH" if sys.platform == "darwin" else "LD_LIBRARY_PATH")
    existing = env.get(variable, "")
    if sys.platform == "darwin" and not existing:
        existing = os.pathsep.join((str(Path.home() / "lib"), "/usr/local/lib", "/usr/lib"))
    env[variable] = os.pathsep.join(dict.fromkeys(paths + ([existing] if existing else [])))
    return env


def atomic_write(path, content):
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(dir=path.parent, prefix=".runit-")
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            stream.write(content)
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def render(report, template):
    # HTML script raw-text parsing must never see an input-provided closing tag.
    data = json.dumps(report, ensure_ascii=True, allow_nan=False).replace("<", "\\u003c")
    return template.replace("__RUNIT_DATA__", data)


def valid_history(runs):
    if not isinstance(runs, list):
        return False
    for run in runs:
        if not isinstance(run, dict):
            return False
        if any(not isinstance(run.get(key), str) for key in ("id", "started", "commit")):
            return False
        duration = run.get("duration")
        if not isinstance(duration, (int, float)) or not math.isfinite(duration) or duration < 0:
            return False
        counts, states = run.get("counts"), run.get("statuses")
        if not isinstance(counts, dict) or not isinstance(states, dict):
            return False
        if any(key not in {"passed", "failed", "broken", "skipped"} or
               not isinstance(value, int) or value < 0 for key, value in counts.items()):
            return False
        if any(value not in {"passed", "failed", "broken", "skipped"} for value in states.values()
               if isinstance(value, str)) or any(not isinstance(value, str) for value in states.values()):
            return False
    return True


def write_report(report, output, history_limit=20):
    output.mkdir(parents=True, exist_ok=True)
    lock = output / ".report.lock"
    try:
        descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError:
        raise RuntimeError("Report directory is in use: " + str(lock)) from None
    try:
        os.close(descriptor)
        history_path = output / "history.json"
        history = []
        if history_path.exists():
            try:
                prior = json.loads(history_path.read_text(encoding="utf-8"))
                if prior.get("schema") == SCHEMA and prior.get("project") == report["project"]:
                    if not valid_history(prior["runs"]):
                        raise ValueError("Invalid history entries")
                    history = prior["runs"][-(history_limit - 1):]
            except (ValueError, KeyError, TypeError, AttributeError):
                print("Warning: invalid history ignored", file=sys.stderr)
        summary = {key: report[key] for key in ("id", "started", "duration", "counts", "commit")}
        summary["statuses"] = {test["id"]: test["status"] for test in report["tests"]}
        report["history"] = history + [summary]
        template = (Path(__file__).parent / "report" / "index.html").read_text(encoding="utf-8")
        atomic_write(output / "report.json", json.dumps(report, indent=2, allow_nan=False))
        atomic_write(output / "index.html", render(report, template))
        atomic_write(history_path, json.dumps({"schema": SCHEMA, "project": report["project"],
                                               "runs": report["history"]}, allow_nan=False))
    finally:
        lock.unlink()


def positive(value):
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise argparse.ArgumentTypeError("must be a finite positive number")
    return number


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", type=Path, default=Path("Cargo.toml"))
    parser.add_argument("--output", type=Path, default=Path("target/runit-report"))
    parser.add_argument("--title", default="RUnit test report")
    parser.add_argument("--workspace", action="store_true")
    parser.add_argument("--package", action="append", default=[])
    parser.add_argument("--features")
    parser.add_argument("--all-features", action="store_true")
    parser.add_argument("--no-default-features", action="store_true")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--locked", action="store_true")
    parser.add_argument("--include-ignored", action="store_true")
    parser.add_argument("--filter", default="", help="substring of the native test name")
    parser.add_argument("--doc-tests", action="store_true", help="also run Cargo doctests as one aggregate entry")
    parser.add_argument("--timeout", type=positive, default=120, help="seconds per test process")
    parser.add_argument("--build-timeout", type=positive, default=600)
    args = parser.parse_args(argv)
    manifest = args.manifest_path.resolve()
    root = manifest.parent
    output = args.output.resolve()
    start = time.monotonic()
    report = {"schema": SCHEMA, "id": str(uuid.uuid4()), "title": args.title,
              "project": str(manifest), "started": datetime.now(timezone.utc).isoformat(),
              "tests": [], "commit": "", "filter": args.filter,
              "mode": "Native libtest · one process per case · serial"}
    git = execute(["git", "rev-parse", "--short", "HEAD"], root, 10)
    if git["code"] == 0:
        report["commit"] = git["output"].strip()
        dirty = execute(["git", "status", "--porcelain"], root, 10)
        if dirty["code"] == 0 and dirty["output"].strip():
            report["commit"] += " + changes"
    flags = ["--manifest-path", str(manifest)]
    for option in ("workspace", "all_features", "no_default_features", "release", "offline", "locked"):
        if getattr(args, option):
            flags.append("--" + option.replace("_", "-"))
    for package in args.package:
        flags.extend(["--package", package])
    if args.features:
        flags.extend(["--features", args.features])
    command = ["cargo", "test", *flags, "--no-run", "--message-format=json"]
    report["command"] = command
    print("Building native test binaries…", flush=True)
    build = execute(command, root, args.build_timeout, 16 * 1024 * 1024)
    report["buildDuration"] = build["duration"]
    report["phases"] = [timeline_event("Cargo build", build, start)]
    binaries = {}
    if build["code"] != 0 or build["timeout"] or build["truncated"]:
        report["tests"].append(case("Infrastructure", "Cargo build", build, "broken", "build"))
    else:
        try:
            messages = []
            for line in build["output"].splitlines():
                try:
                    artifact = json.loads(line)
                except ValueError:
                    continue  # Cargo progress and compiler output on stderr.
                if not isinstance(artifact, dict):
                    continue
                messages.append(artifact)
                if (artifact.get("reason") == "compiler-artifact" and artifact.get("executable")
                        and artifact.get("profile", {}).get("test")):
                    binaries[artifact["executable"]] = artifact
            if not binaries:
                raise ValueError("Cargo returned no native test executables")
            metadata = execute(["cargo", "metadata", *["--manifest-path", str(manifest)],
                                "--no-deps", "--format-version=1", "--offline"], root,
                               args.build_timeout, 16 * 1024 * 1024)
            report["phases"].append(timeline_event("Cargo metadata", metadata, start))
            if metadata["code"] != 0 or metadata["truncated"]:
                raise ValueError("Cannot resolve package working directories: " + metadata["output"])
            metadata_lines = []
            for line in metadata["output"].splitlines():
                try:
                    entry = json.loads(line)
                except ValueError:
                    continue
                if isinstance(entry, dict) and "packages" in entry:
                    metadata_lines.append(entry)
            if len(metadata_lines) != 1:
                raise ValueError("Cargo metadata did not return exactly one package inventory")
            packages = {p["id"]: p for p in metadata_lines[0]["packages"]}
            rust_lib = execute([os.environ.get("RUSTC", "rustc"), "--print", "target-libdir"], root, 30)
            if rust_lib["code"] != 0:
                raise ValueError("Cannot resolve Rust runtime libraries: " + rust_lib["output"])
            for executable, artifact in sorted(binaries.items()):
                package = packages[artifact["package_id"]]
                cwd = Path(package["manifest_path"]).parent
                target = artifact["target"]
                environment = runtime_environment(package, artifact, messages,
                                                  Path(metadata_lines[0]["target_directory"]).resolve(),
                                                  rust_lib["output"].strip())
                suite = package["name"] + " / " + target["name"] + " (" + ",".join(target["kind"]) + ")"
                listing = execute([executable, "--list", "--format", "terse"], cwd, args.timeout,
                                  16 * 1024 * 1024, env=environment)
                report["phases"].append(timeline_event("Test discovery", listing, start, suite))
                if listing["code"] != 0 or listing["timeout"] or listing["truncated"]:
                    report["tests"].append(case(suite, "Test discovery", listing, "broken", "discovery"))
                    continue
                try:
                    names = discover(listing["output"])
                except ValueError as error:
                    listing["output"] += "\n" + str(error)
                    report["tests"].append(case(suite, "Test discovery", listing, "broken", "discovery"))
                    continue
                for name in names:
                    if args.filter not in name:
                        continue
                    invocation = [executable, "--exact", name, "--nocapture", "--color", "never"]
                    if args.include_ignored:
                        invocation.append("--include-ignored")
                    result = case(suite, name, execute_case(invocation, cwd, args.timeout, environment))
                    report["tests"].append(result)
                    print(result["status"].upper() + " " + suite + " :: " + name, flush=True)
            if args.doc_tests:
                docs = execute(["cargo", "test", *flags, "--doc", "--no-fail-fast", "--", args.filter], root, args.build_timeout)
                status = "broken" if docs["timeout"] or docs["code"] is None else (
                    "passed" if docs["code"] == 0 else "failed")
                if status == "passed" and not any(int(passed) for passed, _, _ in SUMMARY.findall(docs["output"])):
                    status = "skipped"
                report["tests"].append(case("Documentation", "Cargo doctests (aggregate)", docs, status, "doctests"))
        except (ValueError, KeyError, TypeError) as error:
            report["tests"].append(case("Infrastructure", "Discovery error", {
                "code": None, "timeout": False, "duration": 0, "output": str(error), "truncated": False}, "broken", "discovery"))
    report["duration"] = time.monotonic() - start
    for test in report["tests"]:
        if "_started" in test:
            test["startOffset"] = test.pop("_started") - start
            test["endOffset"] = test.pop("_ended") - start
    report["counts"] = dict(Counter(test["status"] for test in report["tests"]))
    write_report(report, output)
    print("Report: " + str(output / "index.html"))
    if not report["tests"]:
        print("No matching tests; returning exit code 2", file=sys.stderr)
        return 2
    return int(any(test["status"] in BAD for test in report["tests"]))


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError) as error:
        print("runit-report: " + str(error), file=sys.stderr)
        sys.exit(2)
