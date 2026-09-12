# Следующие 20 возможностей

Реализация продолжает выбранную первую десятку. Нумерация соответствует исходному списку 50 предложений.

| № | Возможность | Реализованный интерфейс |
|---:|---|---|
| 3 | Профили запуска | `profiles`, worker `--profile quick/normal/thorough` |
| 5 | Сравнение Git revisions | `git-compare`, отдельные локальные снимки и каталоги сборки |
| 6 | Последний результат | `history`, `report last`, `compare @main last` |
| 8 | Preview запуска | runner/worker `--dry-run`, без выполнения workload |
| 9 | Фильтры и теги | worker `--exact`, `--glob`, `--exclude`, `--tag`, `Suite::tag` |
| 10 | Preflight окружения | `preflight --plan`, проверка файлов/прав/fixtures/cwd/deadlines/output |
| 13 | Fixtures с разным временем жизни | Lazy `Fixture`, `bench_fixture`, существующий fresh input на операцию |
| 14 | Именованные фазы | `Recorder::phase`, `Recorder::measure` |
| 19 | Единицы работы | `Suite::work_units`, производная throughput в отчёте |
| 20 | Воспроизводимые входы | `Seeded` (SplitMix64), `Suite::seed` в contract |
| 21 | Краткий итог | Число регрессий/неразрешённых метрик; проблемные строки первыми |
| 23 | Сырые графики | Отдельные SVG scatterplots по каждому процессу в HTML |
| 24 | Объяснение неопределённости | Рекомендации по missing/zero baseline/числу процессов/широкому интервалу |
| 25 | Diff контекстов | `context A B`: environment, provenance, case/metric contracts |
| 26 | История производительности | `trend --case --metric`; HTML-графики разделены по context hash |
| 27 | Экспорт | `export --format csv/jsonl` с сырыми значениями и availability |
| 28 | Переносимый результат | `bundle`, `unpack`: JSON-контейнер с SHA-256 каждого файла |
| 29 | Заметки | `note`, `notes`; отдельно от неизменяемого run.json |
| 32 | Политика неопределённости | `compare --check` / `gate --uncertainty fail/warn/record` |
| 33 | CI-шаблон | `ci -o FILE`: ручной GitHub Actions smoke workflow |

## Быстрый рабочий цикл

В workspace теперь зарегистрирован реальный target `rbench/workloads`: два CPU-сценария с общим lazy fixture, seed 42, тегами и единицами работы.

```sh
cargo build --release --workspace --offline
cargo rbench profiles
cargo rbench bench --offline --repetitions 3 -o .rbench/demo -- --profile quick --tag cpu
cargo rbench history
cargo rbench note last "Seed 42; быстрый smoke-прогон"
cargo rbench report last -o .rbench/demo.html
cargo rbench export last --format csv -o .rbench/demo.csv
cargo rbench export last --format jsonl -o .rbench/demo.jsonl
cargo rbench bundle last -o .rbench/demo.bundle.json
cargo rbench unpack .rbench/demo.bundle.json -o .rbench/demo-restored
```

`last` ищет последний Complete run по времени изменения файла в `--store` (по умолчанию `.rbench`). Это удобный навигатор, не стабильная ссылка: для воспроизводимого сравнения сохраните baseline. Если реальный путь `last` существует, он имеет приоритет. Сканирование ограничено глубиной 8 и 20 000 каталогов; `target`, `.git`, `checkouts`, baseline/notes и символические каталоги не обходятся. Повреждённые run-файлы показываются как Invalid. Копирование/восстановление старого run может сделать его последним по filesystem time.

Профили меняют worker samples/warmup/sample time. `--repetitions` отдельно задаёт число независимых процессов. Explicit `--samples`/`--sample-ms`/`--warmup-ms` переопределяют профиль независимо от порядка аргументов. Quick не предназначен для доказательства отсутствия регрессии.

## Фильтр и preview

