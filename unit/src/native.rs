//! Additive assertions for ordinary #[test] functions.
use std::{
    env,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

/// Unique temporary directory removed on [`Drop`].
#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Create a new empty directory under the process temp dir.
    pub fn new() -> io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let serial = TEMP_SERIAL.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "airbug-temp-{}-{stamp}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// Borrowed path of the temporary directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Owned path of the temporary directory.
    pub fn path_buf(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Sets an environment variable and restores the previous value on [`Drop`].
#[derive(Debug)]
pub struct EnvScope {
    key: OsString,
    previous: Option<OsString>,
}

impl EnvScope {
    /// Set `key` to `value` for the lifetime of the returned guard.
    #[allow(unsafe_code)]
    pub fn set(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        let key = key.as_ref().to_os_string();
        let previous = env::var_os(&key);
        // SAFETY: test-only scoped mutation; restored on Drop for this process.
        unsafe {
            env::set_var(&key, value.as_ref());
        }
        Self { key, previous }
    }
}

impl Drop for EnvScope {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: restores the value captured in `set` for this process.
        unsafe {
            match &self.previous {
                Some(value) => env::set_var(&self.key, value),
                None => env::remove_var(&self.key),
            }
        }
    }
}

/// Changes the process working directory and restores it on [`Drop`].
#[derive(Debug)]
pub struct DirScope {
    previous: PathBuf,
}

impl DirScope {
    /// `chdir` into `path` until the guard is dropped.
    pub fn set(path: impl AsRef<Path>) -> io::Result<Self> {
        let previous = env::current_dir()?;
        env::set_current_dir(path.as_ref())?;
        Ok(Self { previous })
    }
}

impl Drop for DirScope {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.previous);
    }
}

/// Check collection membership. Each expression is evaluated exactly once.
#[macro_export]
macro_rules! assert_contains {
    ($collection:expr, $expected:expr $(,)?) => {{
        match (&$collection, &$expected) {
            (collection, expected) => assert!(
                collection.contains(expected),
                "expected {:?} to contain {:?}",
                collection,
                expected
            ),
        }
    }};
}

/// Compare slices ignoring order, but preserving duplicate counts. O(n²).
#[track_caller]
pub fn assert_same_items<T: PartialEq + std::fmt::Debug>(actual: &[T], expected: &[T]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "different collection lengths: actual {actual:?}, expected {expected:?}"
    );
    let mut matched = vec![false; expected.len()];
    for item in actual {
        let index = expected
            .iter()
            .enumerate()
            .position(|(index, value)| !matched[index] && item == value);
        match index {
            Some(index) => matched[index] = true,
            None => panic!(
                "different collection contents: actual {actual:?}, expected {expected:?}; unmatched {item:?}"
            ),
        }
    }
}

/// Compare collections without imposing an ordering or requiring Ord/Hash.
#[macro_export]
macro_rules! assert_same_items {
    ($actual:expr, $expected:expr $(,)?) => {{
        $crate::native::assert_same_items(&$actual, &$expected);
    }};
}

/// Evaluate every boolean check and report all false expressions together.
/// User panics propagate immediately; this does not catch assert! panics.
#[macro_export]
macro_rules! check_all {
    ($($check:expr);+ $(;)?) => {{
        let mut failures = ::std::vec::Vec::<&str>::new();
        $(if !$check { failures.push(stringify!($check)); })+
        assert!(failures.is_empty(), "failed checks:\n{}", failures.join("\n"));
    }};
}
