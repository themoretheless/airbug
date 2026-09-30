# Построение отчётов

Команда работает с сохранёнными данными и не запускает измерения. Источник — `run.json`, каталог одного запуска или дерево экспериментов.

```sh
cargo airbug-bench report .airbug-bench/experiment --title "Forma: performance" -o .airbug-bench/report.html
cargo airbug-bench report .airbug-bench/experiment -o .airbug-bench/report.md
cargo airbug-bench report .airbug-bench/experiment -o .airbug-bench/report.json
cargo airbug-bench report .airbug-bench/new --baseline .airbug-bench/old --threshold 5 --alpha 0.05 -o .airbug-bench/comparison.html
```

Без `-o` выводится Markdown. Существующий выходной файл не перезаписывается.

HTML автономный: сводные карточки, поиск, фильтр по исходу, переходы к запускам, сортировка таблиц, раскрытие деталей и печать. Блок «Effect charts» показывает интервалы изменения (forest plot) и точечные оценки по метрикам; для кроссоверных запусков (`baseline` и `candidate` в одном `run.json`) добавляются диаграммы по независимым единицам — точки по процесс-парам, накопленная доля единиц ниже порога (когда этих единиц не меньше пяти) и дельта каждой пары. Все графики рисуются офлайновым SVG из `bench/src/viz.rs`: без JS-библиотек и внешних ресурсов, с доступными подписями. Подпись каждой диаграммы говорит, чего она не утверждает; отсутствие интервала показывается текстом, а не нулевым эффектом. Для читаемости отображается до 32 интервалов и до 8 диаграмм по независимым единицам на запуск и до 64 серий процессов на документ; таблицы и JSON сохраняют остальные данные.

JSON содержит документ со сводкой, сравнениями, проблемами и загруженными наблюдениями. Для исходных файлов указаны пути и SHA-256. Привязанные к хешу заметки добавляются к данным отчёта; SHA-256 относится к исходному файлу запуска.

## Осмотр состава запуска

`cargo airbug-bench list RUN` печатает идентификаторы кейсов (с `--filter SUBSTR` — подмножество), не запуская измерения. Флаг `--json` добавляет по каждому кейсу его метрики (id, единицу, scope, фазу, статистику, направление) и счётчики наблюдений (`observations`/`available`/`processes`) — удобно, чтобы узнать точные имена метрик для `gate`, `check` или `compare --metric`.

```sh
cargo airbug-bench list .airbug-bench/sort
cargo airbug-bench list .airbug-bench/sort --json --filter unstable
```

`cargo airbug-bench history` перечисляет запуски хранилища в хронологическом порядке; `--status` (например `complete`/`failed`, регистр не важен) и `--limit N` (только последние N) сужают список, `--json` даёт машинный вывод.

`cargo airbug-bench context A B` показывает различия окружения/provenance/контрактов двух запусков, `cargo airbug-bench trend --case C --metric M` — историю медиан по кейсу/метрике. У обоих есть `--json` для машинной обработки (`context` — список `{key,a,b}`, `trend` — точки `{id,revision,median,unit,context}`).

## Интерпретация

- Regression — обнаружена регрессия.
- Error — повреждённый, неудачный или незавершённый запуск, ошибка сопоставления.
- Unavailable — недоступные измерения.
- Inconclusive — недостаточно оснований для вывода.
- No comparison — измерения без пригодного сравнения.
- Diagnostic — запуск под профилировщиком, исключённый из сравнения.
- Passed comparison — сравнение прошло по заданному относительному порогу; это не проверка абсолютного бюджета.

Независимые эксперименты не объединяются в одну выборку. Поправка на множественные сравнения учитывает число запусков кандидата, вариантов и метрик. При добавлении запусков интервалы могут стать шире, а вывод — неопределённым.

Парные и многовариантные запуски сравниваются автоматически. Внешние коллекции baseline и candidate сопоставляются по точному относительному пути. Один baseline нельзя распространить на всю коллекцию. Отсутствующие соответствия видны как проблемы. Незавершённый каталог может принадлежать ещё работающему процессу.

