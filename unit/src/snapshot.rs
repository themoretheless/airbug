//! Text snapshots. Verification never writes; updates require an explicit mode.
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
/// Whether a snapshot run is allowed to write to disk, and how much.
///
/// The default never writes, so a snapshot cannot be accepted by accident in
/// CI. Widen it deliberately when you mean to record new expected output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UpdateMode {
    /// Compare only. A missing or differing snapshot is an error.
    #[default]
    Verify,
    /// Write snapshots that do not exist yet; still fail on a mismatch.
    CreateMissing,
    /// Write every snapshot, replacing whatever was there.
    Overwrite,
    /// Stub for rewriting expected values into test source.
    ///
    /// Source rewrite is not implemented: snapshot checks return
    /// [`SnapshotError::Unsupported`].
    InlineSource,
}
/// Pretty or compact JSON rendering for [`Snapshots::check_json`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JsonMode {
    /// Multi-line indented JSON (default).
    #[default]
    Pretty,
    /// Single-line compact JSON.
    Compact,
}
/// Why a snapshot check did not pass.
#[derive(Debug)]
pub enum SnapshotError {
    /// Reading or writing the snapshot file failed.
    Io(io::Error),
    /// The snapshot name is empty, too long, or has characters that are not
    /// safe in a file name.
    InvalidName(String),
    /// The text is larger than the configured [`Snapshots::max_bytes`] limit.
    TooLarge,
    /// No snapshot on disk, and the mode does not allow creating one.
    Missing(PathBuf),
    /// The text differs from the stored snapshot.
    Mismatch {
        /// File the text was compared against.
        path: PathBuf,
        /// Rendered difference, from [`text_diff`].
        diff: String,
    },
    /// Snapshot files exist that no current check referenced.
    Obsolete {
        /// Paths of unused snapshot files.
        paths: Vec<String>,
    },
    /// Requested feature is not available (for example [`UpdateMode::InlineSource`]).
    Unsupported(&'static str),
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
            Self::Obsolete { paths } => {
                write!(f, "obsolete snapshot files: {}", paths.join(", "))
            }
            Self::Unsupported(message) => write!(f, "unsupported: {message}"),
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
/// A directory of stored text snapshots, plus the policy for comparing
/// against them: update mode, size limit and redactions.
pub struct Snapshots {
    directory: PathBuf,
    mode: UpdateMode,
    replacements: Vec<(String, String)>,
    max_bytes: usize,
    #[cfg(feature = "json")]
    json_mode: JsonMode,
    schema_stamp: Option<u32>,
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
    /// Snapshots stored in `directory`, in [`UpdateMode::Verify`].
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            mode: UpdateMode::Verify,
            replacements: Vec::new(),
            max_bytes: 1_048_576,
            #[cfg(feature = "json")]
            json_mode: JsonMode::Pretty,
            schema_stamp: None,
        }
    }
    /// Choose whether this run may write snapshots.
    pub fn mode(mut self, mode: UpdateMode) -> Self {
        self.mode = mode;
        self
    }
    /// Pretty or compact JSON for [`check_json`](Snapshots::check_json).
    #[cfg(feature = "json")]
    pub fn json_mode(mut self, mode: JsonMode) -> Self {
        self.json_mode = mode;
        self
    }
    /// Prefix stored snapshots with `# airbug-snap-v{n}` when set.
    ///
    /// Comparisons always use the body after stripping a matching header, so
    /// older snaps without a header still compare correctly.
    pub fn schema_stamp(mut self, version: u32) -> Self {
        self.schema_stamp = Some(version);
        self
    }
    /// Refuse text larger than `limit` bytes, measured after redaction.
    pub fn max_bytes(mut self, limit: usize) -> Self {
        self.max_bytes = limit;
        self
    }
    /// Literal replacements run in declaration order on actual text before comparison/storage.
    /// Replace `text` with `replacement` before storing or comparing, so
    /// timestamps, ids and paths do not make snapshots unstable. Redaction runs
    /// before anything is written or reported.
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
    /// Compare `actual` against the snapshot named `name`.
    pub fn check(&self, name: &str, actual: &str) -> Result<(), SnapshotError> {
        Self::name(name)?;
        self.check_path(self.directory.join(format!("test-{name}.snap")), actual)
    }
    /// Snapshot `bytes` as lowercase hex (no spaces) via [`check`](Snapshots::check).
    pub fn check_bytes(&self, name: &str, bytes: &[u8]) -> Result<(), SnapshotError> {
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        self.check(name, &hex)
    }
    /// [`check`](Snapshots::check) for one case of a parameterised test, so
    /// each case gets its own file rather than overwriting a shared one.
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
    /// Compare against a literal written in the test instead of a file.
    /// Applies redaction and the size limit, and never writes anything.
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

