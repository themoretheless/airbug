//! Bounded, opt-in diagnostic allocation stacks. Rust heap only; no object graph.
use serde::{Deserialize, Serialize};
use std::{
    alloc::{GlobalAlloc, Layout},
    cell::Cell,
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    },
};
const CAPACITY: usize = 8192;
const DEPTH: usize = 24;
#[derive(Clone, Copy)]
struct Event {
    ptr: usize,
    old: usize,
    size: usize,
    stack: [usize; DEPTH],
    depth: usize,
}
const EMPTY: Event = Event {
    ptr: 0,
    old: 0,
    size: 0,
    stack: [0; DEPTH],
    depth: 0,
};
struct Buffer {
    events: [Event; CAPACITY],
    len: usize,
}
static BUFFER: Mutex<Buffer> = Mutex::new(Buffer {
    events: [EMPTY; CAPACITY],
    len: 0,
});
static ACTIVE: AtomicBool = AtomicBool::new(false);
static USED: AtomicBool = AtomicBool::new(false);
static DROPPED: AtomicU64 = AtomicU64::new(0);
thread_local! { static ENTERED: Cell<bool> = const { Cell::new(false) }; }
static INSTALLED: AtomicBool = AtomicBool::new(false);
static GATE: AtomicBool = AtomicBool::new(false);
struct Hook {
    entered: bool,
    locked: bool,
}
impl Hook {
    fn enter_trace() -> Self {
        INSTALLED.store(true, Relaxed);
        if !ACTIVE.load(Relaxed) {
            return Self {
                entered: false,
                locked: false,
            };
        }
        let entered = ENTERED.try_with(|e| !e.replace(true)).unwrap_or(false);
        let locked = entered
            && GATE
                .compare_exchange(false, true, std::sync::atomic::Ordering::Acquire, Relaxed)
                .is_ok();
        if entered && !locked {
            DROPPED.fetch_add(1, Relaxed);
        }
        Self { entered, locked }
    }
    fn enter_free() -> Self {
        INSTALLED.store(true, Relaxed);
        if !ACTIVE.load(Relaxed) {
            return Self {
                entered: false,
                locked: false,
            };
        }
        let entered = ENTERED.try_with(|e| !e.replace(true)).unwrap_or(false);
        Self {
            entered,
            locked: false,
        }
    }
}
impl Drop for Hook {
    fn drop(&mut self) {
        if self.locked {
            GATE.store(false, std::sync::atomic::Ordering::Release);
        }
        if self.entered {
            let _ = ENTERED.try_with(|e| e.set(false));
        }
    }
}
fn record(ptr: usize, old: usize, size: usize) {
    // A profiler failure must never unwind across GlobalAlloc.
    if std::panic::catch_unwind(|| {
        let mut e = Event {
            ptr,
            old,
            size,
            ..EMPTY
        };
        if ptr != 0 {
            backtrace::trace(|f| {
                if e.depth == DEPTH {
                    return false;
                }
                e.stack[e.depth] = f.ip() as usize;
                e.depth += 1;
                true
            });
        }
        let mut b = BUFFER.lock().unwrap_or_else(|e| e.into_inner());
        if b.len == CAPACITY {
            DROPPED.fetch_add(1, Relaxed);
            return;
        }
        let i = b.len;
        b.events[i] = e;
        b.len += 1;
    })
    .is_err()
    {
        std::process::abort();
    }
}
pub struct Allocator<A>(pub A);
// SAFETY: pointer/layout operations are forwarded unchanged. The hook never
// dereferences allocations, uses a bounded static buffer and prevents reentry.
unsafe impl<A: GlobalAlloc> GlobalAlloc for Allocator<A> {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let hook = Hook::enter_trace();
        let p = unsafe { self.0.alloc(l) };
        if hook.locked && !p.is_null() {
            record(p as usize, 0, l.size())
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let hook = Hook::enter_trace();
        let p = unsafe { self.0.alloc_zeroed(l) };
        if hook.locked && !p.is_null() {
            record(p as usize, 0, l.size())
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        let hook = Hook::enter_free();
        if hook.entered {
            record(0, p as usize, 0);
        }
        unsafe { self.0.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let hook = Hook::enter_trace();
        let q = unsafe { self.0.realloc(p, l, n) };
        if !q.is_null() {
            if hook.locked {
                record(q as usize, p as usize, n);
            } else if hook.entered && q != p {
                record(0, p as usize, 0);
            }
        }
        q
    }
}
/// Aggregated allocation statistics for one unique call stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Site {
    pub frames: Vec<String>,
    pub allocated_bytes: u64,
    pub allocations: u64,
    pub live_bytes: u64,
    pub live_blocks: u64,
}

impl Site {
    /// Parse rendered stack frames (`"fn_name (path/to/file.rs:42)"` or `"fn_name"`)
    /// into `(function_name, filename, line_number)` tuples (root-to-leaf order).
    pub fn parsed_frames(&self) -> Vec<(String, Option<String>, Option<u32>)> {
        self.frames
            .iter()
            .map(|raw| parse_rendered_frame(raw))
            .collect()
    }
}

fn parse_rendered_frame(raw: &str) -> (String, Option<String>, Option<u32>) {
    let trimmed = raw.trim();
    if let Some(open) = trimmed.rfind(" (")
        && trimmed.ends_with(')')
    {
        let func = trimmed[..open].trim().to_string();
        let loc = &trimmed[open + 2..trimmed.len() - 1];
        if let Some((file, line_str)) = loc.rsplit_once(':')
            && let Ok(line) = line_str.parse::<u32>()
        {
            return (func, Some(file.to_string()), Some(line));
        }
        return (func, Some(loc.to_string()), None);
    }
    (trimmed.to_string(), None, None)
}

/// Allocation profile report captured by [`Session`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub schema: String,
    pub scope: String,
    pub dropped_events: u64,
    pub events: usize,
    pub peak_tracked_bytes: u64,
    pub sites: Vec<Site>,
}

