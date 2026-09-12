# Исследование для собственной benchmark-библиотеки rbench

Дата: 6 сентября 2026.

## Решение

Проектировать **свою библиотеку + отдельный runner + единый протокол результатов**. Основная ценность — поддержка сценариев Forma с корректными границами CPU/GPU/window, воспроизводимыми A/B запусками и явной сопоставимостью метрик. Одна обёртка над Criterion/Divan оставит важнейшие потребности в отдельных самописных скриптах.

Брать архитектурные идеи из BenchmarkDotNet, модель статистического измерения Criterion, эргономику Divan, paired scheduling Tango/Binggan/Zenbench, process workflow Hyperfine, опциональную диагностику Gungraun и отложенные GPU queries wgpu-profiler. Не копировать все возможности в первый релиз.

Главные документы: [требования](../docs/REQUIREMENTS.md), [архитектура](../docs/ARCHITECTURE.md), [методика измерений](../docs/MEASUREMENT.md), [реализация по этапам](../docs/ROADMAP.md), [сравнение ключевых проектов](FOCUSED.md).

## Объём и глубина исследования

| Уровень | Объём | Что действительно сделано |
|---|---:|---|
| Discovery | 779 записей | Восемь запросов GitHub Search + явно выбранные benchmark-проекты; URL, описание, default branch, поиск-источник |
| Получена документация | 763 README | Сохранены локально, SHA-256, URL; автоматическое извлечение тематических фрагментов с номерами строк |
| Итоговый корпус | **614 репозиториев** | Просмотрены описания и извлечённые фрагменты; оставлены benchmark/profiling инструменты и потенциальные workload-примеры |
| Исключены | 148 | False positives, evaluation задач вместо системной производительности, нерелевантные продукты и слишком бедные README |
| Без README | 16 | Не засчитаны в анализ содержимого |
| Alias | 1 | Старое `bheisler/criterion.rs`; канонический `criterion-rs/criterion.rs` считается один раз |
| Снимки исходников | 15 публичных проектов | Скачаны выборочные текстовые исходники из архивов; один дополнительный архив не найден |
| Углублённая проверка движков | 7 проектов | Criterion, Divan, BenchmarkDotNet, Tango, Zenbench, Gungraun, Hyperfine: прочитаны ключевые участки execution/measurement/protocol/stats |
| Расширенный shortlist | 27 проектов | 15 основных + 12 дополнительных; разная глубина источников явно указана в FOCUSED.md |
| Проекты владельца | Forma + 8 других | Forma подробно; 6 существующих measurement implementations прочитаны, UXI/Voxy рассмотрены как потенциальные потребители ограниченной глубины |

Число 614 **не означает** полный source audit 614 проектов или доказательство, что все они — качественные benchmark frameworks. Это широкий документальный скрининг с последующим углублением ключевых архитектур. Никакие upstream throughput/accuracy заявления не перепроверены запуском. GPU/frame/window эксперименты в рамках этого исследования не выполнялись.

## Воспроизводимые артефакты

- [repositories.json](repositories.json): исходный каталог и хеши README.
- [screening.json](screening.json): извлечённые свидетельства с номерами строк, категория и итоговое решение.
- [REPOSITORIES.md](REPOSITORIES.md): карточки всех записей.
- [DECISIONS.tsv](DECISIONS.tsv): retained/excluded/unavailable/alias для каждой записи.
- [manual-exclusions.json](manual-exclusions.json): дополнительный отбор после чтения фрагментов.
- `search-0.json` … `search-7.json`: ответы GitHub Search.
- `evidence-review-*.txt`: компактные фрагменты, прочитанные при отборе; состав соответствует промежуточному корпусу до последнего исключения.
- [focused/manifest.json](focused/manifest.json): хеши архивов ключевых проектов и ошибки получения.
- [audit.json](audit.json): проверка локальных snapshots, счётчиков и ссылок.

