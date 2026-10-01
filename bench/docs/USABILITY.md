# Работа с бенчмарками

Начните с `cargo airbug-bench init` и `cargo bench` в существующем package.
Первый запуск сохраняет отчёт, следующие автоматически сравниваются с предыдущим успешным запуском.

Для повседневной работы:

```sh
cargo bench sort              # только совпадающие кейсы
cargo bench -- --quick        # короткий пробный замер
cargo bench -- --help         # основные команды
cargo bench -- --help-all     # все настройки
```

`--quick` использует адаптивную остановку замера. Профиль `--profile quick`
отдельно задаёт небольшой бюджет выборки. Пробный замер помогает проверить
изменение, но не заменяет полноценное измерение перед выводами о производительности.

## Подключение нового проекта

Отдельный проект не нужен. В существующем package достаточно `cargo airbug-bench init`,
затем `cargo bench`. `init` находит ближайший Cargo package даже из вложенного
каталога и печатает команды для измерения, однократной проверки и списка кейсов.
Явный `--manifest-path` всегда выбирает указанный файл; ошибка в пути не меняет
другой проект.

Установите CLI: `cargo install --path bench/cli --offline` из корня monorepo. Либо используйте абсолютный путь к `target/release/cargo-airbug-bench` после release-сборки. Локальный alias работает только внутри этого workspace.

```sh
cargo airbug-bench init --manifest-path /path/to/project/Cargo.toml
cargo bench --manifest-path /path/to/project/Cargo.toml --bench bench
```

`init` добавляет dev-dependency на локальную библиотеку, `[[bench]]` с `harness=false`, регистрацию target, короткий пример с `#[bench]`, подготовкой данных вне измерения и тремя размерами. Дополнительный JSON-конфиг не создаётся. `init --name sort` добавляет следующий target в `benches/sort.rs`. TOML-комментарии сохраняются. Существующие файлы не заменяются. Для virtual workspace нужно выбрать manifest одного package. Если исходники библиотеки перемещены, передайте `--library-path /path/to/airbug/bench`.

`discover` показывает все Cargo bench targets workspace и их регистрацию. `airbug-bench` сначала собирает **все выбранные targets**, затем измеряет их последовательно. Для существующего bench target добавьте:

```toml
[package.metadata.airbug_bench]
targets = ["my_benchmark"]
```

Не отмечайте обычный libtest/Criterion target: регистрация означает, что executable поддерживает `BENCH_RESULT` protocol. Точный выбор: `--target package/target`, можно повторять. Результаты отдельных targets находятся в пронумерованных подкаталогах; корень содержит `targets.json`. Это коллекция запусков, а не единый Run: для baseline/report/gate выберите подкаталог target.

Настройки worker передаются после `--`, например `-- --samples 8 --warmup-ms 10 --sample-ms 1`. Прогресс идёт в stderr: target, номер процесса, baseline/candidate, оценка оставшегося времени; для долгого процесса показывается последняя фаза worker. ETA оценивается по завершённым процессам, не гарантируется при разных workloads. Диагностика supervisor может немного влиять на host load.

## Автодополнение оболочки

```sh
# bash: подключить на текущую сессию
source <(cargo airbug-bench completions bash)
# zsh: сохранить в каталог из $fpath
cargo airbug-bench completions zsh > ~/.zfunc/_cargo-airbug-bench
# поддерживаются также fish, powershell и elvish
cargo airbug-bench completions fish > ~/.config/fish/completions/cargo-airbug-bench.fish
```

`completions SHELL` печатает скрипт автодополнения в stdout и не изменяет конфигурацию оболочки. Скрипт дополняет установленный бинарник `cargo-airbug-bench`; неизвестное имя оболочки отклоняется. Закрытый downstream-канал (например, `| head`) не считается ошибкой.

## Статистика без дополнительных флагов

Обычный `cargo bench` рассчитывает доверительные интервалы после измерения:
по умолчанию 95%, 10 000 bootstrap-перевыборок, воспроизводимый seed 0.
Настройки группы и кейса сохраняют приоритет над значениями по умолчанию;
явные CLI-настройки меняют только указанные поля. Анализ выполняется вне
измеряемого участка. `--no-bootstrap` отключает его, включая настройки группы;
совместное указание этого флага и явных CLI-настроек bootstrap отклоняется.
Проверка кейсов и профилирование не получают статистический анализ автоматически.
При недостаточной выборке отчёт показывает недоступность оценки.