/// Alias for [`Site`] used by external profilers / OTLP converters.
pub type AllocationSite = Site;

impl Profile {
    /// Slice of recorded allocation stack sites.
    pub fn sites(&self) -> &[Site] {
        &self.sites
    }

    /// Sum of `allocated_bytes` across all tracked sites.
    pub fn total_allocated_bytes(&self) -> u64 {
        self.sites.iter().map(|s| s.allocated_bytes).sum()
    }

    /// Sum of `allocations` across all tracked sites.
    pub fn total_allocations(&self) -> u64 {
        self.sites.iter().map(|s| s.allocations).sum()
    }

    /// Sum of `live_bytes` remaining at session finish across all tracked sites.
    pub fn total_live_bytes(&self) -> u64 {
        self.sites.iter().map(|s| s.live_bytes).sum()
    }

    /// Sum of `live_blocks` remaining at session finish across all tracked sites.
    pub fn total_live_blocks(&self) -> u64 {
        self.sites.iter().map(|s| s.live_blocks).sum()
    }
}
pub struct Session(());
impl Session {
    /// One bounded diagnostic session per process. Stop/join workload threads before finish.
    pub fn start() -> crate::Result<Self> {
        if !INSTALLED.load(Relaxed) {
            return Err(crate::error(
                "install memory::Allocator as global allocator before profiling",
            ));
        }
        if USED.swap(true, Relaxed) {
            return Err(crate::error("memory session already used in this process"));
        }
        ACTIVE.store(true, Relaxed);
        Ok(Self(()))
    }
    pub fn finish(self) -> Profile {
        ACTIVE.store(false, Relaxed);
        let b = BUFFER.lock().unwrap_or_else(|e| e.into_inner());
        let events = b.events[..b.len].to_vec();
        drop(b);
        analyze(&events, DROPPED.load(Relaxed))
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        ACTIVE.store(false, Relaxed);
    }
}
fn analyze(events: &[Event], dropped_events: u64) -> Profile {
    let mut groups: BTreeMap<Vec<usize>, Site> = BTreeMap::new();
    let mut live: BTreeMap<usize, (Vec<usize>, u64)> = BTreeMap::new();
    let mut current = 0u64;
    let mut peak = 0;
    for e in events {
        if let Some((key, n)) = live.remove(&e.old) {
            current -= n;
            if let Some(s) = groups.get_mut(&key) {
                s.live_bytes -= n;
                s.live_blocks -= 1;
            }
        }
        if e.ptr != 0 {
            // Reused addresses after missing frees are not counted twice.
            if let Some((key, n)) = live.remove(&e.ptr) {
                current -= n;
                if let Some(s) = groups.get_mut(&key) {
                    s.live_bytes -= n;
                    s.live_blocks -= 1;
                }
            }
            let key = e.stack[..e.depth].to_vec();
            let site = groups.entry(key.clone()).or_default();
            site.allocated_bytes += e.size as u64;
            site.allocations += 1;
            site.live_bytes += e.size as u64;
            site.live_blocks += 1;
            live.insert(e.ptr, (key, e.size as u64));
            current += e.size as u64;
            peak = peak.max(current);
        }
    }
    let sites = groups
        .into_iter()
        .map(|(stack, mut site)| {
            site.frames = stack
                .into_iter()
                .rev()
                .map(|ip| {
                    let mut name = format!("0x{ip:x}");
                    backtrace::resolve(ip as *mut _, |symbol| {
                        if let Some(n) = symbol.name() {
                            name = n.to_string();
                            if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                                name.push_str(&format!(" ({}:{line})", file.display()));
                            }
                        }
                    });
                    name
                })
                .collect();
            site
        })
        .collect();
    Profile {schema:"bench.memory/1".into(),scope:"Rust allocator events during session; realloc counts as replacement allocation; live = tracked blocks remaining, not proof of leaks; stacks limited to 24 frames; no native/GPU/type/ownership data".into(),dropped_events,events:events.len(),peak_tracked_bytes:peak,sites}
}
impl Profile {
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        crate::model::write_new(path.as_ref(), self)
    }
}
/// Wrap the workload. Normal runs execute without profiling; `run --memory`
/// supplies the output path. Install `memory::Allocator` as the global allocator.
pub fn profile<T>(work: impl FnOnce() -> T) -> crate::Result<T> {
    let Some(path) = std::env::var_os("BENCH_MEMORY_OUTPUT") else {
        return Ok(work());
    };
    let session = Session::start()?;
    let result = work();
    let profile = session.finish();
    profile.save(std::path::Path::new(&path))?;
    let html = profile.html()?;
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(std::path::Path::new(&path).with_extension("html"))?
        .write_all(html.as_bytes())?;
    Ok(result)
}
impl Profile {
    pub fn html(&self) -> crate::Result<String> {
        let data = serde_json::to_string(self)?
            .replace('<', "\\u003c")
            .replace('>', "\\u003e")
            .replace('&', "\\u0026");
        Ok(include_str!("memory-template.html").replace("__PROFILE__", &data))
    }
    pub fn load(path: &std::path::Path) -> crate::Result<Self> {
        if path.is_symlink() || std::fs::metadata(path)?.len() > 16 * 1024 * 1024 {
            return Err(crate::error(
                "memory profile must be a regular file up to 16 MiB",
            ));
        }
        let p: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if p.schema != "bench.memory/1"
            || p.sites.len() > 8192
            || p.sites.iter().any(|s| s.frames.len() > 24)
        {
            return Err(crate::error("unsupported memory profile"));
        }
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_realloc_reuse_and_free() {
        let alloc = |ptr, old, size| Event {
            ptr,
            old,
            size,
            stack: [1; DEPTH],
            depth: 1,
        };
        let p = analyze(
            &[
                alloc(1, 0, 10),
                alloc(2, 1, 20),
                alloc(0, 2, 0),
                alloc(1, 0, 7),
            ],
            0,
        );
        assert_eq!(p.peak_tracked_bytes, 20);
        assert_eq!(p.sites[0].allocated_bytes, 37);
        assert_eq!(p.sites[0].live_bytes, 7);
        assert_eq!(p.sites[0].live_blocks, 1);
    }
    #[test]
    fn escaped_frames_and_incomplete_profile() {
        let p = Profile {
            schema: "bench.memory/1".into(),
            scope: "test".into(),
            dropped_events: 10,
            events: 0,
            peak_tracked_bytes: 0,
            sites: vec![Site {
                frames: vec!["</script><img src=x onerror=alert(1)>".into()],
                ..Site::default()
            }],
        };
        let html = p.html().unwrap();
        assert!(!html.contains("</script><img"));
        assert!(html.contains("\\u003c/script"));
        assert_eq!(p.dropped_events, 10);
    }

    #[test]
    fn parsed_frames_and_totals() {
        let site = Site {
            frames: vec![
                "my_crate::run (src/lib.rs:42)".into(),
                "alloc::raw_vec::finish_grow".into(),
            ],
            allocated_bytes: 1024,
            allocations: 4,
            live_bytes: 256,
            live_blocks: 1,
        };
        let parsed = site.parsed_frames();
        assert_eq!(
            parsed[0],
            ("my_crate::run".into(), Some("src/lib.rs".into()), Some(42))
        );
        assert_eq!(
            parsed[1],
            ("alloc::raw_vec::finish_grow".into(), None, None)
        );

        let p = Profile {
            schema: "bench.memory/1".into(),
            scope: "test".into(),
            dropped_events: 0,
            events: 4,
            peak_tracked_bytes: 1024,
            sites: vec![site],
        };
        assert_eq!(p.total_allocated_bytes(), 1024);
        assert_eq!(p.total_allocations(), 4);
        assert_eq!(p.total_live_bytes(), 256);
        assert_eq!(p.total_live_blocks(), 1);
    }
}
