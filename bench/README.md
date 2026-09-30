# bench

Своя Rust benchmark-библиотека и runner: короткие операции, отдельные процессы и сценарии приложения в одном формате наблюдений. Реализован рабочий прототип 0.1.0; полная приёмка и переносимость ещё проверяются.

Новые удобства: **init, обнаружение workspace targets, именованные baseline, прогресс/ETA, матрицы параметров, проверка результата, HTML-таблицы, бюджеты и Forma golden-сценарии**. [Полный рабочий процесс](docs/USABILITY.md).

Добавлены ещё [20 возможностей рабочего цикла](docs/NEXT20.md): профили, Git-сравнение, `last`, dry-run, фильтры/теги, fixtures/фазы, throughput, seed, история, графики, экспорт/bundle, заметки и CI-политика.

Реализована и третья партия — [ещё 20 функций рабочего цикла](docs/FINAL20.md): приватность, диагностика нестабильности, pilot, A/B/C, атрибуты `#[bench]`, async/threads/pipeline, cold-прогоны, фазы аллокаций, Forma GPU/окно, матрицы аргументов, profiler replay, resume, retention и bisect.

## Быстрый запуск

### Как `cargo test`: функции с атрибутом и `cargo bench`

В своём проекте добавьте зависимость и benchmark target:

```toml
[dev-dependencies]
airbug-bench = { path = "/path/to/airbug/bench", features = ["macros"] }

[[bench]]
name = "collections"
harness = false
```

Файл `benches/collections.rs`:

```rust
#[airbug_bench::suite]
mod collections {
    #[bench]
    fn sort() {
        let mut data = std::hint::black_box(vec![3, 1, 2]);
        data.sort_unstable();
        std::hint::black_box(data);
    }
}
```

`#[airbug_bench::suite]` создаёт `main` и регистрирует функции с `#[bench]`
или `#[airbug_bench::bench]` внутри модуля и вложенных inline-модулей. Один такой модуль
размещается в корне benchmark-файла; ручная регистрация и nightly не нужны.
Функции могут быть синхронными или async и возвращать результат. Generic-кейсы
объявляются через `types` и `consts`.
Обычный `#[bench]` измеряет всё тело функции, включая создание и уничтожение данных.

### Размеры входа и подготовка вне измерения

Отдельный проект не нужен: файл ниже находится в `benches/collections.rs` вашего
существующего Cargo package с настройкой `[[bench]]`, показанной выше.

```rust
#[airbug_bench::suite]
mod collections {
    #[bench(args = [64usize, 1024, 65536], setup = |n| (0..n).rev().collect::<Vec<_>>())]
    fn sort(values: &mut [usize]) {
        values.sort_unstable();
    }
}
```

Это три кейса: `collections/sort/64`, `collections/sort/1024` и
`collections/sort/65536`. Каждый вызов получает свежий вектор. Создание и
уничтожение входа исключены из времени; измеряется тело `sort`.

```sh
cargo bench --bench collections
cargo bench --bench collections collections/sort/1024 -- --exact
cargo bench --bench collections -- --list
```

Опции работают и с `#[airbug_bench::bench(...)]`:

| Атрибут | Сигнатура и поведение |
|---|---|
| `#[bench]` | `fn case()` — измеряется всё тело |
| `#[bench(args = [8u64, 32])]` | `fn case(n: u64)` — отдельный кейс для каждого значения |
| `#[bench(setup = || vec![3, 1, 2])]` | `fn case(input: &mut [i32])` — свежий input по ссылке; `fn case(input: Vec<i32>)` — по владению |
| `#[bench(args = [8usize, 32], setup = make_input)]` | `make_input(n)` готовит input; `fn case(input: &mut Input)` измеряется |
| `name = "sort"` | Меняет имя кейса; значения `args` добавляются после `/` |
| `drop_output = "outside"` | Исключает уничтожение возвращаемого результата из измерения, в том числе без setup; по умолчанию `"inside"` |
| `types = [u32, u64]` | Отдельный кейс для каждого типа `T` |
| `consts = [16, 64]` | Отдельный кейс для каждого значения `const N: usize` |
| `threads = [1, 2, 4]` | Отдельные кейсы с указанным количеством одновременно работающих потоков |
| `bytes = 1024, items = 100, chars = 50, cycles = 200` | Несколько счётчиков полезной работы на операцию; каждый выводится в отчёт |
| `bytes = \|n\| n as u64` | Счётчик, вычисляемый из значения `args` |
| `samples = 30, warmup_ms = 50, sample_ms = 5` | Настройки выборки конкретного кейса |
| `iterations = 100` | Фиксированное число операций на sample и worker, без калибровки |
| `sampling = "flat" / "linear" / "auto"` | Постоянное, возрастающее или автоматически выбранное число операций |
| `min_time_ms = 100, max_time_ms = 1000` | Границы суммарного времени вызовов нагрузки, включая калибровку и прогрев |
| `exclude_external_time = true` | Учитывать только измеряемые интервалы при проверке границ времени |
| `executor = make_executor()` | Executor для async; создаётся лениво вне измерения |
| `custom = true` | `fn case(iterations: u64) -> Duration` — собственные границы измерения |

Значения `args`, передаваемые по значению, должны реализовывать `Copy + Debug`;
можно передать массив, диапазон или другое `IntoIterator`. Для нескольких параметров
передайте кортежи. Функция с аргументом `&T` может заимствовать некопируемые значения;
например, `args = [String::from("hello")]` подходит для `fn case(s: &str)`.
Клонирование входа на каждой измеряемой операции не добавляется.
Выражение `args` вычисляется при регистрации, в том числе при `--list` и
`--dry-run`: держите его дешёвым и без побочных эффектов. `setup` и тело кейса при
этих командах не вызываются. Имена значений строятся через `Debug` и должны быть
уникальными внутри одного кейса. Аргумент без setup передаётся через `black_box`.

Вложенные inline-модули регистрируются автоматически. Для сложных fixtures
доступен `Suite` builder. Обычный `cargo bench` автоматически сравнивает кейсы с предыдущим успешным запуском
и сохраняет HTML-отчёт с результатами и сравнением. Первый запуск создаёт точку
отсчёта; при недостатке данных сравнение явно остаётся неопределённым.
Путь печатается как `Report: …/report.html`; по умолчанию отчёты находятся в
`target/airbug-bench/reports/<run-id>/`. Отдельный runner для этого не нужен.
`--output NEW_DIRECTORY` меняет место сохранения, `--discard` запускает измерения
без сохранения, `--no-history` отключает автоматическое сравнение и отчёт
(явный `--output` продолжает работать).

```sh
cargo bench --bench collections
cargo bench --bench collections sort
cargo bench --bench collections -- --list
cargo bench --bench collections collections/sort -- --exact --profile quick
# Готовый пример в этом репозитории:
cargo bench -p airbug-bench --features macros --bench attributed
```

Cargo использует оптимизированный bench-профиль. Список не исполняет функции.
Прямой `cargo bench` печатает результаты в терминал и записывает прогресс в `target/airbug-report/runs`. Откройте вкладку **Launch details** в hub этого workspace (`/#/launches`). Статусы записываются на границах кейсов, вне измеряемого участка; `AIRBUG_DASHBOARD=0` отключает запись для строгих измерений. `AIRBUG_DASHBOARD_ROOT` позволяет явно задать workspace.
Существующие targets с `Suite::main()` также принимают фильтр по имени от Cargo.

### Группы и ignored-кейсы

```rust
#[airbug_bench::suite(samples = 20, warmup_ms = 10)]
mod benches {
    #[group(name = "strings", samples = 30, bytes = 5)]
    mod text {
        #[bench(setup = || String::from("hello"), drop_output = "outside")]
        fn consume(mut input: String) -> String {
            input.push('!');
            input
        }

        #[bench(samples = 5)]
        #[ignore = "expensive"]
        fn expensive() {
            std::hint::black_box((0..1000).sum::<u64>());
        }
    }
}
```

Имена: `benches/strings/consume` и `benches/strings/expensive`. Настройки suite и
`#[group(...)]` наследуются; значение на кейсе переопределяет соответствующий
параметр. `ignore = true` можно задать всей группе, а `ignore = false` — отдельному
кейсу. `#[ignore]` также поддерживается.

```sh
cargo bench -- --include-ignored  # все выбранные кейсы
cargo bench -- --ignored         # только ignored
cargo bench -- --list            # обычные кейсы, без запуска
cargo bench -- --test            # одна операция на worker, без прогрева/калибровки
```

`--test` помечает результаты как `test_once`: это функциональная проверка,
а не оценка производительности. Зарегистрированная correctness-проверка вызывается
один раз вместо дополнительного запуска workload.

При передаче входа по владению setup исключён из измерения, но уничтожение входа
самой функцией является частью её работы. `drop_output = "outside"` откладывает
только уничтожение возвращённого результата. Вход по владению поддерживается также
для async и потоковых кейсов.

### Типы, async, потоки и своё время

