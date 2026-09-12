# Требования из проектов владельца

Исследовано чтением исходников; существующие проекты не изменялись и их бенчмарки не запускались. Снимок Forma: `themoretheless/forma`, commit `e2ef5f7792af363fe6e625febd44ced0c874cb9f` от текущего remote HEAD на момент получения. Числа из его документации — исторические результаты автора, не новые измерения rbench.

## Forma — первый потребитель и критерий архитектуры

Источники находятся в [локальном снимке](../research/private/forma/README.md). Требования ниже разделяют уже существующий механизм и предлагаемый контракт rbench.

| Подтверждённое поведение | Источник в Forma | Требование rbench |
|---|---|---|
| Native и GPU feature gates, `wgpu = 29.0.4`, также WASM | `vector-ui/Cargo.toml` | Минимальное ядро без wgpu/winit; GPU-адаптер версионируется отдельно; browser — отдельный driver |
| 20 warmup кадров, затем массивы submit/completed, GPU wait после кадра | `vector-ui/examples/perf_bench.rs:124` | Scenario driver с явным completion boundary; single-frame latency отдельно от pipelined throughput |
| GPU timestamp серия из 30 кадров на другом profiled renderer | `vector-ui/examples/perf_bench.rs:170` | Метрика несёт phase ID и instrumentation mode; отдельные серии не суммируются как части одного кадра |
| Resize пересоздаёт target, меняет физический размер | `vector-ui/examples/perf_bench.rs:97` | Стоимость target recreation включается в соответствующий сценарий; параметры resize являются частью case ID |
| Timestamp может отсутствовать или серия может быть неполной | `vector-ui/examples/perf_bench.rs:209` | `Unavailable/Invalid` с причиной, requested/valid counts, без нуля или переполненного времени |
| Успешные present, acquisition failures, occlusion и actual DPI | `vector-ui/examples/window_bench.rs` | Main-thread/event-loop driver, timeout, корректная обработка окна и отдельный счётчик успешных present |
| CPU all-threads, Rust allocations, requested/live/peak bytes, RSS | `vector-ui/examples/bench_support/mod.rs` | Область, тип и единица каждой метрики; RSS lifetime highwater отдельно от phase peak |
| Общая память Apple Silicon; GPU payload неполон | `vector-ui/BENCHMARKS.md` | Запрет автоматического сложения RSS и GPU payload; отсутствие total residency обозначается явно |
| Готовые A/B executable, 4 пары AB/BA, контроль хешей до/после | `scripts/interleave-bench.mjs` | Build отделён от run; неизменяемые executable, fixture manifest, pair ID и schedule |
| Проверка полного набора cases/repeats, metadata, adapter и SHA fixtures | `scripts/compare-bench.mjs` | Compatibility validator перед любым сравнением; duplicate/missing case означает invalid, не тихий пропуск |
| Семь JS batches по 500 компиляций, sampled heap | `scripts/compiler-bench.mjs` | Внешний JS driver и импорт batch результатов; heap delta не переименовывается в allocation bytes |
| Сравнение 54 GPU goldens, visual revision и geometry uploads | `vector-ui/OPTIMIZATION_STEPS.md`, `perf_bench.rs` | Валидация корректности — отдельный связанный результат, с хешем baseline и политикой сравнения |

### Первый набор сценариев Forma

1. **CPU library:** markup/template parsing; сборка display list; tile binning; shaping/outlines; hit testing. Каждый получает собственную границу setup и fixture.
2. **Offscreen serialized:** существующие image/text/nested сцены, 800×400, 1080p, 4K, DPI 2; forced/animation/resize. Миграция сохраняет текущую семантику.
3. **Native window:** actual size/DPI, present mode, события occlusion, интервалы present, readiness. Нельзя подменять offscreen-серией.
4. **Memory/resource:** неизменная анимация без geometry uploads; grow/shrink до плато; счётчик собственных buffers и upload traffic.
5. **Compiler/Studio:** текущий JS compile fixture как process/import driver. Позже расширить размер проекта; нынешний маленький fixture не подтверждает масштабируемость Studio.
6. **Позже, после появления соответствующего runtime:** большие деревья интерактивных контролов, scroll/glyph cache, browser/WebGPU. Это предлагаемые нагрузки, а не уже существующие возможности Forma.

