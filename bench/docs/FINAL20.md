# Третья партия: 20 функций

Реализованы API, команды и интеграции перечисленных пунктов. Это локальная версия: никакой публикации PR-комментариев, удалённого CI или универсальной поддержки всех GPU не подразумевается. Проверки и ограничения ниже являются частью контракта функций.

| № исходного списка | Возможность | Реализация и границы |
|---:|---|---|
| 40 | Защита данных | `plan.privacy`: allowlist окружения, секреты по именам из окружения родителя; literal/JSON-escaped redaction **до** записи логов; `export-preview`. Не распознаёт произвольные преобразования/кодировки секрета. |
| 35 | Нестабильность | `diagnose`: MAD между процессами, сдвиг между первой/второй половинами, AB/BA. Пороговые описательные флаги; не доказательство причинности. Missing отдельно посчитан. |
| 36 | Pilot | `pilot --precision --max-processes`: эвристическая оценка бюджета из точного интервала медианы; новое независимое подтверждение, без pooling/retry-until-pass. Не гарантирует точность или power при дрейфе. |
| 34 | PR | `git-compare` фиксирует base/head; `pr-report` создаёт Markdown со scope, интервалами и неопределённостью. Отправка комментария отсутствует. |
| 30 | A/B/C | `plan.variants`, ротации с обратным порядком, `multi`: сравнения с общей reference, Bonferroni по вариантам **и** метрикам. Полного рейтинга нет. |
| 11 | Атрибуты | Опциональный feature `macros`: `#[rbench::bench]` генерирует `register_NAME(&mut suite)`. Без автозапуска/скрытого registry; builder остаётся источником case IDs. |
| 16 | Async | `Suite::bench_async`, явный `Executor`, минимальный `LocalExecutor`. Future creation/poll/drop включены, создание executor исключено. Tokio требует пользовательского адаптера/собственного runtime. |
| 17 | Потоки | `workloads::parallel`: общий старт после успешного создания всех workers, join всех потоков, ошибки spawn/panic. Создание/завершение потоков включены в wall. |
| 18 | Pipeline | Bounded producer/consumer, FIFO, end-to-end latency до send, send wait+overhead, consumer work, throughput, cooperative cancellation. 1 producer / 1 consumer; не произвольный DAG. |
| 48 | Cold/warm | `Suite::cold()` — ровно один первый вызов одного unchecked case, без pilot/warmup; повтор этого case запрещён. Forma `--cold --filter static|text` включает parse, renderer/resources и первый completed frame. Внешние OS/driver caches не сбрасываются. |
| 46 | Renderer | Прямые frames, geometry/tile uploads, payload bytes, buffer/bind-group creations. Draw calls соответствуют единственному `pass.draw` на успешный frame в данном Forma snapshot. Rebuild/cache-hit counters явно Unsupported: источник их не предоставляет. |
| 47 | Allocation phases | `begin_phase()/finish()`: allocations/realloc/free/requested/live/peak по всему процессу; lifetime peak отдельно. Границы требуют quiescence других allocator users, внутри учитываются все потоки. Не thread-local, не VRAM/ObjC/driver memory. |
| 44 | GPU timestamps | Forma `--gpu-timestamps`: отдельный profiled replay после обычных измерений, serial query consumption, Unsupported/Invalid вместо нулей. Проверен Metal. |
| 45 | Frame deadlines | `deadlines --case --metric`: 60/120/144 Hz, exceedance и p95 только raw individual frames. Не выводит compositor dropped frames; scope completed/present-call/interval сохранён. |
| 49 | Матрица | `matrix --plan`: 1–8 осей CLI-аргументов, до 256 комбинаций; предварительная проверка всех plans, последовательные workers. Forma width/height/DPI/backend входят в contracts; golden разных размеров/DPI разделены. |
| 42 | Окно Forma | Опциональный `window` feature отдельной integration: main-thread winit, FIFO present, resize, occlusion/acquire skips, 30 s deadline, закрытие после 96 измеренных кадров. Present-call не scanout. |
| 50 | Profiler replay | `profile`: один recorded protocol case, проверка hashes и исходного workload descriptor; отдельная диагностическая сессия и profiler contract. Проверен `/usr/bin/time -l`. Profiler должен запускать переданный executable и пропускать stdout-протокол. GPU-only Forma replay требует отдельного single-case worker; обычные Forma и Suite cases поддерживаются. |
| 38 | Resume | `resume`: проверка plan hash, binaries/fixtures, allowlisted env, host/kernel, worker contracts; новая linked session, незаконченная пара/rotation block повторяется целиком. Автоматического объединения нет. Исходный secret env не поддерживается, его неизменность нельзя проверить. |
| 39 | Retention | `retention --keep N`: inventory/preview; `--apply` переносит только завершённые direct runner directories в quarantine. Baselines, последние N, родители/дети sessions защищены. Никакого окончательного удаления. |
| 37 | Revision search | `bisect`: ограниченный **полный аудит** first-parent диапазона вместо предположения о монотонности; общий статистический бюджет; остановка при Inconclusive. При возврате производительности — `nonmonotonic`, culprit отсутствует. Стоимость линейная, максимум 64 commits. |