Успешное создание отчёта не означает прохождение performance gate: команда сохраняет и отчёты с регрессиями. Для CI используйте `compare --check` и `check`.

## Ограничения

Обход пропускает служебные каталоги и символьные ссылки на каталоги. Лимиты: 256 запусков на источник, 64 MiB на файл, 128 MiB суммарных входных данных, глубина 12 и 20 000 посещённых каталогов. При превышении выберите более узкий источник. Отчёт включает уже сохранённые пути, контекст и заметки: перед передачей другим людям проверьте содержимое.

Проверки: интеграционные тесты охватывают ошибки, отсутствие baseline, экранирование и семейную неопределённость; модуль `viz` покрыт тестами на шкалы, тики, экранирование и детерминизм вывода; `node scripts/test-report-ui.mjs` проверяет поведение встроенных элементов управления на DOM-фикстуре, а `scripts/test-live-ui.py` — живые диаграммы и агрегат матрицы через HTTP. Это не визуальная проверка браузером.

## Динамическое построение диаграмм

Отчёт вставляется в страницу как SVG, и каждая диаграмма строится на глазах: метки появляются группами по порядку чтения, около двух секунд на фигуру. Механизм — один блок CSS (`viz::REVEAL_CSS`) и индексы задержки в разметке: `Plot::figure` оборачивает метки в группы не более 64 штук, а браузер анимирует их ключевыми кадрами. JavaScript, SMIL и вторые реализации шкал в браузере не участвуют, офлайн-режим и детеризм вывода сохраняются.

Страница без этого CSS показывает готовый кадр: `Plot::svg` и `Plot::inline` остаются статичными, поэтому живому интерфейсу CSS не выдаётся — его диаграммы обновляются раз в 1,5 секунды, и анимация никогда не завершилась бы. Печать и `prefers-reduced-motion` тоже принудительно рисуют финальный кадр. Стоимость — 22 байта на группу меток (её открывающий и закрывающий теги) и 689 байт самого CSS один раз на страницу; покадровый перерендер стоил бы килобайты на каждую диаграмму.

Второй способ движения — линии и полосы рисуются по собственной длине, а не появляются затуханием. Метка несёт `pathLength="100"` и класс `dv`, поэтому CSS оперирует долями длины и ему не нужно знать измеренную длину кривой; полоса рисуется от нулевой отметки наружу, так что растущая полоса читается как растущее значение. Стоимость — 28 байт на такую метку.

Обе страницы собраны из одного запуска в 13 процессов с одним кейсом и одной метрикой, из них же вырезанием меток движения получен статичный кадр. Страница запуска: 22 983 байта статичного кадра, движение стоит 2 407 байт (+10,5 %), из них 689 — CSS и 1 718 — 79 групп появления; обводимых меток на ней нет вовсе. Страница сравнения: 28 684 против 31 161 байта (+8,6 %), 81 группа и одна обводимая линия накопленной доли. Цену диктует расстановка групп, а не рисование по длине — и это тот параметр, который стоит ужесточать, когда диаграмм станет больше.

## Тепловые карты

`charts::heatmap` рисует матрицу произвольной формы в трёх шкалах: общая для всей матрицы, локальная для строки (когда метрики и единицы разные) и знаковая вокруг нуля. Пропуск остаётся пустой ячейкой — пустое не равно нулю; переполнение ограничивается `max_cells`, недописанные строки называются в подписи. Каждая ячейка несёт всплывающую подпись «строка · столбец = значение», поэтому сокращённые заголовки столбцов не теряют смысл.

В отчётах две такие матрицы. «Change by case and metric» показывает точечные оценки изменения по всем кейсам и метрикам на знаковой шкале и рисуется только вместе с интервалами — тепловая карта без интервала была бы чрезмерным утверждением. «Median per process» показывает медиану каждого процесса по строкам «кейс / метрика / вариант» на шкале своей строки: она годится для поиска выбившегося процесса, а не для сравнения уровней между строками.

## Песочница диаграмм

