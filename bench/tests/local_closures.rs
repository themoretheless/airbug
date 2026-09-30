use airbug_bench::{Config, Sampling, Suite};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[test]
fn local_closure_borrows_mutable_stack_state_and_preserves_it_across_runs() {
    struct LocalOutput<'a>(&'a RefCell<Vec<usize>>, usize, Rc<()>);
    impl Drop for LocalOutput<'_> {
        fn drop(&mut self) {
            assert_eq!(Rc::strong_count(&self.2), 2);
            self.0.borrow_mut().push(self.1);
        }
    }
    let mut count = 0;
    let drops = RefCell::new(Vec::new());
    let not_send = Rc::new(());
    {
        let mut suite = Suite::new("local");
        suite.bench("counter", || {
            count += 1;
            LocalOutput(&drops, count, not_send.clone())
        });
        suite.sampling(Sampling {
            iterations: Some(3),
            ..Default::default()
        });
        suite.bench("unselected", || panic!("unselected closure executed"));
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 3,
        });
        assert_eq!(suite.list("counter"), ["local/counter"]);
        assert!(drops.borrow().is_empty());
        for _ in 0..2 {
            let run = suite.run("counter").unwrap();
            assert_eq!(run.observations.len(), 2);
            assert!(run.observations.iter().all(|o| o.operations == 3));
        }
    }
    assert_eq!(count, 12);
    assert_eq!(*drops.borrow(), (1..=12).collect::<Vec<_>>());
    assert_eq!(Rc::strong_count(&not_send), 1);
}

#[test]
fn local_setup_and_operations_borrow_separate_mutable_state_with_non_send_inputs() {
    use airbug_bench::DropPolicy;
    for owned in [false, true] {
        let mut generated = 0;
        let mut seen = Vec::new();
        {
            let mut suite = Suite::new("local-input");
            let setup = || {
                generated += 1;
                Rc::new(RefCell::new(generated))
            };
            if owned {
                suite.bench_with_owned_input(
                    "owned",
                    setup,
                    |input| {
                        seen.push(*input.borrow());
                        input
                    },
                    DropPolicy::OutsideTiming,
                );
            } else {
                suite.bench_with_input(
                    "borrowed",
                    setup,
                    |input| {
                        seen.push(*input.borrow());
                        *input.borrow_mut() += 100;
                        input.clone()
                    },
                    DropPolicy::OutsideTiming,
                );
            }
            suite.sampling(Sampling {
                iterations: Some(65),
                ..Default::default()
            });
            suite.config(Config {
                samples: 2,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 65,
            });
            assert_eq!(suite.list("").len(), 1);
            suite.run("").unwrap();
        }
        assert_eq!(generated, 130);
        assert_eq!(seen, (1..=130).collect::<Vec<_>>());
    }
}

#[test]
fn shared_fixture_setup_is_untimed_and_context_lives_until_last_owner() {
    use airbug_bench::Fixture;
    use std::{cell::Cell, time::Instant};
    struct Context {
        calls: usize,
        destroyed: Rc<Cell<usize>>,
    }
    impl Drop for Context {
        fn drop(&mut self) {
            assert_eq!(self.calls, 12);
            self.destroyed.set(self.destroyed.get() + 1);
        }
    }
    let setup_ns = Rc::new(Cell::new(0));
    let setups = Rc::new(Cell::new(0));
    let destroyed = Rc::new(Cell::new(0));
    let fixture = Fixture::new({
        let setup_ns = setup_ns.clone();
        let setups = setups.clone();
        let destroyed = destroyed.clone();
        move || {
            let start = Instant::now();
            std::thread::sleep(Duration::from_millis(10));
            setup_ns.set(start.elapsed().as_nanos());
            setups.set(setups.get() + 1);
            Context {
                calls: 0,
                destroyed,
            }
        }
    });
    let mut suite = Suite::new("context");
    for name in ["first", "second"] {
        suite.bench_fixture(name, fixture.clone(), |context| {
            context.calls += 1;
        });
        suite.sampling(Sampling {
            iterations: Some(3),
            ..Default::default()
        });
    }
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 3,
    });
    assert_eq!(suite.list("").len(), 2);
    assert_eq!(setups.get(), 0);
    let start = Instant::now();
    let run = suite.run("").unwrap();
    let wall = start.elapsed().as_nanos();
    let measured: u128 = run
        .observations
        .iter()
        .map(|o| o.value.as_ref().unwrap().parse::<u128>().unwrap())
        .sum();
    assert_eq!(setups.get(), 1);
    assert!(wall >= measured + setup_ns.get());
    drop(suite);
    assert_eq!(
        destroyed.get(),
        0,
        "external fixture owner retains the context"
    );
    drop(fixture);
    assert_eq!(destroyed.get(), 1);
}
