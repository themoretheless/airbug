//! Counter helpers usable directly in attribute expressions or `Suite::work_units`.
//! Counts describe logical work per operation, not automatic memory allocation tracking.

/// Convert byte counts to bits, rejecting overflow instead of wrapping.
pub const fn bits_from_bytes(bytes: u64) -> u64 {
    match bytes.checked_mul(8) {
        Some(bits) => bits,
        None => panic!("bit counter overflow"),
    }
}

pub const fn bytes_of<T>() -> u64 {
    std::mem::size_of::<T>() as u64
}
/// Panics on multiplication overflow, instead of wrapping the declared work count.
pub const fn bytes_of_many<T>(count: u64) -> u64 {
    match bytes_of::<T>().checked_mul(count) {
        Some(bytes) => bytes,
        None => panic!("byte counter overflow"),
    }
}
pub fn bytes_of_val<T: ?Sized>(value: &T) -> u64 {
    std::mem::size_of_val(value) as u64
}
pub fn bytes_of_slice<T>(values: impl AsRef<[T]>) -> u64 {
    bytes_of_val(values.as_ref())
}
pub fn bytes_of_str(value: impl AsRef<str>) -> u64 {
    bytes_of_val(value.as_ref())
}
/// Unicode scalar values, not grapheme clusters or UTF-8 bytes.
pub fn chars_of_str(value: impl AsRef<str>) -> u64 {
    items_of_iter(value.as_ref().chars())
}
/// Consumes the iterator, including its side effects.
pub fn items_of_iter(values: impl IntoIterator) -> u64 {
    values.into_iter().fold(0u64, |count, _| {
        count.checked_add(1).expect("item counter overflow")
    })
}
/// Consumes the iterator and counts the in-memory size of its items.
pub fn bytes_of_iter<T>(values: impl IntoIterator<Item = T>) -> u64 {
    bytes_of_many::<T>(items_of_iter(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sizes_empty_inputs_unicode_and_iterators() {
        assert_eq!(bytes_of::<u32>(), 4);
        assert_eq!(bytes_of_many::<u32>(3), 12);
        assert_eq!(bytes_of_many::<()>(u64::MAX), 0);
        assert_eq!(bytes_of_slice([1u16, 2, 3]), 6);
        assert_eq!(bytes_of_slice::<u8>([]), 0);
        assert_eq!(bytes_of_val(&[1u64, 2][..]), 16);
        assert_eq!(bytes_of_str("é🦀"), 6);
        assert_eq!(chars_of_str("é🦀"), 2);
        assert_eq!(chars_of_str("e\u{301}"), 2);
        assert_eq!(bytes_of_iter([1u32, 2, 3]), 12);
        let mut calls = 0;
        assert_eq!(items_of_iter((0..4).inspect(|_| calls += 1)), 4);
        assert_eq!(calls, 4);
    }
    #[test]
    fn byte_helpers_use_value_layout_and_consume_owned_iterator_items() {
        use std::cell::Cell;
        let text = String::from("e\u{301}🦀");
        assert_eq!(bytes_of_val(&text), bytes_of::<String>());
        assert_eq!(bytes_of_str(&text), 7);
        assert_eq!(chars_of_str(&text), 3);
        let strings = [text];
        assert_eq!(bytes_of_slice(&strings), bytes_of::<String>());
        let erased: &dyn AsRef<str> = &strings[0];
        assert_eq!(bytes_of_str(erased), 7);

        struct Item<'a>(&'a Cell<usize>);
        impl Drop for Item<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let dropped = Cell::new(0);
        let visited = Cell::new(0);
        let items = (0..3).map(|_| {
            visited.set(visited.get() + 1);
            Item(&dropped)
        });
        assert_eq!(bytes_of_iter(items), bytes_of_many::<Item<'_>>(3));
        assert_eq!(visited.get(), 3);
        assert_eq!(dropped.get(), 3);
    }
    #[test]
    #[should_panic(expected = "byte counter overflow")]
    fn size_multiplication_never_wraps() {
        bytes_of_many::<u64>(u64::MAX);
    }
}

thread_local! {
    static WORKER: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}
pub(crate) struct WorkerScope(Option<u64>, std::marker::PhantomData<std::rc::Rc<()>>);
impl WorkerScope {
    pub(crate) fn enter(worker: u64) -> Self {
        Self(
            WORKER.with(|slot| slot.replace(Some(worker))),
            std::marker::PhantomData,
        )
    }
}
impl Drop for WorkerScope {
    fn drop(&mut self) {
        WORKER.with(|slot| slot.set(self.0));
    }
}
pub(crate) fn worker_slot() -> Option<u64> {
    WORKER.with(|slot| slot.get())
}

/// Shared accumulator for actual work performed by prepared inputs. Record counts
/// in setup, then attach a clone with `Suite::input_counters`. Every workload
/// invocation resets it, so calibration/warmup never leak into measured samples.
#[derive(Clone)]
pub struct InputCounters(std::sync::Arc<std::sync::Mutex<InputCounts>>);
struct InputCounts {
    values: std::collections::BTreeMap<String, u128>,
    error: Option<&'static str>,
    workers: std::collections::BTreeMap<u64, std::collections::BTreeMap<String, u128>>,
}
impl InputCounters {
    pub fn new(units: &[&str]) -> crate::Result<Self> {
        let mut values = std::collections::BTreeMap::new();
        for unit in units {
            if unit.is_empty() || values.insert((*unit).to_owned(), 0).is_some() {
                return Err(crate::error(
                    "input counter units must be nonempty and unique",
                ));
            }
        }
        if values.is_empty() {
            return Err(crate::error("input counters require at least one unit"));
        }
        Ok(Self(std::sync::Arc::new(std::sync::Mutex::new(
            InputCounts {
                values,
                workers: Default::default(),
                error: None,
            },
        ))))
    }
    /// Record outside timing. Unknown units or overflow invalidate the sample;
    /// errors are returned by the enclosing Suite run, never silently truncated.
    pub fn add(&self, unit: &str, count: u64) {
        let mut state = self.0.lock().unwrap();
        match state.values.get_mut(unit) {
            Some(value) => match value.checked_add(count as u128) {
                Some(sum) => *value = sum,
                None => state.error = Some("input counter overflow"),
            },
            None => state.error = Some("undeclared input counter unit"),
        }
        if state.error.is_none() {
            if let Some(worker) = worker_slot() {
                if !state.workers.contains_key(&worker) {
                    let zeroes = state.values.keys().map(|unit| (unit.clone(), 0)).collect();
                    state.workers.insert(worker, zeroes);
                }
                let value = state
                    .workers
                    .get_mut(&worker)
                    .unwrap()
                    .get_mut(unit)
                    .unwrap();
                match value.checked_add(u128::from(count)) {
                    Some(sum) => *value = sum,
                    None => state.error = Some("worker input counter overflow"),
                }
            }
        }
    }
    pub(crate) fn reset(&self) {
        let mut state = self.0.lock().unwrap();
        state.values.values_mut().for_each(|v| *v = 0);
        state.error = None;
        state.workers.clear();
    }
    pub(crate) fn worker_snapshot(
        &self,
        count: u64,
    ) -> crate::Result<std::collections::BTreeMap<u64, std::collections::BTreeMap<String, String>>>
    {
        let state = self.0.lock().unwrap();
        if let Some(error) = state.error {
            return Err(crate::error(error));
        }
        if state.workers.len() as u64 != count || state.workers.keys().any(|w| *w >= count) {
            return Ok(Default::default());
        }
        for (unit, total) in &state.values {
            let sum = state
                .workers
                .values()
                .try_fold(0u128, |sum, values| sum.checked_add(values[unit]));
            if sum != Some(*total) {
                return Ok(Default::default());
            }
        }
        Ok(state
            .workers
            .iter()
            .map(|(worker, values)| {
                (
                    *worker,
                    values
                        .iter()
                        .map(|(unit, value)| (unit.clone(), value.to_string()))
                        .collect(),
                )
            })
            .collect())
    }
    pub(crate) fn snapshot(&self) -> crate::Result<std::collections::BTreeMap<String, String>> {
        let state = self.0.lock().unwrap();
        if let Some(error) = state.error {
            return Err(crate::error(error));
        }
        Ok(state
            .values
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect())
    }
}

#[cfg(test)]
mod input_counter_tests {
    use super::InputCounters;
    #[test]
    fn totals_preserve_large_integers_and_overflow_is_explicit() {
        let counts = InputCounters::new(&["bytes"]).unwrap();
        counts.add("bytes", u64::MAX);
        counts.add("bytes", u64::MAX);
        assert_eq!(
            counts.snapshot().unwrap()["bytes"],
            (2 * u64::MAX as u128).to_string()
        );
        counts
            .0
            .lock()
            .unwrap()
            .values
            .insert("bytes".into(), u128::MAX);
        counts.add("bytes", 1);
        assert!(counts.snapshot().is_err());
        counts.reset();
        assert_eq!(counts.snapshot().unwrap()["bytes"], "0");
    }
}

#[cfg(test)]
mod worker_scope_tests {
    use super::*;
    #[test]
    fn scopes_restore_on_unwind_and_unattributed_counts_stay_unknown() {
        let counters = InputCounters::new(&["items"]).unwrap();
        assert_eq!(worker_slot(), None);
        {
            let _outer = WorkerScope::enter(0);
            counters.add("items", 2);
            let _ = std::panic::catch_unwind(|| {
                let _nested = WorkerScope::enter(1);
                counters.add("items", 3);
                panic!("test scope cleanup");
            });
            assert_eq!(worker_slot(), Some(0));
        }
        assert_eq!(worker_slot(), None);
        let known = counters.worker_snapshot(2).unwrap();
        assert_eq!(known[&0]["items"], "2");
        assert_eq!(known[&1]["items"], "3");
        counters.add("items", 1);
        assert!(counters.worker_snapshot(2).unwrap().is_empty());
        counters.reset();
        assert!(counters.worker_snapshot(2).unwrap().is_empty());
        assert_eq!(counters.snapshot().unwrap()["items"], "0");
    }
}