## Именованные baseline

```sh
cargo airbug-bench baseline save main .airbug-bench/first/0-my-package-bench
cargo airbug-bench baseline list
cargo airbug-bench compare @main .airbug-bench/second/0-my-package-bench --check
cargo airbug-bench report @main -o .airbug-bench/main.html
```

Имя разрешает буквы ASCII, цифры, `-` и `_`. Сохранение создаёт ссылку на canonical path и SHA-256 файла завершённого benchmark; данные не копируются. По умолчанию повторное сохранение имени запрещено. `baseline save main NEW_RUN --replace` атомарно заменяет ссылку, сохраняя старый исходный run. `baseline save main NEW_RUN --retain` оставляет существующую проверенную ссылку; если имени нет, создаёт её. При сохранении существующей ссылки новый источник не читается и может отсутствовать. Флаги `--replace` и `--retain` взаимоисключающие. Повреждённая существующая ссылка при `--retain` вызывает ошибку; исправить её можно явным `--replace`. Smoke-запуски не принимаются как baseline. Удаление исходного run сломает ссылку, изменение его содержимого обнаруживается. Хранилище по умолчанию `.bench` относительно текущего каталога; общий каталог задаётся глобальным `--store PATH`.

## Матрицы и проверка правильности

```rust
use airbug_bench::{DropPolicy, Suite};
let mut suite = Suite::new("collections");
suite.matrix("sort", &[("size", &["32", "512"]), ("order", &["forward", "reverse"])],
    |suite, id, params| {
        let size: u64 = params["size"].parse().unwrap();
        let reverse = params["order"] == "reverse";
        suite.bench_checked(id,
            move || {
                let mut v: Vec<_> = (0..size).collect();
                if reverse { v.reverse(); }
                v
            },
            |v| v.sort_unstable(),
            |v, _output| {
                if v.windows(2).all(|w| w[0] <= w[1]) { Ok(()) }
                else { Err(airbug_bench::error("unordered result")) }
            }, DropPolicy::InsideTiming);
    })?;
```

Матрица создаёт декартово произведение, максимум 4096 комбинаций. ID содержит отсортированные параметры с экранированием разделителей; параметры также сохраняются в contract. Замыкание регистрации вызывается сразу, workload остаётся ленивым. Дубли осей/значений отклоняются.

`bench_checked` выполняет одну отдельную операцию на свежем input до калибровки и ещё одну после измерений. Проверка получает изменённый input и output и находится вне timing. Ошибка останавливает запуск. Это выборочная проверка, не проверка каждой измеренной операции. Общая mutable closure-state сохраняется, поэтому для stateful workloads учитывайте две дополнительные операции. Внутренние заимствования для checked API берутся до timing, а не на каждой измеряемой операции.

## Бюджеты

```json
{
  "budgets": [
    {"case": "forma/static", "metric": "geometry.uploads", "unit": "calls", "max": 0},
    {"case": "forma/static", "metric": "frame.completed", "unit": "ns", "max_regression_percent": 5}
  ]
}
```

```sh
cargo airbug-bench gate .airbug-bench/candidate --config budgets.json --baseline @main
```

`case` и `metric` совпадают **точно**. `unit` обязательна. `min`/`max` проверяются на каждом candidate observation, batch totals нормализуются на число операций. `max_regression_percent` использует сравнительный анализ независимых процессов; без `--baseline` требуется paired run. Для нескольких относительных бюджетов применяется общая поправка Bonferroni. Budget на нулевом baseline задавайте абсолютным пределом.

Коды: 0 — все прошли, 1 — хотя бы один провален, 2 — недоступность/неопределённость/ошибка без установленного провала. Отсутствующая метрика, несовпадение единиц и пустая конфигурация не дают успешный результат. Для проверки бюджетов сохраните отдельный конфиг с пределами для своего workload. [Бюджеты Forma](../integrations/forma/budgets.json) проверяют отсутствие geometry uploads в static/hover/animation.

## HTML-отчёт

