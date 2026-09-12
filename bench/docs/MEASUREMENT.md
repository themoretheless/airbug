# Модель измерения и доверия к результатам

Это спецификация будущей реализации. Статистические процедуры должны быть проверены на синтетических данных и контрольных A/A-прогонах до появления строгого CI gate.

## Метрика — больше, чем число

Каждый `MetricDescriptor` содержит:

- Стабильный ID и единицу: ns, bytes, calls, frames, tokens, count; throughput вычисляется из соответствующего объёма работы и интервала.
- Вид: duration, monotonic counter delta, gauge, peak, distribution, ratio.
- Scope: current thread, registered threads, whole process, owned GPU resources, GPU pass, window.
- Phase и границу завершения: например, submit-return, GPU-completed или present-call-return.
- Направление оптимизации: lower/higher/neutral. CPU utilization — neutral, не универсально «меньше лучше».
- Способ наблюдения и clock domain, точность/разрешение, instrumentation mode.
- Доступность: `Available`, `Unsupported`, `PermissionDenied`, `Invalid`, `NotApplicable`, `Incomplete`; причина и requested/valid sample counts.

Пример различий, обязательных для Forma:

| ID | Смысл | Нельзя интерпретировать как |
|---|---|---|
| `cpu.submit.ns` | Время подготовки и подачи команд в объявленной фазе | GPU time |
| `frame.completed.ns` | Wall time до подтверждённого завершения кадра | Время самого GPU pass |
| `gpu.pass.ns` | Разность GPU timestamps одного pass | Полное время resize/present/readback |
| `window.present.calls_per_second` | Частота успешных present calls | Количество уникальных кадров scanout |
| `alloc.requested.bytes` | Трафик успешных alloc/realloc | Удерживаемая память или leak |
| `alloc.live.bytes` | Запрошенные живые Rust bytes | RSS |
| `process.rss.lifetime_peak.bytes` | Highwater за жизнь процесса | Peak данной фазы |
| `gpu.owned_buffer.bytes` | Известные renderer buffers | Полная VRAM/residency |

`null` без статуса недостаточен. Отсутствие обязательного GPU adapter проваливает сценарий. Отсутствие необязательных timestamp queries сохраняет CPU-результат, а GPU-метрику помечает недоступной. Ошибка окна `Occluded` не означает 0 FPS.

## Единица наблюдения

У каждого observation есть `run_id`, `case_id`, `phase_id`, `process_id`, `block_id`, `pair_id?`, `sequence`, `operations`, `elapsed_ticks`, `clock`, `status`.

Batched 100 операций дают один замер средней стоимости batch/100. p99 из таких замеров — p99 средних batch, а не p99 отдельных операций. Кадры имеют временную зависимость; 240 кадров одного процесса не равны 240 независимым процессам. По одному mean/p95 из существующего Forma импортёр не выдумывает raw frames.

Количество измеренных операций, pipeline depth, concurrency, warmup и reset policy сохраняются. При разрешении сравнивать разные sample counts анализ использует соответствующую схему; самодельное усечение длинной серии до короткой запрещено.

## Сбор данных

1. Подготовить и проверить fixture вне основного интервала; сохранить семантику результата, seed и hashes. Input generation входит в timing только по явному запросу.
2. Провести ограниченный pilot: подобрать batch count до измеримой длительности с лимитами времени и памяти. Итерации не растут бесконечно при нулевом/квантованном времени.
3. Выбрать режим cold-start или warmed-state. Для холодного запуска прогрев отсутствует по определению. Для stateful workload нужен reset после pilot, иначе pilot уже изменил базу/кеш.
4. Заморозить confirmatory schedule до основной серии. Сохранять AB/BA order, seeds и process boundaries.
5. Собирать samples в заранее выделенный буфер; сериализовать после batch. Не печатать progress из измеряемой функции.
6. Сохранить все валидные наблюдения в исходном порядке, ошибки и условия остановки. Нельзя выбирать лучший процесс и замалчивать остальные.

Первый portable timer — `Instant`. Сообщать clock resolution и слишком короткие batches. Пустой loop измеряется как диагностический контроль; автоматическое вычитание overhead не должно создавать отрицательные или ложные «точные» времена. Специализированные timers допускаются после измерений read overhead, migration behaviour и совместимости.

## Сравнение A/B

Default для prepared executable — соседние пары с заранее сбалансированным AB/BA и случайным воспроизводимым порядком блоков. Программы не запускаются одновременно, кроме явно заданного contention workload. Парность уменьшает влияние медленного дрейфа, но не гарантирует одинаковую температуру или отсутствие GPU contention.

