# mon

Local host monitor: live metrics, history, alerts.

Package: `airbug-mon` (egui + sysinfo + SQLite). OpenTelemetry OTLP lives in `otel/` (`airbug-otel`);
mon can export host metrics with `--otlp` or `OTEL_EXPORTER_OTLP_ENDPOINT`.

Нативное десктопное приложение мониторинга системы для macOS.

## Запуск

```
cargo run -p airbug-mon --release
```

OTLP export (collector on :4318):

```
cargo run -p airbug-mon --release -- --otlp
# or: export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
```

Headless-режим (без GUI, только сбор в SQLite):

```
cargo run -p airbug-mon --release -- --headless --seconds 15
cargo run -p airbug-mon --release -- --headless --seconds 15 --otlp
```

## Возможности

- **Обзор**: живые графики CPU (общий и по ядрам), RAM/swap, GPU, сети и диска за последние 10 минут.
- **Процессы**: топ-10 процессов по CPU+RAM, сортировка по колонкам, фильтр, мини-график истории выбранного процесса.
- **История**: метрики из SQLite, диапазоны 1ч/6ч/24ч/7д/30д (длинные диапазоны — по минутным агрегатам), экспорт CSV.
- **Алерты**: пороги CPU/RAM/диск с нативными уведомлениями и журналом срабатываний.

Данные хранятся в `~/.local/share/airbug-mon/airbug-mon.db`: сырые сэмплы — 24 часа, минутные агрегаты — 30 дней.

## Ограничения

- **GPU на macOS**: метрики берутся из `powermetrics`, которому нужен sudo. Без него провайдер возвращает `None`, и UI показывает «GPU недоступен (нужен sudo)».
- Первый сэмпл после запуска показывает нулевые rate для сети/диска (нет предыдущей точки для дельты).

## Тесты

```
cargo test -p airbug-mon
```
