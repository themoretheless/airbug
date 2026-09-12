#!/usr/bin/env python3
"""Build and test the actual archives without publishing either workspace crate.

A local patch connects the two unpacked packages, avoiding Cargo's temporary
registry checksum failure for unpublished workspace dependencies.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument('--toolchain', default='stable')
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
cargo = ['cargo', '+' + args.toolchain]

def run(arguments, directory=root, **kwargs):
    return subprocess.run(cargo + arguments, cwd=directory, check=True, **kwargs)

metadata = json.loads(run(['metadata', '--offline', '--no-deps', '--format-version=1'],
                         capture_output=True, text=True).stdout)
packages = {p['name']: p for p in metadata['packages']}
run(['package', '--offline', '--workspace', '--all-features', '--allow-dirty', '--no-verify'])
with tempfile.TemporaryDirectory(prefix='airbug-packages-') as temp:
    temp = Path(temp).resolve()
    unpacked = {}
    for name in ['airbug', 'airbug-macros']:
        folder = name + '-' + packages[name]['version']
        archive = Path(metadata['target_directory']) / 'package' / (folder + '.crate')
        with tarfile.open(archive) as contents:
            # Accept only ordinary directories/files inside the expected package root.
            for member in contents.getmembers():
                destination = (temp / member.name).resolve()
                if not destination.is_relative_to(temp / folder) or not (member.isfile() or member.isdir()):
                    raise RuntimeError('Unexpected archive member: ' + member.name)
            contents.extractall(temp)
        unpacked[name] = temp / folder
    consumer = temp / 'consumer'
    (consumer / 'src').mkdir(parents=True)
    # JSON string syntax is compatible with TOML basic strings for these paths.
    path = lambda name: json.dumps(str(unpacked[name]))
    (consumer / 'Cargo.toml').write_text(
        '[package]\nname="airbug-package-check"\nversion="0.0.0"\nedition="2024"\n'
        '[workspace]\n[dependencies]\n'
        f'helpers={{package="airbug",path={path("airbug")},features=["macros"]}}\n'
        '[patch.crates-io]\n'
        f'airbug-macros={{path={path("airbug-macros")}}}\n', encoding='utf-8')
    (consumer / 'src/lib.rs').write_text('''
#[cfg(test)]
mod tests {
    #[derive(helpers::Generate)]
    struct Data { value: u64 }
    #[helpers::mock]
    trait Store { fn get(&self, id: u64) -> u64; }
    #[helpers::cases(first(7), second(9))]
    fn packaged_helpers(value: u64) {
        let mut ctx = helpers::FixtureContext::new();
        let data = ctx.builder::<Data>().with_value(value).build();
        let store = MockStore::default();
        let sequence = helpers::CallSequence::new("packaged sequence");
        store.get.expect("id", move |(id,)| *id == value).in_sequence(&sequence).returns(value);
        helpers::with_mocks(&[&store, &sequence], || { assert_eq!(store.get(data.value), value); });
    }
}
''', encoding='utf-8')
    environment = dict(os.environ, CARGO_TARGET_DIR=str(temp / 'target'))
    run(['test', '--offline'], consumer, env=environment)
print('Both package archives passed consumer tests.')
