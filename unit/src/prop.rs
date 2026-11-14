//! Seeded property checks with optional counterexample persistence.
//!
//! Shrinking is intentionally minimal (not a full property framework); see
//! [`Shrink`] and [`Prop::shrink_from`].
use crate::fixture::{FixtureContext, Generate, GenerationError};
use crate::strategy::Strategy;
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Why a property check failed.
#[derive(Debug)]
pub enum PropError {
    /// Value generation failed for a trial.
    Generation(GenerationError),
    /// Counterexample I/O failed.
    Io(io::Error),
    /// A trial returned `false` or an error message.
    Failed(PropFailure),
}

impl fmt::Display for PropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generation(e) => write!(f, "property generation: {e}"),
            Self::Io(e) => write!(f, "property I/O: {e}"),
            Self::Failed(fail) => write!(f, "{fail}"),
        }
    }
}

impl std::error::Error for PropError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Generation(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Failed(_) => None,
        }
    }
}

impl From<GenerationError> for PropError {
    fn from(value: GenerationError) -> Self {
        Self::Generation(value)
    }
}

impl From<io::Error> for PropError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// A failed property trial that can be replayed from its seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropFailure {
    /// Seed that reproduced the failure.
    pub seed: u64,
    /// Zero-based trial index within the run.
    pub trial: usize,
    /// Optional detail from the property (or a default message).
    pub message: String,
}

impl fmt::Display for PropFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "property failed on trial {} with seed {}: {}",
            self.trial, self.seed, self.message
        )
    }
}

impl PropFailure {
    /// Persist `seed=<n>` to `path` for later [`Prop::replay`].
    pub fn write_seed_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(path)?;
        writeln!(file, "seed={}", self.seed)?;
        file.sync_all()
    }

    /// Read a seed previously written by [`PropFailure::write_seed_file`].
    pub fn read_seed_file(path: impl AsRef<Path>) -> io::Result<u64> {
        let text = fs::read_to_string(path)?;
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("seed=") {
                return rest
                    .trim()
                    .parse()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "seed file missing seed=<u64> line",
        ))
    }
}

/// Minimal shrinking candidates for a value (not a full property framework).
///
/// Implementors return simpler values to try after a failure. Prefer writing a
/// focused regression over relying on deep shrink trees.
pub trait Shrink: Sized {
    /// Candidate values strictly simpler than `self` (may be empty).
    fn shrink(&self) -> Vec<Self>;
}

macro_rules! impl_shrink_unsigned {
    ($($t:ty),* $(,)?) => {$(
        impl Shrink for $t {
            fn shrink(&self) -> Vec<Self> {
                let mut out = Vec::new();
                if *self > 0 {
                    out.push(0);
                    let mut step = *self / 2;
                    while step > 0 {
                        let candidate = *self - step;
                        if candidate != *self {
                            out.push(candidate);
                        }
                        step /= 2;
                    }
                    let pred = *self - 1;
                    if pred != 0 {
                        out.push(pred);
                    }
                    out.sort_unstable();
                    out.dedup();
                    out.retain(|v| *v < *self);
                }
                out
            }
        }
    )*};
}

impl_shrink_unsigned!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_shrink_signed {
    ($($t:ty),* $(,)?) => {$(
        impl Shrink for $t {
            fn shrink(&self) -> Vec<Self> {
                let mut out = Vec::new();
                if *self == 0 {
                    return out;
                }
                out.push(0);
                if *self < 0 {
                    if let Some(pos) = self.checked_neg() {
                        out.push(pos);
                    }
                    let mut lo = *self;
                    loop {
                        let next = lo / 2;
                        if next != 0 && next != *self {
                            out.push(next);
                        }
                        let mid = lo + (0 - lo) / 2;
                        if mid == lo || mid == 0 {
                            break;
                        }
                        if mid != *self {
                            out.push(mid);
                        }
                        lo = mid;
                    }
                    out.push(self.saturating_add(1));
                    out.retain(|v| v.unsigned_abs() < self.unsigned_abs() || (*v > 0 && (*v as u128) == (self.unsigned_abs() as u128)));
                } else {
                    let mut hi = *self;
                    while hi > 1 {
                        let half = hi / 2;
                        if half != 0 && half != *self {
                            out.push(half);
                        }
                        hi = half;
                    }
                    out.push(self.saturating_sub(1));
                    out.retain(|v| *v >= 0 && *v < *self);
                }
                out.sort_unstable();
                out.dedup();
                out
            }
        }
    )*};
}

