# bench

Своя Rust benchmark-библиотека и runner: короткие операции, отдельные процессы и сценарии приложения в одном формате наблюдений. Реализован рабочий прототип 0.1.0; полная приёмка и переносимость ещё проверяются.

Новые удобства: **init, обнаружение workspace targets, именованные baseline, прогресс/ETA, матрицы параметров, проверка результата, HTML-таблицы, бюджеты и Forma golden-сценарии**. [Полный рабочий процесс](docs/USABILITY.md).

Добавлены ещё [20 возможностей рабочего цикла](docs/NEXT20.md): профили, Git-сравнение, `last`, dry-run, фильтры/теги, fixtures/фазы, throughput, seed, история, графики, экспорт/bundle, заметки и CI-политика.

Реализована и третья партия — [ещё 20 функций рабочего цикла](docs/FINAL20.md): приватность, диагностика нестабильности, pilot, A/B/C, атрибуты `#[bench]`, async/threads/pipeline, cold-прогоны, фазы аллокаций, Forma GPU/окно, матрицы аргументов, profiler replay, resume, retention и bisect.

## Быстрый запуск

```sh
cargo build --release --workspace --offline
cargo build --release --examples --offline
cargo airbug-bench doctor
# Список без исполнения workload:
target/release/examples/sort --list
cargo airbug-bench run --program target/release/examples/sort --protocol \
  --repetitions 12 -o .airbug-bench/sort -- --json
cargo airbug-bench report .airbug-bench/sort -o .airbug-bench/sort.html
```

`--offline` подходит при наличии зависимостей в Cargo cache; при первой сборке его можно убрать. Alias `cargo airbug-bench` настроен в этом workspace. Для других проектов: `cargo install --path bench/cli --offline`.

Веб-интерфейс: `cargo airbug-bench serve .bench`. Сводные отчёты по папке экспериментов: HTML с фильтрами и графиками, Markdown и JSON. [Построение отчётов](docs/REPORTS.md).

Профиль памяти по стекам: `run --memory` и [подключение Rust worker](docs/MEMORY_PROFILER.md).

## API библиотеки

Подключение из этого monorepo:

```toml
airbug-bench = { path = "../bench" }
# или из GitHub monorepo:
# airbug-bench = { git = "https://github.com/themoretheless/airbug", package = "airbug-bench" }
```

Локальный checkout: `airbug-bench = { path = "/path/to/airbug/bench" }`. Для Cargo benchmark target задайте `harness = false`.

CI monorepo: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (MSRV **1.96**, workspace `cargo test` / clippy). Отдельного `release.yml` в этом репозитории нет — версии поднимаются вручную в crate `Cargo.toml`.

```rust
use airbug_bench::{DropPolicy, Suite};
fn main() -> airbug_bench::Result<()> {
    let mut suite = Suite::new("collections");
    suite.bench_with_input(
        "sort/1000",
        || (0..1000u64).rev().collect::<Vec<_>>(),
        |input| input.sort_unstable(),
        DropPolicy::InsideTiming,
    ).parameter("elements", 1000);
    suite.main()
}
```

Каждая операция получает свежий input. Его создание и уничтожение находятся вне измерения; `DropPolicy` определяет уничтожение **результата** операции. Внутренние batch ограничены 64 входами, калибровка ограничена числом операций. Простые замыкания регистрируются через `suite.bench`. `--filter` ищет подстроку, `--samples`, `--sample-ms`, `--warmup-ms` управляют сбором. Исполнение требует release-сборку.

## Сравнение и автоматические проверки

```sh
cargo airbug-bench run --program /path/to/candidate --baseline /path/to/baseline \
  --protocol --repetitions 12 -o .airbug-bench/ab -- --json
cargo airbug-bench compare .airbug-bench/ab --threshold 5 --check
# Или два исторических запуска:
cargo airbug-bench compare .airbug-bench/old .airbug-bench/new --json
# Абсолютный бюджет на каждое наблюдение, без статистического вывода:
cargo airbug-bench check .airbug-bench/forma --metric geometry.uploads --max 0
# Нижняя граница или диапазон на конкретную метрику (можно сузить кейсы через --filter):
cargo airbug-bench check .airbug-bench/forma --metric frame.completed --min 8 --max 16 --filter static
# Производная throughput (units/s; MiB/s для bytes) с бюджетом:
cargo airbug-bench throughput .airbug-bench/run --min 1000
```

