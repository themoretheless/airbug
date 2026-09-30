# Покрытие benchmark-сценариев

Полный реестр незавершённого паритета: [parity/README.md](parity/README.md).

Проверка API: 2026-09-30. Airbug использует собственные измерения и формат результатов.
Совместимость с API Criterion/Divan не является целью; таблица описывает задачи,
которые можно решить внутри Airbug. Она не доказывает равенство точности или overhead.

Источники для перечня задач: [Divan bench](https://docs.rs/divan/latest/divan/attr.bench.html),
[Criterion timing loops](https://bheisler.github.io/criterion.rs/book/user_guide/timing_loops.html),
[Criterion async](https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_async.html).

| Сценарий | Airbug | Проверка / границы |
|---|---|---|
| Обычная функция и Cargo-фильтры | `#[bench]`, `cargo bench` | Реальный Cargo end-to-end script |
| Значения параметров | `args = [...]` | По значению Copy; по ссылке можно String и другие owned-значения |
| Несколько типов | `types = [u32, u64]` | Один type-параметр, включая bounds и where |
| Const generics | `consts = [16, 64]` | Один const-параметр, непосредственный список выражений |
| Матрица параметров | types × consts × args × threads | Отдельные ID, точная фильтрация; builder также имеет matrix |
| Свежий input вне измерения | `setup = ...` | Новый input на каждую операцию, setup и input drop исключены |
| Повторно используемый fixture | `Suite::bench_fixture` | Уже есть; отдельного атрибута общего fixture пока нет |
| Исключение output drop | `drop_output = "outside"` | Буферизация ограничена chunks <=64; память самих объектов не ограничена в байтах |
| Async | `async fn` + опциональный `executor` | Lazy construction, wakeup и borrowed input проверены; runtime-specific reactor предоставляет пользователь |
| Конкурентные потоки | `threads = [1, 2, 4]` | Общий старт, завершение всех workers, корректное суммарное число операций |
| Ручной измеряемый интервал | `custom = true` / `bench_custom` | Принимает n и возвращает Duration; точность границ обеспечивает автор кейса |
| Throughput | `bytes`, `items`, `chars`, `cycles` | Одновременные счётчики, замена по единице; budget выбирает `--unit` |
| Настройки измерения | `samples`, `sample_ms`, `warmup_ms` | CLI overrides и dry-run эффективных настроек проверены |
| Статистика и сравнение | analysis + runner compare | Существующая модель независимых процессов; внутренние batches не выдаются за независимые репликации |
| Baseline / CI-бюджеты | runner baseline, compare/check; автоматическая история `cargo bench` | Изменения относительно прошлого запуска показываются автоматически; один процесс обычно даёт inconclusive, именованные baseline-политики ещё проверяются |
| История и прогресс | Suite::main + hub | Все виды кейсов попадают в единый формат истории |
| Метрики приложения | Recorder, allocator, scenario API | Существующий API; произвольные метрики не добавляются этим макросом |

## Проверки новой реализации

- `cargo test -p airbug-bench --features macros --offline`: контракты lifecycle,
  generic dimensions, lazy async executor, пробуждение future, ручные длительности,
  нормализация concurrent operations, ошибки worker и drop после завершения группы.
- `python3 bench/scripts/test-cargo-bench.py`: реальный `cargo bench`, listing,
  exact-фильтры, типы/константы/параметры, async, потоки, настройки и история всех кейсов.
- Compile-fail doctests проверяют неверные параметры и недопустимые сочетания.
- Strict Clippy для библиотеки, макросов и всех targets.

Это функциональная проверка. Здесь не измерялись преимущества Airbug над другими
движками, частота ложных регрессий или чувствительность к небольшим изменениям.

## Оставшиеся ограничения

- Вложенные inline-модули регистрируются автоматически с наследованием defaults;
  out-of-line modules, несколько generic-параметров одного вида и явные
  lifetime-параметры требуют дальнейшей реализации.
- Async + threads не имеют автоматического совместного режима. Нужный runtime и
  его конкурентное расписание можно явно измерить через custom timing.
- Входы по владению поддерживаются по сигнатуре функции, включая async/threads.
  Их уничтожение самой функцией остаётся внутри измерения; output drop можно отложить.
- По-прежнему нет доказательства паритета overhead и статистической устойчивости
  с Criterion/Divan на контролируемом наборе реальных алгоритмов.
- Новые проверки выполнены на текущем macOS toolchain; они не являются отдельной
  проверкой Windows/Linux или минимальной версии Rust.
