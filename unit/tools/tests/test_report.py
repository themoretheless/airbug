"""Reporter regression tests; uses a temporary Rust crate for end-to-end checks."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

TOOL = Path(__file__).resolve().parents[1] / "airbug_report.py"
SPEC = importlib.util.spec_from_file_location("airbug_report", TOOL)
reporter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(reporter)


class ReporterTests(unittest.TestCase):
    def result(self, output, code=0, timeout=False):
        return {"output": output, "code": code, "timeout": timeout}

    def test_diagnostic_collector_rejects_paths_and_incomplete_steps(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            events = root / 'events.jsonl'
            events.write_text(json.dumps({'type': 'attachment', 'parent': None, 'name': 'x',
                'mediaType': 'text/plain', 'size': 1, 'file': '../secret'}) + '\n')
            data = reporter.collect_diagnostics(root)
            self.assertTrue(data['diagnosticErrors'])
            self.assertEqual(data['attachments'], [])
            events.write_text(json.dumps({'type': 'step_start', 'id': 1, 'parent': None, 'name': 'unfinished'}) + '\n')
            data = reporter.collect_diagnostics(root)
            self.assertEqual(data['steps'][0]['status'], 'broken')
            self.assertIsNone(data['steps'][0]['duration'])
            self.assertTrue(data['diagnosticErrors'])
            events.write_text(json.dumps({'type': 'step_start', 'id': 1, 'parent': 1, 'name': 'cycle'}) + '\n')
            self.assertTrue(reporter.collect_diagnostics(root)['diagnosticErrors'])

    def test_diagnostic_collector_bounds_sizes_and_escapes_attachment_content(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            content = b'</script><script>alert(1)</script>'
            (root / 'attachment-1-1.bin').write_bytes(content)
            event = {'type': 'attachment', 'parent': None, 'name': 'hostile.html',
                     'mediaType': 'text/html', 'size': len(content), 'file': 'attachment-1-1.bin'}
            (root / 'events.jsonl').write_text(json.dumps(event) + '\n')
            data = reporter.collect_diagnostics(root)
            self.assertFalse(data['diagnosticErrors'])
            self.assertEqual(data['attachments'][0]['text'], content.decode())
            self.assertNotIn('</script>', reporter.render(data, '__AIRBUG_DATA__'))
            (root / 'attachment-1-1.bin').write_bytes(b'x' * (1024 * 1024 + 1))
            self.assertTrue(reporter.collect_diagnostics(root)['diagnosticErrors'])

    def test_diff_preserves_line_endings_and_missing_final_newline(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            event = {'type': 'comparison', 'parent': None, 'name': 'endings',
                     'expected': 'paid\r\n', 'actual': 'paid', 'passed': False, 'truncated': False}
            (root / 'events.jsonl').write_text(json.dumps(event) + '\n')
            data = reporter.collect_diagnostics(root)
            self.assertFalse(data['diagnosticErrors'])
            diff = data['comparisons'][0]['diff']
            self.assertIn('-paid\\r', diff)
            self.assertIn('No final newline in actual', diff)

    def test_report_diagnostics_end_to_end(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'src').mkdir()
            dependency = json.dumps(str(TOOL.parents[1]))
            (root / 'Cargo.toml').write_text('[package]\nname="rich-report-fixture"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\nairbug={path=' + dependency + '}\n')
            (root / 'src/lib.rs').write_text(r'''
#[test] fn successful() {
    airbug::report::step("parent", || {
        airbug::report::step("child", || {
            airbug::report::attach_bytes("payload.json", "application/json", br#"{"ok":true}"#).unwrap();
            airbug::report::assert_text_equal("body", "ok", "ok");
        });
    });
}
#[test] fn failed() {
    airbug::report::step("parent", || airbug::report::step("child", || {
        airbug::report::assert_text_equal("body", "paid\n2500\n", "pending\n2400\n");
    }));
}
#[test] fn recovery() {
    airbug::report::step("parent", || {
        let _ = std::panic::catch_unwind(|| airbug::report::step("failed child", || panic!("expected")));
        airbug::report::step("next child", || {});
    });
}
#[test] fn timeout() { airbug::report::step("unfinished", || std::thread::sleep(std::time::Duration::from_secs(30))); }
#[test] fn attachment_limit() { assert!(airbug::report::attach_bytes("large", "application/octet-stream", &vec![0; 1024*1024+1]).is_err()); }
''')
            result = subprocess.run([sys.executable, str(TOOL), '--manifest-path', str(root / 'Cargo.toml'),
                                     '--output', str(root / 'report'), '--offline', '--timeout', '1'],
                                    capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            data = json.loads((root / 'report/report.json').read_text())
            tests = {t['name']: t for t in data['tests']}
            good, bad = tests['successful'], tests['failed']
            self.assertEqual(good['status'], 'passed')
            self.assertEqual([s['status'] for s in good['steps']], ['passed', 'passed'])
            self.assertEqual(good['steps'][1]['parent'], good['steps'][0]['id'])
            self.assertEqual(good['attachments'][0]['parent'], good['steps'][1]['id'])
            self.assertEqual(good['attachments'][0]['text'], '{"ok":true}')
            self.assertEqual(bad['status'], 'failed')
            self.assertEqual([s['status'] for s in bad['steps']], ['failed', 'failed'])
            self.assertIn('-paid', bad['comparisons'][0]['diff'])
            self.assertIn('+pending', bad['comparisons'][0]['diff'])
            self.assertEqual(tests['timeout']['steps'][0]['status'], 'broken')
            self.assertEqual(tests['recovery']['status'], 'passed')
            self.assertEqual(tests['recovery']['steps'][2]['parent'], tests['recovery']['steps'][0]['id'])
            self.assertEqual(tests['attachment_limit']['status'], 'passed')

    def test_status_requires_one_completed_native_case(self):
        self.assertEqual(reporter.classify(self.result("test result: ok. 1 passed; 0 failed; 0 ignored;")), "passed")
        self.assertEqual(reporter.classify(self.result("test result: ok. 0 passed; 0 failed; 1 ignored;")), "skipped")
        self.assertEqual(reporter.classify(self.result("test result: FAILED. 0 passed; 1 failed; 0 ignored;", 101)), "failed")
        for result in [self.result(""), self.result("", -9), self.result("", None),
                       self.result("test result: ok. 0 passed; 0 failed; 0 ignored;"),
                       self.result("test result: ok. 1 passed; 0 failed; 0 ignored;", timeout=True)]:
            self.assertEqual(reporter.classify(result), "broken")

    def test_last_summary_takes_precedence_over_test_output(self):
        text = "test result: FAILED. 0 passed; 1 failed; 0 ignored;\n\ntest result: ok. 1 passed; 0 failed; 0 ignored;"
        self.assertEqual(reporter.classify(self.result(text)), "passed")

    def test_timeline_preserves_real_offsets_and_gaps(self):
        event = reporter.timeline_event("build", {"_started": 12.5, "_ended": 15.0,
                                                   "duration": 2.5, "code": 0, "timeout": False}, 10)
        self.assertEqual(event["startOffset"], 2.5)
        self.assertEqual(event["endOffset"], 5.0)
        self.assertEqual(event["duration"], 2.5)

    def test_discovery_rejects_custom_harness_noise(self):
        self.assertEqual(reporter.discover("module::case: test\nbenchmark: benchmark\n"), ["module::case"])
        with self.assertRaises(ValueError):
            reporter.discover("arbitrary custom harness output")
        with self.assertRaises(ValueError):
            reporter.discover("a: test\na: test")

    def test_report_data_cannot_close_script_element(self):
        payload = {"title": "</script><script>alert(1)</script> & 中文"}
        rendered = reporter.render(payload, "<script>__AIRBUG_DATA__</script>")
        self.assertEqual(rendered.count("</script>"), 1)
        self.assertEqual(json.loads(rendered[8:-9]), payload)

    def test_process_output_is_bounded_and_exit_code_preserved(self):
        result = reporter.execute([sys.executable, "-c", "print('x'*2000); raise SystemExit(7)"], Path.cwd(), 5, 100)
        self.assertEqual(result["code"], 7)
        self.assertTrue(result["truncated"])
        self.assertLessEqual(len(result["output"]), 100)

    def test_process_timeout(self):
        result = reporter.execute([sys.executable, "-c", "import time; time.sleep(30)"], Path.cwd(), .1)
        self.assertTrue(result["timeout"])
        self.assertLess(result["duration"], 5)

    def test_missing_executable(self):
        result = reporter.execute(["airbug-executable-that-does-not-exist"], Path.cwd(), 1)
        self.assertEqual(reporter.classify(result), "broken")

    def test_output_lock_preserves_existing_report(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            (folder / ".report.lock").touch()
            (folder / "index.html").write_text("existing")
            with self.assertRaises(RuntimeError):
                reporter.write_report({}, folder)
            self.assertEqual((folder / "index.html").read_text(), "existing")

    def test_invalid_history_is_rejected(self):
        self.assertFalse(reporter.valid_history({}))
        self.assertFalse(reporter.valid_history([{"duration": float("nan")}]))
        self.assertFalse(reporter.valid_history([{"id": "1", "started": "now", "commit": "",
                                                 "duration": 1, "counts": {}, "statuses": {"x": []}}]))

    def test_proc_macro_runtime_is_discoverable(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "Cargo.toml").write_text('[package]\nname="report-macros"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[lib]\nproc-macro=true\n')
            (root / "src/lib.rs").write_text('#[test] fn smoke() { assert_eq!(std::env::var("CARGO_PKG_NAME").unwrap(), "report-macros"); }')
            result = subprocess.run([sys.executable, str(TOOL), "--manifest-path", str(root / "Cargo.toml"),
                                     "--output", str(root / "report"), "--offline"],
                                    capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            report = json.loads((root / "report/report.json").read_text())
            self.assertEqual(report["counts"], {"passed": 1})

    def test_real_cargo_cases_failures_timeouts_history_and_empty_selection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "marker").write_text("package cwd")
            (root / "Cargo.toml").write_text('[package]\nname="reporter-fixture"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
            (root / "src/lib.rs").write_text('''
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
pub fn documented() {}
#[test] fn passing() { assert!(std::path::Path::new("marker").exists()); println!("</script><script>alert(1)</script>"); }
#[test] fn failing() { panic!("expected regression failure"); }
#[test] #[ignore] fn ignored() {}
#[test] #[should_panic(expected="intentional")] fn expected_panic() { panic!("intentional"); }
#[test] fn timeout() { std::thread::sleep(std::time::Duration::from_secs(30)); }
#[test] fn abnormal_exit() { std::process::exit(0); }
''')
            output = root / "report"
            command = [sys.executable, str(TOOL), "--manifest-path", str(root / "Cargo.toml"),
                       "--output", str(output), "--offline", "--timeout", "1", "--doc-tests"]
            result = subprocess.run(command, capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            report = json.loads((output / "report.json").read_text())
            previous_end = 0
            for test in report["tests"]:
                self.assertGreaterEqual(test["startOffset"], previous_end)
                self.assertLessEqual(test["endOffset"], report["duration"])
                self.assertAlmostEqual(test["endOffset"] - test["startOffset"], test["duration"])
                self.assertNotIn("_started", test)
                self.assertNotIn("_ended", test)
                previous_end = test["endOffset"]
            self.assertEqual(report["phases"][0]["name"], "Cargo build")
            self.assertLessEqual(report["phases"][0]["endOffset"], report["tests"][0]["startOffset"])
            states = {t["name"]: t["status"] for t in report["tests"]}
            self.assertEqual(states, {"passing": "passed", "failing": "failed", "ignored": "skipped",
                                     "expected_panic": "passed", "timeout": "broken", "abnormal_exit": "broken",
                                     "Cargo doctests (aggregate)": "passed"})
            html = (output / "index.html").read_text()
            self.assertNotIn("</script><script>alert(1)</script>", html)
            first_id = next(t["id"] for t in report["tests"] if t["name"] == "passing")
            # A filtered second run preserves prior history and stable case identity.
            result = subprocess.run(command[:-1] + ["--filter", "passing"], capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            report = json.loads((output / "report.json").read_text())
            self.assertEqual(len(report["history"]), 2)
            self.assertEqual(report["tests"][0]["id"], first_id)
            result = subprocess.run(command[:-1] + ["--filter", "no_such_test"], capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 2)
            report = json.loads((output / "report.json").read_text())
            self.assertEqual(report["tests"], [])
            result = subprocess.run(command[:-1] + ["--filter", "ignored", "--include-ignored"], capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            report = json.loads((output / "report.json").read_text())
            self.assertEqual(report["tests"][0]["status"], "passed")
            # Build failures still produce an inspectable report and a non-zero exit.
            (root / "src/lib.rs").write_text("this is invalid rust")
            result = subprocess.run(command, capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 1)
            report = json.loads((output / "report.json").read_text())
            self.assertEqual(report["tests"][0]["kind"], "build")
            self.assertEqual(report["tests"][0]["status"], "broken")


if __name__ == "__main__":
    unittest.main()