### Что нужно улучшить при миграции

В `perf_bench` покадровые массивы вычисляются, но наружу выводятся агрегаты. rbench должен сохранять и сырые samples: из summary нельзя восстановить порядок, автокорреляцию и корректную повторную статистику. Набор из четырёх процессов A/B полезен как диагностика, но сам по себе не даёт убедительного разрешения малых регрессий. Нужен заранее заданный confirmatory план с достаточным числом независимых повторов.

Для профилируемой GPU-фазы размер resize фиксирован последним target и используется другой renderer. Это нужно сохранить в контракте явно, иначе отчёт будет неверно интерпретирован как время каждого resize.

## Другие проекты: найденные повторяющиеся потребности

| Проект и прочитанный файл | Текущее устройство | Что обобщить |
|---|---|---|
| [camera-man](/Users/themoretheless/Documents/Sources/camera-man/src/benchmarking.rs) | Версия схемы, fixture profile, окружение до/после, сырые samples, bootstrap CI, process summaries, copy ledger | Типизированный отчёт, снимки окружения, пользовательские counters, сравнение на уровне процессов |
| [rrrah](/Users/themoretheless/Documents/Sources/rrrah/scripts/bench-harness.py) | Process boundary, JSONL, SHA бинаря и RAW-файла, состояние OS cache unknown | Процессный driver, fixture identity, явный cache policy; «выключен app cache» не равно «холодный OS cache» |
| [allpaka](/Users/themoretheless/Documents/Sources/allpaka/scripts/bench-compare.sh) | Чередование A/B, prefill/decode, модель SHA, проверки tokens/fast path, ограничения эквивалентности | Общий workload contract; объём работы и семантика обязательны до объявления победителя |
| [tokenizer](/Users/themoretheless/Documents/Sources/tokenizer/benches/json_bench.rs) | Самописный Instant loop, warmup, black_box, valid/invalid/Unicode/deep datasets, MiB/s | Первый простой потребитель microbench API; throughput + correctness + параметризация |
| [vbuff](/Users/themoretheless/Documents/Sources/vbuff/crates/vbuff-store/tests/performance_budget.rs) | In-memory store, 1000 insert + 100 search, грубые абсолютные budgets | Разделять smoke budget и сравнительный benchmark; fixture reset, размер данных и disk/in-memory в контракте |
| [dbill](/Users/themoretheless/Documents/Sources/dbill/ui/scripts/benchmark-grid.mjs) | Production TS helpers, 32/128/512 колонок, 40 batches, исходники SHA, явное исключение DOM/RSS | Межъязыковой importer и scope. Percentile batch averages не называть latency отдельной операции |
| [UXI](/Users/themoretheless/Documents/Sources/uxi/packages/egui/src/bin/playground.rs) | Native playground служит UI-сценарием | Потенциальный window consumer, требует отдельной привязки событий и framebuffer validation; полный анализ UI здесь не проводился |
| [Voxy](/Users/themoretheless/Documents/ChatGPT/Voxy/Cargo.toml) | Отдельные crates world/mesher/runtime/render | Потенциальный stage-based CPU/GPU consumer. Проверена структура workspace; конкретные бюджетные значения ещё не определены |

## Приоритеты

**P0:** валидность результата; единый versioned protocol; process runner; Forma import; raw data; batch/frame distinction; units/scope/availability; неизменяемые baselines; отсутствие обязательного GPU в ядре.

**P1:** удобный Rust API и параметры; полноценный Forma adapter; allocation diagnostics; pair-aware statistics; переносимые окружение/CPU/RSS; HTML/Markdown/JSON.

**P2:** GPU timeline, tracing/samply/perf/Valgrind integration; browser worker; workload concurrency; CI history.

**Отложить:** распределённый кластер запусков, собственный profiler UI, универсальный Rust plugin ABI, LLM-объяснения результатов, автоматическое принятие нового baseline.
