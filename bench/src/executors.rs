//! Optional adapters for production async runtimes. Enable only the runtime you
//! use; the default benchmark dependency does not pull in or start a runtime.
//! Construct runtime-dependent futures inside an async block so their creation
//! occurs after the runtime has been entered.
use crate::workloads::Executor;
use std::future::Future;

impl<E: Executor + ?Sized> Executor for &mut E {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        (**self).block_on(future)
    }
}

/// Current-thread executor from the futures crate. Does not supply an I/O reactor.
#[cfg(feature = "async-futures")]
#[derive(Clone, Copy, Debug, Default)]
pub struct FuturesExecutor;
#[cfg(feature = "async-futures")]
impl Executor for FuturesExecutor {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        futures_executor::block_on(future)
    }
}

/// Smol's executor; supports Smol timers and I/O without requiring Send futures.
#[cfg(feature = "async-smol")]
#[derive(Clone, Copy, Debug, Default)]
pub struct SmolExecutor;
#[cfg(feature = "async-smol")]
impl Executor for SmolExecutor {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        smol::block_on(future)
    }
}

#[cfg(feature = "async-tokio")]
impl Executor for tokio::runtime::Runtime {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        tokio::runtime::Runtime::block_on(self, future)
    }
}
#[cfg(feature = "async-tokio")]
impl Executor for &tokio::runtime::Runtime {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        tokio::runtime::Runtime::block_on(self, future)
    }
}
/// A current-thread runtime's Handle needs its Runtime running elsewhere to
/// drive timers/I/O. Prefer the Runtime itself when benchmarking that runtime.
#[cfg(feature = "async-tokio")]
impl Executor for tokio::runtime::Handle {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        tokio::runtime::Handle::block_on(self, future)
    }
}
#[cfg(feature = "async-tokio")]
impl Executor for &tokio::runtime::Handle {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        tokio::runtime::Handle::block_on(self, future)
    }
}
