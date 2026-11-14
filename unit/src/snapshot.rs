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
    /// Rewrite inline snapshot string literals directly in the calling Rust source file.
    ///
    /// Named file checks ([`Snapshots::check`], [`Snapshots::check_json`], etc.) remain
    /// read-only (equivalent to [`UpdateMode::Verify`]) in this mode.
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
    /// Requested feature is not available in the current configuration.
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
    /// Snapshots stored in `directory`, selecting [`UpdateMode::InlineSource`] when
    /// `AIRBUG_UPDATE_INLINE` is truthy (`1`, `true`, `yes`), otherwise [`UpdateMode::Verify`].
    pub fn from_inline_env(directory: impl Into<PathBuf>) -> Self {
        let update = std::env::var("AIRBUG_UPDATE_INLINE").ok().is_some_and(|v| {
            matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
        });
        Self::new(directory).mode(if update {
            UpdateMode::InlineSource
        } else {
            UpdateMode::Verify
        })
    }

    /// Compare against a literal written in the test instead of a file.
    /// Applies redaction and the size limit.
    ///
    /// When [`UpdateMode::InlineSource`] is enabled, a mismatch rewrites the matching
    /// expected string literal in the calling Rust source file atomically.
    #[track_caller]
    pub fn inline(&self, expected: &str, actual: &str) -> Result<(), SnapshotError> {
        if expected.len() > self.max_bytes {
            return Err(SnapshotError::TooLarge);
        }
        let actual = self.prepare(actual)?;
        if expected == actual {
            return Ok(());
        }
        if self.mode == UpdateMode::InlineSource {
            let loc = std::panic::Location::caller();
            return self.rewrite_inline_source(
                Path::new(loc.file()),
                loc.line(),
                loc.column(),
                expected,
                &actual,
                false,
            );
        }
        Err(SnapshotError::Mismatch {
            path: PathBuf::from("<inline>"),
            diff: text_diff(expected, &actual),
        })
    }

    /// Compare `actual` against an inline string literal at explicit source `(file, line, column)`.
    ///
    /// When [`UpdateMode::InlineSource`] is enabled, a mismatch rewrites the matching
    /// string literal in `file` atomically while tracking per-file line deltas across
    /// subsequent updates in the same process.
    pub fn inline_source(
        &self,
        file: impl AsRef<Path>,
        line: u32,
        column: u32,
        expected: &str,
        actual: &str,
    ) -> Result<(), SnapshotError> {
        if expected.len() > self.max_bytes {
            return Err(SnapshotError::TooLarge);
        }
        let actual = self.prepare(actual)?;
        if expected == actual {
            return Ok(());
        }
        let file = file.as_ref();
        if self.mode == UpdateMode::InlineSource {
            return self.rewrite_inline_source(file, line, column, expected, &actual, false);
        }
        Err(SnapshotError::Mismatch {
            path: file.to_path_buf(),
            diff: text_diff(expected, &actual),
        })
    }

    #[doc(hidden)]
    pub fn inline_source_macro(
        &self,
        file: &str,
        line: u32,
        column: u32,
        expected: &str,
        actual: &str,
    ) -> Result<(), SnapshotError> {
        if expected.len() > self.max_bytes {
            return Err(SnapshotError::TooLarge);
        }
        let actual = self.prepare(actual)?;
        if expected == actual {
            return Ok(());
        }
        if self.mode == UpdateMode::InlineSource {
            return self.rewrite_inline_source(
                Path::new(file),
                line,
                column,
                expected,
                &actual,
                true,
            );
        }
        Err(SnapshotError::Mismatch {
            path: PathBuf::from(file),
            diff: text_diff(expected, &actual),
        })
    }

    fn rewrite_inline_source(
        &self,
        file: &Path,
        line: u32,
        column: u32,
        expected: &str,
        actual: &str,
        prefer_last: bool,
    ) -> Result<(), SnapshotError> {
        let resolved =
            resolve_source_path(file).ok_or_else(|| SnapshotError::Missing(file.to_path_buf()))?;
        let canonical = resolved.canonicalize().unwrap_or(resolved);
        let deltas_lock = INLINE_LINE_DELTAS
            .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        let mut deltas = deltas_lock.lock().unwrap_or_else(|e| e.into_inner());
        let delta = *deltas.get(&canonical).unwrap_or(&0);
        let effective_line = (line as isize + delta).max(1) as usize;
        let source = fs::read_to_string(&canonical)?;
        let use_crlf = source.contains("\r\n");
        let literal_span = find_inline_literal_span(
            &source,
            effective_line,
            column as usize,
            expected,
            prefer_last,
        )
        .ok_or(SnapshotError::Unsupported(
            "inline source target string literal not found at call site",
        ))?;
        let mut replacement = format_rust_string_literal(actual);
        if use_crlf && replacement.contains('\n') && !replacement.contains("\r\n") {
            replacement = replacement.replace('\n', "\r\n");
        }
        let old_slice = &source[literal_span.start..literal_span.end];
        let old_lines = old_slice.bytes().filter(|&b| b == b'\n').count() as isize;
        let new_lines = replacement.bytes().filter(|&b| b == b'\n').count() as isize;
        let mut updated = String::with_capacity(
            source.len() - (literal_span.end - literal_span.start) + replacement.len(),
        );
        updated.push_str(&source[..literal_span.start]);
        updated.push_str(&replacement);
        updated.push_str(&source[literal_span.end..]);
        write_atomic_source(&canonical, updated.as_bytes())?;
        *deltas.entry(canonical).or_insert(0) += new_lines - old_lines;
        Ok(())
    }

    /// Canonical JSON snapshot (stable key order). Requires feature `json`.
    #[cfg(feature = "json")]
    pub fn check_json(&self, name: &str, actual: &serde_json::Value) -> Result<(), SnapshotError> {
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
        let body = self.prepare(actual)?;
        let stored = self.with_schema_header(&body);
        if matches!(self.mode, UpdateMode::Verify | UpdateMode::InlineSource) {
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

/// Assert that an expression matches an inline expected string literal, optionally rewriting the
/// caller's source file when `AIRBUG_UPDATE_INLINE=1` or when using a [`Snapshots`] instance
/// configured with [`UpdateMode::InlineSource`].
///
/// # Forms
///
/// - `assert_inline_snapshot!(actual, "expected")`
/// - `assert_inline_snapshot!(actual, @"expected")`
/// - `assert_inline_snapshot!(snapshots => actual, "expected")`
/// - `assert_inline_snapshot!(snapshots => actual, @"expected")`
#[macro_export]
macro_rules! assert_inline_snapshot {
    ($actual:expr, @$expected:literal $(,)?) => {{
        let __snaps = $crate::snapshot::Snapshots::from_inline_env(".");
        $crate::assert_inline_snapshot!(__snaps => $actual, @$expected)
    }};
    ($actual:expr, $expected:literal $(,)?) => {{
        let __snaps = $crate::snapshot::Snapshots::from_inline_env(".");
        $crate::assert_inline_snapshot!(__snaps => $actual, $expected)
    }};
    ($snapshots:expr => $actual:expr, @$expected:literal $(,)?) => {{
        let __actual_str: &str = ::std::convert::AsRef::<str>::as_ref(&($actual));
        if let ::std::result::Result::Err(__err) = $crate::snapshot::Snapshots::inline_source_macro(
            &($snapshots),
            ::std::file!(),
            ::std::line!(),
            ::std::column!(),
            $expected,
            __actual_str,
        ) {
            ::std::panic!("inline snapshot assertion failed: {__err}");
        }
    }};
    ($snapshots:expr => $actual:expr, $expected:literal $(,)?) => {{
        let __actual_str: &str = ::std::convert::AsRef::<str>::as_ref(&($actual));
        if let ::std::result::Result::Err(__err) = $crate::snapshot::Snapshots::inline_source_macro(
            &($snapshots),
            ::std::file!(),
            ::std::line!(),
            ::std::column!(),
            $expected,
            __actual_str,
        ) {
            ::std::panic!("inline snapshot assertion failed: {__err}");
        }
    }};
}

static INLINE_LINE_DELTAS: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<PathBuf, isize>>,
> = std::sync::OnceLock::new();
static INLINE_TMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
struct LiteralSpan {
    start: usize,
    end: usize,
}

fn resolve_source_path(file: &Path) -> Option<PathBuf> {
    if file.is_absolute() {
        return file.exists().then(|| file.to_path_buf());
    }
    if file.exists() {
        return Some(file.to_path_buf());
    }
    if let Ok(cwd) = std::env::current_dir() {
        for base in cwd.ancestors() {
            let candidate = base.join(file);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        for base in manifest_path.ancestors() {
            let candidate = base.join(file);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn write_atomic_source(path: &Path, bytes: &[u8]) -> Result<(), SnapshotError> {
    use std::io::Write;
    let seq = INLINE_TMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("snapshot");
    let temporary = path.with_file_name(format!(".{file_name}.airbug-tmp-{pid}-{seq}"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let cleanup = Remove(temporary.clone());
    file.write_all(bytes).and_then(|_| file.sync_all())?;
    drop(file);
    fs::rename(&temporary, path)?;
    drop(cleanup);
    Ok(())
}

fn line_col_to_offset(src: &str, target_line: usize, target_col: usize) -> Option<usize> {
    let mut current_line = 1usize;
    let mut line_start = 0usize;
    if target_line > 1 {
        for (idx, b) in src.bytes().enumerate() {
            if b == b'\n' {
                current_line += 1;
                line_start = idx + 1;
                if current_line == target_line {
                    break;
                }
            }
        }
        if current_line != target_line {
            return None;
        }
    }
    if target_col <= 1 {
        return Some(line_start);
    }
    let line_end = src[line_start..]
        .find('\n')
        .map_or(src.len(), |rel| line_start + rel);
    let line_slice = &src[line_start..line_end];
    let mut offset_in_line = line_slice.len();
    for (col_idx, (byte_idx, _)) in line_slice.char_indices().enumerate() {
        if col_idx + 1 == target_col {
            offset_in_line = byte_idx;
            break;
        }
    }
    Some(line_start + offset_in_line)
}

fn find_inline_literal_span(
    src: &str,
    line: usize,
    column: usize,
    expected: &str,
    prefer_last: bool,
) -> Option<LiteralSpan> {
    let start = line_col_to_offset(src, line, column)?;
    let expected_lf = expected.replace("\r\n", "\n");
    let bytes = src.as_bytes();
    let mut i = start;
    let mut depth = 0usize;
    let mut entered_call = false;
    let mut candidates = Vec::new();
    let mut seen_lines = 0usize;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\n' {
            seen_lines += 1;
            if !entered_call && seen_lines > 4 {
                break;
            }
            if seen_lines > 128 {
                break;
            }
            i += 1;
            continue;
        }
        // Skip line comments
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Skip block comments (supports nesting)
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            let mut comment_depth = 1usize;
            while i + 1 < bytes.len() && comment_depth > 0 {
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    comment_depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    comment_depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Skip byte / c-string literals without confusing trailing 'r' with raw string
        if matches!(b, b'b' | b'c')
            && !is_ident_continue_before(bytes, i)
            && let Some(next_idx) = skip_prefixed_string(bytes, i + 1)
        {
            i = next_idx;
            continue;
        }
        // Skip char literals (carefully distinguish from lifetimes like `'a`)
        if b == b'\'' {
            if let Some(next_idx) = try_skip_char_literal(src, i) {
                i = next_idx;
                continue;
            }
            i += 1;
            continue;
        }
        // Check raw string literal: r"..." or r#"..."#
        if b == b'r'
            && !is_ident_continue_before(bytes, i)
            && let Some((end_idx, value)) = parse_raw_string_at(src, i)
        {
            if value == expected_lf {
                candidates.push(LiteralSpan {
                    start: i,
                    end: end_idx,
                });
            }
            i = end_idx;
            continue;
        }
        // Check normal string literal: "..."
        if b == b'"' {
            if let Some((end_idx, value)) = parse_quoted_string_at(src, i) {
                if value == expected_lf || value == expected {
                    candidates.push(LiteralSpan {
                        start: i,
                        end: end_idx,
                    });
                }
                i = end_idx;
                continue;
            }
            i += 1;
            continue;
        }
        match b {
            b'(' | b'[' | b'{' => {
                entered_call = true;
                depth += 1;
            }
            b')' | b']' | b'}' => {
                if entered_call {
                    if depth <= 1 {
                        break;
                    }
                    depth -= 1;
                } else {
                    break;
                }
            }
            b';' if !entered_call => break,
            _ => {}
        }
        i += 1;
    }

    if prefer_last {
        candidates.last().copied()
    } else {
        candidates.first().copied()
    }
}

fn is_ident_continue_before(bytes: &[u8], idx: usize) -> bool {
    idx > 0 && (bytes[idx - 1].is_ascii_alphanumeric() || bytes[idx - 1] == b'_')
}

fn skip_prefixed_string(bytes: &[u8], idx: usize) -> Option<usize> {
    if idx >= bytes.len() {
        return None;
    }
    if bytes[idx] == b'"' {
        let mut i = idx + 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i = (i + 2).min(bytes.len());
                continue;
            }
            if bytes[i] == b'"' {
                return Some(i + 1);
            }
            i += 1;
        }
        return None;
    }
    if bytes[idx] == b'r' {
        let mut hashes = 0usize;
        let mut p = idx + 1;
        while p < bytes.len() && bytes[p] == b'#' {
            hashes += 1;
            p += 1;
        }
        if p < bytes.len() && bytes[p] == b'"' {
            p += 1;
            while p < bytes.len() {
                if bytes[p] == b'"'
                    && p + hashes < bytes.len()
                    && bytes[p + 1..=p + hashes].iter().all(|&c| c == b'#')
                {
                    return Some(p + 1 + hashes);
                }
                p += 1;
            }
        }
    }
    None
}

fn try_skip_char_literal(src: &str, quote_idx: usize) -> Option<usize> {
    let rest = src.get(quote_idx + 1..)?;
    if let Some(after_slash) = rest.strip_prefix('\\') {
        let mut chars = after_slash.char_indices();
        let (_, esc) = chars.next()?;
        let after_esc = match esc {
            'x' => {
                let mut hex_end = 1;
                for (offset, ch) in after_slash[1..].char_indices().take(2) {
                    if !ch.is_ascii_hexdigit() {
                        return None;
                    }
                    hex_end = 1 + offset + ch.len_utf8();
                }
                &after_slash[hex_end..]
            }
            'u' => {
                let braced = after_slash.get(1..)?.strip_prefix('{')?;
                let close = braced.find('}')?;
                &braced[close + 1..]
            }
            _ => &after_slash[esc.len_utf8()..],
        };
        if after_esc.starts_with('\'') {
            let consumed = src.len() - after_esc.len() + 1;
            return Some(consumed);
        }
        return None;
    }
    let mut chars = rest.char_indices();
    let (_, ch) = chars.next()?;
    if ch == '\'' || ch == '\n' {
        return None;
    }
    let after = rest.get(ch.len_utf8()..)?;
    after
        .starts_with('\'')
        .then(|| quote_idx + 1 + ch.len_utf8() + 1)
}

fn parse_raw_string_at(src: &str, r_idx: usize) -> Option<(usize, String)> {
    let bytes = src.as_bytes();
    if bytes.get(r_idx) != Some(&b'r') {
        return None;
    }
    let mut hashes = 0usize;
    let mut p = r_idx + 1;
    while p < bytes.len() && bytes[p] == b'#' {
        hashes += 1;
        p += 1;
    }
    if bytes.get(p) != Some(&b'"') {
        return None;
    }
    let body_start = p + 1;
    p = body_start;
    while p < bytes.len() {
        if bytes[p] == b'"'
            && p + hashes < bytes.len()
            && bytes[p + 1..=p + hashes].iter().all(|&c| c == b'#')
        {
            let value = src[body_start..p].replace("\r\n", "\n");
            return Some((p + 1 + hashes, value));
        }
        p += 1;
    }
    None
}

fn parse_quoted_string_at(src: &str, quote_idx: usize) -> Option<(usize, String)> {
    let mut out = String::new();
    let mut chars = src[quote_idx + 1..].char_indices();
    while let Some((idx, ch)) = chars.next() {
        match ch {
            '"' => return Some((quote_idx + 1 + idx + 1, out)),
            '\r' => {
                if matches!(chars.clone().next(), Some((_, '\n'))) {
                    continue;
                }
                out.push('\r');
            }
            '\\' => {
                let (_, esc) = chars.next()?;
                match esc {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\\' => out.push('\\'),
                    '0' => out.push('\0'),
                    '"' => out.push('"'),
                    '\'' => out.push('\''),
                    'x' => {
                        let (_, h1) = chars.next()?;
                        let (_, h2) = chars.next()?;
                        let byte = u8::from_str_radix(&format!("{h1}{h2}"), 16).ok()?;
                        out.push(byte as char);
                    }
                    'u' => {
                        if !matches!(chars.next(), Some((_, '{'))) {
                            return None;
                        }
                        let mut hex = String::new();
                        let mut closed = false;
                        for (_, c) in chars.by_ref() {
                            if c == '}' {
                                closed = true;
                                break;
                            }
                            if c != '_' {
                                hex.push(c);
                            }
                        }
                        if !closed {
                            return None;
                        }
                        let code = u32::from_str_radix(&hex, 16).ok()?;
                        out.push(char::from_u32(code)?);
                    }
                    '\r' | '\n' => {
                        while let Some((_, ws)) = chars.clone().next() {
                            if ws == ' ' || ws == '\t' || ws == '\n' || ws == '\r' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
            }
            other => out.push(other),
        }
    }
    None
}

fn format_rust_string_literal(value: &str) -> String {
    let use_raw = !value.contains('\r')
        && !value.chars().any(|c| c.is_control() && c != '\n')
        && (value.contains('\n') || value.contains('"') || value.contains('\\'));
    if use_raw {
        let mut hashes = 1usize;
        loop {
            let mut closing = String::with_capacity(1 + hashes);
            closing.push('"');
            for _ in 0..hashes {
                closing.push('#');
            }
            if !value.contains(&closing) {
                let mut out = String::with_capacity(value.len() + hashes * 2 + 3);
                out.push('r');
                for _ in 0..hashes {
                    out.push('#');
                }
                out.push('"');
                out.push_str(value);
                out.push_str(&closing);
                return out;
            }
            hashes += 1;
        }
    }
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => {
                for esc in c.escape_default() {
                    out.push(esc);
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
