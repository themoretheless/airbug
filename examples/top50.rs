use airbug::prelude::*;
use std::time::{Duration, SystemTime};
fn main() {
    let mut fixture = FixtureContext::with_seed(42);
    fixture.sequence_u64(1);
    let id = fixture.build::<u64>();
    let capture = Capture::new(4);
    let fetch = Mock::<u64, Option<String>>::new("repository.fetch");
    fetch
        .expect("known id", move |value| *value == id)
        .capture(&capture)
        .returns_sequence([None, Some("ready".into())]);
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    with_mocks(&[&fetch], || {
        let value = Eventually::new(Duration::from_secs(5), Duration::from_secs(1))
            .check(&clock, || fetch.call(id), Option::is_some)
            .unwrap();
        let validator = Validator::<String>::new()
            .rule_for("state", |v| v)
            .not_empty()
            .done();
        validator.validate(value.as_ref().unwrap()).unwrap();
        let mut report = CheckReport::default();
        report.equal("state", &value, &Some("ready".into()));
        report.assert();
        assert_count(&capture.values(), 2, |seen| *seen == id);
        Snapshots::new("snapshots")
            .inline("Some(\"ready\")", &format!("{value:?}"))
            .unwrap();
    });
    assert_eq!(clock.elapsed(), Duration::from_secs(1));
}