impl_shrink_signed!(i8, i16, i32, i64, i128, isize);

macro_rules! impl_shrink_float {
    ($($t:ty),* $(,)?) => {$(
        impl Shrink for $t {
            fn shrink(&self) -> Vec<Self> {
                if *self == 0.0 {
                    return Vec::new();
                }
                if !self.is_finite() {
                    return vec![0.0];
                }
                let mut out = vec![0.0];
                if *self < 0.0 {
                    out.push(-*self);
                }
                let trunc = self.trunc();
                if trunc != *self && trunc.abs() < self.abs() {
                    out.push(trunc);
                }
                let half = *self / 2.0;
                if half.is_finite() && half != *self && half.abs() < self.abs() {
                    out.push(half);
                }
                out.retain(|v| *v != *self && (v.abs() < self.abs() || (*self < 0.0 && *v == -*self)));
                out
            }
        }
    )*};
}

impl_shrink_float!(f32, f64);

impl Shrink for bool {
    fn shrink(&self) -> Vec<Self> {
        if *self { vec![false] } else { Vec::new() }
    }
}

fn char_rank(ch: char) -> (u8, u32) {
    match ch {
        'a' => (0, 0),
        ' ' => (0, 1),
        '0' => (0, 2),
        'a'..='z' => (1, ch as u32),
        '0'..='9' => (1, ch as u32 + 100),
        'A'..='Z' => (2, ch as u32),
        c if c.is_ascii_graphic() => (3, c as u32),
        c if c.is_ascii() => (4, c as u32),
        c => (5, c as u32),
    }
}

impl Shrink for char {
    fn shrink(&self) -> Vec<Self> {
        let my_rank = char_rank(*self);
        let mut candidates = vec!['a', ' ', '0'];
        if self.is_ascii_uppercase() {
            candidates.push(self.to_ascii_lowercase());
        } else if !self.is_ascii() {
            candidates.push('z');
        }
        candidates.retain(|c| char_rank(*c) < my_rank);
        candidates.sort_by_key(|c| char_rank(*c));
        candidates.dedup();
        candidates
    }
}

impl<T: Shrink> Shrink for Option<T> {
    fn shrink(&self) -> Vec<Self> {
        match self {
            None => Vec::new(),
            Some(inner) => {
                let mut out = vec![None];
                out.extend(inner.shrink().into_iter().map(Some));
                out
            }
        }
    }
}

impl<T: Shrink + Clone, E: Shrink + Clone> Shrink for Result<T, E> {
    fn shrink(&self) -> Vec<Self> {
        match self {
            Ok(v) => v.shrink().into_iter().map(Ok).collect(),
            Err(e) => e.shrink().into_iter().map(Err).collect(),
        }
    }
}

impl<T: Shrink + Clone> Shrink for Vec<T> {
    fn shrink(&self) -> Vec<Self> {
        if self.is_empty() {
            return Vec::new();
        }
        let mut out = vec![Vec::new()];
        if self.len() > 1 {
            let mid = self.len() / 2;
            out.push(self[..mid].to_vec());
            out.push(self[mid..].to_vec());
            for idx in 0..self.len() {
                let mut copy = self.clone();
                copy.remove(idx);
                out.push(copy);
            }
        }
        for (idx, item) in self.iter().enumerate() {
            for shrunk in item.shrink() {
                let mut copy = self.clone();
                copy[idx] = shrunk;
                out.push(copy);
            }
        }
        out
    }
}