Primary statistic задаётся до запуска. Для положительных process-level latency можно использовать log ratio `log(B/A)` по парам и обратное преобразование для эффекта в %. Нулевые baseline не имеют относительного изменения; нужен абсолютный эффект. Для независимых исторических baseline нельзя использовать paired test без настоящих pair IDs.

Bootstrap выполняется на уровне независимых experimental units: process pairs, а для отдельных frame-series — подходящие blocks с описанной политикой. Не ресемплировать зависимые кадры как независимые для заявления о стабильности across runs. Для малого числа process pairs показывать ограниченность данных и `Inconclusive`; простой bootstrap не исправляет недостаток независимых наблюдений.

Первый строгий режим — fixed sample plan с CI и практическим threshold. Автоостановка «когда p < 0.05» не допускается: она меняет частоту ложных срабатываний. Exploratory precision stopping возможен с отдельной меткой; confirmatory evidence требует нового плана или статистически корректной sequential procedure.

При множестве cases/метрик заранее выбрать primary outcomes и политику множественных сравнений; кандидат — Holm для family-wise gate. Анализ версии v1 обязан сохранять выбранный метод, confidence level, seed, threshold и число проверок.

Пять основных результатов сравнения:

- **Regression:** совместимые валидные данные, эффект статистически поддержан и превышает практический порог в худшую сторону.
- **Improvement:** то же в лучшую сторону.
- **EquivalentWithinMargin:** заранее заданная процедура подтверждает попадание в область допустимой эквивалентности.
- **Inconclusive:** интервал слишком широк или информации недостаточно. «Не нашли значимости» не означает «одинаково быстро».
- **Incompatible/Invalid:** не совпадает контракт, нарушена корректность или серия неполна. Не становится зелёным CI автоматически.

Absolute performance budgets — отдельный механизм. Например, `geometry_uploads == 0` можно проверять без статистики, если счётчик детерминирован и область корректна. Для шумного `latency < X` нужна заранее заданная политика границы, а не assert на одном sample.

## Compatibility key

Сопоставляются workload contract/version, input hash/size, operation semantics, parameters, target/DPI, renderer backend/adapter/features, completion boundary, present mode, concurrency/pipeline depth, reset/cache policy, instrumentation mode, metric definition/unit и analysis support.

Toolchain, optimization flags, allocator, hardware/OS/power state по умолчанию должны соответствовать политике эксперимента. Если изменяемая переменная — compiler/backend/allocator, это явно объявленная axis: нельзя требовать её равенства и одновременно запретить запрошенное сравнение. Binary hash/commit A и B ожидаемо отличаются. Нежелательная разница фиксируется как mismatch, разрешённая — как experimental variable.

Commit без dirty state и binary hash недостаточен. CPU model недостаточен для полной идентичности машины. Thermal/load readings — наблюдения, не доказательство изоляции. Unknown не преобразуется в false/zero.

## GPU и память

GPU adapter должен выдавать timestamps с ID кадра/query, declared period и проверкой валидности. Отсутствующие/устаревшие/неполные query results не дают wraparound duration. Асинхронный readback с bounded backlog предпочтителен для timeline; serialized benchmark сохраняет явный wait. На разных backends проверяются реальные границы и отсутствие stale timestamps.

Allocation wrapper — `TrackingAllocator<A: GlobalAlloc>`, подключаемый пользователем. Успешный realloc учитывается по объявленной семантике requested bytes; failed realloc не удаляет старую allocation. Thread-local режим требует регистрации worker threads; whole-process атомарный режим имеет другую стоимость. Snapshot атомиков не транзакционен, reset peak допустим только с указанной моделью quiescence. Diagnostic allocation runs отдельно сравниваются с такими же instrumentation runs.

RSS снимается вне per-frame hot loop. Изменение RSS не доказывает leak. На UMA owned GPU buffers и RSS могут перекрываться, поэтому суммарную «RAM+VRAM» библиотека автоматически не строит.

## Корректность и честное представление

Изображение, checksum, semantic assertions и ошибки работы — отдельные артефакты с привязкой к binary/fixture. Для одинакового backend возможны byte-exact goldens; для разных backend требуется заранее заданная tolerance/quality policy. Принять потерю AA ради ускорения без явного изменения контракта нельзя.

Отчёт обязан показывать scope рядом с числом, raw count и independent process count, interval/effect, availability, mismatches и ссылки на raw data. Профили помогают объяснить причину, но сами по себе не доказывают end-to-end ускорение.
