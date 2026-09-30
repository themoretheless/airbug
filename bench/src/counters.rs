//! Counter helpers usable directly in attribute expressions or `Suite::work_units`.
//! Counts describe logical work per operation, not automatic memory allocation tracking.

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
    #[should_panic(expected = "byte counter overflow")]
    fn size_multiplication_never_wraps() {
        bytes_of_many::<u64>(u64::MAX);
    }
}

/// Shared accumulator for actual work performed by prepared inputs. Record counts
/// in setup, then attach a clone with `Suite::input_counters`. Every workload
/// invocation resets it, so calibration/warmup never leak into measured samples.
#[derive(Clone)]
pub struct InputCounters(std::sync::Arc<std::sync::Mutex<InputCounts>>);
struct InputCounts {
    values: std::collections::BTreeMap<String, u128>,
    error: Option<&'static str>,
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
    }
    pub(crate) fn reset(&self) {
        let mut state = self.0.lock().unwrap();
        state.values.values_mut().for_each(|v| *v = 0);
        state.error = None;
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
