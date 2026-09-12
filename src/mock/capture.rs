//! Bounded recording of call arguments.
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

/// Bounded owned argument capture. Clones share values. Oldest values are evicted.
pub struct Capture<A> {
    values: Arc<Mutex<VecDeque<Arc<A>>>>,
    capacity: usize,
}
impl<A> Clone for Capture<A> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            capacity: self.capacity,
        }
    }
}
impl<A: Clone> Capture<A> {
    /// Keep at most `capacity` most recent arguments. Zero records nothing.
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Arc::new(Mutex::new(VecDeque::new())),
            capacity,
        }
    }
    /// Clone one argument in. Cloning happens outside the mock's lock.
    pub fn record(&self, value: &A) {
        if self.capacity == 0 {
            return;
        }
        let value = Arc::new(value.clone());
        let removed = {
            let mut values = self.values.lock().expect("capture lock poisoned");
            let removed = if values.len() == self.capacity {
                values.pop_front()
            } else {
                None
            };
            values.push_back(value);
            removed
        };
        drop(removed);
    }
    /// Snapshot the retained arguments, oldest first.
    pub fn values(&self) -> Vec<A> {
        let values: Vec<_> = self
            .values
            .lock()
            .expect("capture lock poisoned")
            .iter()
            .cloned()
            .collect();
        values.into_iter().map(|value| (*value).clone()).collect()
    }
    /// Drop everything recorded so far, for reuse across phases of a test.
    pub fn clear(&self) {
        let removed = std::mem::take(&mut *self.values.lock().expect("capture lock poisoned"));
        drop(removed);
    }
}