    /// Canonical JSON snapshot (stable key order). Requires feature `json`.
    #[cfg(feature = "json")]
    pub fn check_json(
        &self,
        name: &str,
        actual: &serde_json::Value,
    ) -> Result<(), SnapshotError> {
        self.check(name, &canonical_json(actual, self.json_mode))
    }

    /// [`check_json`](Snapshots::check_json) with a per-case file.
    #[cfg(feature = "json")]
    pub fn check_json_case(
        &self,
        test: &str,
        case: &str,
        actual: &serde_json::Value,
    ) -> Result<(), SnapshotError> {
        self.check_case(test, case, &canonical_json(actual, self.json_mode))
    }

    /// Snapshot files under this directory that were not referenced by `used`.
    ///
    /// `used` are snapshot **names** (as passed to [`check`](Snapshots::check)),
    /// not full paths. Only top-level `test-*.snap` files are considered.
    pub fn list_obsolete(&self, used: &[&str]) -> Result<Vec<PathBuf>, SnapshotError> {
        let used: std::collections::BTreeSet<String> =
            used.iter().map(|n| format!("test-{n}.snap")).collect();
        let mut obsolete = Vec::new();
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(obsolete),
            Err(e) => return Err(e.into()),
        };
        for entry in entries {
            let entry = entry?;
            let meta = entry.metadata()?;
            if !meta.is_file() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.starts_with("test-") && name.ends_with(".snap") && !used.contains(name) {
                obsolete.push(entry.path());
            }
        }
        obsolete.sort();
        Ok(obsolete)
    }

    /// Fail if any top-level snapshot file is not in `used`.
    pub fn assert_no_obsolete(&self, used: &[&str]) -> Result<(), SnapshotError> {
        let obsolete = self.list_obsolete(used)?;
        if obsolete.is_empty() {
            Ok(())
        } else {
            Err(SnapshotError::Obsolete {
                paths: obsolete
                    .into_iter()
                    .map(|p| p.display().to_string())
                    .collect(),
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
    fn with_schema_header(&self, body: &str) -> String {
        match self.schema_stamp {
            Some(version) => format!("# airbug-snap-v{version}\n{body}"),
            None => body.to_string(),
        }
    }
    fn check_path(&self, path: PathBuf, actual: &str) -> Result<(), SnapshotError> {
        if self.mode == UpdateMode::InlineSource {
            return Err(SnapshotError::Unsupported(
                "UpdateMode::InlineSource source rewrite is unsupported",
            ));
        }
        let body = self.prepare(actual)?;
        let stored = self.with_schema_header(&body);
        if self.mode == UpdateMode::Verify {
            let expected = self.read(&path)?;
            let expected_body = expected.as_deref().map(strip_schema_header);
            return Self::compare(&path, expected_body, &body);
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
        let existing_body = existing.as_deref().map(strip_schema_header);
        let result = if existing_body == Some(body.as_str()) {
            Ok(())
        } else if existing.is_some() && self.mode == UpdateMode::CreateMissing {
            Self::compare(&path, existing_body, &body)
        } else {
            let temporary = path.with_extension("tmp");
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            let cleanup = Remove(temporary.clone());
            let written = file
                .write_all(stored.as_bytes())
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

fn strip_schema_header(text: &str) -> &str {
    let Some(rest) = text.strip_prefix("# airbug-snap-v") else {
        return text;
    };
    let Some(newline) = rest.find('\n') else {
        return text;
    };
    let version = &rest[..newline];
    if version.is_empty() || !version.bytes().all(|b| b.is_ascii_digit()) {
        return text;
    }
    &rest[newline + 1..]
}

/// Canonical JSON with recursively sorted object keys for stable snapshots.
#[cfg(feature = "json")]
pub fn canonical_json(value: &serde_json::Value, mode: JsonMode) -> String {
    fn sort(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut keys: Vec<_> = map.keys().cloned().collect();
                keys.sort();
                let mut out = serde_json::Map::new();
                for key in keys {
                    out.insert(key.clone(), sort(&map[&key]));
                }
                serde_json::Value::Object(out)
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(sort).collect())
            }
            other => other.clone(),
        }
    }
    let sorted = sort(value);
    let text = match mode {
        JsonMode::Pretty => serde_json::to_string_pretty(&sorted).expect("Value serializes"),
        JsonMode::Compact => serde_json::to_string(&sorted).expect("Value serializes"),
    };
    format!("{text}\n")
}