## Быстрый старт

```rust
// Cargo.toml: rbench = { path = ".../crates/rbench", features = ["macros"] }
#[rbench::bench]
fn parsing() -> usize { std::hint::black_box("12345").len() }

fn main() -> rbench::Result<()> {
    let mut suite = rbench::Suite::new("app");
    register_parsing(&mut suite); // app/parsing; регистрации не исполняют workload
    suite.main()
}
```

```rust
use rbench::workloads::{LocalExecutor, Cancellation, pipeline, parallel};
fn main() -> rbench::Result<()> {
let mut suite = rbench::Suite::new("requests");
suite.bench_async("local", LocalExecutor, || async { 42 });
let workers = parallel(4, |index| index * index)?;
let flow = pipeline((0..100).collect(), 8, &Cancellation::default(), |n| n * 2)?;
assert_eq!(workers.outputs, [0, 1, 4, 9]);
assert_eq!(flow.processed, 100);
Ok(())
}
```

Для CPU-bound `LocalExecutor` не создаёт concurrency сам. Он паркует текущий поток до wake; futures, требующие I/O reactor/timers конкретного runtime, запускаются адаптером этого runtime. Noncooperative future/closure требует внешнего process timeout. Pipeline cancellation проверяется между элементами, не прерывает произвольный пользовательский код.

```sh
cargo rbench diagnose .rbench/run
cargo rbench pilot .rbench/pilot --precision 5 --max-processes 200
cargo rbench multi .rbench/abc --reference baseline
cargo rbench deadlines .rbench/forma --case forma/static --metric frame.completed
cargo rbench git-compare --repo . --baseline BASE --candidate HEAD \
  --target package/bench --offline -o .rbench/pr
cargo rbench pr-report .rbench/pr/run -o .rbench/pr/comment.md
cargo rbench profile .rbench/run --case suite/case --profiler /usr/bin/time \
  -o .rbench/profile -- -l
cargo rbench resume .rbench/interrupted -o .rbench/continued
cargo rbench retention --keep 20
cargo rbench retention --keep 20 --apply
```

`plan.privacy` пример (значения секретов **не** помещать в JSON):

```json
{
  "candidate": {"path":"/absolute/worker", "args":["--json"], "cwd":null},
  "protocol":true,
  "repetitions":12,
  "privacy":{"allow_env":["PATH","HOME"], "secret_env":["SERVICE_TOKEN"]}
}
```

Allowlisted переменные считаются несекретными и сохраняются для воспроизводимости. `program.env` остаётся явным несекретным override. Секреты 8–4096 bytes; совпадение значения с plan/metadata отвергается до сохранения этих данных. Workload не должен писать секреты в собственные файлы, кодировать их или выводить неизвестные runner секреты. Redaction относится к stdout/stderr, а не к произвольным побочным эффектам workload. Перед передачей bundle используйте `export-preview` и просмотрите run/notes.

A/B/C добавляет к обычному plan:

```json
{"variants":{"alternative":{"path":"/absolute/third-worker","cwd":null}}}
```

Обязателен baseline. Для трёх вариантов repetitions кратно 6: каждая позиция получает одинаковое число запусков. В `multi` сравнения проводятся с заранее выбранной reference, не с автоматически выбранным победителем.