`report RUN -o report.html` создаёт автономную таблицу с поиском строк, сортировкой колонок, раскрываемыми case contracts, окружением и печатью/PDF через браузер. Внешних скриптов/шрифтов нет. Исходные данные остаются в `run.json`. Значения и пользовательские строки экранируются; HTML из metadata не исполняется.

## Сценарии Forma и контроль кадров

```sh
cargo build --release --manifest-path bench/integrations/forma/Cargo.toml --offline
bench/integrations/forma/target/release/bench-forma-example --list
bench/integrations/forma/target/release/bench-forma-example \
  --record-goldens .airbug-bench/my-forma-goldens
# Просмотрите созданные PNG как эталоны, затем:
cargo airbug-bench run --program bench/integrations/forma/target/release/bench-forma-example \
  --protocol --repetitions 3 -o .airbug-bench/forma-checked -- \
  --goldens .airbug-bench/my-forma-goldens --json
cargo airbug-bench gate .airbug-bench/forma-checked --config bench/integrations/forma/budgets.json
```

Golden capture не запускает benchmark protocol и никогда не перезаписывает каталог. Запуск измерения требует эталоны; автоматического принятия нового изображения нет. RGBA8 сохраняется в версионированном `.rbimg`, PNG предназначен для просмотра. Эталоны берутся с известной сборки; их запись сама по себе не доказывает правильность renderer.

| Сценарий | Полезная работа |
|---|---|
| static | Принудительный draw неизменной кнопки; повторные geometry uploads запрещены |
| hover | Установка hover и завершение перехода цвета; update включён в timing |
| animation | Детерминированный tick 16 ms с переключением hover каждые 12 кадров |
| scroll | Сброс и изменение scroll offset внутри clipped viewport |
| resize | Переключение заранее созданных targets 800×400 / 640×320; без создания текстур и оконного resize |
| text | Статический draw сцены с другим текстом; не отдельный font shaping benchmark |
| image | Unsupported: в native snapshot нет Image display-list/GPU primitive |

Каждый реальный сценарий: 20 static warmup кадров, затем 96 кадров. Отдельный предварительный проход сравнивает checkpoints 0 и 95. Новый экземпляр модели и renderer измеряет ту же последовательность; после него проверяется кадр 95. Readback и сравнение pixels находятся вне timing и allocator snapshots. Точная проверка по умолчанию; допустимы явно заданные `--channel-tolerance` (0..255) и `--max-changed-percent` (0..100). Порог относится к доле pixels, где хотя бы один RGBA channel превышает tolerance.

Контракт хранит SHA-256 исходной сцены/component и эталонов, параметры проверки, adapter, viewport и scope. Это контроль checkpoints, а не всех кадров анимации. Нельзя использовать offscreen completed-time как оконный FPS.

### Baseline по кейсам из Cargo

`cargo airbug-bench report @main --baseline @candidate` читает baseline по кейсам,
сохраняя окружение и происхождение каждого результата. Для отчёта кейсы
сопоставляются по стабильным идентификаторам независимо от исходных запусков.
Проверенные представления кешируются в `<store>/baseline-views/`; они исключены
из истории запусков и автоматического выбора `last`.

Команды runner CLI, рассчитанные на один запуск, принимают такой алиас, если
выбранные кейсы происходят из одного артефакта. При нескольких источниках они
возвращают явную ошибку; общий отчёт доступен через `report`, повторный анализ —
через `cargo bench -- --load-baseline NAME`.

При сохранении поверх baseline, созданного из Cargo, runner CLI обновляет
существующий manifest: `baseline save NAME RUN --replace` заменяет кейсы из RUN
и сохраняет остальные. `--retain` оставляет существующий проверенный baseline
без чтения нового источника. Имена старого формата продолжают использовать ссылки
на исходный файл; команда не создаёт старую ссылку поверх manifest по кейсам.
Публикация обоих форматов согласована с retention общей блокировкой хранилища.

## Выгрузка исходных измерений

```sh
cargo airbug-bench export PATH_TO_RUN --format csv -o measurements.csv
```

