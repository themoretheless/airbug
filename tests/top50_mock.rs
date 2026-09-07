use runit::{
    Mock,
    mock::{Capture, Matcher},
};
#[test]
fn sequence_has_exact_order_and_exhaustion() {
    let mock = Mock::<(), _>::new("sequence");
    mock.expect("responses", |_| true)
        .returns_sequence([1, 2, 3]);
    assert_eq!([mock.call(()), mock.call(()), mock.call(())], [1, 2, 3]);
    mock.assert_verified();
    assert!(mock.try_call(()).is_err());
    assert!(mock.reset_counts().is_err());
}
#[test]
fn one_shot_owns_non_clone_response() {
    struct NonClone(u8);
    let mock = Mock::<(), _>::new("once");
    let value = NonClone(7);
    mock.expect("one", |_| true).returning_once(move |_| value);
    assert_eq!(mock.call(()).0, 7);
    mock.assert_verified();
    assert!(mock.try_call(()).is_err());
}
#[test]
fn range_counts_enforce_both_bounds() {
    let mock = Mock::<(), ()>::new("range");
    mock.expect("range", |_| true)
        .times_between(2, 3)
        .returns(());
    assert!(mock.verify().is_err());
    mock.call(());
    assert!(mock.verify().is_err());
    mock.call(());
    mock.assert_verified();
    mock.call(());
    mock.assert_verified();
    assert!(mock.try_call(()).is_err());
}
#[test]
fn at_least_requires_minimum() {
    let mock = Mock::<(), ()>::new("min");
    mock.expect("min", |_| true).at_least(1).returns(());
    assert!(mock.verify().is_err());
    for _ in 0..10 {
        mock.call(());
    }
    mock.assert_verified();
}
#[test]
fn at_most_accepts_zero_and_rejects_excess() {
    let mock = Mock::<(), ()>::new("max");
    mock.expect("max", |_| true).at_most(1).returns(());
    mock.assert_verified();
    mock.call(());
    mock.assert_verified();
    assert!(mock.try_call(()).is_err());
}
#[test]
fn capture_keeps_owned_recent_arguments() {
    let capture = Capture::new(2);
    let mock = Mock::<String, ()>::new("capture");
    mock.expect("all", |_| true)
        .times(3)
        .capture(&capture)
        .returns(());
    for text in ["a", "b", "c"] {
        mock.call(text.into());
    }
    assert_eq!(capture.values(), ["b", "c"]);
    capture.clear();
    assert!(capture.values().is_empty());
}
#[test]
fn journal_records_matched_and_unexpected_calls() {
    let mock = Mock::<u8, ()>::new("journal");
    mock.journal_capacity(5);
    mock.expect("one", |v| *v == 1).returns(());
    mock.call(1);
    let _ = mock.try_call(2);
    let calls = mock.calls();
    assert_eq!(calls.len(), 2);
    assert!(calls[0].accepted);
    assert!(!calls[1].accepted);
    assert_eq!(calls[0].expectation.as_deref(), Some("one"));
}
#[test]
fn journal_is_bounded_and_can_be_disabled() {
    let mock = Mock::<u8, ()>::new("bounded");
    mock.journal_capacity(2);
    mock.expect("all", |_| true).at_least(0).returns(());
    for n in 0..5 {
        mock.call(n);
    }
    assert_eq!(
        mock.calls()
            .iter()
            .map(|c| c.arguments.as_str())
            .collect::<Vec<_>>(),
        ["3", "4"]
    );
    mock.journal_capacity(0);
    mock.call(9);
    assert!(mock.calls().is_empty());
}
#[test]
fn named_matchers_compose_and_reuse() {
    let positive = Matcher::new("positive", |v: &i32| *v > 0);
    let even = Matcher::new("even", |v: &i32| v % 2 == 0);
    let combined = positive.clone().and(even.clone());
    assert!(combined.matches(&2));
    assert!(!combined.matches(&1));
    assert!(positive.or(even).matches(&-2));
    assert!(combined.clone().negate().matches(&1));
    let a = Mock::new("a");
    let b = Mock::new("b");
    a.expect_matcher(combined.clone()).returns(1);
    b.expect_matcher(combined).returns(2);
    assert_eq!(a.call(2), 1);
    assert_eq!(b.call(4), 2);
    a.assert_verified();
    b.assert_verified();
}
#[test]
fn reset_opens_new_counting_phase() {
    let mock = Mock::<(), ()>::new("phases");
    mock.expect("once", |_| true).returns(());
    mock.call(());
    mock.assert_verified();
    mock.reset_counts().unwrap();
    assert!(mock.verify().is_err());
    mock.call(());
    mock.assert_verified();
}
#[test]
fn concurrent_capture_and_counts_remain_consistent() {
    let capture = Capture::new(100);
    let mock = Mock::<usize, ()>::new("threads");
    mock.expect("all", |_| true)
        .times(100)
        .capture(&capture)
        .returns(());
    let threads: Vec<_> = (0..4)
        .map(|n| {
            let mock = mock.clone();
            std::thread::spawn(move || {
                for i in 0..25 {
                    mock.call(n * 25 + i);
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    mock.assert_verified();
    let mut values = capture.values();
    values.sort();
    assert_eq!(values, (0..100).collect::<Vec<_>>());
}

#[test]
fn reset_rejects_in_flight_answer() {
    let entered = std::sync::Arc::new(std::sync::Barrier::new(2));
    let release = std::sync::Arc::new(std::sync::Barrier::new(2));
    let mock = Mock::<(), ()>::new("active");
    let start = entered.clone();
    let finish = release.clone();
    mock.expect("answer", |_| true).returning(move |_| {
        start.wait();
        finish.wait();
    });
    let worker = mock.clone();
    let thread = std::thread::spawn(move || worker.call(()));
    entered.wait();
    assert!(mock.reset_counts().is_err());
    release.wait();
    thread.join().unwrap();
    mock.assert_verified();
    mock.reset_counts().unwrap();
}
