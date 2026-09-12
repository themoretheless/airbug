//! Explicit async/thread/pipeline lifecycle helpers. No hidden runtime dependency.
use crate::{error, Result};
use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};
pub trait Executor {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output;
}
#[derive(Default)]
pub struct LocalExecutor;
impl Executor for LocalExecutor {
    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        struct Signal(std::thread::Thread);
        impl Wake for Signal {
            fn wake(self: Arc<Self>) {
                self.0.unpark();
            }
            fn wake_by_ref(self: &Arc<Self>) {
                self.0.unpark();
            }
        }
        let waker = Waker::from(Arc::new(Signal(std::thread::current())));
        let mut cx = Context::from_waker(&waker);
        let mut f = std::pin::pin!(future);
        loop {
            match f.as_mut().poll(&mut cx) {
                Poll::Ready(v) => return v,
                Poll::Pending => std::thread::park(),
            }
        }
    }
}
#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
#[derive(Debug)]
pub struct ParallelResult<T> {
    pub outputs: Vec<T>,
    pub wall_ns: u128,
    pub workers: usize,
}
/// Thread creation and joins are included; a start barrier synchronizes useful work.
/// A noncooperative closure still needs the process runner timeout.
pub fn parallel<T: Send>(
    workers: usize,
    work: impl Fn(usize) -> T + Sync,
) -> Result<ParallelResult<T>> {
    if workers == 0 || workers > 256 {
        return Err(error("workers must be 1..256"));
    }
    let gate = (Mutex::new(None::<bool>), Condvar::new());
    let start = Instant::now();
    let outputs = std::thread::scope(|scope| -> Result<Vec<T>> {
        let mut handles = vec![];
        let mut spawn_error = None;
        for i in 0..workers {
            let gate = &gate;
            let f = &work;
            match std::thread::Builder::new().spawn_scoped(scope, move || {
                let mut state = gate.0.lock().unwrap();
                while state.is_none() {
                    state = gate.1.wait(state).unwrap();
                }
                let proceed = state.unwrap();
                drop(state);
                proceed.then(|| f(i))
            }) {
                Ok(h) => handles.push(h),
                Err(e) => {
                    spawn_error = Some(e);
                    break;
                }
            }
        }
        *gate.0.lock().unwrap() = Some(spawn_error.is_none());
        gate.1.notify_all();
        let mut outputs = vec![];
        let mut panicked = false;
        for h in handles {
            match h.join() {
                Ok(Some(v)) => outputs.push(v),
                Ok(None) => {}
                Err(_) => panicked = true,
            }
        }
        if let Some(e) = spawn_error {
            return Err(e.into());
        }
        if panicked {
            return Err(error("parallel worker panicked"));
        }
        Ok(outputs)
    })?;
    Ok(ParallelResult {
        outputs,
        wall_ns: start.elapsed().as_nanos(),
        workers,
    })
}
#[derive(Debug)]
pub struct PipelineResult<T> {
    pub outputs: Vec<T>,
    pub wall_ns: u128,
    pub latency_ns: Vec<u128>,
    pub send_wait_and_overhead_ns: u128,
    pub processed: usize,
    pub consumer_work_ns: u128,
}
/// One producer, one consumer, bounded queue. Latency starts BEFORE send (includes backpressure).
/// Cancellation is cooperative, checked around each item. Outputs are bounded by input length.
pub fn pipeline<I: Send, O: Send>(
    inputs: Vec<I>,
    capacity: usize,
    cancel: &Cancellation,
    consume: impl FnMut(I) -> O + Send,
) -> Result<PipelineResult<O>> {
    if capacity == 0 || capacity > 65536 || inputs.len() > 1_000_000 {
        return Err(error("pipeline capacity 1..65536; inputs <=1000000"));
    }
    let start = Instant::now();
    let (tx, rx) = std::sync::mpsc::sync_channel(capacity);
    let (outputs, latency_ns, blocked, consumer_work_ns) =
        std::thread::scope(|scope| -> Result<_> {
            let producer = std::thread::Builder::new().spawn_scoped(scope, move || {
                let mut blocked = 0;
                for item in inputs {
                    if cancel.cancelled() {
                        break;
                    }
                    let begin = Instant::now();
                    let mut packet = (begin, item);
                    loop {
                        match tx.try_send(packet) {
                            Ok(()) => break,
                            Err(std::sync::mpsc::TrySendError::Full(p)) => {
                                packet = p;
                                if cancel.cancelled() {
                                    return blocked;
                                }
                                std::thread::sleep(Duration::from_micros(20));
                            }
                            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => return blocked,
                        }
                    }
                    blocked += begin.elapsed().as_nanos();
                }
                blocked
            })?;
            let consumer = std::thread::Builder::new().spawn_scoped(scope, move || {
                let mut consume = consume;
                let mut outputs = vec![];
                let mut latencies = vec![];
                let mut busy = 0;
                while !cancel.cancelled() {
                    match rx.recv_timeout(Duration::from_millis(1)) {
                        Ok((begin, item)) => {
                            let work = Instant::now();
                            outputs.push(consume(item));
                            busy += work.elapsed().as_nanos();
                            latencies.push(begin.elapsed().as_nanos());
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(_) => continue,
                    }
                }
                (outputs, latencies, busy)
            })?;
            // Sender lives in producer; a normal completion disconnects the channel.
            let blocked = producer.join().map_err(|_| error("producer panicked"))?;
            let (o, l, busy) = consumer.join().map_err(|_| error("consumer panicked"))?;
            Ok((o, l, blocked, busy))
        })?;
    if cancel.cancelled() {
        return Err(error("pipeline cancelled"));
    }
    Ok(PipelineResult {
        processed: outputs.len(),
        consumer_work_ns,
        outputs,
        wall_ns: start.elapsed().as_nanos(),
        latency_ns,
        send_wait_and_overhead_ns: blocked,
    })
}

impl<T> PipelineResult<T> {
    /// End-to-end items/second including worker lifecycle and queue overhead.
    pub fn throughput_per_second(&self) -> Option<f64> {
        (self.wall_ns > 0).then(|| self.processed as f64 * 1e9 / self.wall_ns as f64)
    }
}
