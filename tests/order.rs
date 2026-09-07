use runit::{CallSequence, Mock, with_mocks};
use std::sync::{Arc, Barrier};
#[test]
fn order_across_objects_and_methods() {
    let order = CallSequence::new("checkout");
    let charge = Mock::new("payments.charge");
    let save = Mock::new("orders.save");
    charge
        .expect("charge", |_: &u8| true)
        .in_sequence(&order)
        .returns(1);
    save.expect("save", |_: &String| true)
        .in_sequence(&order)
        .returns(());
    with_mocks(&[&charge, &save, &order], || {
        assert_eq!(charge.call(7), 1);
        save.call("order".into());
    });
}
#[test]
fn wrong_order_does_not_execute_answer_capture_or_consume_counts() {
    let order = CallSequence::new("flow");
    let a = Mock::<(), ()>::new("first");
    let b = Mock::<(), ()>::new("second");
    let capture = runit::mock::Capture::new(3);
    let executed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = executed.clone();
    a.expect("first", |_| true).in_sequence(&order).returns(());
    b.expect("second", |_| true)
        .capture(&capture)
        .in_sequence(&order)
        .returning(move |_| {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
    b.journal_capacity(2);
    let error = b.try_call(()).unwrap_err().to_string();
    assert!(error.contains("expected step 1 first") && error.contains("received step 2 second"));
    assert!(capture.values().is_empty());
    assert_eq!(executed.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert!(!b.calls()[0].accepted);
    a.call(());
    b.call(());
    assert_eq!(executed.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(b.verify().is_err());
    assert!(order.verify().is_err());
}
#[test]
fn missing_steps_are_reported_by_sequence() {
    let order = CallSequence::new("missing");
    let a = Mock::<(), ()>::new("first");
    let b = Mock::<(), ()>::new("second");
    a.expect("a", |_| true).in_sequence(&order).returns(());
    b.expect("b", |_| true).in_sequence(&order).returns(());
    a.call(());
    let error = order.verify().unwrap_err().to_string();
    assert!(error.contains("step 2 second") && error.contains("actual 0"));
}
#[test]
fn same_method_with_distinct_matchers_can_have_multiple_steps() {
    let order = CallSequence::new("same method");
    let mock = Mock::<u8, ()>::new("call");
    mock.expect("one", |v| *v == 1)
        .in_sequence(&order)
        .returns(());
    mock.expect("two", |v| *v == 2)
        .in_sequence(&order)
        .returns(());
    mock.call(1);
    mock.call(2);
    mock.assert_verified();
    order.assert_verified();
}
#[test]
fn repeated_step_must_finish_before_next_step() {
    let order = CallSequence::new("repeat");
    let a = Mock::<(), _>::new("a");
    let b = Mock::<(), ()>::new("b");
    a.expect("two", |_| true)
        .in_sequence(&order)
        .returns_sequence([1, 2]);
    b.expect("next", |_| true).in_sequence(&order).returns(());
    assert_eq!(a.call(()), 1);
    assert!(b.try_call(()).is_err());
    assert_eq!(a.call(()), 2);
    b.call(());
    assert!(order.verify().is_err());
}
#[test]
fn late_registration_is_rejected_without_poisoning_locks() {
    let order = CallSequence::new("sealed");
    let a = Mock::<(), ()>::new("a");
    let b = Mock::<(), ()>::new("b");
    a.expect("a", |_| true).in_sequence(&order).returns(());
    a.call(());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| b
            .expect("late", |_| true)
            .in_sequence(&order)
            .returns(())))
        .is_err()
    );
    order.assert_verified();
    b.expect("plain", |_| true).returns(());
    b.call(());
    b.assert_verified();
}
#[test]
fn invalid_count_does_not_reserve_a_step() {
    for range in [(0, 0), (1, 2)] {
        let order = CallSequence::new("invalid");
        let mock = Mock::<(), ()>::new("m");
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mock
                .expect("bad", |_| true)
                .times_between(range.0, range.1)
                .in_sequence(&order)
                .returns(())))
            .is_err()
        );
        order.assert_verified();
        mock.expect("good", |_| true)
            .in_sequence(&order)
            .returns(());
        mock.call(());
        order.assert_verified();
    }
}
#[test]
fn unordered_calls_can_interleave() {
    let order = CallSequence::new("flow");
    let a = Mock::<(), ()>::new("a");
    let b = Mock::<(), ()>::new("b");
    let logs = Mock::<(), ()>::new("log");
    a.expect("a", |_| true).in_sequence(&order).returns(());
    b.expect("b", |_| true).in_sequence(&order).returns(());
    logs.expect("log", |_| true).at_least(0).returns(());
    logs.call(());
    a.call(());
    logs.call(());
    b.call(());
    logs.call(());
    order.assert_verified();
}
#[test]
fn order_is_admission_not_completion() {
    let order = CallSequence::new("concurrent");
    let a = Mock::<(), ()>::new("a");
    let b = Mock::<(), ()>::new("b");
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let start = entered.clone();
    let finish = release.clone();
    a.expect("blocked answer", |_| true)
        .in_sequence(&order)
        .returning(move |_| {
            start.wait();
            finish.wait();
        });
    b.expect("next", |_| true).in_sequence(&order).returns(());
    let worker = a.clone();
    let thread = std::thread::spawn(move || worker.call(()));
    entered.wait();
    b.call(());
    release.wait();
    thread.join().unwrap();
    a.assert_verified();
    b.assert_verified();
    order.assert_verified();
}
#[test]
fn reentrant_answer_can_call_next_step() {
    let order = CallSequence::new("nested");
    let first = Mock::<(), ()>::new("first");
    let second = Mock::<(), ()>::new("second");
    let nested = second.clone();
    first
        .expect("outer", |_| true)
        .in_sequence(&order)
        .returning(move |_| nested.call(()));
    second
        .expect("inner", |_| true)
        .in_sequence(&order)
        .returns(());
    first.call(());
    order.assert_verified();
}
#[test]
fn ordered_reset_is_rejected() {
    let order = CallSequence::new("reset");
    let mock = Mock::<(), ()>::new("m");
    mock.expect("m", |_| true).in_sequence(&order).returns(());
    mock.call(());
    assert!(mock.reset_counts().is_err());
    order.assert_verified();
}
#[test]
fn excessive_call_is_retained_by_sequence() {
    let order = CallSequence::new("excess");
    let mock = Mock::<(), ()>::new("m");
    mock.expect("m", |_| true).in_sequence(&order).returns(());
    mock.call(());
    assert!(mock.try_call(()).is_err());
    assert!(order.verify().unwrap_err().to_string().contains("exceeded"));
}
#[test]
fn callbacks_panicking_still_consume_admitted_step() {
    let order = CallSequence::new("panic");
    let first = Mock::<(), ()>::new("first");
    let next = Mock::<(), ()>::new("next");
    first
        .expect("panic", |_| true)
        .in_sequence(&order)
        .returning(|_| panic!("answer"));
    next.expect("next", |_| true)
        .in_sequence(&order)
        .returns(());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| first.call(()))).is_err());
    next.call(());
    order.assert_verified();
}
