use airbug_bench::workloads::{Executor, LocalExecutor};
use std::{future::poll_fn, rc::Rc, task::Poll};

async fn local_future() -> Rc<usize> {
    let value = Rc::new(7);
    let mut pending = true;
    poll_fn(|cx| {
        if std::mem::take(&mut pending) {
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    })
    .await;
    value
}
fn check_local(mut executor: impl Executor) {
    assert_eq!(*executor.block_on(local_future()), 7);
}
#[test]
fn borrowed_executor_keeps_non_send_future_and_output() {
    let mut executor = LocalExecutor;
    check_local(&mut executor);
    check_local(executor);
}
#[cfg(feature = "async-futures")]
#[test]
fn futures_executor_drives_wakes_without_send_bounds() {
    check_local(airbug_bench::executors::FuturesExecutor);
}

#[cfg(feature = "async-tokio")]
mod tokio_tests {
    use super::*;
    use std::{cell::Cell, time::Duration};
    async fn runtime_work() -> Rc<usize> {
        let value = Rc::new(7);
        tokio::time::sleep(Duration::from_millis(1)).await;
        assert_eq!(tokio::spawn(async { 42 }).await.unwrap(), 42);
        value
    }
    fn check_runtime(mut executor: impl Executor) {
        check_local(&mut executor);
        assert_eq!(*executor.block_on(runtime_work()), 7);
    }
    #[test]
    fn tokio_owned_borrowed_runtime_and_handles_drive_timers_and_tasks() {
        let mut current = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        check_runtime(&mut current);
        check_runtime(&current);
        check_runtime(current);
        let multi = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        check_runtime(multi.handle());
        check_runtime(multi.handle().clone());
        check_runtime(multi);
    }
    #[test]
    fn suite_constructs_runtime_lazily_and_reuses_it_between_samples() {
        let constructions = Cell::new(0);
        let mut suite = airbug_bench::Suite::new("tokio");
        suite.bench_async_factory(
            "timer",
            || {
                constructions.set(constructions.get() + 1);
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
            },
            runtime_work,
            airbug_bench::DropPolicy::OutsideTiming,
        );
        suite.sampling(airbug_bench::Sampling {
            iterations: Some(1),
            ..Default::default()
        });
        suite.config(airbug_bench::Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 1,
        });
        assert_eq!(suite.list("").len(), 1);
        assert_eq!(constructions.get(), 0);
        let run = suite.run("").unwrap();
        assert_eq!(constructions.get(), 1);
        assert_eq!(run.observations.len(), 2);
        assert!(run.cases[0].contract["async.executor"].contains("Runtime"));
    }
}

#[cfg(feature = "async-smol")]
#[test]
fn smol_executor_drives_timer_spawn_and_local_future() {
    let mut executor = airbug_bench::executors::SmolExecutor;
    check_local(&mut executor);
    let value = executor.block_on(async {
        let local = Rc::new(7);
        smol::Timer::after(std::time::Duration::from_millis(1)).await;
        assert_eq!(smol::spawn(async { 42 }).await, 42);
        local
    });
    assert_eq!(*value, 7);
}

#[cfg(all(unix, feature = "async-tokio"))]
#[test]
fn tokio_executor_drives_os_io() {
    let mut runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let bytes = Executor::block_on(&mut runtime, async {
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            let (reader, writer) = tokio::net::UnixStream::pair().unwrap();
            let write = tokio::spawn(async move {
                let mut written = 0;
                while written < 4 {
                    writer.writable().await.unwrap();
                    match writer.try_write(&b"data"[written..]) {
                        Ok(0) => panic!("write made no progress"),
                        Ok(n) => written += n,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                        Err(e) => panic!("write: {e}"),
                    }
                }
            });
            let mut buffer = [0u8; 4];
            let mut read = 0;
            while read < buffer.len() {
                reader.readable().await.unwrap();
                match reader.try_read(&mut buffer[read..]) {
                    Ok(0) => panic!("unexpected EOF"),
                    Ok(n) => read += n,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => panic!("read: {e}"),
                }
            }
            write.await.unwrap();
            buffer
        })
        .await
        .unwrap()
    });
    assert_eq!(&bytes, b"data");
}

#[cfg(all(unix, feature = "async-smol"))]
#[test]
fn smol_executor_drives_os_io() {
    use smol::io::{AsyncReadExt, AsyncWriteExt};
    let mut executor = airbug_bench::executors::SmolExecutor;
    let bytes = executor.block_on(async {
        smol::future::or(
            async {
                let (reader, writer) = std::os::unix::net::UnixStream::pair().unwrap();
                let mut reader = smol::Async::new(reader).unwrap();
                let mut writer = smol::Async::new(writer).unwrap();
                let write = smol::spawn(async move {
                    writer.write_all(b"data").await.unwrap();
                });
                let mut buffer = [0u8; 4];
                reader.read_exact(&mut buffer).await.unwrap();
                write.await;
                buffer
            },
            async {
                smol::Timer::after(std::time::Duration::from_secs(1)).await;
                panic!("Smol I/O timed out");
            },
        )
        .await
    });
    assert_eq!(&bytes, b"data");
}