```sh
# PROGRAM — путь к executable из plan.json или результата Cargo build.
PROGRAM --list --filter 'workloads/*/4096' --glob --exclude '*zero*' --tag cpu
PROGRAM --dry-run --profile quick --filter workloads/checksum/4096 --exact
cargo rbench run --plan experiment.json --dry-run -o .rbench/not-created
cargo rbench preflight --plan experiment.json -o .rbench/not-created
```

`*` и `?` сопоставляются со всей строкой case ID; shell patterns нужно заключать в кавычки. Повторные `--tag` требуют все теги. `--exclude` всегда glob. Обычный `--filter` сохраняет поиск подстроки; `--exact` и `--glob` несовместимы. Эти флаги относятся к Suite worker; фильтр CLI compare пока остаётся подстрокой.

Worker dry-run показывает известные descriptors и sample budget без setup/validation/calibration. Runner dry-run показывает программы, AB/BA порядок, SHA-256, cwd и конфигурацию; executable он не вызывает и cases из него не угадывает. Значения `plan.env` в preview скрыты; реальный plan.json по-прежнему сохраняет explicit env. Preflight проверяет executable bit на Unix, файлы fixtures, cwd, допустимые deadlines и отсутствие output; показывает `df -k`, когда доступен. Он не гарантирует будущий доступ к GPU, свободное место на весь эксперимент или право записи в ещё не созданные каталоги.

## Fixtures, seed и фазы

```rust
use rbench::{Fixture, Seeded, Suite};
let fixture = Fixture::new(|| Seeded::new(42).bytes(4096));
let mut suite = Suite::new("data");
suite.bench_fixture("checksum", fixture.clone(), |bytes| {
    bytes.iter().map(|b| u64::from(*b)).sum::<u64>()
}).tag("cpu").work_units("bytes", 4096).seed(42);
```

Fixture инициализируется один раз при первом измерении. Клоны разделяют одно mutable значение внутри процесса. Отдельный `Fixture::new` на case даёт отдельное состояние; `bench_with_input`/`bench_checked` продолжают создавать свежий input на каждую операцию. Повторный `Suite::run` в том же процессе не сбрасывает shared fixture. Межпроцессного singleton здесь нет. Setup/заимствование/уничтожение fixture вне timing; output Drop включён. Seeded не криптографический генератор. `seed()` записывает договорённость в contract — caller должен использовать этот же seed для генератора.

Throughput вычисляется из положительных wall batch durations и количества work units на операцию; это batch throughput, а не распределение индивидуальных задержек. Zero/Unavailable durations в производную throughput не входят, исходные наблюдения сохраняются. Для `bytes` показывается MiB/s, остальные имена отображаются как `<unit>/s`. Производную throughput можно получить и проверить командой `cargo rbench throughput RUN` (текст или `--json`, фильтр `--filter`, бюджет `--min`/`--max`, коды выхода как у `check`: 0 — пройдено, 1 — нарушение границы, 2 — ошибка использования). Она остаётся производной величиной: отдельным observation не записывается, исходные наблюдения неизменны. Гейтинг требует завершённого запуска.

```rust
recorder.phase("pipeline", "parse", "CPU parse")?;
let parsed = recorder.measure("pipeline", "parse", || parse(input))?;
```

Case регистрируется заранее; неизвестная фаза отклоняется до вызова closure. Measure измеряет только синхронное замыкание, затем добавляет observation; output Drop снаружи. Один phase descriptor нельзя незаметно заменить другим. [Исполняемый пример фаз](../crates/rbench/examples/phases.rs).

## Git-сравнение

```sh
cargo rbench git-compare --repo /path/to/repo \
  --baseline main --candidate feature \
  --target package/bench --offline --repetitions 12 \
  -o .rbench/git-ab -- --profile normal
cargo rbench compare .rbench/git-ab/run --check
```

