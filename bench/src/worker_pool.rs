//! Persistent OS threads with synchronous, borrowed broadcasts.
//!
//! The only lifetime erasure is below in `broadcast`. A completion is sent only
//! after its callback has returned (or unwound), and a guard drains every
//! dispatched callback before the borrowing frame can leave, including on panic.
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, mpsc},
    thread::{self, JoinHandle},
};

type Callback = Box<dyn FnOnce() + Send + 'static>;
struct Task {
    callback: Callback,
    done: mpsc::Sender<thread::Result<()>>,
}

pub(crate) struct Pool {
    senders: Vec<mpsc::SyncSender<Task>>,
    handles: Vec<JoinHandle<()>>,
}

impl Pool {
    pub(crate) fn new(workers: usize) -> crate::Result<Self> {
        if workers == 0 {
            return Err(crate::error("workers must be positive"));
        }
        let mut pool = Self {
            senders: crate::suite_measure::worker_buffer(workers)?,
            handles: crate::suite_measure::worker_buffer(workers)?,
        };
        for worker in 0..workers {
            let (sender, receiver) = mpsc::sync_channel::<Task>(0);
            let handle = thread::Builder::new()
                .name(format!("airbug-worker-{worker}"))
                .spawn(move || {
                    while let Ok(task) = receiver.recv() {
                        let result = catch_unwind(AssertUnwindSafe(task.callback));
                        // Callback captures have all been dropped before this ack.
                        // The receiving guard remains alive until every ack arrives.
                        let _ = task.done.send(result);
                    }
                })?;
            pool.senders.push(sender);
            pool.handles.push(handle);
        }
        Ok(pool)
    }

    pub(crate) fn len(&self) -> usize {
        self.senders.len()
    }

    pub(crate) fn broadcast<R: Send>(
        &mut self,
        work: impl Fn(usize) -> R + Sync,
    ) -> crate::Result<Vec<R>> {
        let mut results = crate::suite_measure::worker_buffer(self.len())?;
        results.extend((0..self.len()).map(|_| Mutex::new(None)));
        let (done, receiver) = mpsc::channel();
        let mut tasks = crate::suite_measure::worker_buffer(self.len())?;
        for (worker, result) in results.iter().enumerate() {
            let work = &work;
            let callback: Box<dyn FnOnce() + Send + '_> = Box::new(move || {
                let value = work(worker);
                *result.lock().unwrap() = Some(value);
            });
            // SAFETY: only the lifetime is erased. The closure remains Send.
            // `work` and `results` outlive `pending`, whose destructor waits for
            // all dispatched callbacks to finish, even if this method unwinds.
            // Undispatched tasks are dropped before these borrowed locals.
            // No callback or sender is exposed to callers, so this wait cannot
            // be skipped with mem::forget or by leaking a user-visible guard.
            let callback =
                unsafe { std::mem::transmute::<Box<dyn FnOnce() + Send + '_>, Callback>(callback) };
            tasks.push(Task {
                callback,
                done: done.clone(),
            });
        }
        let mut pending = Pending {
            remaining: 0,
            receiver,
        };
        for (sender, task) in self.senders.iter().zip(tasks) {
            pending.remaining += 1;
            // Workers only exit when their sender is dropped, and user panics
            // are caught in their loop. An unexpectedly dead worker violates
            // that invariant: unwinding here could strand a partially issued
            // broadcast at a user barrier. Abort rather than release borrows.
            if sender.send(task).is_err() {
                std::process::abort();
            }
        }
        let mut outcomes = Vec::with_capacity(self.len());
        while pending.remaining > 0 {
            outcomes.push(pending.receive());
        }
        if outcomes.iter().any(Result::is_err) {
            return Err(crate::error("benchmark worker panicked"));
        }
        results
            .into_iter()
            .map(|result| {
                result
                    .into_inner()
                    .unwrap()
                    .ok_or_else(|| crate::error("benchmark worker returned no result"))
            })
            .collect()
    }
}

struct Pending {
    remaining: usize,
    receiver: mpsc::Receiver<thread::Result<()>>,
}
impl Pending {
    fn receive(&mut self) -> thread::Result<()> {
        let result = self
            .receiver
            .recv()
            .unwrap_or_else(|_| std::process::abort());
        self.remaining -= 1;
        result
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        while self.remaining > 0 {
            // On coordinator unwind, even a panic payload with a panicking
            // destructor must not interrupt the wait for borrowed callbacks.
            if let Err(payload) = self.receive() {
                std::mem::forget(payload);
            }
        }
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        self.senders.clear();
        for handle in self.handles.drain(..) {
            // All broadcasts have already finished. Join also completes TLS
            // destruction before a replacement pool starts doing measured work.
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_inputs_and_outputs_survive_broadcast_and_panic() {
        let mut pool = Pool::new(2).unwrap();
        let input = String::from("borrowed stack data");
        let output = pool.broadcast(|_| input.as_str()).unwrap();
        assert_eq!(output, [input.as_str(), input.as_str()]);
        let before = pool.broadcast(|_| thread::current().id()).unwrap();
        assert!(
            pool.broadcast(|worker| {
                assert_ne!(worker, 0, "intentional failure");
                input.as_str()
            })
            .is_err()
        );
        assert_eq!(pool.broadcast(|_| thread::current().id()).unwrap(), before);
        assert_eq!(pool.broadcast(|_| input.as_str()).unwrap(), output);
    }

    #[test]
    fn pending_guard_waits_on_unwind_without_dropping_panic_payloads() {
        use std::sync::atomic::{AtomicBool, Ordering};
        struct BadPayload;
        impl Drop for BadPayload {
            fn drop(&mut self) {
                panic!("payload must not interrupt the pending guard");
            }
        }
        let finished = AtomicBool::new(false);
        thread::scope(|scope| {
            let (done, receiver) = mpsc::channel();
            let first = done.clone();
            scope.spawn(move || {
                first
                    .send(Err(Box::new(BadPayload) as Box<dyn std::any::Any + Send>))
                    .unwrap()
            });
            let finished = &finished;
            scope.spawn(move || {
                thread::sleep(std::time::Duration::from_millis(2));
                finished.store(true, Ordering::Release);
                done.send(Ok(())).unwrap();
            });
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    let _pending = Pending {
                        remaining: 2,
                        receiver,
                    };
                    panic!("coordinator unwind");
                }))
                .is_err()
            );
            // Check before thread::scope does its own joining.
            assert!(finished.load(Ordering::Acquire));
        });
    }

    #[test]
    fn dropping_pool_joins_worker_thread_local_destructors() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        struct Exit(Arc<AtomicUsize>);
        impl Drop for Exit {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        thread_local! { static EXIT: std::cell::RefCell<Option<Exit>> = const { std::cell::RefCell::new(None) }; }
        let exits = Arc::new(AtomicUsize::new(0));
        let mut pool = Pool::new(2).unwrap();
        pool.broadcast(|_| EXIT.with(|slot| *slot.borrow_mut() = Some(Exit(exits.clone()))))
            .unwrap();
        assert_eq!(exits.load(Ordering::SeqCst), 0);
        drop(pool);
        assert_eq!(exits.load(Ordering::SeqCst), 2);
    }
}