```sh
cargo run -p airbug-bench --example viz_gallery
open target/viz-playground.html
```

Страница собирает все диаграммы `viz` из синтетических данных: над каждой фигурой стоит вызов и измеренный размер разметки, поэтому видно, сколько отчёт платит за конкретную диаграмму. Фигуры берутся теми же функциями, что и отчёт, и лежат на том же CSS, включая `REVEAL_CSS`, — это не макет, а реальный вывод рендера.

Переключатели движения (финальный кадр, пауза, замедление, без расстановки по времени, скрыть оговорки) сделаны чистым CSS поверх уже существующих классов через `:has()`. Рендерер о песочнице не знает: в `viz` не добавлено ни одной опции для неё. Печать и перезагрузка страницы работают как проверка финального кадра и повторного проигрывания.

Накопленная доля (`charts::ecdf`) тоже стоит дешевле уже имеющейся диаграммы: на 23 единицы в двух вариантах её разметка — 4,0 KiB против 5,0 KiB у диаграммы точек, потому что у неё нет подписи на каждую точку.

Страница намеренно включает крайние случаи: нечисловые значения, враждебные подписи с тегами, пустые и вырожденные наборы, длинные и кириллические подписи, переполнение бюджета ячеек.

## Веб-интерфейс

```sh
cargo airbug-bench serve .bench --port 8787
```

Откройте адрес, напечатанный сервером. Доступен каталог запусков с поиском, общий отчёт, выбор отдельного запуска и baseline, настройка порога, просмотр и экспорт JSON/Markdown. Для большой коллекции можно передать более узкую папку. Обновление списка выполняется кнопкой; отчёт строится кнопкой или выбором запуска.

Сервер слушает только `127.0.0.1`, адрес содержит новый ключ при каждом старте. Он читает сохранённые результаты и не запускает бенчмарки. Запросы обслуживаются последовательно, действуют ограничения размера и обхода из раздела выше. Остановка — Ctrl+C.

## Интерфейс во время бенчмарка

`cargo airbug-bench run ... -o .airbug-bench/session` автоматически открывает интерфейс при запуске из интерактивного терминала. Пока идут измерения UI переключается в live-режим: на экране только бренд и прогресс процессов, построение отчёта недоступно. После финализации интерфейс возвращается к каталогу запусков, сам строит отчёт и остаётся доступным до Ctrl+C.

Под счётчиком сервер рисует две живые диаграммы: полосы времени по вариантам и тренд завершённых процессов (маршрут `api/live-charts`). Источник — история тех же опросов `progress.json`, что делает браузер, поэтому измерения не получают дополнительной работы; границы полос приблизительны, это наглядность, а не доказательство. В report-режиме живые диаграммы скрываются.

- `--ui` включает интерфейс и при перенаправленном выводе.
- `--no-ui` отключает его, например для CI или измерений без браузера.
- `--no-open` оставляет только ссылку без автоматического открытия браузера.
- `--dry-run` не запускает сервер и не создаёт результаты.

`cargo airbug-bench matrix --plan PLAN -o DIR` принимает те же три флага и показывает одну страницу на всю сессию: счётчик суммирует процессы всех ячеек, а полоса диаграммы подписывается комбинацией осей (`--cpu 2 · candidate`). Ячейки при этом остаются обычными запусками в `DIR/0`, `DIR/1`, … — сервер читает их `progress.json` и `status-final.json` сам, измерительный путь не меняется. Отчёт открывается только когда все ячейки закончены; упавшая или прерванная ячейка видна как итоговый статус агрегата.

В каталоге запуска автоматически сохраняются `run.json`, `report.html`, `report.json`, `report.md`, логи и финальный статус. Ошибки/отмена после начала измерений также сохраняются с соответствующим статусом. После закрытия runner HTML можно открыть самостоятельно либо запустить `serve` на сохранённом каталоге. Ошибки до создания запуска могут не иметь артефактов.

Счётчик обновляется между измерениями; UI опрашивает маленький статус раз в 1,5 секунды. Браузер и сервер всё равно потребляют ресурсы системы: для строгих измерений используйте `--no-ui`.

