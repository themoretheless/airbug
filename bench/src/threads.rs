//! Worker-count selection for benchmark attributes and manual registration.
use std::borrow::Borrow;

/// Available host parallelism. If the OS
/// cannot report parallelism, use one worker. This never creates worker threads.
pub fn available() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
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
        assert!(available() > 0);
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
            Ok(count)
        })
        .collect::<crate::Result<_>>()?;
    Ok(resolve(requested, available()))
}

pub(crate) fn from_args(args: &[String]) -> crate::Result<Option<Vec<usize>>> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h" | "--help-all"))
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
        assert_eq!(
            parse_list(&format!("257,{}", usize::MAX)).unwrap(),
            [257, usize::MAX]
        );
        for invalid in ["", "1,", ",1", "-1", "one"] {
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

/// Macro dispatch for optionally threaded callbacks. Autoref selection keeps local
/// callbacks legal when they capture values which cannot be shared by workers.
/// Dispatch uses the argument type as well as the callback: closure auto-traits
/// alone are resolved too late for autoref fallback during method selection.
#[doc(hidden)]
pub struct InheritedCallback<T, F>(std::cell::RefCell<Option<F>>, std::marker::PhantomData<T>);
#[doc(hidden)]
pub fn argument_marker<T>(_: &T) -> std::marker::PhantomData<T> {
    std::marker::PhantomData
}
impl<T, F> InheritedCallback<T, F> {
    pub fn new(marker: std::marker::PhantomData<T>, callback: F) -> Self {
        Self(std::cell::RefCell::new(Some(callback)), marker)
    }
}
#[doc(hidden)]
pub trait RegisterInheritedCallback<'a, O> {
    fn register_inherited(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        workers: usize,
        drop: crate::DropPolicy,
    );
}
impl<'a, T: Sync, F: Fn() -> O + Sync + 'a, O: 'a> RegisterInheritedCallback<'a, O>
    for &&InheritedCallback<T, F>
{
    fn register_inherited(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        let callback = self
            .0
            .borrow_mut()
            .take()
            .expect("callback registered once");
        suite.bench_threads_with_local_input(
            name,
            workers,
            || (),
            move |_: &mut ()| callback(),
            drop,
        );
    }
}
impl<'a, T, F: Fn() -> O + 'a, O: 'a> RegisterInheritedCallback<'a, O>
    for &InheritedCallback<T, F>
{
    fn register_inherited(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        if workers == 1 {
            let callback = self
                .0
                .borrow_mut()
                .take()
                .expect("callback registered once");
            suite.bench_batched_ref(
                name,
                || (),
                move |_| callback(),
                drop,
                suite.registration_batch_policy(),
            );
            suite.local_thread_contract();
            return;
        }
        suite.unavailable_thread_case(name, "cannot share its arguments between workers; use thread-safe arguments or threads = false");
    }
}

/// Allocated equivalent of callback dispatch; local captures remain valid for
/// one caller-thread worker and fail before execution for larger counts.
#[doc(hidden)]
pub trait RegisterInheritedAllocated<'a, A: std::alloc::GlobalAlloc + 'a, O> {
    fn register_inherited_allocated(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        drop: crate::DropPolicy,
    );
}
impl<'a, T: Sync, A: std::alloc::GlobalAlloc + Sync + 'a, F: Fn() -> O + Sync + 'a, O: 'a>
    RegisterInheritedAllocated<'a, A, O> for &&InheritedCallback<T, F>
{
    fn register_inherited_allocated(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        let callback = self
            .0
            .borrow_mut()
            .take()
            .expect("callback registered once");
        suite.bench_threads_allocated_with_local_input(
            name,
            allocator,
            workers,
            || (),
            move |_| callback(),
            drop,
        );
    }
}
impl<'a, T, A: std::alloc::GlobalAlloc + 'a, F: Fn() -> O + 'a, O: 'a>
    RegisterInheritedAllocated<'a, A, O> for &InheritedCallback<T, F>
{
    fn register_inherited_allocated(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        if workers == 1 {
            let callback = self
                .0
                .borrow_mut()
                .take()
                .expect("callback registered once");
            suite.bench_allocated_batched_ref(
                name,
                allocator,
                || (),
                move |_| callback(),
                drop,
                suite.registration_batch_policy(),
            );
            suite.local_thread_contract();
        } else {
            suite.unavailable_thread_case(name, "cannot share its arguments between workers; use thread-safe arguments or threads = false");
        }
    }
}

