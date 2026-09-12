# Первые десять улучшений

Реализованы пункты 1, 2, 4, 7, 12, 15, 22, 31, 41 и 43 из списка предложений. У Forma image-сценарий пока явно Unsupported: в исследованном native renderer нет Image primitive. Оконный режим не входит в эту итерацию.

## Подключение нового проекта

Установите CLI: `cargo install --path crates/cargo-rbench --offline` из checkout rbench. Либо используйте абсолютный путь к `target/release/cargo-rbench` после release-сборки. Локальный alias работает только внутри checkout rbench.

```sh
cargo rbench init --manifest-path /path/to/project/Cargo.toml
cargo rbench discover --manifest-path /path/to/project/Cargo.toml --offline
cargo rbench bench --manifest-path /path/to/project/Cargo.toml \
  --offline --repetitions 12 -o .rbench/first
```

`init` добавляет dev-dependency на локальную библиотеку, `[[bench]]` с `harness=false`, регистрацию target, пример проверяемой сортировки с тремя размерами и `rbench.json`. TOML-комментарии сохраняются. Существующие файлы не заменяются. Для virtual workspace нужно выбрать manifest одного package. Если исходники библиотеки перемещены, передайте `--library-path /path/to/rbench/crates/rbench`.

`discover` показывает все Cargo bench targets workspace и их регистрацию. `bench` сначала собирает **все выбранные targets**, затем измеряет их последовательно. Для существующего rbench target добавьте:

```toml
[package.metadata.rbench]
targets = ["my_benchmark"]
```

Не отмечайте обычный libtest/Criterion target: регистрация означает, что executable поддерживает `RBENCH_RESULT` protocol. Точный выбор: `--target package/target`, можно повторять. Результаты отдельных targets находятся в пронумерованных подкаталогах; корень содержит `targets.json`. Это коллекция запусков, а не единый Run: для baseline/report/gate выберите подкаталог target.

Настройки worker передаются после `--`, например `-- --samples 8 --warmup-ms 10 --sample-ms 1`. Прогресс идёт в stderr: target, номер процесса, baseline/candidate, оценка оставшегося времени; для долгого процесса показывается последняя фаза worker. ETA оценивается по завершённым процессам, не гарантируется при разных workloads. Диагностика supervisor может немного влиять на host load.

## Автодополнение оболочки

```sh
# bash: подключить на текущую сессию
source <(cargo rbench completions bash)
# zsh: сохранить в каталог из $fpath
cargo rbench completions zsh > ~/.zfunc/_cargo-rbench
# поддерживаются также fish, powershell и elvish
cargo rbench completions fish > ~/.config/fish/completions/cargo-rbench.fish
```

`completions SHELL` печатает скрипт автодополнения в stdout и не изменяет конфигурацию оболочки. Скрипт дополняет установленный бинарник `cargo-rbench`; неизвестное имя оболочки отклоняется. Закрытый downstream-канал (например, `| head`) не считается ошибкой.

## Именованные baseline

```sh
cargo rbench baseline save main .rbench/first/0-my-package-rbench
cargo rbench baseline list
cargo rbench compare @main .rbench/second/0-my-package-rbench --check
cargo rbench report @main -o .rbench/main.html
```

Имя разрешает буквы ASCII, цифры, `-` и `_`. Сохранение создаёт неизменяемую ссылку на canonical path и SHA-256 файла результата; данные не копируются. Перезапись имени запрещена: используйте новое имя. Удаление исходного run сломает ссылку, изменение его содержимого обнаруживается. Хранилище по умолчанию `.rbench` относительно текущего каталога; общий каталог задаётся глобальным `--store PATH`.

## Матрицы и проверка правильности

```rust
use rbench::{DropPolicy, Suite};
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
                else { Err(rbench::error("unordered result")) }
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
cargo rbench gate .rbench/candidate --config budgets.json --baseline @main
```

`case` и `metric` совпадают **точно**. `unit` обязательна. `min`/`max` проверяются на каждом candidate observation, batch totals нормализуются на число операций. `max_regression_percent` использует сравнительный анализ независимых процессов; без `--baseline` требуется paired run. Для нескольких относительных бюджетов применяется общая поправка Bonferroni. Budget на нулевом baseline задавайте абсолютным пределом.

Коды: 0 — все прошли, 1 — хотя бы один провален, 2 — недоступность/неопределённость/ошибка без установленного провала. Отсутствующая метрика, несовпадение единиц и пустая конфигурация не дают успешный результат. Пример, созданный `init`, содержит демонстрационный предел; настройте его под свой workload. [Бюджеты Forma](../integrations/forma/budgets.json) проверяют отсутствие geometry uploads в static/hover/animation.

## HTML-отчёт

`report RUN -o report.html` создаёт автономную таблицу с поиском строк, сортировкой колонок, раскрываемыми case contracts, окружением и печатью/PDF через браузер. Внешних скриптов/шрифтов нет. Исходные данные остаются в `run.json`. Значения и пользовательские строки экранируются; HTML из metadata не исполняется.

## Сценарии Forma и контроль кадров

```sh
cargo build --release --manifest-path integrations/forma/Cargo.toml --offline
integrations/forma/target/release/rbench-forma-example --list
integrations/forma/target/release/rbench-forma-example \
  --record-goldens .rbench/my-forma-goldens
# Просмотрите созданные PNG как эталоны, затем:
cargo rbench run --program integrations/forma/target/release/rbench-forma-example \
  --protocol --repetitions 3 -o .rbench/forma-checked -- \
  --goldens .rbench/my-forma-goldens --json
cargo rbench gate .rbench/forma-checked --config integrations/forma/budgets.json
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
