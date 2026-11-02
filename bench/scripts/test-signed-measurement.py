#!/usr/bin/env python3
"""Signed custom measurements through Cargo, saved baselines and offline reports."""
import csv
import math
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
env = {k: v for k, v in os.environ.items() if not k.startswith('AIRBUG_BENCH_')}
env.update(AIRBUG_DASHBOARD='0', AIRBUG_BENCH_HISTORY='0',
           CARGO_TARGET_DIR=str(ROOT / 'target/parity-external'))
with tempfile.TemporaryDirectory(prefix='airbug-signed-') as directory:
    root = Path(directory)
    (root / 'benches').mkdir()
    (root / 'Cargo.toml').write_text(f'''[package]
name="signed-measurement-fixture"
version="0.1.0"
edition="2021"
[workspace]
[dev-dependencies]
airbug-bench={{path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="signed"
harness=false
''', encoding='utf-8')
    source = (ROOT / 'bench/examples/custom_measurement.rs').read_text(encoding='utf-8')
    source = source.replace('    fn metric(&self)', '''    fn value_domain(&self) -> airbug_bench::measurement::ValueDomain {
        airbug_bench::measurement::ValueDomain::Signed
    }
    fn metric(&self)''')
    source = source.replace('Ok(*value as f64)', 'Ok(*value as f64 * std::env::var("SIGN").unwrap().parse::<f64>().unwrap())')
    source = source.replace('counter.set(counter.get() + 3);', '''assert!(std::env::var_os("NO_WORK").is_none());
        counter.set(counter.get() + std::env::var("STEP").unwrap().parse::<u64>().unwrap());''')
    source = source.replace('    suite.work_units("items", 6);', '')
    (root / 'benches/signed.rs').write_text(source, encoding='utf-8')
    base = ['cargo', 'bench', '--offline', '--manifest-path', str(root / 'Cargo.toml'),
            '--bench', 'signed', '--', '--samples', '4', '--warmup-ms', '0',
            '--sample-ms', '1', '--sampling', 'linear', '--resamples', '32',
            '--bootstrap-distributions', '--baseline-store', str(root / 'baselines')]
    def execute(command, extra=None):
        current = dict(env, SIGN='-1', STEP='3')
        current.update(extra or {})
        result = subprocess.run(command, cwd=ROOT, env=current,
                                capture_output=True, text=True, encoding='utf-8', timeout=240)
        assert result.returncode == 0, result.stdout + result.stderr
        return result
    for name, flags, extra, expected in [
        ('negative', ['--save-baseline', 'negative'], {}, -3),
        ('loaded', ['--load-baseline', 'negative'], {'NO_WORK': '1'}, -3),
        ('decreased', ['--baseline', 'negative'], {'STEP': '6'}, -6),
        ('crossed', ['--baseline', 'negative'], {'SIGN': '1'}, 3),
        ('zero', ['--save-baseline', 'zero'], {'SIGN': '0'}, 0),
        ('from-zero', ['--baseline', 'zero'], {}, -3),
    ]:
        execute([*base, *flags, '--output', str(root / name)], extra)
        run = json.loads((root / name / 'run.json').read_text(encoding='utf-8'))
        rows = [o for o in run['observations'] if o['metric'] == 'work_units']
        assert len(rows) == 4 and all(float(o['value']) / o['operations'] == expected for o in rows), rows
        estimates = json.loads((root / name / 'estimates.json').read_text(encoding='utf-8'))
        row = next(r for r in estimates['rows'] if r['metric'] == 'work_units')
        for statistic in ['mean', 'median']:
            assert row['estimates'][statistic]['point'] == expected, row
        assert row['regressions'], row
        for regression in row['regressions']:
            assert math.isclose(regression['fit']['slope']['point'], expected, abs_tol=1e-10), regression
        formatted = json.loads((root / name / 'formatted.json').read_text(encoding='utf-8'))[0]
        assert all(r['value'] == expected for r in formatted['machine']), formatted
        with (root / name / 'formatted.csv').open(encoding='utf-8', newline='') as f:
            assert all(float(r['value']) == expected for r in csv.DictReader(f))
        if name in ('decreased', 'crossed', 'from-zero'):
            relative = json.loads((root / name / 'relative-distributions.json').read_text(encoding='utf-8'))
            relative = next(r for r in relative if r['metric'] == 'work_units')['report']
            percent = {'decreased': 100, 'crossed': -200, 'from-zero': None}[name]
            for statistic in ('mean', 'median'):
                assert relative[statistic]['point_percent'] == percent, relative
                if percent is None:
                    assert relative[statistic]['interval_percent'] is None
                    assert relative[statistic]['undefined_draws'] == 32
                else:
                    assert relative[statistic]['interval_percent'] == [percent, percent]
        assert (root / name / 'report.html').is_file()
    cli = ['cargo', 'run', '--offline', '--locked', '-p', 'cargo-airbug-bench', '--']
    execute([*cli, 'report', str(root / 'negative/run.json'), '--output', str(root / 'restored')], {'NO_WORK': '1'})
    for command in [
        ['compare', str(root / 'negative/run.json'), str(root / 'decreased/run.json'), '--metric', 'work_units', '--html'],
        ['report', str(root / 'decreased/run.json'), '--baseline', str(root / 'negative/run.json')],
    ]:
        target = root / (command[0] + '-restored.html')
        execute([*cli, *command, '--output', str(target)], {'NO_WORK': '1'})
        html = target.read_text(encoding='utf-8')
        assert 'Saved formatter comparison' in html and 'work_units' in html
        assert 'formatted regression comparison' in html
    print('Signed Cargo: negative/zero/sign-crossing totals, saved baseline, statistics and standalone report passed')
