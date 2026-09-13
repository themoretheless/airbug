//! Thread-local scope + process-wide breadcrumb ring.
use crate::event::{Breadcrumb, Severity, User};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    sync::{Mutex, OnceLock},
};

const DEFAULT_MAX_BREADCRUMBS: usize = 100;

/// Mutable capture context (tags, user, extra, fingerprint override).
#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub tags: BTreeMap<String, String>,
    pub user: Option<User>,
    pub extra: BTreeMap<String, serde_json::Value>,
    pub fingerprint: Option<Vec<String>>,
    pub level: Option<Severity>,
}

impl Scope {
    pub fn set_tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.tags.insert(key.into(), value.into());
    }

    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    pub fn set_extra(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.extra.insert(key.into(), value);
    }

    pub fn set_fingerprint(&mut self, parts: impl IntoIterator<Item = impl Into<String>>) {
        self.fingerprint = Some(parts.into_iter().map(Into::into).collect());
    }

    pub fn set_level(&mut self, level: Severity) {
        self.level = Some(level);
    }

    pub fn clear(&mut self) {
        *self = Scope::default();
    }
}

thread_local! {
    static SCOPE: RefCell<Scope> = RefCell::new(Scope::default());
}

static BREADCRUMBS: OnceLock<Mutex<BreadcrumbRing>> = OnceLock::new();

#[derive(Debug)]
struct BreadcrumbRing {
    max: usize,
    items: VecDeque<Breadcrumb>,
}

impl BreadcrumbRing {
    fn new(max: usize) -> Self {
        Self {
            max: max.max(1),
            items: VecDeque::new(),
        }
    }

    fn push(&mut self, crumb: Breadcrumb) {
        if self.items.len() >= self.max {
            self.items.pop_front();
        }
        self.items.push_back(crumb);
    }

    fn snapshot(&self) -> Vec<Breadcrumb> {
        self.items.iter().cloned().collect()
    }
}

fn ring() -> &'static Mutex<BreadcrumbRing> {
    BREADCRUMBS.get_or_init(|| Mutex::new(BreadcrumbRing::new(DEFAULT_MAX_BREADCRUMBS)))
}

pub(crate) fn configure_max_breadcrumbs(max: usize) {
    if let Ok(mut g) = ring().lock() {
        g.max = max.max(1);
        while g.items.len() > g.max {
            g.items.pop_front();
        }
    }
}

/// Record a breadcrumb kept until the next captured event.
pub fn add_breadcrumb(category: impl Into<String>, message: impl Into<String>, level: Severity) {
    let crumb = Breadcrumb {
        timestamp: crate::iso_now(),
        category: category.into(),
        message: message.into(),
        level,
    };
    if let Ok(mut g) = ring().lock() {
        g.push(crumb);
    }
}

pub(crate) fn take_breadcrumbs() -> Vec<Breadcrumb> {
    ring().lock().map(|g| g.snapshot()).unwrap_or_default()
}

/// Mutate the current thread's scope.
pub fn configure_scope<F, R>(f: F) -> R
where
    F: FnOnce(&mut Scope) -> R,
{
    SCOPE.with(|cell| f(&mut cell.borrow_mut()))
}

pub(crate) fn with_scope<F, R>(f: F) -> R
where
    F: FnOnce(&Scope) -> R,
{
    SCOPE.with(|cell| f(&cell.borrow()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumb_ring_overflow_drops_oldest() {
        let mut ring = BreadcrumbRing::new(3);
        for msg in ["1", "2", "3", "4"] {
            ring.push(Breadcrumb {
                timestamp: "t".into(),
                category: "a".into(),
                message: msg.into(),
                level: Severity::Info,
            });
        }
        let crumbs = ring.snapshot();
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs[0].message, "2");
        assert_eq!(crumbs[2].message, "4");
    }
}
