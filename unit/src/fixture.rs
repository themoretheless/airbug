//! Deterministic object generation with explicit, fallible graph edges.
use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    fmt,
    rc::Rc,
};

type Factory = Rc<dyn Fn(&mut FixtureContext) -> Result<Box<dyn Any>, GenerationError>>;
/// Implement with `ctx.try_build()?` for each child. Use `build` only at test boundaries.
pub trait Generate: Sized + 'static {
    /// Build one value, asking `ctx` for each child with
    /// [`try_build`](FixtureContext::try_build) so limits and cycle detection
    /// apply to the whole graph.
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError>;
}
/// Why generation stopped. The graph limits exist so a mistake in a factory
/// fails the test instead of hanging or exhausting memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationErrorKind {
    /// A type asked to build itself, directly or through its children.
    Cycle,
    /// Nesting passed [`FixtureContext::max_depth`].
    DepthLimit,
    /// No factory was registered for a type built by registration.
    MissingFactory,
    /// The graph produced more values than [`FixtureContext::max_nodes`].
    NodeLimit,
    /// A factory rejected the request itself, via [`GenerationError::custom`].
    Custom(String),
}
/// A failed generation, with enough context to reproduce it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationError {
    /// Type names from the root down to where it failed.
    pub path: Vec<&'static str>,
    /// What went wrong.
    pub kind: GenerationErrorKind,
    /// Seed of the context, so the failure can be replayed exactly.
    pub seed: Option<u64>,
}
impl GenerationError {
    /// An error raised by a factory itself. The path and seed are filled in as
    /// it propagates back up.
    pub fn custom(message: impl Into<String>) -> Self {
        Self {
            path: Vec::new(),
            seed: None,
            kind: GenerationErrorKind::Custom(message.into()),
        }
    }
}
impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {} (seed={:?})",
            self.kind,
            self.path.join(" -> "),
            self.seed
        )
    }
}
impl std::error::Error for GenerationError {}
/// Per-test context; deliberately not Send/Sync. Factories may capture Rc values.
pub struct FixtureContext {
    factories: HashMap<TypeId, Factory>,
    stack: Vec<(TypeId, &'static str)>,
    state: u64,
    collection_len: usize,
    max_depth: usize,
    max_nodes: usize,
    nodes: usize,
    seed: u64,
}
impl Default for FixtureContext {
    fn default() -> Self {
        Self::with_seed(1)
    }
}
impl FixtureContext {
    /// A context seeded with 1, so an unseeded test is still reproducible.
    pub fn new() -> Self {
        Self::default()
    }
    /// A context whose random choices are determined by `seed`. The same seed
    /// always produces the same values.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            factories: HashMap::new(),
            stack: Vec::new(),
            state: seed,
            collection_len: 3,
            max_depth: 64,
            max_nodes: 100_000,
            nodes: 0,
            seed,
        }
    }
    /// How many elements generated collections get. Defaults to 3.
    pub fn collection_len(&mut self, len: usize) -> &mut Self {
        self.collection_len = len;
        self
    }
    /// Deepest nesting allowed before [`GenerationErrorKind::DepthLimit`].
    /// Defaults to 64.
    pub fn max_depth(&mut self, depth: usize) -> &mut Self {
        self.max_depth = depth;
        self
    }
    /// Supply values of `T` from a factory instead of its [`Generate`] impl.
    pub fn register<T: 'static>(
        &mut self,
        factory: impl Fn(&mut Self) -> T + 'static,
    ) -> &mut Self {
        self.register_fallible(move |ctx| Ok(factory(ctx)))
    }
    /// [`register`](FixtureContext::register) for a factory that can refuse.
    pub fn register_fallible<T: 'static>(
        &mut self,
        factory: impl Fn(&mut Self) -> Result<T, GenerationError> + 'static,
    ) -> &mut Self {
        self.factories.insert(
            TypeId::of::<T>(),
            Rc::new(move |ctx| factory(ctx).map(|v| Box::new(v) as Box<dyn Any>)),
        );
        self
    }
    /// Drop the factory for `T`, returning whether there was one.
    pub fn remove<T: 'static>(&mut self) -> bool {
        self.factories.remove(&TypeId::of::<T>()).is_some()
    }
    /// Clone a fixed value. Rc/Arc values preserve shared identity.
    pub fn reuse<T: Clone + 'static>(&mut self, value: T) -> &mut Self {
        self.register(move |_| value.clone())
    }
    /// [`try_build`](FixtureContext::try_build), panicking on failure. Use it
    /// at the top of a test, not inside a [`Generate`] impl.
    #[track_caller]
    pub fn build<T: Generate>(&mut self) -> T {
        self.try_build().unwrap_or_else(|e| panic!("{e}"))
    }
    /// Build a `T`, counting it against the depth, node and cycle limits.
    /// This is what a [`Generate`] impl should call for its children.
    pub fn try_build<T: Generate>(&mut self) -> Result<T, GenerationError> {
        self.run(true, T::generate)
    }
    /// [`try_build_registered`](FixtureContext::try_build_registered),
    /// panicking on failure.
    #[track_caller]
    pub fn build_registered<T: 'static>(&mut self) -> T {
        self.try_build_registered()
            .unwrap_or_else(|e| panic!("{e}"))
    }
    /// Build a `T` from its registered factory, failing with
    /// [`GenerationErrorKind::MissingFactory`] if there is none. Unlike
    /// [`try_build`](FixtureContext::try_build) this needs no [`Generate`] impl.
    pub fn try_build_registered<T: 'static>(&mut self) -> Result<T, GenerationError> {
        self.run(true, |_| {
            Err(GenerationError {
                path: Vec::new(),
                seed: None,
                kind: GenerationErrorKind::MissingFactory,
            })
        })
    }
    fn run<T: 'static>(
        &mut self,
        use_registry: bool,
        fallback: impl FnOnce(&mut Self) -> Result<T, GenerationError>,
    ) -> Result<T, GenerationError> {
        if self.stack.is_empty() {
            self.nodes = 0;
        }
        let id = TypeId::of::<T>();
        let name = type_name::<T>();
        let kind = if self.stack.iter().any(|(current, _)| *current == id) {
            Some(GenerationErrorKind::Cycle)
        } else if self.stack.len() >= self.max_depth {
            Some(GenerationErrorKind::DepthLimit)
        } else if self.nodes >= self.max_nodes {
            Some(GenerationErrorKind::NodeLimit)
        } else {
            None
        };
        if let Some(kind) = kind {
            let mut path: Vec<_> = self.stack.iter().map(|(_, name)| *name).collect();
            path.push(name);
            return Err(GenerationError {
                path,
                kind,
                seed: Some(self.seed),
            });
        }
        self.nodes += 1;
        self.stack.push((id, name));
        let factory = if use_registry {
            self.factories.get(&id).cloned()
        } else {
            None
        };
        // Catch only to restore traversal state; user panics are never converted into errors.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let result = if let Some(factory) = factory {
                factory(self).map(|value| {
                    *value
                        .downcast::<T>()
                        .expect("internal factory type mismatch")
                })
            } else {
                fallback(self)
            };
            result.map_err(|mut error| {
                if error.seed.is_none() {
                    error.seed = Some(self.seed);
                }
                if error.path.is_empty() {
                    error.path = self.stack.iter().map(|(_, name)| *name).collect();
                }
                error
            })
        }));
        self.stack.pop();
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
macro_rules! integers { ($($ty:ty),*) => { $(impl Generate for $ty {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> { Ok(ctx.next() as Self) }
})* }; }
integers!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
impl Generate for u128 {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok((u128::from(ctx.next()) << 64) | u128::from(ctx.next()))
    }
}
impl Generate for i128 {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok(u128::generate(ctx)? as i128)
    }
}
impl Generate for bool {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok(ctx.next() & 1 == 1)
    }
}
impl Generate for String {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok(format!("fixture-{:016x}", ctx.next()))
    }
}
impl Generate for f64 {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok((ctx.next() >> 11) as f64 / ((1u64 << 53) as f64))
    }
}
impl Generate for f32 {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok((ctx.next() >> 40) as f32 / ((1u32 << 24) as f32))
    }
}
impl<T: Generate> Generate for Vec<T> {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        (0..ctx.collection_len).map(|_| ctx.try_build()).collect()
    }
}
impl<T: Generate> Generate for Option<T> {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        ctx.try_build().map(Some)
    }
}
impl<T: Generate> Generate for Box<T> {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        ctx.try_build().map(Box::new)
    }
}
impl<T: Generate> Generate for Rc<T> {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        ctx.try_build().map(Rc::new)
    }
}
impl<T: Generate> Generate for std::sync::Arc<T> {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        ctx.try_build().map(std::sync::Arc::new)
    }
}

