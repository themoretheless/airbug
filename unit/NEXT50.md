# NEXT50: следующий слой поверх TOP50

TOP50 (v0.4) закрыт. Ниже 50 следующих улучшений из ROADMAP / README gaps.
Статусы: **done** = есть API + тест в этом срезе; **stub-done** = осознанный stub
(ошибка Unsupported / Err «not implemented» / docs-only gap) с тестом или явной пометкой.

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
| 9 | Snapshots | Inline source update mode | `UpdateMode::InlineSource` → `Unsupported` | stub-done |
| 10 | Snapshots | Binary / hex snapshots | `Snapshots::check_bytes` | done |
| 11 | Prop | Shrinking | `Shrink` + `Prop::shrink_from` (minimal) | done |
| 12 | Prop | Multi-arg strategies | `Prop::for_all2` | done |
| 13 | Prop | Strategy combinators | `Strategy::map` / `filter` | done |
| 14 | Mock | Generic method escape hatch | `ExpectationBuilder::map_args` | done |
| 15 | Mock | Borrowed return adapter | `mock::borrowed` | done |
| 16 | Mock | Associated type stub helper | docs in `mock::borrowed` | done |
| 17 | Mock macro | One generic method pattern | compile_error → manual Mock + map_args/borrowed | stub-done |
| 18 | Mock | Completion-order barrier | `CompletionBarrier` stub (no fake `await_done`) | stub-done |
| 19 | Mock | Pending/cancelled async probes | `Mock::pending` / `pending_response` | done |
| 20 | Validation | Async validate | `Validator::validate_async` | done |
| 21 | Validation | Cancel token | `ValidationCancel` | done |
| 22 | Validation | Concurrent field rules | `validate_parallel` | done |
| 23 | Fixture | Weighted choose | `FixtureContext::choose_weighted` | done |
| 24 | Fixture | From distribution | `draw_f64` / ranges | done |
| 25 | Fixture | Persisted counterexample seed | `AIRBUG_PROP_FAILURE_DIR` / `persist_failures` | done |
| 26 | Checks | JSON path equal | `assert_json_path` | done |
| 27 | Checks | Soft assert batch | `SoftAssert` | done |
| 28 | Checks | Approx duration | `assert_duration_eq` | done |
| 29 | Checks | Approx Instant/SystemTime | `assert_near` / `assert_instant_near` | done |
| 30 | Time | Fake sleep park | `ParkClock` | done |
| 31 | Time | Deadline helper | `Deadline::remaining` | done |
| 32 | Time | Retry policy | `Retry::exponential` | done |
| 33 | Report | Flaky label helper | `report::flaky` | done |
| 34 | Report | Secret redaction presets | `redact::secrets` | done |
| 35 | Report | Attach JSON blob | `attach_json` | done |
| 36 | Containers | HTTP wait strategy | `Wait::http` | done |
| 37 | Containers | Ready JSON probe | `Wait::http_json` | done |
| 38 | Containers | CI skip policy docs | README + soft-skip | done |
| 39 | Containers | Postgres app-shaped example | `examples/postgres_wait.rs` | done |
| 40 | Native | Temp dir fixture | `TempDir` | done |
| 41 | Native | Env var scope | `EnvScope` | done |
| 42 | Native | Working dir scope | `DirScope` | done |
| 43 | Order | Partial-order DAG | `CallDag` | done |
| 44 | Order | Happens-before assert | `happened_before` | done |
| 45 | Snapshot | Pretty vs compact JSON modes | `JsonMode` | done |
| 46 | Snapshot | Schema version stamp in snap | `Snapshots::schema_stamp` | done |
| 47 | Prelude | Re-export Prop / Eventually async | prelude | done |
| 48 | Docs | NEXT50 example binary | `examples/next50.rs` | done |
| 49 | Docs | ROADMAP sync with done rows | ROADMAP.md | done |
| 50 | Docs | Maturity note in README | README | done |

Срез закрыт: shrinking минимальный; `InlineSource` и `CompletionBarrier` — явные stubs;
`#[mock]` generics не кодогенерятся, а указывают на manual Mock + `map_args` / `borrowed`.