Матрица оборачивает обычный plan:

```json
{
  "plan": {
    "candidate": {"path":"/absolute/rbench-forma-example", "args":["--goldens","/absolute/goldens","--filter","static"], "cwd":null},
    "protocol":true, "repetitions":1
  },
  "axes":{"--width":["640","800"],"--dpi":["1","2"],"--backend":["metal","dx12"]}
}
```

```sh
cargo rbench matrix --plan matrix.json -o .rbench/matrix
cargo build --manifest-path integrations/forma/Cargo.toml --release --features window
cargo rbench run --program integrations/forma/target/release/rbench-forma-window \
  --protocol --repetitions 1 --timeout-ms 35000 -o .rbench/window -- --json
```

Размеры Forma — физические pixels, DPI задаёт logical scale. Фиксированная сцена может обрезаться в маленьком logical viewport; это часть workload, а не автоматический responsive layout. Записывайте и визуально проверяйте отдельные goldens для каждой комбинации до измерений. Без backend capability получается Unsupported; ошибка golden остаётся ошибкой.

## Проверки

**51 проверка пройдена:** 50 workspace tests/doctests + 1 Forma query-result test; strict Clippy проходит для обоих workspaces.

Автотесты: controlled spread/drift/order, independent pilot confirmation simulation (2 000 seeded trials), A/B/C family correction, async wake, concurrent execution/join/panic/cancellation, first-invocation lifecycle, realloc/cross-thread free/phase peak, macro identity, secret redaction across byte boundaries, absent unlisted env, bundle redaction, balanced positions, resume immutability, protected retention. Все тестовые значения synthetic помечены соответствующим scope.

Локальные артефакты текущей проверки, намеренно исключённые из Git:

- `.rbench/final20-metal-final`: финальный smoke с 576 normal + 576 GPU frames; draw-call mapping проверен, отсутствующие rebuild/cache counters явно Unsupported.
- `.rbench/final20-allocator-v2`: system / static tracking allocator / active phase, 12 balanced repetitions (36 processes). По 5% margin оба instrumentation comparisons Inconclusive; стоимость не объявлена нулевой и нет автоматического добора до значимости.
- `.rbench/final20-profile-forma`: выбран только `forma/static` из GPU-enabled source; обычный replay под time проходит проверку полного descriptor.
- `.rbench/final20-profile-final`: повтор с финальной проверкой полного descriptor и hashes, 8 samples одного case.
- `.rbench/final20-metal`: 2 × 6 × 96 = **1 152** normal frames и столько же отдельных GPU query samples; checkpoint golden checks. Static: 0 geometry/tile uploads, 96 renderer frames и 4 608 payload bytes за phase.
- `.rbench/final20-window-v2`: **96** measured frames, 3 resize events, 1 occlusion event, 1 skipped frame/acquire; настоящее Metal surface. Первый прогон выявил лишний queued redraw после exit; исправлен верхний предел кадров.
- `.rbench/final20-cold`: 2 новых процесса, первый текстовый кадр, без предыдущего draw; post-golden validation.
- `.rbench/final20-matrix`: 4 Metal combinations с golden equality + 4 Unsupported DX12. Четыре reference PNG визуально просмотрены.
- `.rbench/final20-bisect`: реальная контрольная задержка 1 → 3 ms, правильно найден `5ad2b329a335e1d804e7fec934f82e8031c2fd1d`; 12 независимых process pairs.
- `.rbench/final20-nonmonotonic`: synthetic protocol fixture 100 → 200 → 100; оба commits проверены против reference, результат `nonmonotonic`, culprit `null`.
- `.rbench/final20-profile-v2`: настоящий `/usr/bin/time -l` replay; первый sandbox-прогон честно Failed из-за запрета `sysctl kern.clockrate`.

В этой версии оконные изображения не проверяются pixel golden; корректность отрисовки подтверждена offscreen checkpoints. CPU/GPU/window числа — локальные smoke evidence, не универсальные оценки FPS или заявления о превосходстве реализации. HTML сохраняет прежнее ограничение: структурная проверка, без нового browser visual proof.