impl<T: Shrink + Clone, const N: usize> Shrink for [T; N] {
    fn shrink(&self) -> Vec<Self> {
        let mut out = Vec::new();
        for idx in 0..N {
            for shrunk in self[idx].shrink() {
                let mut copy = self.clone();
                copy[idx] = shrunk;
                out.push(copy);
            }
        }
        out
    }
}

impl<A: Shrink + Clone> Shrink for (A,) {
    fn shrink(&self) -> Vec<Self> {
        self.0.shrink().into_iter().map(|a| (a,)).collect()
    }
}

impl<A: Shrink + Clone, B: Shrink + Clone> Shrink for (A, B) {
    fn shrink(&self) -> Vec<Self> {
        let mut out = Vec::new();
        for a in self.0.shrink() {
            out.push((a, self.1.clone()));
        }
        for b in self.1.shrink() {
            out.push((self.0.clone(), b));
        }
        out
    }
}

impl<A: Shrink + Clone, B: Shrink + Clone, C: Shrink + Clone> Shrink for (A, B, C) {
    fn shrink(&self) -> Vec<Self> {
        let mut out = Vec::new();
        for a in self.0.shrink() {
            out.push((a, self.1.clone(), self.2.clone()));
        }
        for b in self.1.shrink() {
            out.push((self.0.clone(), b, self.2.clone()));
        }
        for c in self.2.shrink() {
            out.push((self.0.clone(), self.1.clone(), c));
        }
        out
    }
}

impl<A: Shrink + Clone, B: Shrink + Clone, C: Shrink + Clone, D: Shrink + Clone> Shrink
    for (A, B, C, D)
{
    fn shrink(&self) -> Vec<Self> {
        let mut out = Vec::new();
        for a in self.0.shrink() {
            out.push((a, self.1.clone(), self.2.clone(), self.3.clone()));
        }
        for b in self.1.shrink() {
            out.push((self.0.clone(), b, self.2.clone(), self.3.clone()));
        }
        for c in self.2.shrink() {
            out.push((self.0.clone(), self.1.clone(), c, self.3.clone()));
        }
        for d in self.3.shrink() {
            out.push((self.0.clone(), self.1.clone(), self.2.clone(), d));
        }
        out
    }
}

impl Shrink for String {
    fn shrink(&self) -> Vec<Self> {
        let mut out = Vec::new();
        if self.is_empty() {
            return out;
        }
        out.push(String::new());
        let char_count = self.chars().count();
        if char_count > 1 {
            let mid_char = char_count / 2;
            let split_byte = self
                .char_indices()
                .nth(mid_char)
                .map_or(self.len(), |(idx, _)| idx);
            out.push(self[..split_byte].to_owned());
            if split_byte < self.len() {
                out.push(self[split_byte..].to_owned());
            }
            let mut chars: Vec<char> = self.chars().collect();
            chars.pop();
            out.push(chars.into_iter().collect());
        }
        let chars: Vec<char> = self.chars().collect();
        for (idx, ch) in chars.iter().enumerate() {
            for simpler in ch.shrink() {
                let mut next = chars.clone();
                next[idx] = simpler;
                out.push(next.into_iter().collect());
            }
        }
        out.sort();
        out.dedup();
        out.retain(|s| s != self);
        out
    }
}

/// Deterministic property runner over [`Generate`] values.
#[derive(Debug, Clone)]
pub struct Prop {
    seed: u64,
    trials: usize,
    max_shrinks: usize,
    failure_path: Option<PathBuf>,
}

