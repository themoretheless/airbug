//! Method experiment: what a systematic drift does to a comparison.
//!
//! Both variants run the *same* workload; the machine gets slower over the session (thermal
//! drift, a background job). An honest method must not call that a regression. With the
//! crossover schedule the runner uses — baseline and candidate adjacent in time, order
//! alternating AB/BA — the drift cancels inside each pair. Measuring all baselines first and
//! all candidates afterwards turns the same drift into a "regression" almost every time.
//!
//! Deterministic: values are synthetic, so this needs no quiet machine.
use airbug_bench::{
    Availability, Case, Metric, Observation, Run, Status,
    analysis::{self, Decision},
};
use std::collections::BTreeMap;

const PAIRS: u32 = 12;
const SESSIONS: u64 = 200;
/// The machine ends the session this much slower than it started.
const DRIFT: f64 = 0.16;
const NOISE: f64 = 0.01;
const THRESHOLD_PERCENT: f64 = 5.0;
const ALPHA: f64 = 0.05;

/// xorshift64*: enough for ±1% measurement noise, and reproducible.
struct Noise(u64);
impl Noise {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let unit = (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64;
        1.0 + NOISE * (2.0 * unit - 1.0)
    }
}

/// `slots[t]` is what ran at position `t` of the session: (variant, pair).
fn session(seed: u64, slots: &[(&str, u32)]) -> Run {
    let mut noise = Noise(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut run = Run::new();
    run.id = format!("drift-{seed}");
    run.status = Status::Complete;
    run.cases = vec![Case {
        id: "work".into(),
        contract: BTreeMap::new(),
        metrics: vec![Metric::duration("wall", "operation", "median")],
    }];
    let last = (slots.len() - 1) as f64;
    for (position, (variant, pair)) in slots.iter().enumerate() {
        let slowdown = 1.0 + DRIFT * position as f64 / last;
        run.observations.push(Observation {
            case: "work".into(),
            metric: "wall".into(),
            variant: variant.to_string(),
            process: position as u32,
            pair: Some(*pair),
            sequence: position as u64 + 1,
            value: Some((1000.0 * slowdown * noise.next()).to_string()),
            operations: 1,
            worker_work_totals: Default::default(),
            work_totals: BTreeMap::new(),
            availability: Availability::Available,
        });
    }
    run
}

/// How many sessions were called a regression, and the largest estimated change, percent.
fn verdicts(slots: &[(&str, u32)]) -> (u64, f64) {
    let mut regressions = 0;
    let mut largest: f64 = 0.0;
    for seed in 0..SESSIONS {
        let run = session(seed, slots);
        let rows = analysis::compare(&run, None, THRESHOLD_PERCENT, ALPHA).unwrap();
        assert_eq!(rows.len(), 1);
        regressions += u64::from(rows[0].decision == Decision::Regression);
        largest = largest.max(rows[0].change_percent.unwrap_or(0.0).abs());
    }
    (regressions, largest)
}

#[test]
fn crossover_cancels_a_drift_that_a_blocked_schedule_reports_as_a_regression() {
    // AB, BA, AB, … — what `runner::run` schedules.
    let crossover: Vec<(&str, u32)> = (0..PAIRS)
        .flat_map(|pair| {
            if pair % 2 == 0 {
                [("baseline", pair), ("candidate", pair)]
            } else {
                [("candidate", pair), ("baseline", pair)]
            }
        })
        .collect();
    // Every baseline first, then every candidate.
    let blocked: Vec<(&str, u32)> = (0..PAIRS)
        .map(|pair| ("baseline", pair))
        .chain((0..PAIRS).map(|pair| ("candidate", pair)))
        .collect();

    let (crossover_regressions, crossover_change) = verdicts(&crossover);
    let (blocked_regressions, blocked_change) = verdicts(&blocked);
    println!(
        "drift {DRIFT}: crossover {crossover_regressions}/{SESSIONS} regressions, change up to \
         {crossover_change:.2}%; blocked {blocked_regressions}/{SESSIONS}, change up to {blocked_change:.2}%"
    );
    // Identical workloads: every regression verdict here is false.
    assert_eq!(crossover_regressions, 0);
    assert!(
        crossover_change < 1.5,
        "the drift should cancel inside a pair: {crossover_change}"
    );
    // Blocks see the second half of the session, ~8% slower than the first.
    assert!(
        blocked_regressions * 10 >= SESSIONS * 9,
        "{blocked_regressions}"
    );
    assert!(blocked_change > THRESHOLD_PERCENT, "{blocked_change}");
}
