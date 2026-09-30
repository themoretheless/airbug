//! Worker-count selection for benchmark attributes and manual registration.
use std::borrow::Borrow;

/// Available host parallelism, capped at the supported 256 workers. If the OS
/// cannot report parallelism, use one worker. This never creates worker threads.
pub fn available() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(256)
}

#[doc(hidden)]
pub struct Scalar;
#[doc(hidden)]
pub struct Iterable;
/// Accepted worker count sources. The marker allows scalar and iterable inputs
/// without overlapping implementations; callers normally use `counts`.
pub trait ThreadCounts<Marker> {
    fn into_counts(self) -> Vec<usize>;
}
impl ThreadCounts<Scalar> for usize {
    fn into_counts(self) -> Vec<usize> {
        vec![self]
    }
}
impl ThreadCounts<Scalar> for bool {
    fn into_counts(self) -> Vec<usize> {
        vec![if self { 0 } else { 1 }]
    }
}
impl<I> ThreadCounts<Iterable> for I
where
    I: IntoIterator,
    I::Item: Borrow<usize>,
{
    fn into_counts(self) -> Vec<usize> {
        self.into_iter().map(|n| *n.borrow()).collect()
    }
}
/// Resolve zero to available parallelism and remove duplicate effective counts,
/// keeping the requested order. Explicit out-of-range counts are retained so
/// Suite validation can report them before executing a workload.
pub fn counts<T: ThreadCounts<M>, M>(value: T) -> Vec<usize> {
    resolve(value.into_counts(), available())
}
fn resolve(requested: Vec<usize>, automatic: usize) -> Vec<usize> {
    let mut seen = std::collections::BTreeSet::new();
    requested
        .into_iter()
        .map(|n| if n == 0 { automatic } else { n })
        .filter(|n| seen.insert(*n))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_iterable_auto_and_duplicate_selection() {
        assert_eq!(counts(2usize), [2]);
        assert_eq!(counts(false), [1]);
        assert_eq!(counts(true), [available()]);
        assert_eq!(counts(0usize), [available()]);
        assert_eq!(counts(2..=4), [2, 3, 4]);
        assert_eq!(counts(&[2usize, 3][..]), [2, 3]);
        assert_eq!(resolve(vec![0, 1, 4, 0, 2, 4], 4), [4, 1, 2]);
        assert_eq!(counts([257usize]), [257]);
        assert!((1..=256).contains(&available()));
    }
}