/// Implemented by derive(Generate) for structs. Builders override individual fields.
pub trait FixtureBuild: Sized + 'static {
    /// Generated builder type, borrowing the context it will build from.
    type Builder<'ctx>;
    /// Start a builder whose fields default to generated values.
    fn fixture_builder(ctx: &mut FixtureContext) -> Self::Builder<'_>;
}
impl FixtureContext {
    /// A builder for `T`, to set some fields explicitly and generate the rest.
    pub fn builder<T: FixtureBuild>(&mut self) -> T::Builder<'_> {
        T::fixture_builder(self)
    }
    /// Generate with a one-call root factory, bypassing that root's registration.
    /// Nested types still use their registered rules; graph limits still apply.
    pub fn try_build_with<T: 'static>(
        &mut self,
        factory: impl FnOnce(&mut Self) -> Result<T, GenerationError>,
    ) -> Result<T, GenerationError> {
        self.run(false, factory)
    }
}

/// Replay point for built-in random state and configuration. Captured factory
/// state remains shared through Rc; external side effects cannot be rewound.
#[derive(Clone)]
pub struct FixtureCheckpoint {
    factories: HashMap<TypeId, Factory>,
    state: u64,
    seed: u64,
    collection_len: usize,
    max_depth: usize,
    max_nodes: usize,
}
impl FixtureContext {
    /// The seed this context was created with, worth printing when a
    /// generated test fails.
    pub fn seed(&self) -> u64 {
        self.seed
    }
    /// Most values one build may produce before
    /// [`GenerationErrorKind::NodeLimit`]. Defaults to 100_000.
    pub fn max_nodes(&mut self, limit: usize) -> &mut Self {
        self.max_nodes = limit;
        self
    }
    /// Capture factories, configuration and random state, to replay the same
    /// values later. Panics unless the fixture is idle.
    pub fn checkpoint(&self) -> FixtureCheckpoint {
        assert!(self.stack.is_empty(), "checkpoint requires an idle fixture");
        FixtureCheckpoint {
            factories: self.factories.clone(),
            state: self.state,
            seed: self.seed,
            collection_len: self.collection_len,
            max_depth: self.max_depth,
            max_nodes: self.max_nodes,
        }
    }
    /// Rewind to a [`checkpoint`](FixtureContext::checkpoint). Panics unless
    /// the fixture is idle.
    pub fn restore(&mut self, checkpoint: FixtureCheckpoint) {
        assert!(self.stack.is_empty(), "restore requires an idle fixture");
        self.factories = checkpoint.factories;
        self.state = checkpoint.state;
        self.seed = checkpoint.seed;
        self.collection_len = checkpoint.collection_len;
        self.max_depth = checkpoint.max_depth;
        self.max_nodes = checkpoint.max_nodes;
        self.nodes = 0;
    }
    /// Inherit rules/configuration with a fresh random sequence. Factory captures are shared.
    pub fn child(&self, seed: u64) -> Self {
        let mut child = Self::with_seed(seed);
        child.factories = self.factories.clone();
        child.collection_len = self.collection_len;
        child.max_depth = self.max_depth;
        child.max_nodes = self.max_nodes;
        child
    }
    /// Restore registrations and limits on normal return or panic; keep RNG progress.
    pub fn scoped<R>(&mut self, body: impl FnOnce(&mut Self) -> R) -> R {
        let factories = self.factories.clone();
        let limits = (self.collection_len, self.max_depth, self.max_nodes);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(self)));
        self.factories = factories;
        (self.collection_len, self.max_depth, self.max_nodes) = limits;
        match result {
            Ok(value) => value,
            Err(error) => std::panic::resume_unwind(error),
        }
    }
    /// Uniform choice using rejection sampling. Empty input is an error.
    pub fn choose<'a, T>(&mut self, values: &'a [T]) -> Result<&'a T, GenerationError> {
        if values.is_empty() {
            let mut error = GenerationError::custom("cannot choose from an empty slice");
            error.seed = Some(self.seed);
            return Err(error);
        }
        let bound = values.len() as u64;
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let n = self.next();
            if n >= threshold {
                return Ok(&values[(n % bound) as usize]);
            }
        }
    }
    /// Weighted choice via cumulative weights. Empty slices, length mismatch, or
    /// all-zero weights are errors.
    pub fn choose_weighted<'a, T>(
        &mut self,
        values: &'a [T],
        weights: &[u32],
    ) -> Result<&'a T, GenerationError> {
        if values.is_empty() {
            let mut error = GenerationError::custom("cannot choose_weighted from an empty slice");
            error.seed = Some(self.seed);
            return Err(error);
        }
        if values.len() != weights.len() {
            let mut error = GenerationError::custom("choose_weighted values/weights length mismatch");
            error.seed = Some(self.seed);
            return Err(error);
        }
        let total: u64 = weights.iter().map(|&w| u64::from(w)).sum();
        if total == 0 {
            let mut error = GenerationError::custom("choose_weighted requires a nonzero weight");
            error.seed = Some(self.seed);
            return Err(error);
        }
        let threshold = total.wrapping_neg() % total;
        let pick = loop {
            let n = self.next();
            if n >= threshold {
                break n % total;
            }
        };
        let mut cumulative = 0u64;
        for (index, &weight) in weights.iter().enumerate() {
            cumulative += u64::from(weight);
            if pick < cumulative {
                return Ok(&values[index]);
            }
        }
        Ok(&values[values.len() - 1])
    }
    /// Uniform `u64` in the inclusive range `[min, max]`.
    pub fn draw_u64(&mut self, min: u64, max: u64) -> Result<u64, GenerationError> {
        if min > max {
            let mut error = GenerationError::custom("draw_u64 requires min <= max");
            error.seed = Some(self.seed);
            return Err(error);
        }
        let span = max.wrapping_sub(min).wrapping_add(1);
        if span == 0 {
            return Ok(self.next());
        }
        let threshold = span.wrapping_neg() % span;
        loop {
            let n = self.next();
            if n >= threshold {
                return Ok(min + n % span);
            }
        }
    }
    /// Uniform `f64` in `[0, 1)`, matching [`Generate`] for `f64`.
    pub fn draw_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / ((1u64 << 53) as f64)
    }
    /// Registers checked sequential u64 values. The last u64 value is emitted once.
    pub fn sequence_u64(&mut self, start: u64) -> &mut Self {
        let next = std::cell::Cell::new(Some(start));
        self.register_fallible(move |_| {
            let value = next
                .get()
                .ok_or_else(|| GenerationError::custom("sequence exhausted"))?;
            next.set(value.checked_add(1));
            Ok(value)
        })
    }
    /// Generate an ordered pair within an inclusive unsigned range.
    pub fn ordered_pair(&mut self, min: u64, max: u64) -> Result<(u64, u64), GenerationError> {
        if min > max {
            let mut error = GenerationError::custom("ordered_pair requires min <= max");
            error.seed = Some(self.seed);
            return Err(error);
        }
        let span = max.wrapping_sub(min).wrapping_add(1);
        let mut draw = || {
            if span == 0 {
                return self.next();
            }
            let threshold = span.wrapping_neg() % span;
            loop {
                let n = self.next();
                if n >= threshold {
                    return min + n % span;
                }
            }
        };
        let a = draw();
        let b = draw();
        Ok((a.min(b), a.max(b)))
    }
    /// Lazy fallible generation, one object per next(). No collection allocation.
    pub fn stream<T: Generate>(&mut self) -> impl Iterator<Item = Result<T, GenerationError>> + '_ {
        std::iter::from_fn(|| Some(self.try_build()))
    }
}
/// Deterministic edge sets for table-driven tests, not random distributions.
pub mod boundaries {
    /// Extremes and their neighbours, plus the signs and zero.
    pub const I64: [i64; 7] = [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX];
    /// Zero, one, and the top of the range with its neighbour.
    pub const U64: [u64; 4] = [0, 1, u64::MAX - 1, u64::MAX];
    /// Infinities, both zeros, the smallest positive value and NaN, which are
    /// the cases naive float code tends to get wrong.
    pub const F64: [f64; 10] = [
        f64::NEG_INFINITY,
        f64::MIN,
        -1.0,
        -0.0,
        0.0,
        f64::MIN_POSITIVE,
        1.0,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
    ];
}