### Relative bootstrap distributions from saved runs

For two independent runs, export relative mean and median changes without rerunning
workloads:

```sh
cargo airbug-bench compare baseline candidate --json --relative-distributions \
  --hypothesis-resamples 10000 --hypothesis-seed 7 --alpha 0.05
```

This opt-in output is an object with `comparisons` (the ordinary comparison rows)
and `relative` (bootstrap reports). Without the flag, JSON remains an array of
comparison rows. `--hypothesis-distribution` can additionally retain Welch null
draws in `comparisons`.

Relative draws use independent resampling of each run's process medians, after
normalizing batch totals by operation counts. A process with more batches does
not receive more weight. Each report records process counts, seed, resample count,
and confidence level (`1 - alpha`). These are per-statistic percentile intervals,
not family-adjusted regression decisions; `--check` still uses the ordinary
comparison decisions. Undefined percentage ratios remain JSON `null`; an interval
is unavailable if any draw has an undefined ratio. Missing or insufficient process
observations also produce explicit unavailable reports. This flag requires both a
candidate run and `--json`; paired single-run relative export is not supported.

`compare baseline candidate --html` also includes relative mean/median density
charts. Blue shading is the bootstrap confidence interval; amber shading marks
`[-threshold, +threshold]` percent (the existing `--threshold`, default 5). A gray
line marks zero and a red line marks the original point estimate. Positive changes
mean an increase in the measured quantity, which is not necessarily a regression
for every metric. Charts use the same `--hypothesis-resamples`,
`--hypothesis-seed`, and `--alpha` controls as relative JSON export. Undefined
ratios suppress the complete density chart and show an explicit reason; finite
subsets are never silently plotted as the full population. Constant distributions
are marked as point masses. These charts do not change `--check` decisions.

The unified `report candidate --baseline baseline --output report.html` includes
relative mean/median charts using its confidence and threshold settings. Direct
Cargo suites with a named `--baseline`, `--resamples N`, and `--output DIR` include
the same charts in `regression-comparison.html` and save retained draws in
`relative-distributions.json`. Each case uses its own baseline snapshot and
resolved practical-change threshold; the bootstrap confidence, seed, and sample
count come from the requested bootstrap configuration. Comparison export happens
before baseline promotion. Incompatible or absent case baselines remain explicitly
unavailable and are not synthesized into relative distributions.

### Violin summaries

Bootstrap HTML reports now include violin summaries. They use the full retained
measurement population, including outliers. Multi-process runs contribute one
median per process; single-process runs show normalized observations and make no
claim of process independence. Missing observations suppress that population and
retain the reason in the report. Compatible metric contracts and sampling units
share a chart; legacy JSON without metric contracts remains separated by row.

```sh
cargo airbug-bench analyze saved-run --format html --summary-scale logarithmic \
  --output analysis.html
```

The default summary scale is linear. `--summary-scale` on `analyze` requires HTML
output. This setting controls violin summaries; other diagnostic plots keep their
own axes. Direct Cargo bootstrap `estimates.html` includes violin summaries, linear by default.
Each violin's width is normalized to its own peak density, not its sample count.

Unified experiment HTML includes candidate violin summaries with or without a
baseline. `--summary-scale logarithmic` controls violin axes independently of
`--summary-parameter`. Direct Cargo passes the same summary scale
to `estimates.html`; selecting a summary scale enables bootstrap reporting even
without `--resamples`. It requires `--output` and a measurement or loaded baseline. Nonpositive populations
on log axes are explicitly unavailable; no positive-only subset is substituted.

Rust registration can set suite, group and individual case scales:

```rust
use airbug_bench::{Suite, viz::charts::AxisScale};
let mut suite = Suite::new("sorting");
suite.summary_scale(AxisScale::Logarithmic);
suite.with_summary_scale_defaults(AxisScale::Linear, |suite| {
    suite.group("small", |suite| {
        suite.bench("case", || std::hint::black_box(1));
    });
});
```

