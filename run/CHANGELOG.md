# Changelog

## Unreleased

## 0.1.0

- The run format (`airbug.run/1` manifests, progress, `airbug.test-report/1` reports) and the store under `.airbug/runs/<id>/`: atomic writes, bounded reads, listing and pruning by kind.
- `hub`: probe whether a hub serves a root and build the address of a run page.