```rust
#[airbug_bench::suite]
mod examples {
    #[bench(types = [u32, u64], consts = [16, 64], items = N as u64)]
    fn fill<T: Default + Copy, const N: usize>() -> [T; N] {
        [std::hint::black_box(T::default()); N]
    }

    #[bench(args = [64usize, 1024], setup = |n| vec![0u8; n], bytes = |n| n as u64)]
    async fn async_work(input: &mut [u8]) {
        std::future::ready(()).await;
        input.fill(7);
    }

    static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    #[bench(threads = [1, 2, 4], items = 1)]
    fn contention() -> u64 {
        COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    #[bench(custom = true)]
    fn measured_region(iterations: u64) -> std::time::Duration {
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            std::hint::black_box(123u64.wrapping_mul(std::hint::black_box(456)));
        }
        start.elapsed()
    }
}
```

`types` × `consts` × `args` × `threads` образуют отдельные кейсы с уникальными
именами. Для нескольких type-параметров задавайте строки типов:
`types = [(u8, u16), (u32, u64)]` для `fn case<T, U>()`. Каждая строка задаёт
одну комбинацию в порядке type-параметров функции. Для одного `T` tuple в списке
остаётся одним tuple-типом. Несколько const-параметров задаются аналогично:
`consts = [(16, false), (64, true)]` для `fn case<const N: usize, const FLAG: bool>()`.
Порядок элементов строки соответствует порядку const-параметров, даже если между
ними объявлены type-параметры. Строки `types` и `consts` образуют декартово произведение.
Явные lifetime-параметры функции поддерживаются, например
`fn parse<'input, T: From<&'input str>>(input: &'input str) -> T` с
`types = [String]` и `args = [String::from("text")]`. Lifetime заимствованного
входа выводится для каждого вызова; `where`-ограничения сохраняются и проверяются
Rust для выбранных типов. Поддерживаются также связанные lifetimes и `for<'a>`
в ограничениях. Зарегистрированные type-аргументы остаются `'static`; это не
требование `'static` к заимствованию входа. Список `types` записывается непосредственно
в атрибуте. Для `consts` можно указать внешний массив или срез: `consts = SIZES`,
где `const SIZES: &[usize] = &[16, 64]`. Внешний список содержит от 1 до 20 элементов;
список прямо в атрибуте не имеет этого ограничения. Для нескольких const-параметров
внешний список содержит tuple с соответствующими типами и порядком полей.
Значения `consts` могут быть const-выражениями, включая обращения к внешним
массивам и вызовы `const fn`.
`args` допускает константу, функцию или iterator, например `args = INPUTS` или
`args = (1..5).map(|n| (n, n * 2))`. Tuple передаётся одним аргументом:
`fn case((width, height): (usize, usize))`. Генератор выполняется при регистрации;
тело бенчмарка и `setup` при перечислении не запускаются.

Async по умолчанию использует `LocalExecutor`: он поддерживает `Future` и wakeup,
но не устанавливает Tokio/другой runtime, таймеры или I/O reactor. Для таких задач
передайте свой тип с реализацией `airbug_bench::workloads::Executor` через
`executor = ...`. Создание executor не происходит при `--list`, `--dry-run` или
выборе другого кейса. С `setup` async-функция получает свежий вход по `&mut`
или во владение, в зависимости от сигнатуры.

Готовые адаптеры включаются отдельными Cargo features:

| Feature | Значение `executor` |
| --- | --- |
| `async-tokio` | `tokio::runtime::Runtime`, `Handle`, `&Runtime` или `&Handle` |
| `async-futures` | `airbug_bench::executors::FuturesExecutor` |
| `async-smol` | `airbug_bench::executors::SmolExecutor` |

Например, с `features = ["macros", "async-tokio"]` и зависимостью `tokio`:

```rust,ignore
#[bench(executor = tokio::runtime::Builder::new_current_thread()
    .enable_all().build().unwrap())]
async fn timer() {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
}
```

Factory вызывается лениво; в однопоточном кейсе runtime переиспользуется между
samples. Создавайте Tokio timers/I/O внутри async-функции, после входа в runtime.
Для current-thread Tokio используйте сам `Runtime`: один `Handle` не обслуживает
его таймеры и I/O, если runtime не выполняется в другом потоке. `FuturesExecutor`
обслуживает futures и wakeups, но не предоставляет reactor для Tokio I/O.
Любой адаптер можно передать через `&mut`; futures и их результаты могут быть
без `Send`. Без этих features дополнительные runtimes не подключаются.

Async также поддерживает `threads = [1, 4]`, включая `args`, `setup`, borrowed/owned
входы и динамические счётчики. Executor создаётся на каждом worker перед каждой
волной (до 64 операций на worker), а уничтожается там же после завершения всех
измерений волны. Создание executor и setup исключены из измерения. Futures идут
последовательно внутри worker, несколько workers выполняются одновременно.
Входы, futures, executor и outputs не обязаны быть `Send`; захваты фабрик и
операции должны быть `Sync`. Для async+threads setup всегда worker-local.
Явный `setup_thread = "coordinator"` в этом сочетании отклоняется.

```rust,ignore
#[bench(threads = [1, 4], setup = || std::rc::Rc::new(42u64), input_items = |_| 1)]
async fn work(input: &mut std::rc::Rc<u64>) -> u64 {
    std::future::ready(**input).await
}
```

Потоковый кейс ждёт готовности всех workers и измеряет интервал от общего старта
до завершения последнего. Создание и join потоков исключены, пробуждение и
планировщик включены. Число операций равно `итерации × workers`; ns/op — величина,
обратная общей пропускной способности, а не latency отдельного worker.
`threads` или `threads = true` выбирает доступный параллелизм хоста (не более
256 workers; если ОС не сообщает значение — один). `threads = 4` задаёт одно
число; `threads = [1, 2, 4]`, диапазон или срез задают несколько кейсов.
Ноль в таком списке означает автоматический выбор. Одинаковые итоговые числа
объединяются, сохраняя порядок, поэтому `[0, 4]` не создаёт два одинаковых кейса
на четырёхпоточном хосте. В именах и contracts записывается фактическое число.
Литерал `threads = false` отключает унаследованную параллельность и выбирает
локальное выполнение, в том числе с аргументами без `Send`/`Sync`.
Для ручной регистрации доступно `airbug_bench::threads::available()`.
Явные числа workers должны быть в пределах 1..256. С `setup` измерение разбивается на
волны до 64 операций на worker; длительности волн суммируются. Входы и отложенные
outputs освобождаются после завершения всех workers. По умолчанию setup работает
на координаторе, поэтому входы и результаты должны быть `Send`.

`setup_thread = "worker"` создаёт и освобождает входы на измеряющем worker-потоке.
Это поддерживает локальные значения вроде `Rc<Cell<_>>`, включая результаты:

```rust,ignore
#[bench(threads = [1, 4], setup_thread = "worker",
        setup = || std::rc::Rc::new(std::cell::Cell::new(0u64)))]
fn local_state(input: &mut std::rc::Rc<std::cell::Cell<u64>>) {
    input.set(input.get() + 1);
}
```

Setup исключён из измерения. Захваченное состояние фабрики должно быть `Sync`,
поскольку её вызывают несколько workers. Переданные по значению входы уничтожаются
внутри операции; заимствованные входы и отложенные результаты — после остановки
всех таймеров, на своём worker. Паника в setup отменяет волну до запуска операций;
паника операции освобождает ожидающие потоки и завершает запуск с ошибкой.

Для синхронных и async-бенчмарков параметр `batch` управляет числом одновременно
живущих входов и отложенных результатов:

```rust,ignore
#[bench(setup = || vec![0u8; 4096], batch = airbug_bench::BatchPolicy::PerIteration)]
fn update(input: &mut Vec<u8>) { input.reverse(); }
```

`BatchPolicy::Iterations(NonZeroU64)` задаёт размер партии, `Batches(NonZeroU64)` —
целевое число партий на sample. `SmallInput` делит sample примерно на 10 партий,
`LargeInput` — на 1000; `PerIteration` подготавливает и измеряет каждый вход отдельно.
Последняя партия может быть неполной. Это ограничение числа значений, а не байтов
в их heap-буферах. Меньшие партии требуют больше обращений к таймеру.
Без параметра сохраняется размер партии до 64. Для ручной регистрации доступны
`Suite::bench_batched_ref` и `Suite::bench_batched`, а для async —
`Suite::bench_async_batched_ref` и `Suite::bench_async_batched`. Async-операции
внутри партии выполняются последовательно до завершения каждой future.
Для функций без `setup` политика управляет удержанием возвращаемых значений:

```rust,ignore
#[bench(drop_output = "outside", batch = airbug_bench::BatchPolicy::LargeInput)]
fn create_buffer() -> Vec<u8> { vec![0; 1024 * 1024] }
```

Память для контейнера результатов выделяется до начала таймера; сами операции и
сохранение результатов входят в замер, а уничтожение результатов — после него.
С `drop_output = "inside"` каждый результат уничтожается сразу в измеряемом участке.
Для builder API можно передать `|| ()` как setup в `bench_batched` или
`bench_async_batched`.
Threads с явной политикой партий пока не поддерживаются.

`custom` передаёт число операций и принимает их суммарный `Duration`. Пользователь
отвечает за исполнение ровно этого числа операций и ожидание их завершения.
`custom` поддерживает `async fn` и параметр `executor`: future выполняется до конца,
а функция возвращает суммарную длительность. Executor создаётся лениво один раз.
`custom` не комбинируется с `setup`, `drop_output` или `threads`: эти границы
задаются внутри функции. Для `async fn` параметр `threads` запускает futures на нескольких workers.

`input_bytes`, `input_items`, `input_chars`, `input_cycles` принимают функцию от
`&Input`, созданного `setup`. Счётчик вызывается после подготовки каждого входа,
до запуска таймера и до изменения/потребления входа операцией:

```rust,ignore
#[bench(setup = make_buffer, input_bytes = |input: &Vec<u8>| input.len() as u64)]
fn parse(input: &mut Vec<u8>) { /* измеряемая работа */ }
```

Callback создаётся один раз при регистрации кейса и может сохранять состояние.
Для worker-local setup нужен `Fn + Sync`, для остальных вариантов допустим `FnMut`.
Поддерживаются sync, async, borrowed/owned и оба места threaded setup. В режиме
`setup_thread = "worker"` подсчёт тоже происходит на worker. Суммы всех workers
и волн сохраняются в `observations[].work_totals` как десятичные строки; throughput
делит фактическую сумму sample на его длительность. Calibration и warmup не входят
в эти суммы. Динамический счётчик заменяет фиксированный счётчик той же единицы;
нулевой объём допустим. Отсутствующие/некорректные суммы считаются ошибкой данных.

Для ручной регистрации используйте `counters::InputCounters::new(&["bytes"])?`,
вызов `add("bytes", count)` в setup и `suite.input_counters(handle)` после регистрации
кейса. Clone handle передайте в setup; используйте отдельный accumulator для каждого
одновременно исполняемого кейса. Вызов `add` должен находиться вне измеряемой операции.

`bytes`, `items`, `chars`, `cycles` — неотрицательные числа `u64` на одну операцию.
Они могут использоваться одновременно; с `args` допускается closure от аргумента.
Повторный `Suite::work_units` заменяет счётчик только с той же единицей.
Для CI-бюджета с несколькими единицами выберите одну, например
`cargo airbug-bench throughput DIR --unit MiB --min 100`.
 `samples`, `sample_ms`
и `warmup_ms` задают локальные defaults. Явный CLI-флаг переопределяет соответствующий
параметр, `--profile quick|normal|thorough` — все три. `--dry-run` показывает
эффективные настройки в contract каждого кейса. Ни профиль, ни число samples не
гарантируют статистическую точность.

[Покрытие сценариев и оставшиеся ограничения](docs/FEATURES.md).

### Запуск через runner

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

CI monorepo: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (MSRV **1.96**, workspace `cargo test` / clippy). Версии семейства обновляются вместе в Cargo.toml; `.github/workflows/release.yml` создаёт GitHub Release и обновляет ветку `release`.

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

### Границы времени и расписание выборки

Настройки выше наследуются от suite/group; локальный атрибут переопределяет поле.
CLI предоставляет `--iterations`, `--sampling`, `--min-time-ms`, `--max-time-ms`,
`--exclude-external-time` и `--include-external-time` с приоритетом над атрибутами.
По умолчанию используется flat и время всего вызова нагрузки, включая setup/drop.
Максимум имеет приоритет над минимумом. Проверка происходит между вызовами:
начатая операция не прерывается и может превысить максимум. Если бюджет исчерпан
до первого sample, запуск возвращает ошибку. Минимум может увеличить число samples
(до защитного предела 100000); linear при продлении повторяет последний размер.
Фиксированные iterations несовместимы с linear и ограничены допустимым размером
нагрузки конкретного кейса. `--test` выполняет один вызов, независимо от лимитов.

### Выбор кейсов через регулярные выражения

```sh
cargo bench --bench attributed -- '^collections/(sum|sort_only/[0-9]+)$' --regex --list
cargo bench --bench attributed -- --exclude-exact collections/sum --exclude-regex '/threads=2$'
```

Обычный фильтр ищет подстроку; `--exact`, `--glob` и `--regex` взаимоисключающие.
Regex применяется ко всему пути кейса, но совпадение может занимать его часть:
для точного пути используйте `^` и `$`. Несколько исключений объединяются по OR.
`--exclude` сохраняет glob-семантику, `--exclude-exact` исключает только указанный
путь, `--exclude-regex` исключает совпадения выражения. Параметры работают одинаково
для списка, dry-run, smoke и измерения. Неверный regex останавливает запуск до setup.
В Rust используйте `Selection::default().with_regex(pattern)?`, `skip_regex(pattern)?`
и `exclude_exact`; выражения компилируются один раз перед обходом кейсов.

### Явное потребление результатов

`airbug_bench::black_box` предоставляет стандартный барьер оптимизации Rust.
`airbug_bench::black_box_drop(value)` пропускает значение через барьер и уничтожает
его в точке вызова. Например, `iterator.for_each(airbug_bench::black_box_drop)`
выполняет ленивый итератор целиком, включая уничтожение каждого результата.
Это полезно, когда уничтожение должно входить в измеряемую операцию.

### Порядок кейсов

`--sort registration|lexical|natural|source|kind` выбирает порядок для списка, dry-run и выполнения.
По умолчанию сохраняется порядок регистрации. `--reverse` обращает выбранный порядок.
Natural сравнивает последовательности ASCII-цифр по величине без ограничения разрядности:
`case2` идёт перед `case10`; при равных числовых значениях порядок уточняется написанием.
В Rust задайте `Selection { sort: SortOrder::Natural, reverse: true, ..Default::default() }`.
Сортировка запуска не меняет исходный порядок регистрации в `Suite`.

Целиком числовые сегменты пути (например, аргументы `-10.0`, `1.01`, `1e100`)
сравниваются как десятичные числа без преобразования в `f64`. Поддерживаются
произвольное число цифр и показатель степени в диапазоне `i64`. Числовые сегменты
идут перед текстовыми; равные значения уточняются написанием. Нечисловые сегменты
сохраняют сравнение последовательностей цифр, описанное выше.

`--sort source` упорядочивает по файлу, строке и столбцу объявления. Атрибуты
передают эти данные автоматически; для ручной регистрации доступен
`Suite::source_location(file, line, column)`. При одинаковом месте объявления
сохраняется порядок регистрации аргументов и специализаций. Кейсы без места
объявления идут после известных. Местоположение не входит в контракт измерения.

### Бенчмарки в нескольких файлах

В `benches/support/sorting.rs` объявите переиспользуемую группу:

```rust
#[airbug_bench::group(name = "sorting", samples = 20)]
pub mod cases {
    #[bench(args = [32usize, 128])]
    fn sort(n: usize) {
        let mut values: Vec<_> = (0..n).rev().collect();
        values.sort_unstable();
    }
}
```

В основном benchmark target подключите модуль обычным способом Rust:

```rust
#[path = "support/sorting.rs"]
mod sorting;

#[airbug_bench::suite(groups = [crate::sorting::cases])]
mod benches {}
```

Запуск: `cargo bench --bench <имя target>`. Отдельный проект не нужен.
Группы также можно экспортировать из библиотеки и подключать по пути crate.
`groups = [...]` работает и внутри самостоятельного `#[airbug_bench::group]`;
вложенные кейсы регистрируются автоматически. Группа создаёт функцию регистрации,
а `suite` — единственный `main`.

Импортированные группы используют настройки своего объявления. Настройки
родительского inline-модуля сейчас распространяются на его inline-потомков;
CLI-переопределения применяются и к импортированным кейсам. Импортированные группы
добавляются после локальных кейсов в порядке списка `groups`.

### Bootstrap-интервалы

```sh
cargo bench --bench attributed -- collections/sum --exact \
  --resamples 10000 --confidence-level 0.95 --analysis-seed 42 \
  --output target/sum-with-estimates
```

Любой из трёх параметров анализа включает bootstrap после измерений. По умолчанию
для включённого анализа используются 10000 повторных выборок, уровень 0.95 и seed 0.
Число повторов — от 2 до 1000000, уровень доверия — строго между 0 и 1.
В директории результата сохраняются `run.json` и `estimates.json`; с `--json`
выводятся строки `BENCH_RESULT=` и `BENCH_ESTIMATES=`. Обычный текстовый вывод
содержит таблицу оценок, percentile-интервалы и стандартную ошибку.

Доступны среднее, медиана, выборочное стандартное отклонение и медианное абсолютное
отклонение с коэффициентом 1.4826. JSON также содержит минимум и максимум.
Выборки берутся с возвращением; границы — линейно интерполированные квантили.
Seed делает повторный анализ одних и тех же наблюдений воспроизводимым.

Если есть несколько процессов, каждый даёт одну медиану. Для одного процесса
используются отдельные нормализованные наблюдения: это исследовательский интервал,
покрытие которого может нарушаться при автокорреляции. Он не описывает задержку
отдельных операций и не подтверждает воспроизводимость между процессами.
Меньше двух наблюдений или недоступная метрика дают `estimates: null` с пояснением.

Для повторного анализа сохранённых данных используйте
`bootstrap::analyze(&run, &bootstrap::Config { ... })`; для отдельного массива —
`bootstrap::estimate(&values, &config)`. Анализ не меняет существующие правила
сравнения независимых процессов.

### Регрессионная оценка времени операции

При `--sampling linear --resamples 10000` отчёт дополнительно оценивает наклон
модели `общее время = наклон × число операций`. Это метод наименьших квадратов
с прямой через начало координат. В `estimates.json` поле `regressions` содержит
оценку для каждого процесса: наклон, percentile-интервал, стандартную ошибку,
число samples и центрированный R².