`summary_scale_case` sets an explicit scale on the last registered case. Nearest
nested group defaults win; an explicit `Linear` overrides inherited
`Logarithmic`. Defaults also wrap registration functions imported from other
modules or crates. `summary_scales` resolves selected cases without executing
workloads. Direct Cargo bootstrap HTML uses these settings when requested with
`--resamples` and `--output`; an explicit CLI `--summary-scale` overrides all
registered settings. Resolved presentation defaults are saved in `run.json` provenance under
`airbug.presentation.summary_scales.v1`, separately from workload compatibility
contracts. `analyze --format html` and unified `report` HTML restore them; an explicit CLI scale wins.
Legacy runs without this metadata retain linear defaults. Invalid stored scale
values produce an error. The process runner propagates preferences agreed across workers. If workers
disagree (including a legacy worker with implicit linear scale), it keeps the
measurements, records a note, and leaves that case at the default scale unless
explicitly overridden. Original worker provenance remains available.

The same defaults are available through attributes:

```rust
#[airbug_bench::suite(summary_scale = "logarithmic")]
mod benchmarks {
    #[group(summary_scale = "linear")]
    mod small {
        #[bench]
        fn operation() { std::hint::black_box(1); }
    }
    #[bench(summary_scale = "linear")]
    fn explicit_override() { std::hint::black_box(2); }
}
```

`summary_scale` accepts only `linear` or `logarithmic`. Repeated options and invalid
values fail at compilation. Inline and imported groups use the same nearest-default
precedence; a case attribute overrides both. Registration and scale inspection do
not execute benchmark functions.

### Declared input families

After registering a case, call `suite.summary_family("sort")` and
`suite.parameter("size", "1024")` to include it in a numeric input family.
Family names are scoped to the current group. The resolved mapping is saved in
`airbug.presentation.summary_families.v1` provenance, outside workload contracts.
`--summary-parameter size` joins estimates from the same declared family and
variant, ordered by numeric input. Metric contracts remain separate. Unassigned
cases stay individual points. Duplicate input values in a family produce an
explicit unavailable line summary instead of averaging different cases.

The process runner preserves family identities agreed across all workers. A
conflicting or missing identity leaves that case as separate points and adds a
report note; measurements remain usable. Attributes support `summary_family = "name"` on a bench, group or suite.
A child declaration overrides inherited defaults, including imported groups.
For scalar `args`, `--summary-parameter arg` uses the generated numeric argument
labels as inputs. Choose distinct family names for different functions that
share the same input values; otherwise the report rejects duplicate inputs.

Numeric input summaries also include throughput lines for declared work counters.
They use median batch throughput within each process, then the median across
processes, so unequal batch counts do not change process weights. Bytes use fixed
MiB/s across cases; other counter units stay separate. Work totals reuse the same
calculation as throughput tables. Charts require positive, complete `wall`
`batch_total` observations in nanoseconds; unsupported or incomplete populations
are omitted with a diagnostic. On logarithmic axes, zero work rates cannot form
a line and produce an explicit unavailable chart.

`bench/examples/input_summary.rs` generates deterministic time and throughput
summary figures without running workloads.

### HTML без графиков

`cargo airbug-bench report RUN --baseline BASELINE --no-plots --output report.html`
сохраняет таблицы и результаты сравнения, пропуская bootstrap-анализ для графиков
и генерацию SVG. Флаг доступен для HTML-команды `report`; он несовместим с
`--summary-parameter`, `--summary-scale` и `--summary-estimator`.

Тот же режим доступен для сохранённых измерений:

```sh
cargo airbug-bench analyze RUN --format html --no-plots --output estimates.html
cargo airbug-bench compare BASELINE CANDIDATE --html --no-plots --output comparison.html
```

`analyze` продолжает рассчитывать численные bootstrap-оценки. `compare` сохраняет
обычные статистические решения, но не удерживает массивы нулевых распределений
для графиков. `--no-plots` требует HTML; в `analyze` он несовместим с
`--summary-scale`.