impl Prop {
    /// Start from `seed` with 100 trials, up to 1,000 shrink steps, and no persistence.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            trials: 100,
            max_shrinks: 1_000,
            failure_path: None,
        }
    }

    /// Set the maximum number of shrink steps attempted after a property failure.
    pub fn max_shrinks(mut self, max_shrinks: usize) -> Self {
        self.max_shrinks = max_shrinks;
        self
    }

    /// How many values to generate (at least 1).
    pub fn trials(mut self, trials: usize) -> Self {
        self.trials = trials.max(1);
        self
    }

    /// When a trial fails, write `seed=<n>` to this path.
    pub fn persist_failures(mut self, path: impl Into<PathBuf>) -> Self {
        self.failure_path = Some(path.into());
        self
    }

    /// Run `property` on `trials` generated `T` values.
    ///
    /// Returning `Err(message)` or `Ok(false)` fails the property. Seeds for
    /// trial `i` are `seed.wrapping_add(i as u64)`.
    pub fn for_all<T, F>(self, property: F) -> Result<(), PropError>
    where
        T: Generate,
        F: FnMut(&T) -> Result<bool, String>,
    {
        self.for_all_strategy(Strategy::<T>::from_generate(), property)
    }

    /// Like [`for_all`](Prop::for_all), but draws values from `strategy`.
    pub fn for_all_strategy<T, F>(
        self,
        strategy: Strategy<T>,
        mut property: F,
    ) -> Result<(), PropError>
    where
        T: 'static,
        F: FnMut(&T) -> Result<bool, String>,
    {
        for trial in 0..self.trials {
            let trial_seed = self.seed.wrapping_add(trial as u64);
            let mut fixture = FixtureContext::with_seed(trial_seed);
            let value = strategy.draw(&mut fixture)?;
            match property(&value) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(self.fail(trial_seed, trial, "property returned false".into()));
                }
                Err(message) => return Err(self.fail(trial_seed, trial, message)),
            }
        }
        Ok(())
    }

    /// Like [`for_all`](Prop::for_all), generating a pair `(A, B)` from one
    /// [`FixtureContext`] per trial (same seed, sequential draws).
    pub fn for_all2<A, B, F>(self, mut property: F) -> Result<(), PropError>
    where
        A: Generate,
        B: Generate,
        F: FnMut(&A, &B) -> Result<bool, String>,
    {
        for trial in 0..self.trials {
            let trial_seed = self.seed.wrapping_add(trial as u64);
            let mut fixture = FixtureContext::with_seed(trial_seed);
            let a = A::generate(&mut fixture)?;
            let b = B::generate(&mut fixture)?;
            match property(&a, &b) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(self.fail(trial_seed, trial, "property returned false".into()));
                }
                Err(message) => return Err(self.fail(trial_seed, trial, message)),
            }
        }
        Ok(())
    }

    /// Replay a single trial from an explicit seed (or a seed file via
    /// [`PropFailure::read_seed_file`]).
    pub fn replay<T, F>(seed: u64, mut property: F) -> Result<(), PropError>
    where
        T: Generate,
        F: FnMut(&T) -> Result<bool, String>,
    {
        let mut fixture = FixtureContext::with_seed(seed);
        let value = T::generate(&mut fixture)?;
        match property(&value) {
            Ok(true) => Ok(()),
            Ok(false) => Err(PropError::Failed(PropFailure {
                seed,
                trial: 0,
                message: "property returned false".into(),
            })),
            Err(message) => Err(PropError::Failed(PropFailure {
                seed,
                trial: 0,
                message,
            })),
        }
    }

    /// Minimal shrink loop from a known failing seed (not a full property framework).
    ///
    /// Regenerates `T` from `failure_seed`, asserts the property still fails,
    /// then repeatedly replaces the value with the first [`Shrink::shrink`]
    /// candidate that also fails. Returns the smallest failing value found.
    /// Run `property` on `trials` generated `T` values, automatically shrinking the first
    /// failing counterexample via [`Shrink`].
    ///
    /// Returns [`PropError::Failed`] with the failure seed, trial index, and the minimal
    /// shrunk counterexample formatted into the message.
    pub fn for_all_shrunk<T, F>(self, property: F) -> Result<(), PropError>
    where
        T: Generate + Shrink + fmt::Debug,
        F: FnMut(&T) -> Result<bool, String>,
    {
        self.for_all_shrunk_strategy(Strategy::<T>::from_generate(), property)
    }

    /// Like [`for_all_shrunk`](Prop::for_all_shrunk), drawing values from `strategy`.
    pub fn for_all_shrunk_strategy<T, F>(
        self,
        strategy: Strategy<T>,
        mut property: F,
    ) -> Result<(), PropError>
    where
        T: Shrink + fmt::Debug + 'static,
        F: FnMut(&T) -> Result<bool, String>,
    {
        for trial in 0..self.trials {
            let trial_seed = self.seed.wrapping_add(trial as u64);
            let mut fixture = FixtureContext::with_seed(trial_seed);
            let value = strategy.draw(&mut fixture)?;
            let initial_msg = match property(&value) {
                Ok(true) => continue,
                Ok(false) => "property returned false".to_owned(),
                Err(message) => message,
            };
            let (minimal, steps, final_msg) =
                shrink_counterexample(value, initial_msg, self.max_shrinks, &mut property);
            return Err(self.fail(
                trial_seed,
                trial,
                format!("{final_msg} (shrunk counterexample after {steps} shrinks: {minimal:?})"),
            ));
        }
        Ok(())
    }

    /// Minimal failing input starting from `failure_seed`.
    pub fn shrink_from<T, F>(failure_seed: u64, mut property: F) -> Result<T, PropError>
    where
        T: Generate + Shrink,
        F: FnMut(&T) -> Result<bool, String>,
    {
        let mut fixture = FixtureContext::with_seed(failure_seed);
        let current = T::generate(&mut fixture)?;
        if matches!(property(&current), Ok(true)) {
            return Err(PropError::Failed(PropFailure {
                seed: failure_seed,
                trial: 0,
                message: "shrink_from seed does not fail the property".into(),
            }));
        }
        let (minimal, _, _) = shrink_counterexample(current, String::new(), 10_000, &mut property);
        Ok(minimal)
    }

    fn fail(&self, seed: u64, trial: usize, message: String) -> PropError {
        let failure = PropFailure {
            seed,
            trial,
            message,
        };
        if let Some(path) = &self.failure_path {
            if let Err(e) = failure.write_seed_file(path) {
                return PropError::Io(e);
            }
        } else if let Ok(dir) = std::env::var("AIRBUG_PROP_FAILURE_DIR") {
            let path = PathBuf::from(dir).join(format!("prop-fail-{seed}.seed"));
            if let Err(e) = failure.write_seed_file(path) {
                return PropError::Io(e);
            }
        }
        PropError::Failed(failure)
    }
}

fn shrink_counterexample<T, F>(
    mut current: T,
    mut last_message: String,
    max_shrinks: usize,
    property: &mut F,
) -> (T, usize, String)
where
    T: Shrink,
    F: FnMut(&T) -> Result<bool, String>,
{
    let mut steps = 0usize;
    let mut checks = 0usize;
    let max_checks = max_shrinks.saturating_mul(64);
    while steps < max_shrinks && checks < max_checks {
        let mut progressed = false;
        for candidate in current.shrink() {
            if checks >= max_checks {
                break;
            }
            checks += 1;
            match property(&candidate) {
                Ok(true) => {}
                Ok(false) => {
                    current = candidate;
                    last_message = "property returned false".to_owned();
                    steps += 1;
                    progressed = true;
                    break;
                }
                Err(msg) => {
                    current = candidate;
                    last_message = msg;
                    steps += 1;
                    progressed = true;
                    break;
                }
            }
        }
        if !progressed {
            break;
        }
    }
    (current, steps, last_message)
}