Bootstrap переносит пару «число операций, общее время» целиком. Разные процессы
получают отдельные регрессии. Подход применяется к метрикам `batch_total`, когда
число операций меняется между samples. При постоянных общих временах R² равен
`null`; отрицательный R² допустим и означает плохую аппроксимацию этой моделью.
Допущение независимости выборок внутри процесса остаётся тем же, что у bootstrap.
Для массивов пар доступен `regression::fit(&samples, &bootstrap_config)`.

### Выбросы и HTML-анализ

При включённом bootstrap `--output` также сохраняет `estimates.html`: автономную
таблицу оценок и SVG-график классифицированных наблюдений. В JSON поле `outliers`
содержит квартильные границы, пять счётчиков и все исходные значения с индексами
и метками `low_severe`, `low_mild`, `normal`, `high_mild`, `high_severe`.

Классификация Tukey использует границы `Q1 − 1.5×IQR`, `Q3 + 1.5×IQR` для
умеренных и `Q1 − 3×IQR`, `Q3 + 3×IQR` для сильных выбросов. Квартили линейно
интерполируются; точка ровно на границе относится к менее строгой категории.
При нулевом IQR отличающиеся от ядра значения считаются сильными выбросами.
Наблюдения не отбрасываются из среднего, bootstrap или регрессии.
Для нескольких процессов классифицируются их медианы; для одного — нормализованные
наблюдения, как указано в `resampling_unit`.

Публичный API: `outliers::classify(&values)` и `outliers::figure(...)`.
Демонстрационный отчёт на синтетических данных:
`cargo run -p airbug-bench --example outlier_report > outliers.html`.

### Счётчики пустых входов и функции подсчёта

Ноль — допустимый объём полезной работы. `bytes = 0` или `items = 0` сохраняется
в контракте и даёт нулевую пропускную способность при положительном времени.
Пустая единица измерения и некорректное числовое значение остаются ошибками.

Модуль `airbug_bench::counters` предоставляет:

- `bytes_of::<T>()`, `bytes_of_many::<T>(count)` и `bytes_of_val(&value)`;
- `bytes_of_slice(values)`, `bytes_of_str(text)` и `bytes_of_iter(iterator)`;
- `chars_of_str(text)` и `items_of_iter(iterator)`.

Функции для итераторов потребляют их и выполняют побочные эффекты. Размер типа
определяется через `size_of`; переполнение количества байтов вызывает явную панику.
Символы считаются как Unicode scalar values: `é🦀` содержит 2 символа и 6 UTF-8-байтов.

```rust
#[airbug_bench::bench(args = [String::new(), String::from("é🦀")],
    bytes = |s: &str| airbug_bench::counters::bytes_of_str(s),
    chars = |s: &str| airbug_bench::counters::chars_of_str(s))]
fn count_chars(input: &str) -> usize { input.chars().count() }
```

### Формат единиц пропускной способности

```sh
cargo bench --bench attributed -- --bytes-format decimal
cargo bench --bench attributed -- --bytes-format binary --json --output target/rates
```

`decimal` использует B/KB/MB/GB… с основанием 1000; `binary` — B/KiB/MiB/GiB…
с основанием 1024. Префикс выбирается по медиане серии и применяется ко всем её
значениям. Нулевой поток отображается в B/s. Число операций, символов и циклов
сохраняет свои единицы.

При `--json` добавляется `BENCH_THROUGHPUT=`; с `--output` сохраняется
`throughput.json`. Форматирование не меняет `run.json` и контракты сравнения.
Без флага сохраняется прежний фиксированный MiB/s. Библиотечные функции:
`report::throughput_with_format` и `report::markdown_with_bytes_format`,
аргумент — `report::BytesFormat::Decimal` или `Binary`.

### Переименованная зависимость

```toml
[dev-dependencies]
ab = { package = "airbug-bench", version = "0.9", features = ["macros"] }
```

```rust
#[ab::suite]
mod benches {
    #[ab::bench]
    fn sample() -> usize { std::hint::black_box(42) }
}
```

Макросы определяют имя `ab` из Cargo.toml. Для реэкспортирующего модуля или
библиотеки можно задать путь явно:

```rust
pub use ab as bench_api;
#[ab::suite(crate = crate::bench_api)]
mod benches { #[bench] fn sample() {} }
```

Параметр `crate` поддерживается у `suite`, `group` и самостоятельного `bench`;
наследуется вложенными inline-группами. Короткие `#[bench]`/`#[group]` внутри suite
и квалифицированные атрибуты через имя зависимости работают одинаково.

### Timed profiling

```sh
cargo bench --bench collections collections/sort -- --exact --profile-time-ms 5000 --output target/sort-profile
```

`--profile-time-ms` repeats each selected case for the requested wall-clock duration.
Setup, executor construction and synchronization count toward this duration. The loop
stops between complete batches, so a slow operation may exceed the requested time.
Explicit iteration counts are honored; otherwise batch sizes adapt up to the configured
maximum. Custom timing cannot shorten the session by reporting a different duration.

Install an in-process profiler with `Suite::profiler(impl profiling::Profiler)`.
Its `start(case, directory)` and `stop(case, directory)` hooks bracket each case's
work; correctness checks run outside the hooks. `stop` is attempted even when
`start` or the workload returns an error or unwinds. It must tolerate partial
initialization. The default hooks do nothing, allowing an external profiler to attach.

The output directory must be new. Each case gets a numbered subdirectory for profiler
artifacts and `summary.json`; the session writes `profile.json`. `--json` prints
`PROFILE_RESULT`. These files contain actual elapsed time and operation counts, without
statistical estimates. Without `--output`, artifacts go under `target/airbug-profile/`.
`--list` and `--dry-run` execute no hooks and create no artifacts. Profiling rejects
cold cases and combinations with `--test`, bootstrap or throughput formatting options.
The separate `--profile quick|normal|thorough` option configures ordinary sampling.

`--sort kind` ставит кейсы перед подгруппами на каждом уровне, с естественной
сортировкой имён внутри одного типа. Группы задаются через `Suite::group` или
атрибуты групп; `/` в имени отдельного кейса не создаёт подгруппу. `--reverse`
разворачивает порядок. Семейства generic-кейсов сортируются как группы; варианты обычных runtime-аргументов остаются кейсами. Атрибуты задают семейство автоматически; для ручной регистрации есть `Suite::ordering_family`.

### Local closure state

`Suite::bench`, `bench_with_input` and `bench_with_owned_input` accept `FnMut`
closures borrowing local variables. Inputs and outputs can contain `Rc` or other
non-`Send` values. The closure state persists across samples and repeated runs of
the same suite; drop the suite to release its mutable borrows. Setup remains lazy.
Use `DropPolicy::OutsideTiming` with the input APIs to defer output destruction;
`|| ()` supplies a unit input when only closure state is needed.

### Shared reusable context

`Fixture::new(move || context)` with `Suite::bench_fixture` creates a context
lazily, on its first selected execution. Clone the fixture to share mutable state
between cases. Initialization and borrowing are outside the timed loop; each
operation sees the state left by the preceding operation. Output destruction is
included. The context is destroyed after the final fixture owner is dropped, so
an external clone can retain it beyond the suite's lifetime. Use ordinary local
closures when the context borrows stack variables instead of owning its state.

### Descriptive statistics without resampling

`airbug_bench::bootstrap::describe(&values)` computes count, minimum, maximum,
mean, median, sample standard deviation and median absolute deviation scaled by
1.4826. It accepts a single finite observation, reporting `None` for sample
standard deviation. Empty/non-finite inputs and unrepresentable statistics return
an error. Values must already use the intended unit (for example, ns per operation).
This function performs no bootstrap resampling; use `bootstrap::estimate` for
confidence intervals. Direct `cargo bench` prints the descriptive table by default; `--json` emits
`BENCH_SUMMARY` and `--output` writes `summary.json`. Rows remain separated by
case, metric, variant and process. Batch totals are normalized per operation;
they are not individual-operation latency samples. Any unavailable observation
makes that row unavailable instead of silently summarizing a subset.

Descriptive rows also include `operations` and `operation_weighted_mean` for
`batch_total` metrics. The sample mean gives every normalized batch equal weight;
the operation mean weights each batch by its actual operation count (equivalent
to total measured value divided by total operations). Missing observations make
both estimates unavailable. Neither is a distribution of individual latencies.

`associated_counters` in each wall-time summary records work per operation for
the fastest and slowest normalized samples, the central one or two samples by
time, and the operation-weighted mean. These are counters associated with timing,
not counters sorted independently. Dynamic input totals override fixed counts;
missing timing suppresses the associated statistics. The text report includes
the same table. Allocation counters use the separate timing-associated allocation table.

Allocation snapshots and phase results distinguish `allocated_bytes`,
`deallocated_bytes`, `grow_operations`, `shrink_operations`, `grown_bytes` and
`shrunk_bytes`. Growth/shrink byte counts are size differences; allocation/free
byte counts exclude reallocations. Failed reallocations change none of these
counters. Same-size reallocations increment the general realloc count and the grow count,
with zero grown bytes. Process-wide snapshots and per-case/per-worker collection
are available; direct Cargo summaries include timing-associated allocation data.

`TrackingAllocator::begin_thread_phase()` starts allocation accounting on the
current thread for that allocator instance. The guard cannot move to another
thread; `finish()` returns `ThreadStats`, and dropping the guard stops tracking,
including during unwinding. Concurrent threads may each hold a phase. Nested
phases on one thread are rejected. Successful operations are counted on the
thread performing them, so freeing memory allocated before the phase or on another
thread can produce negative `net_live_bytes`. `peak_above_start_bytes` is the
maximum positive net balance, not total process memory. Arithmetic overflow sets
`overflowed`; values then saturate. Local, async and worker Suite integrations are described below.