CSV содержит исходные значения, число операций и единицы измерения. Последние
три колонки — JSON: `case_contract` с параметрами и объявленными счётчиками,
`work_totals` с фактической работой и `worker_work_totals` с работой каждого worker.
Целочисленные счётчики сохраняются десятичными строками без округления.
`formatted-csv` отдельно выгружает сохранённые пользовательские единицы.
Для итогов батча `value` нормализован на операцию (`normalized_per_operation=true`),
а `raw_value`, `raw_unit` и `operations` сохраняют исходный итог, единицы и число
операций. Также включены `pair` и три JSON-колонки с контекстом. Файл
`formatted.csv` от обычного `cargo bench` имеет тот же расширенный формат.
Старые сохранённые запуски читаются без миграции; изменённые исходные измерения
с устаревшим форматированием отклоняются.

## Результаты многопоточного бенча

```sh
cargo bench -- --threads 4 --output-format tree
```

Дерево отдельно показывает общее время и `worker-slot wall`: минимум, максимум,
медиану и среднее времени worker на операцию. Для одного слота сначала суммируются
его волны в пределах выборки. Строка `worker-slot throughput` связывает счётчики
работы с этими временами; её `aggregate` делит суммарную работу на суммарное время
worker. Это отличается от throughput всего параллельного запуска.

При включённом учёте аллокаций строки `worker-wave` показывают значения отдельных
волн worker и идентификаторы самых быстрых/медленных измерений. `wave peak` и
`sample peak` подписаны отдельно. Worker в одном процессе не считаются независимыми
повторами процесса; эти данные не являются распределением задержки отдельных операций.

## Сравнение throughput пользовательской метрики

Если у кейса зарегистрирован `ValueFormatter` и фиксированный счётчик работы,
обычное сравнение с предыдущим запуском и `--baseline NAME` теперь показывают
процентное изменение throughput с доверительным интервалом. Дополнительный флаг
не нужен. Форматтер получает пары средних/медиан исходной метрики на общей шкале;
интервал рассчитывается по преобразованным bootstrap-выборкам. Нелинейный
форматтер учитывается явно. Это отличается от среднего отдельных скоростей.

Метод расчёта сохраняется рядом с оценкой в JSON и описывается в отчёте. Нулевой
объём работы и недостаточная выборка дают недоступную оценку. Динамические
счётчики требуют отдельного парного расчёта и пока не поддержаны в этом сравнении.

### Время жизни рабочих потоков

При `threads > 1` Airbug переиспользует пул с постоянным номером каждого
worker. Потоки сохраняются между калибровкой, прогревом, волнами и выборками,
а также повторными запусками на том же вызывающем потоке с тем же числом workers.
Смена числа workers пересоздаёт пул. При `threads = 1` работа выполняется
на вызывающем потоке.

Свежие входы по-прежнему готовятся для каждой операции; пул сохраняет локальное
состояние потока, но не сами входы. Подготовка и уничтожение worker-local входов
происходят на исходном worker; уничтожение отложенных значений начинается после
остановки всех таймеров волны. По умолчанию волна содержит до 64 операций на поток; `batch` меняет
этот размер.

Исполнитель использует обычные потоки ОС и не устанавливает контекст Rayon.
Вложенные Rayon-операции используют собственный явно заданный или глобальный
пул приложения. В результатах это отмечено контрактом
`threads.executor = os-pool-v1`.
В отличие от Divan, при нескольких workers вызывающий поток не участвует в
измеряемой работе.

### Размер batch в потоковых бенчмарках

`batch` можно сочетать с `threads` на bench или группе, содержащей функции:

```rust
#[airbug_bench::suite(threads = 2, samples = 20)]
mod benches {
    #[bench(batch = airbug_bench::BatchPolicy::Iterations(16.try_into().unwrap()))]
    fn work() -> Vec<u8> { vec![0; 128] }
}
```

Политика задаёт число операций каждого worker в одной волне. Остаток выполняется
отдельной короткой волной. `Batches(1.try_into().unwrap())` выполняет всю выборку
одной волной; `PerIteration` оставляет одну операцию. `SmallInput`, `LargeInput`,
`Iterations` и `Batches` имеют тот же смысл, что в последовательных бенчмарках.
Подготовка входов остаётся вне таймера; `drop_output` управляет уничтожением выходов.

Поддержаны sync/async, подготовка входов на worker или вызывающем потоке и учёт
аллокаций. Для ручной регистрации используйте `Suite::with_batch_defaults`
вокруг вызовов потоковых методов с подготовкой входов. Политика сохраняется
в `threads.batch_policy`, в том числе в `--dry-run`.