Runner чередует AB/BA, последовательно запускает процессы, сохраняет stdout/stderr, план, SHA-256 бинарников/fixtures и сырые наблюдения. Каталог результата должен быть новым: существующие данные не перезаписываются. Без `--protocol` измеряется длительность процесса целиком, включая запуск и ожидание завершения, с разрешением polling около 1 ms.

Сравнение использует независимые процессы, медианы и непараметрические интервалы с поправкой на множество метрик. Недостаточные данные дают `Inconclusive`. `compare --check`: 0 — пройдено, 1 — регрессия, 2 — неопределённость/недоступность/ошибка. `--filter` и `--metric` позволяют заранее выбрать проверяемое семейство. Изменённые контракты и окружение отклоняются; метрики с baseline=0 требуют абсолютного бюджета.

Для разных argv/env/cwd, fixtures и контрактов используйте `run --plan plan.json`; схема примера — [docs/example-plan.json](docs/example-plan.json). Таймауты, ошибки, отмена и недоступные измерения сохраняются явно. `status-final.json` — окончательный статус; `status.json` — начальная запись, её наличие само по себе не означает успех.

## Forma и дополнительные метрики

- `Recorder` принимает наблюдения из собственного event loop приложения; сбор записей можно вынести за измеряемую фазу.
- `TrackingAllocator<System>` подключается явно и считает Rust allocations/reallocations/live/lifetime peak; native/driver allocations в него не входят.
- `import-forma` импортирует завершённые старые normal/paired результаты известной схемы Forma, сохраняя scope и отсутствующие значения.
- [Реальный offscreen-пример](integrations/forma/src/main.rs) использует renderer Forma и wgpu/Metal. Его зависимости изолированы от основного workspace. Сборка: `cargo build --release --offline --manifest-path bench/integrations/forma/Cargo.toml`; затем бинарник запускается runner с `--protocol`.

Интеграция сейчас ссылается на локальный исследовательский снимок `bench/research/private/forma/vector-ui`; для другого checkout исправьте path в её Cargo.toml. Интеграция включает static, hover, animation, scroll, resize и text; image явно Unsupported. Перед запуском нужно записать и просмотреть golden PNG, затем передать `--goldens DIR`. Измеряются submit/completed frames, allocations и geometry uploads с проверкой контрольных кадров. Оконный FPS и GPU timestamp duration этим примером не измеряются.

## Исследование и статус

[Проверки и ограничения](docs/VALIDATION.md) · [Roadmap](docs/ROADMAP.md) · [Архитектура](docs/ARCHITECTURE.md) · [Правила измерений](docs/MEASUREMENT.md)

Исследование охватило 779 записей; 614 репозиториев оставлены после скрининга. Это анализ документации с углублёнными выборочными просмотрами исходников ключевых движков, а не полный аудит или сравнительный запуск сотен библиотек.

[Итоги и методика](research/REPORT.md) · [Ключевые решения](research/FOCUSED.md) · [Потребности проектов, особенно Forma](docs/REQUIREMENTS.md) · [Реестр](research/REPOSITORIES.md) · [Решение по каждой записи](research/DECISIONS.tsv)

## Advanced workflows

The final twenty features are implemented with explicit capability limits: privacy policies,
process diagnostics and pilot planning, A/B/C, optional benchmark attributes, async/thread/pipeline
helpers, cold runs, allocation phases, Forma GPU/window measurement, argument matrices,
profiler replay, linked continuation, reversible retention and bounded revision searches.
See [FINAL20.md](docs/FINAL20.md) for runnable examples, validation evidence and limitations.