#[doc(hidden)]
pub trait RegisterInheritedAsync<'a, E, O> {
    fn register_inherited_async(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        workers: usize,
        drop: crate::DropPolicy,
    );
}
impl<'a, T: Sync, X, F, E, O> RegisterInheritedAsync<'a, E, O> for &&InheritedCallback<T, (X, F)>
where
    X: Fn() -> E + Sync + 'a,
    F: AsyncFn() -> O + Sync + 'a,
    E: crate::workloads::Executor + 'a,
    O: 'a,
{
    fn register_inherited_async(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        let (executor, callback) = self
            .0
            .borrow_mut()
            .take()
            .expect("callback registered once");
        suite.bench_async_threads(name, workers, executor, callback, drop);
    }
}
impl<'a, T, X, F, E, O> RegisterInheritedAsync<'a, E, O> for &InheritedCallback<T, (X, F)>
where
    X: Fn() -> E + 'a,
    F: AsyncFn() -> O + 'a,
    E: crate::workloads::Executor + 'a,
    O: 'a,
{
    fn register_inherited_async(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        if workers == 1 {
            let (executor, callback) = self
                .0
                .borrow_mut()
                .take()
                .expect("callback registered once");
            suite.bench_async_batched_ref(
                name,
                executor,
                || (),
                async move |_| callback().await,
                drop,
                suite.registration_batch_policy(),
            );
            suite.local_thread_contract();
            return;
        }
        suite.unavailable_thread_case(name, "cannot share its arguments or executor factory between workers; use thread-safe captures or threads = false");
    }
}

#[doc(hidden)]
pub trait RegisterInheritedAsyncAllocated<'a, A: std::alloc::GlobalAlloc + 'a, E, O> {
    fn register_inherited_async_allocated(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        drop: crate::DropPolicy,
    );
}
impl<'a, T: Sync, A: std::alloc::GlobalAlloc + Sync + 'a, X, F, E, O>
    RegisterInheritedAsyncAllocated<'a, A, E, O> for &&InheritedCallback<T, (X, F)>
where
    X: Fn() -> E + Sync + 'a,
    F: AsyncFn() -> O + Sync + 'a,
    E: crate::workloads::Executor + 'a,
    O: 'a,
{
    fn register_inherited_async_allocated(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        let (executor, callback) = self
            .0
            .borrow_mut()
            .take()
            .expect("callback registered once");
        suite.bench_async_threads_allocated_with_input(
            name,
            allocator,
            workers,
            executor,
            || (),
            async move |_| callback().await,
            drop,
        );
    }
}
impl<'a, T, A: std::alloc::GlobalAlloc + 'a, X, F, E, O>
    RegisterInheritedAsyncAllocated<'a, A, E, O> for &InheritedCallback<T, (X, F)>
where
    X: Fn() -> E + 'a,
    F: AsyncFn() -> O + 'a,
    E: crate::workloads::Executor + 'a,
    O: 'a,
{
    fn register_inherited_async_allocated(
        self,
        suite: &mut crate::Suite<'a>,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        drop: crate::DropPolicy,
    ) {
        if workers == 1 {
            let (executor, callback) = self
                .0
                .borrow_mut()
                .take()
                .expect("callback registered once");
            suite.bench_async_allocated_batched_ref(
                name,
                allocator,
                executor,
                || (),
                async move |_| callback().await,
                drop,
                suite.registration_batch_policy(),
            );
            suite.local_thread_contract();
        } else {
            suite.unavailable_thread_case(name, "cannot share its arguments or executor factory between workers; use thread-safe captures or threads = false");
        }
    }
}

