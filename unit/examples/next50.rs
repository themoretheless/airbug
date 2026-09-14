//! NEXT50 surface demo: Prop, SoftAssert, snapshots, Eventually, choose_weighted.
use airbug::checks::SoftAssert;
use airbug::fixture::FixtureContext;
use airbug::prop::Prop;
use airbug::snapshot::{Snapshots, UpdateMode};
use airbug::time::{Clock, Eventually, ManualClock};
use std::time::{Duration, SystemTime};

fn main() {
    Prop::new(7)
        .trials(8)
        .for_all::<u64, _>(|n| Ok(*n == *n))
        .expect("prop");

    let mut soft = SoftAssert::default();
    soft.equal("answer", &42u64, &42u64);
    soft.assert();

    let dir = std::env::temp_dir().join(format!(
        "airbug-next50-example-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let snaps = Snapshots::new(&dir).mode(UpdateMode::CreateMissing);
    snaps.check("demo", "ready").expect("snapshot");
    let obsolete = snaps.list_obsolete(&["demo"]).expect("obsolete");
    assert!(obsolete.is_empty());
    snaps.assert_no_obsolete(&["demo"]).expect("no obsolete");
    let _ = std::fs::remove_dir_all(&dir);

    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    let mut ticks = 0u8;
    let value = Eventually::new(Duration::from_secs(3), Duration::from_secs(1))
        .check(
            &clock,
            || {
                ticks += 1;
                ticks
            },
            |v| *v >= 2,
        )
        .expect("eventually");
    assert_eq!(value, 2);
    assert_eq!(clock.elapsed(), Duration::from_secs(1));

    let mut ctx = FixtureContext::with_seed(9);
    let values = ["a", "b", "c"];
    let picked = ctx.choose_weighted(&values, &[0, 10, 0]).expect("weighted");
    assert_eq!(*picked, "b");
}