Повторение сбора: `python3 research/scripts/collect.py`, затем `screen.py`, `curate.py`, `finalize.py`. Сбор использует публичные endpoints, без выполнения скачанного кода. Скрипт архивов `focus.py` — отдельный шаг. Private Forma копировалась через авторизованный GitHub CLI и исключена из будущего Git содержимого rbench.

## Поисковая стратегия и ограничения

Запросы: `benchmark language:Rust`, `benchmark framework`, `microbenchmark`, `profiling language:Rust`, `performance testing`, `rendering language:Rust`, `database language:Rust`, `benchmark language:C++`. По каждому — до 100 результатов по stars; совпадения объединены по полному имени, GitHub-fork результаты отсеяны. Старый адрес Criterion дополнительно исключён.

Stars определяют доступный срез discovery, а не качество методики. Корпус смещён к популярным и хорошо описанным проектам; слово profiling даёт ложные совпадения с account profiles, benchmark — с AI evaluation. Это причина документированного отсева, а не основание считать такие проекты источником архитектуры.

README keywords — указатели для чтения. Например, `setup.py` не доказывает lifecycle hooks, «JSON» не доказывает экспорт benchmark результатов. Отсутствие сигнала в коротком README Divan также не означает отсутствие возможностей. Ни один keyword count не используется как feature score.

Снимки получены по default branch/HEAD в разные моменты и не являются атомарным снимком всего GitHub. Их локальное содержимое фиксируется хешами. Полный commit SHA установлен для Forma; для публичных архивов не утверждается commit pinning.

## Что дал широкий корпус

1. **Microbench API и экспериментальный runner — разные задачи.** Много небольших библиотек хорошо измеряют функцию, но пользовательские проекты отдельно реализуют процессы, входные manifests и отчёты. Это обосновывает выделение управляющего слоя.
2. **Есть существенные подходы за пределами первоначальной тройки.** Tango и Binggan расширяют варианты interleaving; Hyperfine, BenchExec и Bencher показывают разные части execution/изоляции/history. Нельзя выбирать архитектуру только из Criterion/Divan/Callgrind.
3. **Workload contract важнее единого синтаксиса.** Serialization benchmarks требуют одинаковой семантики output; storage — reset/cache policy и смеси read/write; renderer — одинакового качества и completion boundary. Поэтому Case должен описывать выполненную работу.
4. **UI performance не сводится к CPU loop.** Источники для векторной графики, текста, retained rendering, browser UI и terminal output показывают разные точки наблюдения. В частности, вывод в PTY не измеряет завершение отрисовки терминалом. Аналогичная граница существует между GPU submit и present Forma.
5. **Диагностика должна подключаться независимо.** CPU sampling, allocation tracking, GPU timers и traces имеют собственную область и стоимость. Их результаты сохраняются как отдельные типизированные observations и артефакты.
6. **Скорость, ресурсы и корректность не образуют один score.** Быстрее с худшим AA, другим input token stream или исключённым transfer — другой эксперимент. rbench нужен validity gate, а не только p-value.

## Что особенно важно в Forma

В актуальном прочитанном коде уже есть хорошие заготовки общего runner: AB/BA по готовым executable, manifests, checksums, проверки adapter/DPI и полный case set. Необходимость своей библиотеки проявляется в ручной сборке JSON, дублируемых collectors, потере raw frame arrays при экспорте и разрозненных фазовых метриках.

Первый полезный результат реализации — импорт существующих Forma runs с сохранением их семантики, затем общий process runner. После этого — Rust microbench API и нативный scenario adapter. Такой порядок проверит архитектуру на реальной потребности до того, как будет закреплён удобный, но слишком узкий macro API.

## Что остаётся проверить при реализации

Накладные расходы hot loop, корректность статистического coverage/false-positive rate, overhead instrumentation, portable allocator semantics, реальное завершение GPU queries и native window lifecycle. Это требования к последующим экспериментам, а не выполненные проверки нынешнего исследования.
