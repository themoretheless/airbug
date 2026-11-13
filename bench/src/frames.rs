//! Frame lifecycle for application-owned render loops, and an adapter for GPU timestamps.
//!
//! The application keeps its event loop and its renderer; this only records where a frame's
//! boundaries were. Three boundaries, three metrics, never added together:
//!
//! | metric | from → to |
//! |---|---|
//! | `cpu.submit.ns` | [`Frames::begin`] → [`Frames::submitted`]: preparing and submitting commands |
//! | `frame.completed.ns` | `begin` → [`Frames::completed`]: until the application confirmed completion |
//! | `gpu.pass.ns` | GPU timestamps of one pass, reported later by [`Frames::gpu`] |
//!
//! plus `window.present.interval.ns` between successful [`Frames::presented`] calls and a count
//! of failed acquisitions. Nothing here needs a GPU: timestamps arrive as ticks with a
//! declared period, from wgpu's `Queue::get_timestamp_period`, a Metal counter, or a test.
//!
//! GPU results are asynchronous and may never arrive. A missing, stale, duplicated or
//! backwards result becomes an *unavailable* observation with its reason — never a zero and
//! never a wrapped-around duration.
use crate::{Availability, Case, Metric, Recorder, Result, Run, error};
use std::collections::BTreeMap;
use std::time::Instant;

pub const SUBMIT: &str = "cpu.submit.ns";
pub const COMPLETED: &str = "frame.completed.ns";
pub const GPU_PASS: &str = "gpu.pass.ns";
pub const PRESENT_INTERVAL: &str = "window.present.interval.ns";

/// How many frames a GPU result may lag before it is stale. A readback queue is bounded;
/// a result older than this belongs to a frame the application has long moved past.
pub const DEFAULT_BACKLOG: u64 = 8;

/// One resolved pair of GPU timestamp queries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuSpan {
    /// The frame the queries were written in.
    pub frame: u64,
    pub begin_ticks: u64,
    pub end_ticks: u64,
    /// Nanoseconds per tick, as the device declares it.
    pub period_ns: f64,
}

struct Open {
    begun: Instant,
    submitted: bool,
    completed: bool,
    gpu: bool,
}

/// Records frames of one scenario case.
pub struct Frames {
    recorder: Recorder,
    case: String,
    warmup: u64,
    backlog: u64,
    expect_gpu: bool,
    next: u64,
    open: BTreeMap<u64, Open>,
    last_present: Option<Instant>,
    presents: u64,
    failed_presents: u64,
}

impl Frames {
    /// `warmup` frames are driven like any other but leave no observations. With
    /// `expect_gpu`, every measured frame must end up with a GPU result or an explicit
    /// unavailable one; without it the GPU metric is declared unsupported once.
    pub fn new(
        case: &str,
        contract: BTreeMap<String, String>,
        warmup: u64,
        expect_gpu: bool,
    ) -> Result<Self> {
        let metric = |id: &str, scope: &str, statistic: &str, phase: &str| {
            let mut m = Metric::duration(id, scope, statistic);
            m.phase = phase.into();
            m
        };
        let mut contract = contract;
        contract.insert("frames.warmup".into(), warmup.to_string());
        contract.insert(
            "completion".into(),
            "application-confirmed frame completion".into(),
        );
        let mut recorder = Recorder::new();
        recorder.case(Case {
            id: case.into(),
            contract,
            metrics: vec![
                metric(
                    SUBMIT,
                    "calling thread; begin to submit return",
                    "individual frame",
                    "submit",
                ),
                metric(
                    COMPLETED,
                    "calling thread; begin to confirmed completion",
                    "individual frame",
                    "completed",
                ),
                metric(
                    GPU_PASS,
                    "GPU pass; difference of two timestamp queries",
                    "individual pass",
                    "gpu",
                ),
                metric(
                    PRESENT_INTERVAL,
                    "window; between successful present calls",
                    "individual interval",
                    "present",
                ),
            ],
        })?;
        Ok(Self {
            recorder,
            case: case.into(),
            warmup,
            backlog: DEFAULT_BACKLOG,
            expect_gpu,
            next: 0,
            open: BTreeMap::new(),
            last_present: None,
            presents: 0,
            failed_presents: 0,
        })
    }

    /// Frames a GPU result may lag before it is refused as stale.
    pub fn backlog(mut self, frames: u64) -> Self {
        self.backlog = frames.max(1);
        self
    }

    fn measured(&self, frame: u64) -> bool {
        frame >= self.warmup
    }

    /// Start a frame; the returned id names it in every later call.
    pub fn begin(&mut self) -> u64 {
        let frame = self.next;
        self.next += 1;
        self.open.insert(
            frame,
            Open {
                begun: Instant::now(),
                submitted: false,
                completed: false,
                gpu: false,
            },
        );
        frame
    }

