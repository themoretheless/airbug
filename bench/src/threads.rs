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

/// Parse a comma-separated runtime worker list. Zero selects host parallelism;
/// duplicate effective values are removed while retaining requested order.
pub fn parse_list(value: &str) -> crate::Result<Vec<usize>> {
    let requested: Vec<usize> = value
        .split(',')
        .map(|item| {
            let count = item.trim().parse::<usize>().map_err(|_| {
                crate::error("--threads requires a comma-separated list of integers")
            })?;
            if count > 256 {
                return Err(crate::error(
                    "workers must be 1..256 (or 0 for available parallelism)",
                ));
            }
            Ok(count)
        })
        .collect::<crate::Result<_>>()?;
    Ok(resolve(requested, available()))
}

pub(crate) fn from_args(args: &[String]) -> crate::Result<Option<Vec<usize>>> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return Ok(None);
    }
    let mut requested = Vec::new();
    let mut found = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--threads" {
            found = true;
            requested
                .extend(parse_list(args.next().ok_or_else(|| {
                    crate::error("--threads requires a worker list")
                })?)?);
        }
    }
    Ok(found.then(|| resolve(requested, available())))
}

pub(crate) fn validate_registered_request(
    requested: &[usize],
    registered: Option<&[usize]>,
) -> crate::Result<()> {
    match registered {
        Some(actual) if actual != requested => Err(crate::error(
            "CLI thread matrix differs from the registered matrix; use main_registered to apply CLI settings before registration",
        )),
        None if requested.len() > 1 => Err(crate::error(
            "thread lists require main_registered so the matrix is configured before registration",
        )),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::*;
    #[test]
    fn runtime_matrix_never_silently_ignores_requested_counts() {
        assert!(validate_registered_request(&[1, 2], None).is_err());
        assert!(validate_registered_request(&[1, 2], Some(&[2, 1])).is_err());
        assert!(validate_registered_request(&[1], Some(&[1, 2])).is_err());
        assert!(validate_registered_request(&[1, 2], Some(&[1, 2])).is_ok());
        assert!(validate_registered_request(&[4], None).is_ok());
    }
    #[test]
    fn runtime_lists_preserve_order_deduplicate_and_validate_every_count() {
        assert_eq!(parse_list(" 3,1,3,2 ").unwrap(), [3, 1, 2]);
        assert_eq!(parse_list("0,0").unwrap(), [available()]);
        for invalid in ["", "1,", ",1", "-1", "257", "1,257", "one"] {
            assert!(parse_list(invalid).is_err(), "{invalid}");
        }
        let args = ["--threads", "3,1", "--threads", "2,3"].map(String::from);
        assert_eq!(from_args(&args).unwrap(), Some(vec![3, 1, 2]));
        assert!(from_args(&["--threads".into()]).is_err());
        assert_eq!(
            from_args(&["--help".into(), "--threads".into()]).unwrap(),
            None
        );
    }
}