For a synchronous local case, register `suite.bench_allocated("case", &ALLOCATOR,
|| work())`, where `ALLOCATOR` is the installed global `TrackingAllocator<System>`.
Each measured sample records wall time plus allocation/free/reallocation counts,
byte totals, growth/shrinkage, net growth/release and peak net growth. Reporting
allocations are outside the phase; output destruction is inside it. Pilot and
warmup counters are discarded. Metrics share case/process/sequence identifiers
with wall time. Net growth and net release use separate nonnegative fields;
`alloc.peak_above_start_bytes` is a sample peak, not a per-operation total.
This entry point supports local synchronous work; an async counterpart is described
below. Fresh-input, drop-policy and worker variants are also described below.

`Suite::bench_async_allocated(name, &ALLOCATOR, executor_factory, async || work())`
collects the same thirteen metrics on the thread polling the futures. The executor
is constructed once, lazily, before the allocation phase. Future creation,
polling, executor work inside `block_on` and output destruction are included.
Non-Send futures are supported. Allocations by tasks dispatched to other threads
are excluded; this is calling-thread accounting, not runtime-wide accounting.

Attribute suites accept the same allocator directly:

```rust,ignore
#[global_allocator]
static ALLOCATOR: airbug_bench::alloc::TrackingAllocator<std::alloc::System> =
    airbug_bench::alloc::TrackingAllocator::new(std::alloc::System);

#[airbug_bench::suite]
mod benches {
    #[bench(allocator = &crate::ALLOCATOR)]
    fn make_buffer() -> Vec<u8> { vec![std::hint::black_box(1); 64] }
}
```

`allocator` can be a group default and supports plain sync/async functions with
runtime/type/const arguments. Supply a reference to the installed allocator.
Setup, input counters, explicit batching and both output drop policies compose
with allocation accounting for local sync/async functions. Without an explicit
batch policy, setup or deferred output uses batches of at most 64 operations.
Synchronous `threads` cases also support `allocator`; fresh inputs preserve the
default coordinator setup, with `setup_thread = "worker"` available for worker-local
values. Async workers construct their executor and inputs on each worker.
Custom timing with `allocator` is currently rejected at compile time.

Wall summary rows include `associated_allocations`, pairing each `alloc.*` metric
with wall observations by case, variant, process and sequence. Missing samples
produce `null` (shown as n/a in text); inconsistent operation counts are errors.
Fastest/slowest and central one or two samples are selected by normalized wall
time. Batch totals are normalized per operation and their mean is operation-weighted.
Sample peaks retain their original units and use an unweighted sample mean.

`Suite::bench_allocated_batched_ref(name, &ALLOCATOR, setup, operation, drop, batch)`
and `bench_allocated_batched` support synchronous fresh borrowed/owned inputs.
Input setup, borrowed input destruction and harness buffer allocation are excluded
from time and allocation counts. Owned input destruction performed by the operation
is included. `DropPolicy` controls output destruction. Counters sum across batches;
peak net growth is the maximum within a measured batch, since excluded cleanup
separates the batches. Net growth/release is the sum of measured event deltas and
need not equal the process memory change including excluded setup/cleanup.
These lifecycles are also available through attributes and async batch methods.

`Suite::bench_async_allocated_batched_ref` and `bench_async_allocated_batched`
add an executor factory before the setup argument. The executor is initialized
lazily before accounting starts. Future creation, polling and executor work on the
calling thread are counted; setup, harness buffers and executor construction are
excluded. Borrowed input destruction is excluded; consumed owned input destruction
is counted when performed by the operation. Output destruction follows
`DropPolicy`. Each future completes before the next operation, and each batch is
cleaned up before preparing the next one. Tasks on other threads are not counted.

`Suite::bench_threads_allocated_with_local_input(name, &ALLOCATOR, workers,
setup, operation, drop)` collects allocator events inside each worker's operation
loop. Inputs are created and destroyed on their worker and may be non-Send.
Waves contain at most 64 operations per worker. Setup, input destruction, harness
buffers and thread creation/joining are excluded. Deferred outputs remain alive
until all worker timers stop. Counts and bytes sum across workers and waves;
operations normalize by the total across workers. The peak metric is the maximum
across waves of the **sum of individual worker peaks**, an upper bound rather
than a simultaneous process-wide peak. Failed worker operations return an error
without emitting a partial successful allocation sample.

The owned counterpart, `bench_threads_allocated_with_local_owned_input`, moves
fresh inputs into the operation. Destruction of consumed inputs is included.
Both APIs support non-Send inputs and outputs. Attribute equivalents preserve the
same accounting and input counters:

```rust,ignore
#[bench(allocator = &crate::ALLOCATOR, threads = [1, 4],
    setup = || vec![0u8; 1024], setup_thread = "worker",
    input_bytes = |v: &Vec<u8>| v.len() as u64, drop_output = "outside")]
fn transform(input: Vec<u8>) -> Vec<u8> {
    input.into_iter().map(|byte| byte.wrapping_add(1)).collect()
}
```

`bench_threads_allocated_with_input` and
`bench_threads_allocated_with_owned_input` prepare inputs on the coordinator and
transfer them to workers. Their setup closures may borrow mutable, non-Sync
coordinator state; inputs and outputs must be Send. Borrowed inputs and deferred
outputs are returned and destroyed on the coordinator after all worker timers
stop. Owned inputs are destroyed wherever the operation consumes them. These
APIs use the same per-worker counts and upper-bound peak aggregation as the
worker-local variants. Attributes select coordinator setup by default or with
`setup_thread = "coordinator"`.

`bench_async_threads_allocated_with_input` and
`bench_async_threads_allocated_with_owned_input` add an executor factory after
`workers`. Executors are created on each worker for each wave before measurement;
their construction and destruction are excluded from allocation accounting.
Future creation, polling and executor work inside `block_on` are included on that
worker. Executors, inputs, futures and outputs may be non-Send. The same behavior
is available through async `#[bench(allocator = &ALLOCATOR, threads = [1, 4])]`,
with optional setup, input counters and `drop_output`. As with other async worker
cases, setup is worker-local and explicit coordinator setup is rejected. Tasks
spawned onto additional threads are outside these worker allocation counters.

`alloc.peak_above_start_count` reports the peak positive net number of live blocks
during measurement. Reallocating a block does not change this count. Like the
byte peak, it remains an absolute sample peak; worker samples use the maximum
wave sum of individual worker peaks. The signed `ThreadStats::net_live_count`
can be negative when the measured code frees blocks created before measurement.

A successful same-size `realloc` increments both `realloc_count` and `grow_count`,
with zero `grown_bytes`, matching Divan's allocation operation classification.
It changes neither the live block balance nor the live byte balance. Failed
reallocation changes none of these counters. The same rule applies to thread
phases, process snapshots and process phases.

Allocation peak summaries also include `per_operation` in JSON and a separately
labelled row in text. Fastest, slowest and median-time sample peaks are divided
by the operation count of each corresponding sample. `operation_mean` is the
sum of sample peaks divided by the total operation count. Absolute peak values
remain available alongside this normalized view. A normalized peak is an
amortized sample statistic, not an independently measured peak for each operation.

Threaded allocation runs preserve `worker_allocations` in `run.json`. Each record
identifies its case, variant, process, sample sequence, wave and worker slot,
with operation count, elapsed `wall_ns`, and exact decimal-string allocation
metrics. Slots identify workers within a wave; they are not persistent OS thread
identities across waves. Warmup/calibration records are discarded. Empty records
are omitted, and old run files without this field remain readable. Aggregate
observations continue to summarize all workers. The process runner remaps worker
records to the enclosing process and variant when collecting protocol results.

New allocated worker cases declare `alloc.worker_records = "wave-v1"`. Validation
requires complete contiguous waves with every declared worker, equal per-worker
operation counts within a wave, and agreement with aggregate operations, timing,
counters and peaks. Opposite signed net balances cancel before comparison.
Overflow and missing/mismatched records are errors. Legacy cases without the
contract may omit records; if records are present, their consistency is checked.

Wall summary rows include optional `worker_allocations`: a separate summary over
worker waves, ranked by each worker's elapsed time per operation. It includes
fastest/slowest and central one or two source identities (`sequence`, `wave`,
`worker`), all allocation metrics, and absolute plus normalized peaks. Text
reports show the same metrics under “Worker allocations associated with timing”.
Means of totals are weighted by operations; absolute peaks use sample means.
Worker waves are not independent process repetitions and do not expand the
sample size used for process-level confidence intervals.

Inline `#[suite]` and `#[group]` modules accept an `executor` default. It is
inherited by async cases and nested inline groups, with case/group overrides.
Sync cases in a mixed group ignore the inherited executor; explicitly specifying
an executor on a sync case is still rejected. Executor construction remains lazy
and does not run during listing. For an executor factory outside the nested
module, use a path visible there, such as `crate::make_executor()`.

Imported `groups = [...]` inherit missing sampling settings from the importing
suite/group, including across crate boundaries. Each field follows nearest
explicit child, then importing parent, then suite defaults; CLI overrides win.
This includes sample count, warmup/sample durations, iterations, sampling mode,
time limits and external-time accounting. `Suite::group_with_sampling` exposes
this fallback behavior for manual registration. Unrelated cases are unaffected.
Other inherited options across imported groups remain under the parity audit.

