# Реализация и проверка

Реализован первый рабочий срез этапов 1–7: модель, импорт Forma, runner, microbench API, offline-анализ, recorder, allocator и CLI. Полная приёмка этапов ниже ещё не закрыта. Текущие проверки и ограничения перечислены в [VALIDATION.md](VALIDATION.md); этот документ сохраняет расширенные критерии следующих итераций.

## 1. Model + importer Forma

Сделать descriptors, observations, manifests, availability, completeness и immutable run storage. Первым потребителем будет импорт текущих `metadata.json/results.json` и paired run files Forma.

Приёмка: существующие сравнения получают те же исходные значения; явно сохраняются median-of-process-p95, отдельная GPU-фаза, null timestamps, scope CPU fallback. Missing/duplicate cases, zero baseline и mismatch fixtures отклоняются. Импорт не генерирует отсутствующие raw frame samples.

## 2. Process runner

Запуск подготовленных executable, deterministic AB/BA schedule, argv, deadlines, resource leases, logs, binary/fixture hashes и complete/partial state. Компиляция отдельной командой.

Приёмка: реальные child fixtures success/panic/abort/hang/partial JSON; остановка и уборка процессов; первая ошибка сохранена; binary/fixture mutation выявляется; запуск разных A/B сборок без Rust ABI. Измерительные процессы не перекрываются по exclusive resource.

## 3. Rust microbench API

Suite builder, stable case IDs, параметры, bounded calibration, raw batches, setup/reset/drop policies. Macro sugar добавляется после проверки builder на tokenizer и mutable sort fixture.

Приёмка: корректный подсчёт операций, setup вне выбранного timing scope, reset возвращает исходный input, Drop включается/исключается согласно политике, память bounded. Hot-loop overhead сравнивается с простым handwritten loop, Criterion и Divan на одной машине и одинаковой полезной работе. Никаких обещаний «нулевого overhead» до измерений.

## 4. Статистика и gates

Offline analysis, process-pair estimates, compatibility policy, практический threshold, CI, multiple-testing policy, explicit inconclusive. Фиксированный confirmatory plan прежде sequential stopping.

Приёмка: reproducible synthetic IID/heteroscedastic/drift/autocorrelated/quantized/missing данных; заданные известные эффекты и нулевая гипотеза. Измерить false-positive rate и coverage с Monte Carlo uncertainty; задать допустимые диапазоны заранее. A/A реальные запуски на локальной машине и CI, затем контролируемый A/B с известной добавочной работой. Bootstrap unit строго соответствует process/block IDs.

## 5. Полноценный Forma scenario adapter

Заменить ручную сборку JSON в отдельных benchmark executables на recorder; начать с offscreen, затем window main-thread lifecycle. Добавить raw submit/completed observations и phase-aware GPU data. Сценарии и renderer не менять ради сравнения.

Приёмка: same-image validation до/после миграции; совпадение существующих счётчиков в пределах объявленной методики; actual Metal run; offscreen и native window отдельно. Occluded/timeout/no-adapter/missing-query — проверенные отрицательные пути. Успех одного offscreen теста не закрывает native window acceptance.

## 6. Метрики и диагностика

Generic allocator wrapper, OS CPU/RSS providers, собственные GPU counters, timestamp adapter; затем tracing/samply/Gungraun/perf.

Приёмка: failed realloc/shrink/zeroed allocation/thread coverage; фазовый peak и lifetime peak не смешиваются; profiler perturbation измеряется отдельным A/B. Реальные GPU tests на нужных backends, корректная привязка query к кадру. Unsupported — полноценный результат capability negotiation.

## 7. Удобство и переносимость

`cargo rbench list/build/run/compare/import/report`, macro API, автономный HTML, JSON/Markdown, документация и примеры tokenizer/Forma/rrrah. Browser driver и async workloads после native контракта.

Приёмка: пользователь может повторно проанализировать run offline; отсутствуют обязательные Node/GPU/network зависимости CPU-библиотеки; default core собирается на macOS/Linux/Windows. Feature matrix проверяет отсутствие wgpu/winit/Tokio в минимальном графе.

## Первая полезная версия

Этапы 1–3 дают практическую ценность: общий runner и единый формат вместо дублирования скриптов. Строгие заявления о статистической регрессии появляются после этапа 4. Название «готово для Forma» требует также этапа 5 и реального оконного измерения.

Не начинать с dashboard, proc macro DSL или попытки переписать profiler. Самый рискованный ранний контракт — связь workload, фаз, наблюдений и сравнимости; именно его проверяем первым.
