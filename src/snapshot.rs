//! Text snapshots. Verification never writes; updates require an explicit mode.
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UpdateMode {
    #[default]
    Verify,
    CreateMissing,
    Overwrite,
}
#[derive(Debug)]
pub enum SnapshotError {
    Io(io::Error),
    InvalidName(String),
    TooLarge,
    Missing(PathBuf),
    Mismatch { path: PathBuf, diff: String },
}
impl From<io::Error> for SnapshotError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "snapshot I/O: {e}"),
            Self::InvalidName(name) => write!(
                f,
                "invalid snapshot name {name:?}; use 1..=100 ASCII letters, digits, '_' or '-'"
            ),
            Self::TooLarge => write!(f, "snapshot exceeds byte limit"),
            Self::Missing(path) => write!(
                f,
                "missing snapshot {}; explicitly use CreateMissing to create it",
                path.display()
            ),
            Self::Mismatch { path, diff } => {
                write!(f, "snapshot differs at {}\n{diff}", path.display())
            }
        }
    }
}
impl std::error::Error for SnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
/// Bounded line-oriented diagnostic. Escaping makes whitespace differences visible.
pub fn text_diff(expected: &str, actual: &str) -> String {
    let mut result = String::new();
    let expected: Vec<_> = expected.split('\n').collect();
    let actual: Vec<_> = actual.split('\n').collect();
    let mut differences = 0;
    for index in 0..expected.len().max(actual.len()) {
        let old = expected.get(index);
        let new = actual.get(index);
        if old != new {
            if differences == 32 {
                result.push_str("... remaining differences omitted\n");
                break;
            }
            differences += 1;
            result.push_str(&format!(
                "@@ line {} @@\n- {:?}\n+ {:?}\n",
                index + 1,
                old,
                new
            ));
        }
    }
    result
}
#[derive(Debug, Clone)]
pub struct Snapshots {
    directory: PathBuf,
    mode: UpdateMode,
    replacements: Vec<(String, String)>,
    max_bytes: usize,
}
struct LockedFile {
    path: PathBuf,
    file: Option<fs::File>,
}
impl Drop for LockedFile {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}
struct Remove(PathBuf);
impl Drop for Remove {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
impl Snapshots {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            mode: UpdateMode::Verify,
            replacements: Vec::new(),
            max_bytes: 1_048_576,
        }
    }
    pub fn mode(mut self, mode: UpdateMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn max_bytes(mut self, limit: usize) -> Self {
        self.max_bytes = limit;
        self
    }
    /// Literal replacements run in declaration order on actual text before comparison/storage.
    pub fn redact(mut self, text: impl Into<String>, replacement: impl Into<String>) -> Self {
        let text = text.into();
        assert!(!text.is_empty(), "redaction text cannot be empty");
        self.replacements.push((text, replacement.into()));
        self
    }
    fn prepare(&self, actual: &str) -> Result<String, SnapshotError> {
        if actual.len() > self.max_bytes {
            return Err(SnapshotError::TooLarge);
        }
        let mut value = actual.to_string();
        for (from, to) in &self.replacements {
            // Check expansion before allocating its result.
            let count = value.matches(from).count();
            let len = value
                .len()
                .checked_sub(count * from.len())
                .and_then(|n| {
                    count
                        .checked_mul(to.len())
                        .and_then(|extra| n.checked_add(extra))
                })
                .ok_or(SnapshotError::TooLarge)?;
            if len > self.max_bytes {
                return Err(SnapshotError::TooLarge);
            }
            value = value.replace(from, to);
        }
        Ok(value)
    }
    fn name(name: &str) -> Result<(), SnapshotError> {
        if name.is_empty()
            || name.len() > 100
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(SnapshotError::InvalidName(name.into()));
        }
        Ok(())
    }
    pub fn check(&self, name: &str, actual: &str) -> Result<(), SnapshotError> {
        Self::name(name)?;
        self.check_path(self.directory.join(format!("test-{name}.snap")), actual)
    }
    pub fn check_case(&self, test: &str, case: &str, actual: &str) -> Result<(), SnapshotError> {
        Self::name(test)?;
        Self::name(case)?;
        self.check_path(
            self.directory
                .join(format!("test-{test}"))
                .join(format!("case-{case}.snap")),
            actual,
        )
    }
    /// Compare an inline literal; never edits source files, regardless of update mode.
    pub fn inline(&self, expected: &str, actual: &str) -> Result<(), SnapshotError> {
        if expected.len() > self.max_bytes {
            return Err(SnapshotError::TooLarge);
        }
        let actual = self.prepare(actual)?;
        if expected == actual {
            Ok(())
        } else {
            Err(SnapshotError::Mismatch {
                path: PathBuf::from("<inline>"),
                diff: text_diff(expected, &actual),
            })
        }
    }
    fn read(&self, path: &Path) -> Result<Option<String>, SnapshotError> {
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                if !meta.file_type().is_file() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "snapshot must be a regular file",
                    )
                    .into());
                }
                if meta.len() > self.max_bytes as u64 {
                    return Err(SnapshotError::TooLarge);
                }
                let value = fs::read_to_string(path)?;
                if value.len() > self.max_bytes {
                    return Err(SnapshotError::TooLarge);
                }
                Ok(Some(value))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    fn compare(path: &Path, expected: Option<&str>, actual: &str) -> Result<(), SnapshotError> {
        match expected {
            Some(expected) if expected == actual => Ok(()),
            Some(expected) => Err(SnapshotError::Mismatch {
                path: path.to_path_buf(),
                diff: text_diff(expected, actual),
            }),
            None => Err(SnapshotError::Missing(path.to_path_buf())),
        }
    }
    fn check_path(&self, path: PathBuf, actual: &str) -> Result<(), SnapshotError> {
        let actual = self.prepare(actual)?;
        if self.mode == UpdateMode::Verify {
            return Self::compare(&path, self.read(&path)?.as_deref(), &actual);
        }
        let parent = path.parent().expect("snapshot parent");
        fs::create_dir_all(parent)?;
        if fs::symlink_metadata(parent)?.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot parent cannot be a symlink",
            )
            .into());
        }
        let lock_path = path.with_extension("lock");
        let lock = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)?;
        let lock_cleanup = LockedFile {
            path: lock_path,
            file: Some(lock),
        };
        let existing = self.read(&path)?;
        let result = if existing.as_deref() == Some(&actual) {
            Ok(())
        } else if existing.is_some() && self.mode == UpdateMode::CreateMissing {
            Self::compare(&path, existing.as_deref(), &actual)
        } else {
            let temporary = path.with_extension("tmp");
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            let cleanup = Remove(temporary.clone());
            let written = file
                .write_all(actual.as_bytes())
                .and_then(|_| file.sync_all());
            drop(file);
            written?;
            fs::rename(&temporary, &path)?;
            drop(cleanup);
            Ok(())
        };
        drop(lock_cleanup);
        result
    }
}