/// Coordinator-prepared inputs must cross the worker boundary. Carry their
/// concrete types in the receiver so autoref can reject only concurrent use.
#[doc(hidden)]
pub struct InheritedSetup<I, O, S, F> {
    callbacks: std::cell::RefCell<Option<(S, F)>>,
    values: std::marker::PhantomData<fn() -> (I, O)>,
}
macro_rules! inherited_setup_dispatch {
    ($constructor:ident, $trait:ident, $method:ident, $input:ty, $runner:ident, $local:ident) => {
        #[doc(hidden)]
        pub fn $constructor<I, O, S, F>(setup: S, work: F) -> InheritedSetup<I, O, S, F>
        where S: FnMut() -> I, F: Fn($input) -> O {
            InheritedSetup { callbacks: std::cell::RefCell::new(Some((setup, work))), values: std::marker::PhantomData }
        }
        #[doc(hidden)]
        pub trait $trait<'a> {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str, workers: usize, drop: crate::DropPolicy);
        }
        impl<'a, I: Send + 'a, O: Send + 'a, S, F> $trait<'a> for &&InheritedSetup<I, O, S, F>
        where S: FnMut() -> I + 'a, F: Fn($input) -> O + Sync + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str, workers: usize, drop: crate::DropPolicy) {
                let (setup, work) = self.callbacks.borrow_mut().take().expect("setup registered once");
                suite.$runner(name, workers, setup, work, drop);
            }
        }
        impl<'a, I: 'a, O: 'a, S, F> $trait<'a> for &InheritedSetup<I, O, S, F>
        where S: FnMut() -> I + 'a, F: Fn($input) -> O + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str, workers: usize, drop: crate::DropPolicy) {
                if workers == 1 {
                    let (setup, work) = self.callbacks.borrow_mut().take().expect("setup registered once");
                    suite.$local(name, setup, work, drop, suite.registration_batch_policy());
                    suite.local_thread_contract();
                    return;
                }
                suite.unavailable_thread_case(name, "cannot transfer setup inputs or outputs between workers; declare worker-local setup on its group or use threads = false");
            }
        }
    };
}
inherited_setup_dispatch!(
    inherited_setup,
    RegisterInheritedSetup,
    register_inherited_setup,
    &mut I,
    bench_threads_with_input,
    bench_batched_ref
);
inherited_setup_dispatch!(
    inherited_owned_setup,
    RegisterInheritedOwnedSetup,
    register_inherited_owned_setup,
    I,
    bench_threads_with_owned_input,
    bench_batched
);

macro_rules! inherited_allocated_setup_dispatch {
    ($trait:ident, $method:ident, $input:ty, $runner:ident, $local:ident) => {
        #[doc(hidden)]
        pub trait $trait<'a, A: std::alloc::GlobalAlloc + 'a> {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str,
                allocator: &'a crate::alloc::TrackingAllocator<A>, workers: usize, drop: crate::DropPolicy);
        }
        impl<'a, A: std::alloc::GlobalAlloc + Sync + 'a, I: Send + 'a, O: Send + 'a, S, F>
            $trait<'a, A> for &&InheritedSetup<I, O, S, F>
        where S: FnMut() -> I + 'a, F: Fn($input) -> O + Sync + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str,
                allocator: &'a crate::alloc::TrackingAllocator<A>, workers: usize, drop: crate::DropPolicy) {
                let (setup, work) = self.callbacks.borrow_mut().take().expect("setup registered once");
                suite.$runner(name, allocator, workers, setup, work, drop);
            }
        }
        impl<'a, A: std::alloc::GlobalAlloc + 'a, I: 'a, O: 'a, S, F>
            $trait<'a, A> for &InheritedSetup<I, O, S, F>
        where S: FnMut() -> I + 'a, F: Fn($input) -> O + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str,
                allocator: &'a crate::alloc::TrackingAllocator<A>, workers: usize, drop: crate::DropPolicy) {
                if workers == 1 {
                    let (setup, work) = self.callbacks.borrow_mut().take().expect("setup registered once");
                    suite.$local(name, allocator, setup, work, drop,
                        suite.registration_batch_policy());
                    suite.local_thread_contract();
                } else {
                    suite.unavailable_thread_case(name, "cannot transfer setup inputs or outputs between workers; declare worker-local setup on its group or use threads = false");
                }
            }
        }
    };
}
inherited_allocated_setup_dispatch!(
    RegisterInheritedAllocatedSetup,
    register_inherited_allocated_setup,
    &mut I,
    bench_threads_allocated_with_input,
    bench_allocated_batched_ref
);
inherited_allocated_setup_dispatch!(
    RegisterInheritedAllocatedOwnedSetup,
    register_inherited_allocated_owned_setup,
    I,
    bench_threads_allocated_with_owned_input,
    bench_allocated_batched
);