Imported groups also inherit missing `ignore` and fixed bytes/items/chars/cycles
counters. Explicit child `ignore = false` and zero counters override parents;
dynamic input counters take precedence over parent fixed counters of the same
unit. `Suite::group_with_defaults` combines these fallbacks with sampling.
Argument-dependent counter closures cannot yet cross an imported-group boundary;
declare those on the imported group or its cases. The macro reports this
unsupported combination instead of silently dropping the counter.

### Adaptive quick measurements

`Suite::quick(QuickConfig::default())` skips calibration/warmup and doubles
measured batch sizes from one operation up to the configured iteration cap.
It stops after 100 ms when the relative residual of two adjacent batches is
below 5%, or after 5 seconds of elapsed wall time. These defaults follow
Criterion's quick measurement strategy. At the cap, equal-sized batches continue.
`QuickConfig` exposes `min_time`, `max_time` and `relative_deviation`.
The residual is a stopping heuristic, not a confidence interval.

All real batches are retained. A first invocation exceeding the deadline yields
one sample; no duplicate measurement is invented. Deadlines are cooperative and
cannot interrupt a workload call. Existing maximum time budgets can also stop
collection; a 100,000-sample safety cap bounds storage. The contract records the
actual sample count and `quick.stop` reason. Fixed iterations and linear/auto
schedules conflict with this mode. Test/cold single-invocation modes take priority.
Use `cargo bench -- --quick` for adaptive measurement. Optional
`--quick-min-ms`, `--quick-max-ms` and `--quick-relative-deviation` enable and tune
it. `--profile quick` remains the existing fixed preset. Adaptive quick mode
conflicts with timed profiling and fixed/linear/auto schedules; `--test` still
runs once. Listing and dry-run execute no workload. Attributes accept `quick = true/false` and `quick_config = QuickConfig { ... }`.

Quick attributes inherit through inline and imported groups. An explicitly
configured child case or imported child group wins; `quick = false` disables an
inherited mode. `quick_config` without `quick` enables the mode. A configuration
is a complete `QuickConfig` value; use `..Default::default()` for omitted fields.
The builder equivalent is `quick_case(Some(config))` or `quick_case(None)` for
an explicit disable. Suite-level `quick(config)` is a fallback. CLI quick flags
override case choices for selected benchmarks. Conflicts are checked across all
selected cases before any workload executes. Dry-run describes each case's
resolved mode and limits without assigning a fixed sample count to adaptive work.

When quick mode and sampling budgets are combined, the sampling maximum budget
is checked first after each call. Stability stops only after both the quick
wall-time minimum and sampling minimum have been met. The quick maximum wall
deadline still stops a run whose sampling minimum is unmet. A subsequent run
starts its adaptive schedule from one operation; disabling quick mode removes
its old contract metadata and restores the selected ordinary sampling policy.

### Explicit low-level clocks

`airbug_bench::timer::Timer::os()` uses the monotonic OS clock.
`Timer::cpu()` lazily validates and caches a supported CPU counter's frequency;
it returns an error when unavailable. The current implementations cover ARM64
macOS/Linux architectural counters and invariant x86/x86_64 TSC with RDTSCP.

```rust
use airbug_bench::timer::Timer;
let timer = Timer::cpu()?;
let calibration = timer.calibrate()?;
let start = timer.start();
// Work to measure.
let elapsed = start.elapsed()?;
# Ok::<(), airbug_bench::BenchError>(())
```

Calibration reports the minimum empty interval and smallest nonzero observed
interval over 256 immediate trials. If every interval is zero, it tries 32
intervals at each power-of-two delay from 1 through 1024 black-box iterations.
This is bounded to 608 resolution trials total. The first successful delay gives
an observed estimate; exhausting the retries leaves resolution unknown. Delayed
intervals do not change the reported empty-read cost. Results include the retry
count, final delay and `resolution_status = observed|unresolved`.
It also measures 100 black-box loops of 10,000 iterations
and retains the smallest batch interval. The ratio `loop_batch_ns / loop_iterations`
represents the per-iteration estimate without rounding sub-nanosecond costs to zero.
These diagnostics do not change raw observations. Optional compensation is described below.
Backward/wrapped CPU timestamps return an error. CPU ticks are converted with
integer arithmetic into nanoseconds. A start timestamp can be shared by workers;
this does not guarantee synchronization on every host or virtual machine.

Use `suite.timer(Timer::cpu()?)` to select CPU timing for Suite workloads.
Selection applies to local/async operations, borrowed/owned batches, fixtures,
allocation accounting and synchronized worker waves, including already registered
cases and subsequent runs. OS time remains the default and drives warmup, quick
mode deadlines and external-time budgets. Custom measurements retain their
caller-reported duration. Clock errors propagate after workers join.

Results record `timer.clock` in case contracts (`caller` for custom durations),
and the selected clock and optional frequency in run provenance. Frequency is
not a case contract, so calibration variation does not change case identity.
Direct Cargo targets also accept `cargo bench -- --timer cpu` or `--timer os`.
The CLI overrides the suite clock. `--list` and `--dry-run` validate the name but
never resolve or calibrate the CPU clock; dry-run records the requested clock.
Unsupported CPU clocks fail before workloads and live-history creation, without
falling back silently. Repeated `--timer` flags and unknown clock names are errors.
Attributes accept `timer = "os"` or `timer = "cpu"` on benches, groups and suites:

```rust,ignore
#[airbug_bench::suite(timer = "cpu")]
mod measurements {
    #[bench]
    fn cpu_default() {}

    #[bench(timer = "os")]
    fn os_override() {}
}
```

The nearest case/group setting wins, including imported groups. CLI `--timer`
overrides all selected cases. `Suite::timer` supplies the fallback for cases
without an attribute or `timer_case(TimerKind::Cpu/Os)` setting;
`with_timer_defaults(kind, register)` supplies defaults across imported groups.
These per-case choices remain lazy during registration and listing. Execution
resolves all selected clocks before any workload, so a later unavailable CPU
case cannot leave earlier cases partially executed. Mixed OS/CPU runs record
`mixed` in run provenance and the concrete clock in each case contract.
ARM64 native runtime, x86 Windows cross-compilation, and x86_64 macOS runtime
through Rosetta have been checked. Rosetta evidence does not replace native x86
platform testing.

Ordinary Suite runs cache these diagnostics once per clock per process and record
`timer.<os|cpu>.calibration.*` in run provenance. The records include sample counts,
empty-interval cost, optional observed resolution, and the loop ratio. Clock
diagnostics happen before workload callbacks and do not consume workload budgets.
Cold runs, smoke tests and caller-only measurements omit them. The explicit `timer.overhead_policy` field records whether compensation was
requested; each case records whether it was applied.

`airbug_bench::alloc::calibrate_overhead(timer)` measures the profiler's own
bookkeeping with a private scratch tracker. Timed loops call the same TLS and
atomic accounting helpers as successful allocations, frees and reallocations;
they never call the underlying allocator. Grow and shrink have separate estimates.
The estimate describes an uncontended thread and does not model shared-counter
contention. Synthetic events do not modify the application's tracker.

`AllocationOverhead::tally_ns(&ThreadStats)` subtracts the calibration-loop cost
from each event estimate, weights by event counts, then rounds the summed ratio
once. It rejects inconsistent realloc counts and arithmetic overflow. Noise can
make an event estimate smaller than the loop estimate; its contribution is then
zero. Suite can apply these estimates through the optional compensation mode.

### Overhead compensation

Use `suite.compensate_overhead(true)` or `cargo bench -- --overhead subtract`.
The default is `--overhead raw`. Compensation preserves `wall` and adds
`wall.adjusted`, in nanoseconds per batch. Reports include both; comparison can
select `--metric wall.adjusted`.

Loop and allocation-tally costs are calibrated outside workload intervals and
cached per real clock. Their fractional costs are combined before rounding to
nanoseconds. Each measured interval clamps the subtraction at zero; batches sum
those adjusted intervals. Concurrent workers are adjusted individually, then each
wave takes the maximum adjusted duration before waves are summed. The worker with
the greatest adjusted duration can differ from the worker with the greatest raw
duration. Allocation worker records retain both times, and validation checks the
adjusted aggregate against them. Correction coefficients are stored in provenance.

Custom durations, cold cases and smoke tests remain raw. Listing/dry-run do not
calibrate. Timed profiling rejects CLI compensation because it emits no statistical
samples. Sampling schedules, quick stopping and time budgets continue to use raw
durations. Switching compensation off removes adjusted metrics from later runs.
These are estimates of uncontended instrumentation costs; shared atomic contention
and workload-specific cache effects are not modeled.

Compensation also supports `overhead = "subtract"` or `overhead = "raw"` in bench,
group and suite attributes. The nearest case/group setting wins, including
imported groups; an explicit `raw` prevents inheritance. CLI `--overhead` overrides
all selected cases. Builder equivalents are `compensate_case(bool)` and
`with_overhead_defaults(bool, register)`, with `compensate_overhead(bool)` as the
suite fallback. Dry-run reports effective case policies without calibration.

### Automatic comparison with the previous Cargo run