    /// Commands for `frame` were handed to the device.
    pub fn submitted(&mut self, frame: u64) -> Result<()> {
        let now = Instant::now();
        let open = self
            .open
            .get_mut(&frame)
            .ok_or_else(|| error("submitted: unknown frame"))?;
        if std::mem::replace(&mut open.submitted, true) {
            return Err(error("frame submitted twice"));
        }
        let ns = now.duration_since(open.begun).as_nanos();
        if self.measured(frame) {
            self.recorder.observe(&self.case, SUBMIT, ns)?;
        }
        Ok(())
    }

    /// The application confirmed `frame` is done (a device poll returned, a fence signalled).
    pub fn completed(&mut self, frame: u64) -> Result<()> {
        let now = Instant::now();
        let open = self
            .open
            .get_mut(&frame)
            .ok_or_else(|| error("completed: unknown frame"))?;
        if !open.submitted {
            return Err(error("frame completed before it was submitted"));
        }
        if std::mem::replace(&mut open.completed, true) {
            return Err(error("frame completed twice"));
        }
        let ns = now.duration_since(open.begun).as_nanos();
        if self.measured(frame) {
            self.recorder.observe(&self.case, COMPLETED, ns)?;
        }
        self.retire();
        Ok(())
    }

    /// A present call returned: `true` if a surface texture was acquired and presented.
    pub fn presented(&mut self, frame: u64, success: bool) -> Result<()> {
        let now = Instant::now();
        if !success {
            self.failed_presents += 1;
            return Ok(());
        }
        self.presents += 1;
        if let Some(previous) = self.last_present.replace(now)
            && self.measured(frame)
        {
            self.recorder.observe(
                &self.case,
                PRESENT_INTERVAL,
                now.duration_since(previous).as_nanos(),
            )?;
        }
        Ok(())
    }

    /// A GPU timestamp pair was read back. Invalid results are recorded as unavailable.
    pub fn gpu(&mut self, span: GpuSpan) -> Result<()> {
        let refuse = |reason: String| Availability::Invalid(reason);
        let verdict = if span.frame >= self.next {
            Err(refuse(format!(
                "timestamp for frame {} that was never begun",
                span.frame
            )))
        } else if self.next - 1 - span.frame > self.backlog {
            Err(refuse(format!(
                "stale timestamp: frame {} read back {} frames late",
                span.frame,
                self.next - 1 - span.frame
            )))
        } else if !self.open.contains_key(&span.frame) {
            Err(refuse(format!(
                "timestamp for retired frame {}",
                span.frame
            )))
        } else if self.open[&span.frame].gpu {
            Err(refuse(format!(
                "duplicate timestamp for frame {}",
                span.frame
            )))
        } else if !(span.period_ns.is_finite() && span.period_ns > 0.0) {
            Err(refuse(format!(
                "timestamp period {} is not a positive number",
                span.period_ns
            )))
        } else if span.end_ticks < span.begin_ticks {
            // A counter that wrapped or two queries from different frames: no duration.
            Err(refuse(format!(
                "end tick {} precedes begin tick {}",
                span.end_ticks, span.begin_ticks
            )))
        } else {
            Ok(((span.end_ticks - span.begin_ticks) as f64 * span.period_ns).round() as u128)
        };
        if let Some(open) = self.open.get_mut(&span.frame) {
            open.gpu = true;
        }
        if !self.measured(span.frame) && span.frame < self.next {
            return Ok(());
        }
        match verdict {
            Ok(ns) => self.recorder.observe(&self.case, GPU_PASS, ns)?,
            Err(why) => self.recorder.unavailable(&self.case, GPU_PASS, why)?,
        }
        self.retire();
        Ok(())
    }

    /// Drop frames that are complete and can no longer receive a GPU result.
    fn retire(&mut self) {
        let newest = self.next.saturating_sub(1);
        let (backlog, expect_gpu, warmup) = (self.backlog, self.expect_gpu, self.warmup);
        let mut lost = Vec::new();
        self.open.retain(|frame, open| {
            let waiting = expect_gpu && !open.gpu;
            let expired = newest - *frame > backlog;
            if open.completed && (!waiting || expired) {
                if waiting && *frame >= warmup {
                    lost.push(*frame);
                }
                return false;
            }
            true
        });
        for frame in lost {
            let why = Availability::Incomplete(format!(
                "no GPU timestamp for frame {frame} within {backlog} frames"
            ));
            let _ = self.recorder.unavailable(&self.case, GPU_PASS, why);
        }
    }