macro_rules! inherited_async_setup_dispatch {
    ($constructor:ident, $trait:ident, $method:ident, $input:ty, $runner:ident, $local:ident) => {
        #[doc(hidden)]
        pub fn $constructor<T, E, I, O, X, S, F>(marker: std::marker::PhantomData<T>, executor: X, setup: S, work: F) -> InheritedCallback<T, (X, S, F)>
        where X: Fn() -> E, S: Fn() -> I, F: AsyncFn($input) -> O, E: crate::workloads::Executor {
            InheritedCallback::new(marker, (executor, setup, work))
        }
        #[doc(hidden)]
        pub trait $trait<'a, E, I, O> {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str, workers: usize, drop: crate::DropPolicy);
        }
        impl<'a, T: Sync, E, I: 'a, O: 'a, X, S, F> $trait<'a, E, I, O> for &&InheritedCallback<T, (X, S, F)>
        where X: Fn() -> E + Sync + 'a, S: Fn() -> I + Sync + 'a, F: AsyncFn($input) -> O + Sync + 'a, E: crate::workloads::Executor + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str, workers: usize, drop: crate::DropPolicy) {
                let (executor, setup, work) = self.0.borrow_mut().take().expect("setup registered once");
                suite.$runner(name, workers, executor, setup, work, drop);
            }
        }
        impl<'a, T, E, I: 'a, O: 'a, X, S, F> $trait<'a, E, I, O> for &InheritedCallback<T, (X, S, F)>
        where X: Fn() -> E + 'a, S: Fn() -> I + 'a, F: AsyncFn($input) -> O + 'a, E: crate::workloads::Executor + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str, workers: usize, drop: crate::DropPolicy) {
                if workers == 1 {
                    let (executor, setup, work) = self.0.borrow_mut().take().expect("setup registered once");
                    suite.$local(name, executor, setup, work, drop, suite.registration_batch_policy());
                    suite.local_thread_contract();
                    return;
                }
                suite.unavailable_thread_case(name, "cannot share setup arguments between workers; use thread-safe arguments or threads = false");
            }
        }
    };
}
inherited_async_setup_dispatch!(
    inherited_async_setup,
    RegisterInheritedAsyncSetup,
    register_inherited_async_setup,
    &mut I,
    bench_async_threads_with_input,
    bench_async_batched_ref
);
inherited_async_setup_dispatch!(
    inherited_async_owned_setup,
    RegisterInheritedAsyncOwnedSetup,
    register_inherited_async_owned_setup,
    I,
    bench_async_threads_with_owned_input,
    bench_async_owned_inherited
);

macro_rules! inherited_async_allocated_setup_dispatch {
    ($trait:ident, $method:ident, $input:ty, $runner:ident, $local:ident) => {
        #[doc(hidden)]
        pub trait $trait<'a, A: std::alloc::GlobalAlloc + 'a, E, I, O> {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str,
                allocator: &'a crate::alloc::TrackingAllocator<A>, workers: usize, drop: crate::DropPolicy);
        }
        impl<'a, A: std::alloc::GlobalAlloc + Sync + 'a, T: Sync, E, I: 'a, O: 'a, X, S, F>
            $trait<'a, A, E, I, O> for &&InheritedCallback<T, (X, S, F)>
        where X: Fn() -> E + Sync + 'a, S: Fn() -> I + Sync + 'a,
            F: AsyncFn($input) -> O + Sync + 'a, E: crate::workloads::Executor + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str,
                allocator: &'a crate::alloc::TrackingAllocator<A>, workers: usize, drop: crate::DropPolicy) {
                let (executor, setup, work) = self.0.borrow_mut().take().expect("setup registered once");
                suite.$runner(name, allocator, workers, executor, setup, work, drop);
            }
        }
        impl<'a, A: std::alloc::GlobalAlloc + 'a, T, E, I: 'a, O: 'a, X, S, F>
            $trait<'a, A, E, I, O> for &InheritedCallback<T, (X, S, F)>
        where X: Fn() -> E + 'a, S: Fn() -> I + 'a,
            F: AsyncFn($input) -> O + 'a, E: crate::workloads::Executor + 'a {
            fn $method(self, suite: &mut crate::Suite<'a>, name: &str,
                allocator: &'a crate::alloc::TrackingAllocator<A>, workers: usize, drop: crate::DropPolicy) {
                if workers == 1 {
                    let (executor, setup, work) = self.0.borrow_mut().take().expect("setup registered once");
                    suite.$local(name, allocator, executor, setup, work, drop,
                        suite.registration_batch_policy());
                    suite.local_thread_contract();
                } else {
                    suite.unavailable_thread_case(name, "cannot share setup arguments between workers; use thread-safe arguments or threads = false");
                }
            }
        }
    };
}
inherited_async_allocated_setup_dispatch!(
    RegisterInheritedAsyncAllocatedSetup,
    register_inherited_async_allocated_setup,
    &mut I,
    bench_async_threads_allocated_with_input,
    bench_async_allocated_batched_ref
);
inherited_async_allocated_setup_dispatch!(
    RegisterInheritedAsyncAllocatedOwnedSetup,
    register_inherited_async_allocated_owned_setup,
    I,
    bench_async_threads_allocated_with_owned_input,
    bench_async_allocated_batched
);
