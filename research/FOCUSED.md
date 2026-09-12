# Ключевые подходы

Дата: 2026-09-06. Ниже — решения по проверенным фрагментам исходников и документации. «Взять» означает реализовать подход под требования rbench, а не скопировать код или признать чужие численные заявления доказанными. Локальные снимки исходников: `focused/`; контрольные суммы архивов: [manifest](focused/manifest.json). Архивы получены через HEAD и фиксируются хешами содержимого, не привязаны к подтверждённым commit SHA.

| Проект / прочитанный источник | Полезный подход | Решение для rbench |
|---|---|---|
| [Criterion](https://github.com/criterion-rs/criterion.rs), `src/routine.rs`, `measurement.rs`, `bencher.rs`, `stats/` | Прогрев, подбор числа итераций, различные границы setup/drop, независимый измеритель, статистика | Взять разделение измерения и анализа; сохранять исходные batches. Не строить весь протокол вокруг единственного скаляра времени |
| [Divan](https://github.com/nvzqz/divan), `src/stats/sample.rs`, `alloc.rs`, `time/timer.rs`, README | Атрибуты, параметры, счётчики, предварительно выделенные samples, TLS allocator | Взять эргономику; области учёта аллокаций описывать явно. В исходнике отмечено отсутствие учёта чужих unmanaged threads: это важно для wgpu/async |
| [BenchmarkDotNet](https://github.com/dotnet/BenchmarkDotNet), `Engines/Engine.cs`, `EngineStage.cs` | Замороженная конфигурация job, этапы, lifecycle, диагностика, сохранение первоначальной ошибки при сбое cleanup | Взять job/worker/protocol. JIT/GC-специфичные стадии не переносить в Rust по умолчанию |
| [Tango](https://github.com/bazhenov/tango), `tango-bench/src/{worker,protocol}.rs`, README | Парные сравнения разных executable, команды worker, общий seed и подготовка sampler | Взять пары и контроль идентичности; собственный первый runner работает через процессы и сообщения. Shared-memory синхронизацию не включать до измерения необходимости |
| [Zenbench](https://github.com/imazen/zenbench), `src/{engine,stats}.rs`, README | Перемешивание раундов, paired differences, floor разрешения таймера, анализ дрейфа | Взять сохранённый порядок и диагностику. Не брать выбор лучшего из процессов как основной регрессионный результат; sequential stopping потребует отдельной статистической проверки |
| [Zench](https://github.com/envidera/zench), README | Программный доступ к результату, компактный API, проверки бюджета | Взять API отчёта. Обычный параллельный `cargo test` не делать строгим измерительным режимом |
| [Gungraun](https://github.com/gungraun/gungraun), `crates/gungraun-runner/src/summary/model.rs`, README | Отдельный runner, типизированные результаты разных Valgrind tools, абсолютные и относительные регрессии | Опциональный backend для инструкций/симуляции; метрики не преобразовывать в реальное время. Возможность зависит от платформы |
| [Iai](https://github.com/bheisler/iai), README | One-shot instruction-oriented profiling | Исторический источник подхода; новый адаптер ориентировать на Gungraun |
| [Google Benchmark](https://github.com/google/benchmark), README, дерево `src/` | Отдельный harness и богатая экосистема benchmark fixtures | Источник API-кейсов; runtime Rust не должен получать C++-зависимость |
| [Hyperfine](https://github.com/sharkdp/hyperfine), `src/benchmark/{executor,mod,scheduler}.rs`, `timer/wall_clock_timer.rs` | Командные сценарии, prepare/setup/cleanup, разные способы запуска | Process driver нужен для Forma, rrrah и JS-компилятора. Аргументы передавать массивом, shell только явно |
| [Bencher](https://github.com/bencherdev/bencher), README | Отдельные CLI/API/хранилище/представление, testbed и история | Локальный формат и экспорт изначально; сервер и облако не требуются для первого релиза |
| [HdrHistogram Rust](https://github.com/HdrHistogram/HdrHistogram_rust), README | Ограниченная память для больших latency-потоков, известная точность квантования | Для event latency; обычные microbench batches сохранять сырыми. Фиксировать bounds, precision, overflow и auto-resize |
| [measureme](https://github.com/rust-lang/measureme), README, дерево исходников | Профилирование компилятора — отдельная диагностическая задача | Файл профиля как привязанный артефакт; не смешивать его с wall-time статистикой |
| [tracing](https://github.com/tokio-rs/tracing), дерево crates | Отдельный слой инструментирования и потребителей событий | Адаптер диагностики; не форматировать события в горячем измерительном цикле |
| [Tracy](https://github.com/wolfpld/tracy), README, дерево исходников | Детальный CPU/GPU timeline | Внешний профиль для разбора причин, не обязательная зависимость ядра |

## Дополнительные кандидаты, проверенные по документации

| Источник | Применение |
|---|---|
| [wgpu-profiler](https://github.com/Wumpf/wgpu-profiler) | Вложенные timer scopes, пул queries и отложенное получение результатов без остановки устройства. Хороший ориентир GPU-адаптера; не готовая замена оконного benchmark Forma |
| [profiling](https://github.com/aclysma/profiling) | Тонкий фасад над profiler backends. Поддерживает решение держать диагностику за feature gates |
| [Binggan](https://github.com/PSeitz/binggan) | Плагины, peak memory, interleaving, perf. Сильный Rust-кандидат помимо трёх первоначальных библиотек |
| [Bustle](https://github.com/jonhoo/bustle) | Смеси операций и параметры конкурентной нагрузки для коллекций. Нужен сценарий workload, а не только функция без состояния |
| [BenchExec](https://github.com/sosy-lab/benchexec) | Управление ресурсами и процессами; portable ядро отдельно от Linux-изоляции |
| [nanobench](https://github.com/martinus/nanobench) | Компактность и диагностика ненадёжного результата; не объявлять точность по красивой единице ns |
| [BenchmarkTools.jl](https://github.com/JuliaCI/BenchmarkTools.jl) | Разделение групп, выполнения и сравнения; шумоустойчивость как часть методики |
| [CodSpeed](https://github.com/CodSpeedHQ/codspeed) | CI-проверки и отдельные способы измерения. Заявления о дисперсии — заявления авторов, здесь не перепроверены |
| [dhat-rs](https://github.com/nnethercote/dhat-rs) | Точные ожидания числа allocations и пиковой heap-памяти. В README есть предупреждение об экспериментальном статусе; не делать обязательным фундаментом |
| [samply](https://github.com/mstange/samply) | Удобный внешний CPU-профиль на macOS/Linux/Windows |
| [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark) | Реальные операции UI и проверка реализации. Не сравнивать retained/native и DOM по разным контрактам работы |
| [Benchopt](https://github.com/benchopt/benchopt) | Разделение входных данных, алгоритма и воспроизводимого эксперимента; межъязыковой driver |

## Проверенные ограничения чужих сравнительных таблиц

Таблица Zenbench в README приписывает Criterion отсутствие baseline и deferred drop. Это нельзя использовать как источник: в локальном Criterion есть `save_baseline`, `retain_baseline`, `iter_with_large_drop` и `iter_batched`. Архитектуру выбираем по собственным исходникам каждого инструмента, а не по маркетинговой таблице конкурента.

Ни одна из изученных библиотек сама по себе не делает CPU fallback Forma и GPU present семантически одинаковыми. Проблема решается контрактом эксперимента и проверкой результата, а не выбором статистического теста.

## Почему своя библиотека оправдана

Причина — объединение уже существующих механизмов Forma/camera-man/rrrah в общий протокол: разные области времени и памяти, окна с main-thread lifecycle, готовые A/B executable, проверка fixtures, частично недоступные метрики и связанные проверки изображения. Сам по себе новый `#[bench]` не даёт достаточной ценности. Criterion или Divan могут остаться совместимыми входными форматами/адаптерами, но не должны ограничивать модель rbench.
