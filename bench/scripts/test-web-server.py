"""Live HTTP smoke test against saved report fixtures; no browser required."""
import json
import subprocess
import sys
import urllib.request
import urllib.error

proc = subprocess.Popen(['target/release/cargo-rbench', 'serve', sys.argv[1], '--port', '0'], stdout=subprocess.PIPE, text=True)
try:
    url = proc.stdout.readline().strip().split('Report interface: ')[1]
    def get(path):
        return urllib.request.urlopen(url + path, timeout=20).read()
    assert b'<iframe' in get('')
    rows = json.loads(get('api/runs'))
    assert rows
    for row in rows:
        if row.get('memory'):
            assert b'rbench.memory/1' in get('memory?id=' + row['id'])
    first = rows[0]['id']
    assert len(first) == 64
    assert b'<!doctype html>' in get('report?id=' + first).lower()
    doc = json.loads(get('report?format=json'))
    assert len(doc['entries']) == len(rows)
    single = json.loads(get('report?id=' + first + '&format=json'))
    assert len(single['entries']) == 1
    assert b'Benchmark report' in get('report?id=' + first + '&format=md')
    for path in ['report?id=missing', 'report?path=/etc/passwd', '../', 'report?threshold=nan']:
        try:
            get(path)
            raise AssertionError('unexpected success: ' + path)
        except urllib.error.HTTPError as e:
            assert e.code in (400, 404)
    print('HTTP: catalog, HTML, collection JSON, single JSON, Markdown and rejected requests passed')
finally:
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
        raise
