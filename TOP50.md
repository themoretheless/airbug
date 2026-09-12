# Топ-50: реализованные улучшения

Выбор из первоначальных 200 идей: приоритет повторяемой подготовке данных, диагностике и контролю взаимодействий. Все пункты дополняют обычные Rust-тесты; свой раннер не добавлен.

Номера в колонке «Идея» соответствуют первоначальному списку. Проверки находятся в указанных интеграционных тестах.

| № | Идея | Улучшение | API | Проверка |
|---|---|---|---|---|
| 1 | 1 | Prelude | `prelude::*` | [prelude_supports_native_tests](tests/fixture.rs) |
| 2 | 21 | Scoped-настройки с восстановлением | `FixtureContext::scoped` | [scoped_overrides_restore_on_success_and_panic](tests/fixture.rs) |
| 3 | 23 | Дочерний Fixture | `FixtureContext::child` | [child_inherits_rules_without_mutating_parent](tests/fixture.rs) |
| 4 | 26 | Воспроизведение состояния | `checkpoint / restore` | [checkpoint_replays_rng_and_rules](tests/fixture.rs) |
| 5 | 27 | Seed в ошибке генерации | `GenerationError::seed` | [node_budget_protects_wide_graph_and_errors_include_seed](tests/fixture.rs) |
| 6 | 31 | Лимит объектов графа | `FixtureContext::max_nodes` | [node_budget_protects_wide_graph_and_errors_include_seed](tests/fixture.rs) |
| 7 | 51 | Выбор из набора | `FixtureContext::choose` | [choice_is_repeatable_and_empty_is_error](tests/fixture.rs) |
| 8 | 52 | Последовательные ID | `FixtureContext::sequence_u64` | [sequential_ids_stop_at_overflow](tests/fixture.rs) |
| 9 | 56 | Граничные числовые наборы | `fixture::boundaries` | [boundary_sets_include_special_values](tests/fixture.rs) |
| 10 | 58 | Согласованные диапазоны | `FixtureContext::ordered_pair` | [ordered_pairs_cover_full_range_without_overflow](tests/fixture.rs) |
| 11 | 80 | Ленивая генерация | `FixtureContext::stream` | [stream_is_lazy_and_fallible](tests/fixture.rs) |
| 12 | 81 | Последовательные ответы | `returns_sequence` | [sequence_has_exact_order_and_exhaustion](tests/mock.rs) |
| 13 | 82 | Одноразовый ответ без Clone | `returning_once` | [one_shot_owns_non_clone_response](tests/mock.rs) |
| 14 | 83 | Диапазон количества вызовов | `times_between` | [range_counts_enforce_both_bounds](tests/mock.rs) |
| 15 | 84 | Минимум вызовов | `at_least` | [at_least_requires_minimum](tests/mock.rs) |
| 16 | 85 | Максимум вызовов | `at_most` | [at_most_accepts_zero_and_rejects_excess](tests/mock.rs) |
| 17 | 92 | Захват аргументов | `Capture / capture` | [capture_keeps_owned_recent_arguments](tests/mock.rs) |
| 18 | 91 | Журнал взаимодействий | `Mock::calls` | [journal_records_matched_and_unexpected_calls](tests/mock.rs) |
| 19 | 118 | Ограничение журнала | `journal_capacity` | [journal_is_bounded_and_can_be_disabled](tests/mock.rs) |
| 20 | 94 | Композиция matchers | `Matcher::and / or / negate` | [named_matchers_compose_and_reuse](tests/mock.rs) |
| 21 | 96 | Переиспользуемые matchers | `Matcher / expect_matcher` | [named_matchers_compose_and_reuse](tests/mock.rs) |
| 22 | 90 | Этапы проверки счётчиков | `reset_counts` | [reset_opens_new_counting_phase](tests/mock.rs) |
| 23 | 121 | Проверки вложенных полей | `CheckReport::field_equal` | [field_report_preserves_path_actual_and_expected](tests/checks.rs) |
| 24 | 127 | Отсутствие дубликатов | `assert_unique` | [unique_rejects_duplicate_indexes](tests/checks.rs) |
| 25 | 128 | Сортировка коллекции | `assert_sorted` | [sorted_accepts_equal_values_and_rejects_nan](tests/checks.rs) |
| 26 | 129 | Проверка каждого элемента | `assert_all` | [all_and_count_have_actionable_failures](tests/checks.rs) |
| 27 | 130 | Количество подходящих элементов | `assert_count` | [all_and_count_have_actionable_failures](tests/checks.rs) |
| 28 | 131 | Подмножество и надмножество | `assert_subset / assert_superset` | [subset_and_superset_preserve_multiplicity](tests/checks.rs) |
| 29 | 133 | Относительная погрешность | `assert_relative_eq` | [relative_comparison_avoids_overflow](tests/checks.rs) |
| 30 | 135 | Нормализация переносов строк | `assert_text_eq` | [newline_normalization_preserves_other_whitespace](tests/checks.rs) |
| 31 | 137 | Цепочка источников ошибки | `assert_error_chain_contains` | [error_chain_finds_nested_message](tests/checks.rs) |
| 32 | 138 | Проверка panic | `assert_panics` | [panic_check_rejects_missing_wrong_and_non_string_payloads](tests/checks.rs) |
| 33 | 140 | Структурированные отчёты | `CheckReport / CheckFailure` | [report_accumulates_structured_errors](tests/checks.rs) |
| 34 | 141 | Стабильные коды валидации | `ValidationError::code / with_code` | [stable_codes_are_independent_of_messages](tests/validation.rs) |
| 35 | 145 | Правила нескольких полей | `Validator::check` | [cross_field_predicate_compares_fields](tests/validation.rs) |
| 36 | 146 | Условные группы правил | `group_when` | [conditional_group_is_skipped_and_preserves_order](tests/validation.rs) |
| 37 | 148 | Валидация Some | `Validator::optional` | [optional_child_checks_only_present_values](tests/validation.rs) |
| 38 | 150 | Уникальность по ключу | `Validator::unique_by` | [collection_uniqueness_uses_selected_key](tests/validation.rs) |
| 39 | 154 | Бюджет ошибок | `Validator::max_errors` | [error_budget_skips_nested_rules_after_limit](tests/validation.rs) |
| 40 | 155 | Глобальная остановка | `Validator::stop_on_first_failure` | [global_stop_skips_later_checks_even_on_same_field](tests/validation.rs) |
| 41 | 157 | Преобразование ошибок приложения | `ValidationErrors::map` | [errors_convert_to_application_format](tests/validation.rs) |
| 42 | 161 | Текстовые snapshots | `Snapshots::check` | [snapshots_verify_never_creates_files](tests/snapshot.rs) |
| 43 | 162 | Читаемый snapshot diff | `text_diff` | [diff_identifies_line_and_values](tests/snapshot.rs) |
| 44 | 163 | Явное обновление эталонов | `UpdateMode` | [snapshots_only_update_explicitly](tests/snapshot.rs) |
| 45 | 164 | Inline snapshots | `Snapshots::inline` | [inline_snapshots_compare_without_writes](tests/snapshot.rs) |
| 46 | 165 | Маскирование нестабильных значений | `Snapshots::redact` | [redaction_applies_before_storage_and_diagnostics](tests/snapshot.rs) |
| 47 | 168 | Snapshots именованных cases | `Snapshots::check_case` | [cases_have_separate_snapshot_paths](tests/snapshot.rs) |
| 48 | 181 | Управляемые часы | `ManualClock::advance` | [manual_clock_advances_without_sleeping](tests/time.rs) |
| 49 | 182 | Разделение календарного и монотонного времени | `set_wall_time / elapsed` | [wall_adjustments_do_not_change_monotonic_time](tests/time.rs) |
| 50 | 184 | Ожидание eventual consistency | `Eventually::check` | [eventual_success_and_timeout_keep_bounded_observations](tests/time.rs) |

Дополнительные проверки покрывают конкуренцию, сбои обновления snapshots, отсутствие продвижения времени и попытку сброса активного мока.

Границы: checkpoint не откатывает внешнее состояние фабрик; captures хранят собственные копии аргументов; snapshots текстовые с литеральным маскированием; polling синхронный и не прерывает зависший пользовательский код.
