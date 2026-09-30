//! Small reusable helpers without an async runtime or platform dependencies.
use crate::{Case, Config, Result, error};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
    time::Duration,
};

impl Config {
    /// Profiles set sampling effort, never guarantee statistical precision.
    pub fn profile(name: &str) -> Result<Self> {
        let (samples, warmup, sample) = match name {
            "quick" => (8, 10, 1),
            "normal" => (30, 50, 5),
            "thorough" => (100, 200, 10),
            _ => return Err(error("profile must be quick, normal or thorough")),
        };
        Ok(Self {
            samples,
            warmup: Duration::from_millis(warmup),
            sample_time: Duration::from_millis(sample),
            ..Self::default()
        })
    }
}
/// Order applies equally to listing, dry-run and workload execution.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Registration,
    Lexical,
    Natural,
    Source,
    /// At every group level, cases precede subgroups; names break ties.
    Kind,
}
impl SortOrder {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "registration" => Ok(Self::Registration),
            "lexical" => Ok(Self::Lexical),
            "natural" => Ok(Self::Natural),
            "source" => Ok(Self::Source),
            "kind" => Ok(Self::Kind),
            _ => Err(error(
                "sort must be registration, lexical, natural, source or kind",
            )),
        }
    }
    pub(crate) fn compare(self, a: &str, b: &str) -> std::cmp::Ordering {
        match self {
            Self::Registration | Self::Source | Self::Kind => std::cmp::Ordering::Equal,
            Self::Lexical => a.cmp(b),
            Self::Natural => crate::ordering::natural_path_cmp(a, b),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Selection {
    pub pattern: String,
    pub sort: SortOrder,
    pub reverse: bool,
    pub exact: bool,
    pub glob: bool,
    /// A compiled expression, matched anywhere in the full case path. Use anchors for exact paths.
    pub regex: Option<regex_automata::meta::Regex>,
    pub exclude_exact: Vec<String>,
    pub exclude_regex: Vec<regex_automata::meta::Regex>,
    pub exclude: Vec<String>,
    pub tags: Vec<String>,
    pub include_ignored: bool,
    pub only_ignored: bool,
}
impl Selection {
    /// Compile once before listing or measuring. Invalid expressions return an error.
    pub fn with_regex(mut self, pattern: &str) -> Result<Self> {
        self.regex = Some(
            regex_automata::meta::Regex::new(pattern)
                .map_err(|e| error(format!("invalid benchmark regex: {e}")))?,
        );
        self.validate()?;
        Ok(self)
    }
    pub fn skip_regex(&mut self, pattern: &str) -> Result<&mut Self> {
        self.exclude_regex.push(
            regex_automata::meta::Regex::new(pattern)
                .map_err(|e| error(format!("invalid exclusion regex: {e}")))?,
        );
        Ok(self)
    }

    pub fn validate(&self) -> Result<()> {
        if usize::from(self.exact) + usize::from(self.glob) + usize::from(self.regex.is_some()) > 1
        {
            return Err(error("exact, glob and regex are mutually exclusive"));
        }
        Ok(())
    }
    pub fn matches(&self, c: &Case) -> bool {
        let matches = if let Some(regex) = &self.regex {
            regex.is_match(&c.id)
        } else if self.exact {
            c.id == self.pattern
        } else if self.glob {
            glob(&self.pattern, &c.id)
        } else {
            c.id.contains(&self.pattern)
        };
        let ignored = c
            .contract
            .get("ignored")
            .is_some_and(|value| value == "true");
        matches
            && if self.only_ignored {
                ignored
            } else {
                self.include_ignored || !ignored
            }
            && !self.exclude_exact.contains(&c.id)
            && !self.exclude_regex.iter().any(|r| r.is_match(&c.id))
            && !self.exclude.iter().any(|p| glob(p, &c.id))
            && self
                .tags
                .iter()
                .all(|t| c.contract.contains_key(&format!("tag.{t}")))
    }
}
/// Full-string '*'/'?' glob with Unicode scalar matching. No filesystem expansion.
pub fn glob(pattern: &str, text: &str) -> bool {
    let text: Vec<_> = text.chars().collect();
    let mut row = vec![false; text.len() + 1];
    row[0] = true;
    for p in pattern.chars() {
        let mut next = vec![false; row.len()];
        if p == '*' {
            next[0] = row[0];
        }
        for i in 1..row.len() {
            next[i] = if p == '*' {
                row[i] || next[i - 1]
            } else {
                row[i - 1] && (p == '?' || p == text[i - 1])
            };
        }
        row = next;
    }
    row[text.len()]
}
/// A lazy process-local fixture, shareable across cases; initialization and borrowing precede timing.
type Initializer<T> = Rc<RefCell<Option<Box<dyn FnOnce() -> T>>>>;
pub struct Fixture<T> {
    pub(crate) value: Rc<OnceCell<RefCell<T>>>,
    pub(crate) setup: Initializer<T>,
}
impl<T> Clone for Fixture<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            setup: self.setup.clone(),
        }
    }
}
impl<T> Fixture<T> {
    pub fn new(setup: impl FnOnce() -> T + 'static) -> Self {
        Self {
            value: Rc::new(OnceCell::new()),
            setup: Rc::new(RefCell::new(Some(Box::new(setup)))),
        }
    }
    pub(crate) fn get(&self) -> &RefCell<T> {
        self.value.get_or_init(|| {
            RefCell::new(self.setup.borrow_mut().take().expect("fixture initializer")())
        })
    }
}
/// Reproducible SplitMix64 generator. Not cryptographic; seed=0 is valid.
pub struct Seeded {
    state: u64,
}
impl Seeded {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    pub fn bytes(&mut self, len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len);
        while out.len() < len {
            let bytes = self.next_u64().to_le_bytes();
            out.extend_from_slice(&bytes[..(len - out.len()).min(8)]);
        }
        out
    }
}
