# NEXT50: следующий слой поверх TOP50

TOP50 (v0.4) закрыт. Ниже 50 следующих улучшений из ROADMAP / README gaps.
Статусы: **done** = есть API + тест в этом срезе; **plan** = в бэклоге.

| № | Тема | Улучшение | API | Статус |
|---|---|---|---|---|
| 1 | Snapshots | Canonical JSON snapshot | `Snapshots::check_json` | done |
| 2 | Snapshots | Список устаревших `.snap` | `Snapshots::list_obsolete` | done |
| 3 | Snapshots | Assert нет obsolete | `Snapshots::assert_no_obsolete` | done |
| 4 | Snapshots | JSON case snapshots | `Snapshots::check_json_case` | done |
| 5 | Prop | Seeded property runner | `Prop::for_all` | done |
| 6 | Prop | Persist counterexample | `Prop::persist_failures` | done |
| 7 | Prop | Replay from seed/file | `Prop::replay` / `PropFailure` | done |
| 8 | Time | Async eventual polling | `Eventually::check_async` | done |
| 9 | Snapshots | Inline source update mode | `UpdateMode::InlineSource` | plan |
| 10 | Snapshots | Binary / hex snapshots | `Snapshots::check_bytes` | plan |
| 11 | Prop | Shrinking | `Prop::shrink` | plan |
| 12 | Prop | Multi-arg strategies | `Prop::for_all2` | plan |
| 13 | Prop | Strategy combinators | `Strategy::map/filter` | plan |
| 14 | Mock | Generic method escape hatch | `MockMethod::map_args` | plan |
| 15 | Mock | Borrowed return adapter | `mock::borrowed` | plan |
| 16 | Mock | Associated type stub helper | docs + adapter | plan |
| 17 | Mock macro | One generic method pattern | `#[mock]` expand | plan |
| 18 | Mock | Completion-order barrier | `CallSequence::await_done` | plan |
| 19 | Mock | Pending/cancelled async probes | `Mock::pending` | plan |
| 20 | Validation | Async validate | `Validator::validate_async` | plan |
| 21 | Validation | Cancel token | `ValidationCancel` | plan |
| 22 | Validation | Concurrent field rules | `validate_parallel` | plan |
| 23 | Fixture | Weighted choose | `FixtureContext::choose_weighted` | plan |
| 24 | Fixture | From distribution | `draw_f64` / ranges | plan |
| 25 | Fixture | Persisted counterexample seed | auto from Prop | plan |
| 26 | Checks | JSON path equal | `assert_json_path` | plan |
| 27 | Checks | Soft assert batch | `SoftAssert` | plan |
| 28 | Checks | Approx duration | `assert_duration_eq` | plan |
| 29 | Checks | Approx Instant/SystemTime | `assert_near` | plan |
| 30 | Time | Fake sleep park | `ParkClock` | plan |
| 31 | Time | Deadline helper | `Deadline::remaining` | plan |
| 32 | Time | Retry policy | `Retry::exponential` | plan |
| 33 | Report | Flaky label helper | `report::flaky` | plan |
| 34 | Report | Secret redaction presets | `redact::secrets` | plan |
| 35 | Report | Attach JSON blob | `step.attach_json` | plan |
| 36 | Containers | HTTP wait strategy | `Wait::http` | plan |
| 37 | Containers | Ready JSON probe | `Wait::http_json` | plan |
| 38 | Containers | CI skip policy docs | README + test | plan |
| 39 | Containers | Postgres app-shaped example | examples | plan |
| 40 | Native | Temp dir fixture | `TempDir` helper | plan |
| 41 | Native | Env var scope | `EnvScope` | plan |
| 42 | Native | Working dir scope | `DirScope` | plan |
| 43 | Order | Partial-order DAG | `CallDag` | plan |
| 44 | Order | Happens-before assert | `happened_before` | plan |
| 45 | Snapshot | Pretty vs compact JSON modes | `JsonMode` | plan |
| 46 | Snapshot | Schema version stamp in snap | header | plan |
| 47 | Prelude | Re-export Prop / Eventually async | prelude | done |
| 48 | Docs | NEXT50 example binary | `examples/next50.rs` | plan |
| 49 | Docs | ROADMAP sync with done rows | ROADMAP.md | done |
| 50 | Docs | Maturity note in README | README | done |

Границы среза 1–8: без shrinking, без правок исходников inline, без serde в default path кроме feature `json`.