Используются локально существующие commits/refs. Ветки не переключаются, fetch/reset/stash не выполняются. Незакоммиченные изменения не включаются. Читаются tracked blobs в новые snapshots; обе сборки заканчиваются до измерений. `CARGO_TARGET_DIR` задаётся отдельно для каждого snapshot, включая случай унаследованной общей настройки. Полные SHA commits записываются в git.json и provenance run. `--manifest-path` — относительный путь внутри snapshot.

Сейчас symlinks/submodules отклоняются; для таких репозиториев используйте подготовленные бинарники и `run --baseline`. LFS checkout/smudge не выполняется. Absolute/path dependencies вне снимка, системные библиотеки и зависимости без зафиксированного lockfile не становятся автоматически воспроизводимыми. Snapshot-путь доступен build scripts и `file!()`. Cargo config/build scripts исполняются как при обычной сборке доверенного проекта. Это изоляция checkout/build outputs, не sandbox выполнения чужого кода.

## История, контекст и отчёт

```sh
cargo rbench context @main last
cargo rbench trend --case workloads/checksum/4096 --metric wall -o .rbench/trend.html
```

Trend показывает медиану process medians, а не объединяет все внутренние samples. Отдельный context hash включает окружение и descriptor/contract выбранного case; разные контексты рисуются отдельно. Commit отображается, если записан через git-compare. Это описательная история, не причинное доказательство влияния commit. Дрейф окружения возможен даже при совпадающем hash.

Raw plots разделены по case/metric/variant/process, каждая имеет свою шкалу. По оси X sequence, по Y исходное значение (wall batches нормализованы на operation). Отображается до 64 plots и до 1000 точек на plot с прореживанием; экспорт сохраняет всё. Графики находятся в раскрываемых секциях и не делают статистических выводов. Сообщения Inconclusive различают нулевой baseline, недостаток независимых единиц и широкий интервал; не предлагают повторять измерение до желательного результата.

## Переносимость и заметки

Bundle содержит точные bytes run.json, заново генерируемый report.html и notes.json с SHA-256. Бинарники, stdout/stderr и внешний plan.json не включаются: это архив для анализа, а не полностью исполняемая копия эксперимента. Unpack проверяет schema, имена, дубли, checksum, модель Run и привязку notes; HTML заново создаётся из Run. Output должен быть новым. Входной bundle ограничен 128 MiB. Checksum обнаруживает повреждение, но не подтверждает доверие к автору, способному пересчитать hash.

Notes привязаны к hash неизменяемого run.json; базовые файлы и baseline не переписываются. Экспорт CSV заключает поля в кавычки и защищает начальные spreadsheet formula characters апострофом; JSONL сохраняет исходные строки без такой модификации. Каждая JSONL строка содержит observation и metric descriptor; полный contract хранится в Run/bundle.

## Политика CI

```sh
cargo rbench compare @main last --check --uncertainty warn
cargo rbench gate last --config budgets.json --uncertainty fail
cargo rbench ci -o rbench-ci.yml
```

`fail` — стандартный код 2 при Inconclusive. `warn` разрешает Inconclusive с предупреждением; `record` оставляет результат в отчёте и возвращает 0. Регрессия/провал всегда даёт 1. Missing/Unsupported/ошибка остаются кодом 2 при любой политике. Приоритет провала выше неопределённости.

CI-файл — готовый вручную запускаемый smoke workflow для этого workspace; команда не устанавливает его и не запускает удалённый CI. Включены сборка, quick benchmark и сохранение отчётов/artifacts, в том числе hidden `.rbench`. Для другого repo адаптируйте установку CLI и бюджеты. Shared GitHub runner не даёт контролируемой performance acceptance.

Версии действий сверены с первичными источниками: [actions/checkout](https://github.com/actions/checkout), [actions/upload-artifact](https://github.com/actions/upload-artifact), [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain). Workflow uses checkout/upload-artifact v7 и rust-toolchain stable; это не полностью закреплённая цепочка инструментов.
