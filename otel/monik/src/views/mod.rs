use std::collections::VecDeque;

use crate::sampler::Sample;
use crate::storage::Storage;

pub mod alerts;
pub mod history;
pub mod overview;
pub mod processes;

pub const RING_CAPACITY: usize = 600;

pub struct SharedState {
    pub ring: VecDeque<Sample>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            ring: VecDeque::with_capacity(RING_CAPACITY + 1),
        }
    }

    pub fn push(&mut self, sample: Sample) {
        if self.ring.len() >= RING_CAPACITY {
            self.ring.pop_front();
        }
        self.ring.push_back(sample);
    }

    pub fn latest(&self) -> Option<&Sample> {
        self.ring.back()
    }
}

pub fn fmt_bytes(b: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = b;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}

pub fn history_storage() -> Option<Storage> {
    Storage::open(&crate::storage::default_db_path()).ok()
}