`batch` наследуется и через импортированные группы. Настройка конкретного bench
переопределяет родительскую. При одном worker локальные значения без `Send`
сохраняют ту же политику подготовки входов и удержания выходов.
Размер batch ограничивает число одновременно удерживаемых значений, а не память,
которую эти значения сами выделяют. Большие batch могут потребовать много памяти.

### Нелинейное отображение статистик

Зарегистрированный `ValueFormatter` автоматически применяется и к bootstrap-отчёту
обычного `cargo bench`. Линейное преобразование меняет единицы статистик.
Возрастающее нелинейное преобразование или смещение добавляет отдельную таблицу
`formatter(исходная статистика)`; исходные оценки, единицы и классификация выбросов
сохраняются. Например, при `scale_values: v → v²` строка `formatter(mean)` означает
квадрат исходного среднего, а не среднее квадратов наблюдений.

```sh
cargo bench --bench mybench -- --bootstrap-distributions --output results
```

`formatted-estimates.json` сохраняет представление, а `formatted-estimates.html`
показывает его рядом с исходными статистиками. Стандартная ошибка преобразованной
оценки считается по преобразованным bootstrap-выборкам. Без
`--bootstrap-distributions` она недоступна; преобразовывать исходную стандартную
ошибку тем же formatter математически неверно.

Регрессионные графики преобразуют всю модель и границы интервала. Сохранённый
JSON можно загрузить как `bootstrap::Report` и передать в `bootstrap::html` без
экземпляра formatter. Изменение исходных оценок или настроек делает сохранённое
представление недействительным. `--no-plots` сохраняет таблицы, `--no-html` — JSON.

Преобразование должно возвращать конечные значения и сохранять порядок на всём
отображаемом диапазоне. Это проверяется на переданных значениях и дополнительных
точках; между ними монотонность остаётся контрактом пользовательской функции.
Кривые регрессии рисуются по сетке с добавлением всех измеренных значений X.

При `--baseline NAME` файл `regression-comparison.html` также содержит
форматированные регрессионные кривые baseline и нового запуска на общей шкале.
Для регрессии нужны разные числа операций в выборках, например `--sampling linear`.
Одинаковые номера процессов по обе стороны остаются отдельными сериями;
исходные fit и доверительные интервалы не пересчитываются в преобразованных единицах.

Сравнение с именованным baseline сохраняет форматированные кривые в `run.json`.
Их можно восстановить без запуска benchmark и без кода formatter:

```sh
cargo airbug-bench compare before/run.json after/run.json --html --output comparison.html
cargo airbug-bench report after/run.json --baseline before/run.json --output report.html
```

Сохранённое представление относится к конкретным исходным значениям и настройкам
bootstrap. Для другой базы или настроек отчёт сообщает, почему график недоступен.
Изменённые исходные значения candidate или повреждённые координаты приводят к
ошибке; существующий выходной файл остаётся на месте.

То же сохранение работает для автоматического сравнения с предыдущим запуском.
Основной `report.html` показывает форматированные регрессионные кривые, а
`run.json` в выходном каталоге и архиве истории содержит одинаковые снимки.
Их можно использовать в отдельной команде `compare` после завершения benchmark.
Если formatter завершился ошибкой, частично подготовленный снимок не публикуется
и предыдущая запись истории остаётся базой следующего запуска.

### Zero time budgets

`max_time = 0` (or `--max-time-ms 0`) disables a case before setup or work, including smoke mode. A positive override enables it again.

With `exclude_external_time`, a workload that reports zero duration advances the time budget by 1 ns per call. Recorded durations stay zero; the report explains the accounting floor. This prevents an unchanging minimum-time budget, but a very large minimum can still require many calls. Maximum time takes precedence over minimum time.

### Statistical settings

`--confidence-level` controls absolute bootstrap intervals. Changing it keeps the same seeded draws, point estimates and standard errors; only interval endpoints change. `--resamples` controls the number of bootstrap draws. One draw is allowed, but cannot estimate a standard error.