Ordinary `cargo bench` targets using `Suite::main()` now preserve completed results
and compare each selected case with its previous completed result automatically.
The default store is `target/airbug-bench/history` relative to the benchmark process
working directory. `--history-dir DIR` or `AIRBUG_BENCH_HISTORY_DIR` chooses another
root (CLI wins); `--no-history` or `AIRBUG_BENCH_HISTORY=0` disables reads and writes.
Suite name and executable filename form the history namespace, so different Cargo
target identities do not silently share baselines.

Console output includes a previous-run comparison. JSON output adds
`BENCH_COMPARISON=...` with `first_run`, `compared` or `incompatible` per case.
A filtered run updates only its selected cases. Changed environment, metric or
workload contracts produce an explicit incompatibility; the implementation does
not search past that latest result for a more convenient older baseline. Realized
sample count and quick-stop reason are ignored for compatibility, while original
records remain intact. Existing process-level statistics provide changes and
uncertainty: two individual direct runs normally yield an inconclusive verdict,
not a statistically established regression from many batches in one process.

History entries contain `run.json`, `comparison.json` and a final `COMMITTED`
marker with content hashes. Incomplete entries are ignored; changed committed
artifacts produce an error. Smoke tests, timed profiles, listing and dry-run do
not create history. Workload failures and requested-output failures do not promote
a result. Concurrent writers publish separate entries and use their own snapshots
of committed history. The library equivalent is `history::History::record(&run)`.

### Named baselines in direct Cargo benchmarks

Register an existing completed result, then select it in the usual Cargo target:

```sh
cargo airbug-bench baseline save main path/to/result
cargo bench --bench mybench -- --baseline main
cargo bench --bench mybench -- --baseline-lenient main
```

Both commands use `.bench` in the current directory by default. For another store,
use global `--store PATH` in `cargo airbug-bench` and `--baseline-store PATH` in the
benchmark target. Strict `--baseline` requires the name and every selected case
before setup or measurement starts. `--baseline-lenient` reports missing cases as
`FirstRun`. Both reject changed, missing or invalid referenced artifacts; a missing
name is the only absent-file condition accepted by lenient loading.

Named comparison reads a verified snapshot and never changes the named reference
or automatic previous-run history. With `--json`, `BENCH_BASELINE` contains the name
and per-case report. Incompatible contracts are reported explicitly. A single
process still produces descriptive differences and an inconclusive significance
verdict. Listing and dry runs do not load baseline artifacts. Smoke tests and timed
profiling reject named comparison options.

`cargo bench --bench mybench -- --discard` executes measurements and prints their
results without writing result artifacts, automatic benchmark history or dashboard
launch history, even when those histories are enabled in the environment. It
rejects `--output`, named baseline comparison and timed profiling before executing
work. Existing baseline files and histories remain intact. This is broader than
`--no-history`, which disables only automatic previous-run comparison/storage.

Save the measured result directly from Cargo:

```sh
cargo bench --bench mybench -- --save-baseline main
cargo bench --bench mybench -- --replace-baseline main
cargo bench --bench mybench -- --retain-baseline main
# Compare to the old snapshot, then replace it after successful measurement:
cargo bench --bench mybench -- --baseline main --replace-baseline main
```

`--save-baseline` requires a new name. `--replace-baseline` updates measured cases while preserving unselected cases
and earlier snapshots. `--retain-baseline` keeps existing case references and adds
missing cases; measurements still execute. All use
`--baseline-store` and publish only after successful measurement and requested
artifact/history writes. Snapshots live in `baseline-runs/` under that store.
Listing/dry runs write nothing. Smoke, discard and timed profiling reject save
options. A publication failure can leave an unreferenced completed snapshot for
recovery; it does not overwrite an existing source artifact.

Reanalyze an existing snapshot without running setup, calibration or workload:

```sh
cargo bench --bench mybench -- --load-baseline candidate --baseline main --json
cargo bench --bench mybench -- --load-baseline candidate --output new-analysis
```

`--load-baseline` requires every selected case in the saved snapshot. Selection
uses the target's registered case names; stored observations, contracts and run
identity are preserved. Report options such as resampling and throughput formatting
operate on those saved values. The command creates no dashboard/automatic history
entry and does not promote a baseline. Explicit `--output` exports the selected
snapshot and reports to a new directory. Saving another baseline, smoke mode and
profiling are rejected in this mode.

Case baseline manifests can refer to several original runs. Loading such a baseline
keeps each source artifact separate: console/JSON report records are emitted per
source, and `--output DIRECTORY` writes `DIRECTORY/0/`, `DIRECTORY/1/`, etc. A single
source keeps the usual output layout. Cases from the same artifact are analyzed
together; matching run IDs alone never cause independent artifacts to be merged.

### Comparison significance and practical margin

Direct Cargo comparisons accept `--significance-level FRACTION` (default `0.05`)
and `--noise-threshold-percent PERCENT` (default `5`). These configure named
baseline comparisons and automatic previous-run comparisons. The significance
level must be strictly between zero and one; the margin must be finite and nonnegative; values of 100 percent or more are
allowed. Both are recorded in comparison JSON and printed in text.

The margin describes a practically meaningful change; it does not remove noisy
samples. Significance controls the family-adjusted uncertainty calculation across
selected cases and metrics. Independent processes remain the statistical units:
increasing samples within a single process does not establish significance.
`--confidence-level` separately configures bootstrap estimate intervals.

Independent-run comparisons additionally report a two-sided Welch statistic and
pooled-bootstrap p-value, computed from per-process medians after normalizing batch
totals by operation counts. Comparison JSON includes sample counts, seed, resample
count, the family-adjusted significance threshold and the zero-effect test result.
The pooled null assumes exchangeability and independent process observations.

This p-value tests the difference in means of process medians. The existing
practical-change decision uses median effect intervals and the configured margin;
these answer different questions and are both reported. Fewer than two independent
processes on either side, or distinct constant groups with zero variance, produce
an explicit unavailable test. Paired A/B runs retain their paired interval analysis.
Library callers can choose repeat count and seed with `analysis::compare_with_hypothesis`.

Use `--hypothesis-resamples N` (2 through 1,000,000; default 10,000) and
`--hypothesis-seed N` (default 0) to configure Welch pooled-bootstrap calibration
from direct Cargo. These options apply to previous-run and named-baseline
comparisons, including `--load-baseline` reanalysis. The comparison configuration
and each test record include both values. They are independent of descriptive
bootstrap `--resamples` and `--analysis-seed` settings.

Library suites can configure comparison scopes using `comparison_config` for the
suite fallback, `with_comparison_defaults` around group registration, and
`comparison_case` for the last registered case. `history::ComparisonOptions` holds
optional margin, significance, hypothesis resamples and seed fields. Each field
inherits independently: explicit case value, nearest group value, suite fallback.
Explicit CLI values take precedence over these settings for that field only.
Comparison reports include the effective configuration for each selected case.
Invalid selected-case settings fail before workloads or history publication.

The same fields are available on `#[airbug_bench::suite(...)]`,
`#[airbug_bench::group(...)]` and `#[bench(...)]`:

```rust,ignore
#[airbug_bench::group(
    significance_level = 0.01,
    noise_threshold_percent = 2.0,
    hypothesis_resamples = 20_000,
    hypothesis_seed = 7
)]
mod comparisons {
    #[bench(noise_threshold_percent = 0.0)]
    fn work() { std::hint::black_box(1 + 2); }
}
```

Defaults cross imported-group and crate boundaries. Explicit zero margins/seeds
remain overrides. Values are Rust expressions and are validated before selected
workloads execute. `Suite::comparison_settings` resolves library settings for
inspection without running workloads.

For raw null-distribution exports, use `hypothesis::welch_distribution` or
`analysis::compare_with_distribution`. Every resample is retained in generation
order. JSON uses tagged finite values and explicit positive/negative infinity
variants; unavailable tests have an empty distribution with a reason. Summary-only
comparisons retain no distribution vector. The benchmark crate enables exact JSON
float round trips so exported statistics preserve their original numeric values.

Use `--hypothesis-distribution` to include every null resample in direct Cargo
comparison reports. Named-baseline and previous-run comparisons write
`comparison.json` alongside other artifacts when `--output` is supplied. In JSON
console mode the same distribution is included in `BENCH_BASELINE` or
`BENCH_COMPARISON`. Summary-only output remains the default. An unavailable test
has an empty distribution and an explicit reason.

```sh
cargo bench --bench mybench -- --load-baseline candidate --baseline main \
  --hypothesis-distribution --hypothesis-resamples 20000 --output new-analysis
```

The flag requires a named comparison or enabled previous-run history. Smoke and
profiling modes reject it. Comparison export failures do not publish a new
previous-run baseline. Multi-source reanalysis writes the file separately for
each source under the existing numbered output directories.

With `--hypothesis-distribution --output <dir>`, direct Cargo comparisons also
write `<dir>/comparison.html` beside `comparison.json`. The portable HTML embeds
Welch null-distribution SVG charts for each compared case and metric, including
observed t, two-sided p, tail counts and independent-unit counts. Unavailable tests
are labelled explicitly. No browser or external chart service is needed.
An export failure prevents history publication or named-baseline replacement;
already written output files may remain for diagnosis. Existing files are never
overwritten. Library users can call `HistoryReport::export` for the same output.

The process runner can render the same charts from two saved runs, without
executing benchmark programs:

```sh
cargo airbug-bench compare baseline/run.json candidate/run.json \
  --html --hypothesis-resamples 10000 --hypothesis-seed 7 \
  --output comparison.html
cargo airbug-bench compare baseline/run.json candidate/run.json \
  --json --hypothesis-distribution --hypothesis-resamples 10000 \
  --hypothesis-seed 7 --output comparison.json
```

`--html` and `--json` are mutually exclusive. `--hypothesis-distribution` requires
`--json`; HTML captures draws automatically. Paired comparisons keep their paired
analysis and explicitly report that an independent-sample Welch chart is absent.

For custom summary charts, `viz::charts::scatter_scaled` accepts independent
`AxisScale::Linear` or `AxisScale::Logarithmic` settings for X and Y. All series
share axis bounds. Log axes require positive values; invalid point pairs are
removed before fitting or thinning and the caption reports the omitted count.
The existing `scatter` helper keeps linear axes by default.

`report::html_run_with_summary(&run, Some(report::SummaryPlot { estimator: Default::default(), parameter: "size",
scale: viz::charts::AxisScale::Logarithmic }))` adds input-size summary charts to
an ordinary run report. Register the numeric input with `.parameter("size", n)`.
X uses that contract value; Y is the median of independent process medians.
Different metric contracts receive separate plots. Missing or nonnumeric inputs
are counted explicitly. `report::parameter_charts` renders only this section.

The runner exposes these summaries for saved runs and collections:

```sh
cargo airbug-bench report ./results --summary-parameter size \
  --summary-scale logarithmic --output report.html
```

The default summary scale is `linear`. `--summary-scale` requires
`--summary-parameter`; summaries require an `.html` output path. Each loaded run
gets its own summary section, keeping runs and their measurement contracts apart.

Direct Cargo targets accept the same summary options:

```sh
cargo bench --bench mybench -- --summary-parameter size \
  --summary-scale logarithmic --output results
```

This writes `results/report.html` alongside the recorded run and summaries.
Summary charts require `--output` and cannot be combined with smoke tests or
profiling. They also work with `--load-baseline` without executing workloads.

`compare baseline candidate --html` also overlays per-process regressions when
both runs contain varying operation counts for a matching batch-total metric.
Each process keeps its own samples, fit and confidence wedge. Axes are shared;
lines stop at each process's largest observed batch. The HTML states the
per-process confidence levels (derived from `1 - alpha`, bounded by floating-point
precision); these descriptive slope intervals are not family-adjusted tests.

HTML experiment reports from `report candidate --baseline baseline -o report.html`
also include these per-process regression overlays. They use the baseline snapshot
already loaded and checked for comparison. Incompatible workload or metric
contracts remain explicit report issues and do not produce a regression overlay.

Direct Cargo baseline comparisons with bootstrap analysis also write
`regression-comparison.html`:

```sh
cargo bench --bench mybench -- --baseline previous --resamples 10000 \
  --output results
```

The chart uses the selected immutable baseline for each compatible case and keeps
its processes separate from candidate processes. Missing/incompatible baselines
are labelled. With no eligible varying-operation fits, the report explains that
no regression overlay is available. Export completes before any requested named
baseline is saved or replaced.

Bootstrap HTML reports include Gaussian kernel-density curves and a Tukey-labelled
observation rug. Each curve uses every observation in its stated population:
process medians for multiple processes, normalized batches for one process.
Constant populations are labelled point masses; empty or numerically
unrepresentable densities are explicit. Compatible baseline comparisons overlay
both populations on shared axes, keeping their population labels visible.

To retain the absolute bootstrap draws in direct Cargo output, add
`--bootstrap-distributions`:

```sh
cargo bench --bench mybench -- --resamples 10000 \
  --bootstrap-distributions --output results --json
```

`estimates.json` and `BENCH_ESTIMATES` include aligned arrays for mean, median,
standard deviation and median absolute deviation, in original units and generation
order. Each array has `resamples` entries. The flag enables bootstrap analysis
with defaults when no other analysis options are supplied, requires `--output` or
`--json`, and rejects smoke/profile modes. Without it, these arrays are omitted.

With retained bootstrap draws, `estimates.html` includes one density chart for
each statistic. The shaded region uses the reported percentile confidence bounds;
the red line is the original sample estimate. KDE smoothing does not recalculate
the interval. Constant bootstrap distributions remain explicit point masses.

For varying-operation batch totals, `--bootstrap-distributions` also retains
`slope_distribution` on each process regression. These are paired-bootstrap draws
in metric units per operation. `estimates.html` plots each slope distribution with
its point estimate and confidence interval. The optional JSON field is absent
when capture is disabled; older reports continue to deserialize.

Analyze a saved run without launching any workload:

```sh
cargo airbug-bench analyze results/run.json --resamples 10000 \
  --analysis-seed 7 --bootstrap-distributions --output analysis.json
cargo airbug-bench analyze results/run.json --bootstrap-distributions \
  --format html --output analysis.html
```

`analyze` accepts `--confidence-level`, `--resamples`, `--analysis-seed` and optional
`--bootstrap-distributions`. Formats are `json` (default), `html`, and `markdown`.
It uses existing run/baseline resolution, leaves source artifacts unchanged and
refuses to overwrite an existing output file.

### Расчёт точек на сводном графике

Для графика по числовому параметру используйте `--summary-parameter arg --output NEW_DIRECTORY`.
По умолчанию точка — медиана медиан независимых процессов. Опция
`--summary-estimator mean` выбирает арифметическое среднее нормализованных замеров,
как на сводных графиках Criterion: каждый batch имеет одинаковый вес.
Throughput в этом режиме — средняя работа / среднее время, а не среднее скоростей.
Выбор влияет только на сводный график, не на решения о регрессиях.
Опция доступна и в `cargo bench -- …`, и в `cargo airbug-bench report …`.

При именованном сравнении `cargo bench -- --baseline NAME --output NEW_DIRECTORY`
основной `report.html` также показывает замеры baseline и candidate на общих осях.
Каждый процесс — отдельная серия; этот график не требует `--resamples`.
Несовместимые кейсы не накладываются друг на друга.

Чтобы сохранить численные отчёты без генерации графиков, используйте
`cargo bench -- --no-plots`. Флаг действует на основной HTML, bootstrap-оценки
и сравнения с baseline. JSON и явно запрошенные распределения сохраняются;
`--no-plots` несовместим с настройками `--summary-*`.

В builder API задайте `suite.plots(false)` перед `suite.main()`, чтобы отключить
графики по умолчанию. CLI `--plots` включает их обратно, `--no-plots` явно
отключает. Одновременно указывать оба флага нельзя. Настройка относится ко всему
отчёту suite и не меняет измерения или критерии сравнения.

### Пользовательская измеряемая величина

`measurement::Measurement` задаёт тип состояния начала замера, тип значения,
`start/end`, `zero/add` и преобразование результата в `f64`.
`Suite::bench_measured(name, measurement, BatchPolicy, workload)` измеряет
синхронную функцию и сохраняет пользовательскую метрику рядом с `wall`.
Метрика должна иметь отдельный идентификатор и статистику `batch_total`.
Время калибровки и лимитов остаётся в наносекундах; значение пользовательской
метрики не используется как длительность. Уничтожение результата функции входит
в измеряемую операцию. `bench_measured_with_input` дополнительно принимает setup, функцию с `&mut Input`
и `DropPolicy`: создание и уничтожение входа вне измерения, уничтожение результата
внутри или снаружи по выбранной политике. `bench_measured_with_owned_input` передаёт свежий вход по владению: уничтожение
входа самой функцией входит в замер, возвращённого входа — следует `DropPolicy`.
`bench_async_measured_with_input` принимает фабрику executor и async-функцию
с заимствованным входом. Executor создаётся лениво вне замера; создание и опрос
future входят в измеряемый интервал. `bench_async_measured` измеряет async-функцию без входа;
`bench_async_measured_with_owned_input` передаёт вход во владение future, сохраняя
правила `DropPolicy`. Форматирование и threaded-варианты этого API ещё не реализованы.

Полный пример собственного счётчика: `examples/custom_measurement.rs`.
Его можно запустить в этом репозитории:

```sh
cargo run -p airbug-bench --release --example custom_measurement -- --iterations 130 --samples 3 --warmup-ms 0
```

Он записывает `work_units` рядом с `wall`: синтетические 390 единиц на 130 операций.
Это пример подключения API, а не аппаратный счётчик инструкций.

Пользовательский `measurement::ValueFormatter` задаёт преобразования для человека,
throughput и машинного вывода. `measurement::format_observations` применяет его
к выбранной метрике сохранённого `Run`, нормализуя batch totals и сохраняя пропуски.
Результат можно сериализовать или передать в `measurement::markdown`, затем
`report::html`. Исходный `Run` не меняется. `suite.formatter("metric_id", formatter)?` подключает его к последнему кейсу.
`Suite::main` выводит человеческую таблицу в терминал и основной HTML, а также
сохраняет оба представления в `formatted.json` и `BENCH_FORMATTED` при `--json`.
Загруженные baseline должны иметь совпадающий контракт метрики. В `run.json` сохраняется также снимок представления с хешем исходных
наблюдений и контракта метрики. Команда `report` восстанавливает эту таблицу
без пользовательского кода; при изменении исходных данных показывает явное
сообщение об устаревшем представлении. Применение форматтера к графикам
и повторный расчёт форматирования в standalone-командах пока не подключены.
