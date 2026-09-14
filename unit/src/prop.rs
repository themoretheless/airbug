//! Seeded property checks with optional counterexample persistence (no shrinking yet).
use crate::fixture::{FixtureContext, Generate, GenerationError};
use std::{
    fmt,
    fs,
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

/// Deterministic property runner over [`Generate`] values.
#[derive(Debug, Clone)]
pub struct Prop {
    seed: u64,
    trials: usize,
    failure_path: Option<PathBuf>,
}

impl Prop {
    /// Start from `seed` with 100 trials and no persistence.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            trials: 100,
            failure_path: None,
        }
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
    pub fn for_all<T, F>(self, mut property: F) -> Result<(), PropError>
    where
        T: Generate,
        F: FnMut(&T) -> Result<bool, String>,
    {
        for trial in 0..self.trials {
            let trial_seed = self.seed.wrapping_add(trial as u64);
            let mut fixture = FixtureContext::with_seed(trial_seed);
            let value = T::generate(&mut fixture)?;
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
        }
        PropError::Failed(failure)
    }
}
