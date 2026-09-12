//! Opt-in bounded log of the calls a mock has seen.
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallRecord {
    pub arguments: String,
    pub expectation: Option<String>,
    pub accepted: bool,
}

/// Ring buffer of call records. Capacity zero disables recording entirely.
#[derive(Default)]
pub(super) struct Journal {
    records: VecDeque<CallRecord>,
    capacity: usize,
}
impl Journal {
    pub(super) fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
        while self.records.len() > capacity {
            self.records.pop_front();
        }
    }
    pub(super) fn push(&mut self, record: CallRecord) {
        if self.capacity == 0 {
            return;
        }
        if self.records.len() == self.capacity {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }
    pub(super) fn records(&self) -> Vec<CallRecord> {
        self.records.iter().cloned().collect()
    }
    pub(super) fn clear(&mut self) {
        self.records.clear();
    }
}