    /// Close the series. Frames still waiting for a GPU result are recorded as incomplete.
    pub fn finish(mut self) -> Result<Run> {
        let waiting: Vec<u64> = self
            .open
            .iter()
            .filter(|(frame, open)| self.expect_gpu && !open.gpu && **frame >= self.warmup)
            .map(|(frame, _)| *frame)
            .collect();
        for frame in waiting {
            let why = Availability::Incomplete(format!(
                "no GPU timestamp for frame {frame} before the series ended"
            ));
            self.recorder.unavailable(&self.case, GPU_PASS, why)?;
        }
        if !self.expect_gpu {
            let why = Availability::Unsupported(
                "the scenario did not request GPU timestamp queries".into(),
            );
            self.recorder.unavailable(&self.case, GPU_PASS, why)?;
        }
        if self.presents == 0 {
            let why =
                Availability::NotApplicable("no successful present: offscreen or occluded".into());
            self.recorder
                .unavailable(&self.case, PRESENT_INTERVAL, why)?;
        }
        let unfinished = self.open.values().filter(|o| !o.completed).count();
        self.recorder.note(format!(
            "frames: {} begun, {} warm-up without observations, {} never completed; presents: {} succeeded, {} failed. \
             Frames are a dependent series: submit, completed and GPU times are separate boundaries and are not summed.",
            self.next, self.warmup.min(self.next), unfinished, self.presents, self.failed_presents
        ));
        self.recorder.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(run: &Run, metric: &str, available: bool) -> usize {
        run.observations
            .iter()
            .filter(|o| {
                o.metric == metric && (o.availability == Availability::Available) == available
            })
            .count()
    }

    fn drive(frames: &mut Frames) -> u64 {
        let frame = frames.begin();
        frames.submitted(frame).unwrap();
        frames.completed(frame).unwrap();
        frame
    }

    #[test]
    fn warmup_leaves_no_observations_and_boundaries_stay_ordered() {
        let mut frames = Frames::new("scene", BTreeMap::new(), 2, false).unwrap();
        for _ in 0..5 {
            let frame = drive(&mut frames);
            frames.presented(frame, true).unwrap();
        }
        let failed = frames.begin();
        assert!(
            frames.completed(failed).is_err(),
            "completion needs a submit"
        );
        frames.submitted(failed).unwrap();
        assert!(frames.submitted(failed).is_err(), "one submit per frame");
        frames.presented(failed, false).unwrap();
        let run = frames.finish().unwrap();
        assert_eq!(
            count(&run, SUBMIT, true),
            4,
            "frames 2..=5; two warm-up frames dropped"
        );
        assert_eq!(count(&run, COMPLETED, true), 3);
        assert_eq!(
            count(&run, PRESENT_INTERVAL, true),
            3,
            "intervals between presents of measured frames"
        );
        assert_eq!(count(&run, GPU_PASS, false), 1, "unsupported, said once");
        assert!(
            run.notes
                .iter()
                .any(|n| n.contains("1 never completed") && n.contains("1 failed")),
            "{:?}",
            run.notes
        );
        assert_eq!(run.cases[0].contract["frames.warmup"], "2");
    }

    #[test]
    fn gpu_ticks_scale_by_the_period_and_bad_results_never_become_durations() {
        let mut frames = Frames::new("scene", BTreeMap::new(), 0, true)
            .unwrap()
            .backlog(2);
        let span = |frame, begin, end| GpuSpan {
            frame,
            begin_ticks: begin,
            end_ticks: end,
            period_ns: 2.5,
        };
        let first = frames.begin();
        frames.submitted(first).unwrap();
        frames.gpu(span(first, 100, 500)).unwrap();
        frames.gpu(span(first, 100, 500)).unwrap(); // duplicate, while the frame is still open
        frames.completed(first).unwrap();
        frames.gpu(span(first, 100, 500)).unwrap(); // and again after it was retired
        let wrapped = drive(&mut frames);
        frames.gpu(span(wrapped, u64::MAX - 5, 10)).unwrap(); // counter wrapped
        frames.gpu(span(99, 0, 1)).unwrap(); // never begun
        let bad_period = drive(&mut frames);
        frames
            .gpu(GpuSpan {
                period_ns: 0.0,
                ..span(bad_period, 1, 2)
            })
            .unwrap();
        let lost = drive(&mut frames); // its result never arrives
        for _ in 0..4 {
            drive(&mut frames);
        }
        frames.gpu(span(lost, 1, 2)).unwrap(); // far too late: stale
        let run = frames.finish().unwrap();

        let good: Vec<&str> = run
            .observations
            .iter()
            .filter(|o| o.metric == GPU_PASS && o.availability == Availability::Available)
            .filter_map(|o| o.value.as_deref())
            .collect();
        assert_eq!(good, ["1000"], "400 ticks x 2.5 ns");
        let reasons: Vec<String> = run
            .observations
            .iter()
            .filter(|o| o.metric == GPU_PASS)
            .filter_map(|o| match &o.availability {
                Availability::Invalid(why) | Availability::Incomplete(why) => Some(why.clone()),
                _ => None,
            })
            .collect();
        for expected in [
            "duplicate timestamp",
            "precedes begin tick",
            "never begun",
            "not a positive number",
            "stale timestamp",
            "no GPU timestamp for frame",
        ] {
            assert!(
                reasons.iter().any(|r| r.contains(expected)),
                "{expected}: {reasons:?}"
            );
        }
        assert!(
            run.observations
                .iter()
                .all(|o| o.value.as_deref() != Some("0") || o.metric != GPU_PASS)
        );
    }
}
