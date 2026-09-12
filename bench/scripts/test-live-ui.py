"""Verify live lifecycle, persistence and cancellation through real HTTP requests."""
import json
import pathlib
import signal
import subprocess
import tempfile
import time
import urllib.request
import urllib.error

with tempfile.TemporaryDirectory(prefix='rbench-live-') as tmp:
    for cancel in (False, True):
        out = pathlib.Path(tmp) / ('cancelled' if cancel else 'complete')
        proc = subprocess.Popen(['target/release/cargo-rbench', 'run', '--ui', '--no-open', '--program', '/bin/sleep', '--repetitions', '2', '-o', str(out), '--', '0.5'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            url = proc.stdout.readline().strip().split('Live benchmark: ')[1]
            def state():
                return json.load(urllib.request.urlopen(url + 'api/live', timeout=5))
            deadline = time.monotonic() + 15
            while state()['state'] != 'running':
                assert time.monotonic() < deadline
                time.sleep(0.02)
            try:
                urllib.request.urlopen(url + 'report?format=json')
                raise AssertionError('report allowed during measurement')
            except urllib.error.HTTPError as e:
                assert e.code == 400
            if cancel:
                proc.send_signal(signal.SIGINT)
                proc.wait(timeout=15)
                assert proc.returncode != 0
            else:
                while state()['state'] == 'running':
                    assert time.monotonic() < deadline
                    time.sleep(0.05)
                assert state()['state'] == 'complete'
                assert json.load(urllib.request.urlopen(url + 'report?format=json'))['entries'][0]['status'] == 'complete'
                while not (out / 'report.md').exists():
                    assert time.monotonic() < deadline
                    time.sleep(0.05)
                proc.send_signal(signal.SIGINT)
                proc.wait(timeout=10)
                assert proc.returncode == 0
            assert json.loads((out / 'run.json').read_text())['status'] == ('cancelled' if cancel else 'complete')
            for name in ('report.html', 'report.json', 'report.md', 'status-final.json'):
                assert (out / name).is_file(), name
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait()
print('Live HTTP: running progress, deferred report, completion, cancellation and persisted exports passed')
