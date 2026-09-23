//! Strategy combinators over [`crate::fixture::Generate`] draws.
use crate::fixture::{FixtureContext, Generate, GenerationError};

/// Default filter redraw budget when [`Strategy::filter`] rejects a value.
const FILTER_RETRIES: usize = 64;

/// A fallible value source for property tests and fixtures.
pub struct Strategy<T> {
    draw_fn: Draw<T>,
}

type Draw<T> = Box<dyn Fn(&mut FixtureContext) -> Result<T, GenerationError>>;

impl<T: 'static> Strategy<T> {
    /// Draw via [`T::generate`](Generate::generate).
    pub fn from_generate() -> Self
    where
        T: Generate,
    {
        Self {
            draw_fn: Box::new(|ctx| T::generate(ctx)),
        }
    }

    /// Build from an arbitrary draw function.
    pub fn from_fn(
        f: impl Fn(&mut FixtureContext) -> Result<T, GenerationError> + 'static,
    ) -> Self {
        Self {
            draw_fn: Box::new(f),
        }
    }

    /// Transform each drawn value.
    pub fn map<U: 'static>(self, f: impl Fn(T) -> U + 'static) -> Strategy<U> {
        Strategy {
            draw_fn: Box::new(move |ctx| (self.draw_fn)(ctx).map(&f)),
        }
    }

    /// Redraw until `pred` holds, up to a fixed retry budget.
    pub fn filter(self, pred: impl Fn(&T) -> bool + 'static) -> Strategy<T> {
        Strategy {
            draw_fn: Box::new(move |ctx| {
                for _ in 0..FILTER_RETRIES {
                    let value = (self.draw_fn)(ctx)?;
                    if pred(&value) {
                        return Ok(value);
                    }
                }
                Err(GenerationError::custom(format!(
                    "strategy filter rejected {FILTER_RETRIES} draws"
                )))
            }),
        }
    }

    /// Draw one value from this strategy.
    pub fn draw(&self, ctx: &mut FixtureContext) -> Result<T, GenerationError> {
        (self.draw_fn)(ctx)
    }
}