For baseline comparisons, `--significance-level` and `--noise-threshold-percent` answer separate questions: statistical evidence of change and its practical size. The native default practical margin is 5%; use `--noise-threshold-percent 1` for a 1% margin. Confidence/significance must be strictly between 0 and 1; the practical margin must be finite and nonnegative.

### Console output

Use `cargo bench -- --quiet` for one median result per metric, or `--verbose` to include environment and measurement contracts. `--output-format tree --quiet` keeps the tree and omits the statistical detail. Quiet mode still prints results and report paths.

`--json` keeps machine records regardless of console detail. `--color auto` uses color only on a terminal without a nonempty `NO_COLOR`; `--color always` explicitly overrides that environment setting. JSON, listing and saved reports stay plain.

These settings belong to the direct benchmark harness. The separate `cargo airbug-bench` process runner uses command-specific output and live progress; it does not expose global quiet/verbose/color switches.

### Selecting benchmarks

A plain filter is a substring: `cargo bench -- sum`. Use `cargo bench -- '^collections/sum$' --regex` for a regular expression, or `cargo bench -- collections/sum --exact` for a complete identity. Multiple positive filters select their union; exclusions are applied afterwards.

Airbug registers cases before running the selected set. Shared borrowed inputs do not require cloning; use setup when each operation needs a fresh input. Listing and dry-run do not execute benchmark bodies or setup.

Statistical display also accepts monotone decreasing transforms such as `100 - value`: interval endpoints reverse, and standard error is computed from transformed bootstrap draws. Raw statistics stay unchanged. This also applies to formatted regression curves: the displayed confidence band reverses its endpoints while the original fit remains unchanged.

`--output` writes a new result directory. If that path already exists, Airbug now reports the conflict before running setup or benchmark work; choose a fresh path to preserve earlier results. `--dry-run` can still preview the same command without writing anything.

### Sample count and warmup

Airbug defaults to 30 samples and 50 ms of warmup per case. Criterion 0.8.2 defaults to 100 samples and 3 seconds of warmup. For a longer run, set these explicitly: `cargo bench -- --samples 100 --warmup-ms 3000`. Matching these two settings alone does not make the libraries' sampling policies identical.

`--warmup-ms 0` skips warmup; calibration remains enabled unless an iteration count is fixed. `--samples 0` disables the case. Calibration and warmup calls do not appear among collected observations or their throughput totals.

Criterion 0.8.2's `without_plots()` removes its HTML reporter. Use Airbug's `--no-html` for that artifact policy. Airbug additionally supports `--no-plots`, which retains numerical HTML reports and skips SVG graphs.

Named-baseline HTML comparisons now include baseline/candidate bootstrap distributions in the registered formatter's units. Both sides use one formatter call to choose a common scale. With only one collected sample, the report explains that the transformed statistical comparison is unavailable. `--no-plots` and `--no-html` skip these graphs. Automatic history also adds these overlays to its main report. The shared-scale distributions are saved in the comparison snapshot, so standalone `compare --html` and `report --baseline` restore them without loading formatter code. A changed source or different baseline/settings triggers the same checks as saved regression graphs.

For a nonmonotone statistical formatter, such as `(value - center)^2`, transformed interval endpoints do not describe the entire transformation. With retained bootstrap draws, statistical displays instead use percentiles of the transformed draws and identify this method in JSON (`interval_method`) and tables. If draws were not retained, Airbug reproduces them from the report’s observations, seed and settings and verifies that the original estimates match. This does not rerun the benchmark. Older reports without the required source data must be regenerated with `--bootstrap-distributions`. For nonmonotone regression bands, implement the optional `ValueFormatter::scale_intervals` method. Return `Some(FormattedIntervals { bounds, unit })`, with one finite `[minimum, maximum]` image for each supplied closed interval, using the same display unit and typical value as `scale_values`. Include interior extrema: `(value - center)^2` has minimum zero whenever the interval contains `center`. Airbug checks the shape, unit and containment of transformed endpoints; the formatter is responsible for enclosing every interior value. Monotone formatters can keep the default `None`. Without interval bounds, a nonmonotone regression transform still returns an error.

When `--output-format tree` writes to a terminal, deeply nested rows cap their indentation to about one third of the terminal width. A label such as `[64]` records the original nesting depth. Names and result values remain complete; output redirected to a file or pipe keeps the full indentation.
