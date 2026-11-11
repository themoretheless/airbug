//! Full OpenTelemetry Metrics instruments (all 7 sync & observable kinds) with instrument caching.

use opentelemetry::{
    KeyValue, global,
    metrics::{
        AsyncInstrument, Counter, Gauge, Histogram, Meter, ObservableCounter, ObservableGauge,
        ObservableUpDownCounter, UpDownCounter,
    },
};
use std::{
    collections::HashMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

static METER_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Hash, PartialEq, Eq)]
struct InstrumentKey {
    generation: u64,
    scope: &'static str,
    name: &'static str,
    unit: &'static str,
    description: &'static str,
}

#[derive(Default)]
struct InstrumentCache {
    u64_counters: HashMap<InstrumentKey, Counter<u64>>,
    f64_counters: HashMap<InstrumentKey, Counter<f64>>,
    i64_up_down: HashMap<InstrumentKey, UpDownCounter<i64>>,
    f64_up_down: HashMap<InstrumentKey, UpDownCounter<f64>>,
    f64_histograms: HashMap<InstrumentKey, Histogram<f64>>,
    u64_histograms: HashMap<InstrumentKey, Histogram<u64>>,
}

static CACHE: OnceLock<Mutex<InstrumentCache>> = OnceLock::new();

fn cache() -> &'static Mutex<InstrumentCache> {
    CACHE.get_or_init(|| Mutex::new(InstrumentCache::default()))
}

/// Invalidate cached metric instruments when a new `MeterProvider` is installed or shut down.
pub(crate) fn invalidate_instrument_cache() {
    METER_GENERATION.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut guard) = cache().lock() {
        *guard = InstrumentCache::default();
    }
}

/// Global meter for `scope` (after [`crate::init`]).
pub fn meter(scope: &'static str) -> Meter {
    global::meter(scope)
}

/// Increment a cached `u64` counter on the global meter.
pub fn add_counter(scope: &'static str, name: &'static str, value: u64, attrs: &[KeyValue]) {
    let key = InstrumentKey {
        generation: METER_GENERATION.load(Ordering::Relaxed),
        scope,
        name,
        unit: "",
        description: "",
    };
    let counter = if let Ok(mut guard) = cache().lock() {
        guard
            .u64_counters
            .entry(key)
            .or_insert_with(|| meter(scope).u64_counter(name).build())
            .clone()
    } else {
        meter(scope).u64_counter(name).build()
    };
    counter.add(value, attrs);
}

/// Increment a cached `f64` counter on the global meter.
pub fn add_f64_counter(scope: &'static str, name: &'static str, value: f64, attrs: &[KeyValue]) {
    let key = InstrumentKey {
        generation: METER_GENERATION.load(Ordering::Relaxed),
        scope,
        name,
        unit: "",
        description: "",
    };
    let counter = if let Ok(mut guard) = cache().lock() {
        guard
            .f64_counters
            .entry(key)
            .or_insert_with(|| meter(scope).f64_counter(name).build())
            .clone()
    } else {
        meter(scope).f64_counter(name).build()
    };
    counter.add(value, attrs);
}

/// Add a signed delta to a cached `i64` up-down counter on the global meter.
pub fn add_up_down_counter(
    scope: &'static str,
    name: &'static str,
    value: i64,
    attrs: &[KeyValue],
) {
    let key = InstrumentKey {
        generation: METER_GENERATION.load(Ordering::Relaxed),
        scope,
        name,
        unit: "",
        description: "",
    };
    let counter = if let Ok(mut guard) = cache().lock() {
        guard
            .i64_up_down
            .entry(key)
            .or_insert_with(|| meter(scope).i64_up_down_counter(name).build())
            .clone()
    } else {
        meter(scope).i64_up_down_counter(name).build()
    };
    counter.add(value, attrs);
}

/// Add a signed delta to a cached `f64` up-down counter on the global meter.
pub fn add_f64_up_down_counter(
    scope: &'static str,
    name: &'static str,
    value: f64,
    attrs: &[KeyValue],
) {
    let key = InstrumentKey {
        generation: METER_GENERATION.load(Ordering::Relaxed),
        scope,
        name,
        unit: "",
        description: "",
    };
    let counter = if let Ok(mut guard) = cache().lock() {
        guard
            .f64_up_down
            .entry(key)
            .or_insert_with(|| meter(scope).f64_up_down_counter(name).build())
            .clone()
    } else {
        meter(scope).f64_up_down_counter(name).build()
    };
    counter.add(value, attrs);
}

/// Record a `f64` histogram observation on a cached instrument on the global meter.
pub fn record_histogram(scope: &'static str, name: &'static str, value: f64, attrs: &[KeyValue]) {
    let key = InstrumentKey {
        generation: METER_GENERATION.load(Ordering::Relaxed),
        scope,
        name,
        unit: "",
        description: "",
    };
    let histogram = if let Ok(mut guard) = cache().lock() {
        guard
            .f64_histograms
            .entry(key)
            .or_insert_with(|| meter(scope).f64_histogram(name).build())
            .clone()
    } else {
        meter(scope).f64_histogram(name).build()
    };
    histogram.record(value, attrs);
}

/// Record a `u64` histogram observation on a cached instrument on the global meter.
pub fn record_u64_histogram(
    scope: &'static str,
    name: &'static str,
    value: u64,
    attrs: &[KeyValue],
) {
    let key = InstrumentKey {
        generation: METER_GENERATION.load(Ordering::Relaxed),
        scope,
        name,
        unit: "",
        description: "",
    };
    let histogram = if let Ok(mut guard) = cache().lock() {
        guard
            .u64_histograms
            .entry(key)
            .or_insert_with(|| meter(scope).u64_histogram(name).build())
            .clone()
    } else {
        meter(scope).u64_histogram(name).build()
    };
    histogram.record(value, attrs);
}

/// Monotonic `u64` counter wrapper.
#[derive(Clone, Debug)]
pub struct U64Counter {
    inner: Counter<u64>,
}

impl U64Counter {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).u64_counter(name).build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.u64_counter(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        Self { inner: b.build() }
    }

    pub fn add(&self, value: u64, attrs: &[KeyValue]) {
        self.inner.add(value, attrs);
    }
}

/// Monotonic `f64` counter wrapper.
#[derive(Clone, Debug)]
pub struct F64Counter {
    inner: Counter<f64>,
}

impl F64Counter {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).f64_counter(name).build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.f64_counter(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        Self { inner: b.build() }
    }

    pub fn add(&self, value: f64, attrs: &[KeyValue]) {
        self.inner.add(value, attrs);
    }
}

/// Signed `i64` up-down counter wrapper (e.g. active connections, in-flight requests).
#[derive(Clone, Debug)]
pub struct I64UpDownCounter {
    inner: UpDownCounter<i64>,
}

impl I64UpDownCounter {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).i64_up_down_counter(name).build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.i64_up_down_counter(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        Self { inner: b.build() }
    }

    pub fn add(&self, value: i64, attrs: &[KeyValue]) {
        self.inner.add(value, attrs);
    }
}

/// Signed `f64` up-down counter wrapper.
#[derive(Clone, Debug)]
pub struct F64UpDownCounter {
    inner: UpDownCounter<f64>,
}

impl F64UpDownCounter {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).f64_up_down_counter(name).build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.f64_up_down_counter(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        Self { inner: b.build() }
    }

    pub fn add(&self, value: f64, attrs: &[KeyValue]) {
        self.inner.add(value, attrs);
    }
}

/// `f64` histogram wrapper with optional explicit bucket boundaries, unit, and description.
#[derive(Clone, Debug)]
pub struct F64Histogram {
    inner: Histogram<f64>,
}

impl F64Histogram {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).f64_histogram(name).build(),
        }
    }

    pub fn with_boundaries(scope: &'static str, name: &'static str, boundaries: Vec<f64>) -> Self {
        Self {
            inner: meter(scope)
                .f64_histogram(name)
                .with_boundaries(boundaries)
                .build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
        boundaries: Option<Vec<f64>>,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.f64_histogram(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        if let Some(bounds) = boundaries {
            b = b.with_boundaries(bounds);
        }
        Self { inner: b.build() }
    }

    pub fn record(&self, value: f64, attrs: &[KeyValue]) {
        self.inner.record(value, attrs);
    }
}

/// `u64` histogram wrapper with optional explicit bucket boundaries, unit, and description.
#[derive(Clone, Debug)]
pub struct U64Histogram {
    inner: Histogram<u64>,
}

impl U64Histogram {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).u64_histogram(name).build(),
        }
    }

    pub fn with_boundaries(scope: &'static str, name: &'static str, boundaries: Vec<f64>) -> Self {
        Self {
            inner: meter(scope)
                .u64_histogram(name)
                .with_boundaries(boundaries)
                .build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
        boundaries: Option<Vec<f64>>,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.u64_histogram(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        if let Some(bounds) = boundaries {
            b = b.with_boundaries(bounds);
        }
        Self { inner: b.build() }
    }

    pub fn record(&self, value: u64, attrs: &[KeyValue]) {
        self.inner.record(value, attrs);
    }
}

/// Opaque `f64` gauge that hides the OpenTelemetry type from callers.
#[derive(Clone, Debug)]
pub struct F64Gauge {
    inner: Gauge<f64>,
}

impl F64Gauge {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).f64_gauge(name).build(),
        }
    }

    pub fn with_description(
        scope: &'static str,
        name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            inner: meter(scope)
                .f64_gauge(name)
                .with_description(description)
                .build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.f64_gauge(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        Self { inner: b.build() }
    }

    pub fn record(&self, value: f64, attrs: &[KeyValue]) {
        self.inner.record(value, attrs);
    }
}

/// Opaque `u64` gauge that hides the OpenTelemetry type from callers.
#[derive(Clone, Debug)]
pub struct U64Gauge {
    inner: Gauge<u64>,
}

impl U64Gauge {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).u64_gauge(name).build(),
        }
    }

    pub fn with_description(
        scope: &'static str,
        name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            inner: meter(scope)
                .u64_gauge(name)
                .with_description(description)
                .build(),
        }
    }

    pub fn with_metadata(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
    ) -> Self {
        let m = meter(scope);
        let mut b = m.u64_gauge(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        Self { inner: b.build() }
    }

    pub fn record(&self, value: u64, attrs: &[KeyValue]) {
        self.inner.record(value, attrs);
    }
}

/// Opaque `i64` gauge wrapper.
#[derive(Clone, Debug)]
pub struct I64Gauge {
    inner: Gauge<i64>,
}

impl I64Gauge {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).i64_gauge(name).build(),
        }
    }

    pub fn with_description(
        scope: &'static str,
        name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            inner: meter(scope)
                .i64_gauge(name)
                .with_description(description)
                .build(),
        }
    }

    pub fn record(&self, value: i64, attrs: &[KeyValue]) {
        self.inner.record(value, attrs);
    }
}

/// Ergonomic observer passed into [`ObservableF64Gauge`], [`ObservableU64Counter`], and [`ObservableI64UpDownCounter`] callbacks.
pub struct MetricObserver<'a, T>(&'a dyn AsyncInstrument<T>);

impl<T> MetricObserver<'_, T> {
    /// Record an asynchronous observation during metric collection.
    pub fn observe(&self, measurement: T, attrs: &[KeyValue]) {
        self.0.observe(measurement, attrs);
    }
}

/// Asynchronous `f64` observable gauge (keep this handle alive for the lifetime of the callback).
#[derive(Clone, Debug)]
pub struct ObservableF64Gauge {
    _inner: ObservableGauge<f64>,
}

impl ObservableF64Gauge {
    pub fn new<F>(scope: &'static str, name: &'static str, callback: F) -> Self
    where
        F: Fn(&MetricObserver<'_, f64>) + Send + Sync + 'static,
    {
        Self::with_metadata(scope, name, "", "", callback)
    }

    pub fn with_metadata<F>(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
        callback: F,
    ) -> Self
    where
        F: Fn(&MetricObserver<'_, f64>) + Send + Sync + 'static,
    {
        let m = meter(scope);
        let mut b = m.f64_observable_gauge(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        let inner = b
            .with_callback(move |inst| callback(&MetricObserver(inst)))
            .build();
        Self { _inner: inner }
    }
}

/// Asynchronous `u64` observable counter (keep this handle alive for the lifetime of the callback).
#[derive(Clone, Debug)]
pub struct ObservableU64Counter {
    _inner: ObservableCounter<u64>,
}

impl ObservableU64Counter {
    pub fn new<F>(scope: &'static str, name: &'static str, callback: F) -> Self
    where
        F: Fn(&MetricObserver<'_, u64>) + Send + Sync + 'static,
    {
        Self::with_metadata(scope, name, "", "", callback)
    }

    pub fn with_metadata<F>(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
        callback: F,
    ) -> Self
    where
        F: Fn(&MetricObserver<'_, u64>) + Send + Sync + 'static,
    {
        let m = meter(scope);
        let mut b = m.u64_observable_counter(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        let inner = b
            .with_callback(move |inst| callback(&MetricObserver(inst)))
            .build();
        Self { _inner: inner }
    }
}

/// Asynchronous `i64` observable up-down counter (keep this handle alive for the lifetime of the callback).
#[derive(Clone, Debug)]
pub struct ObservableI64UpDownCounter {
    _inner: ObservableUpDownCounter<i64>,
}

impl ObservableI64UpDownCounter {
    pub fn new<F>(scope: &'static str, name: &'static str, callback: F) -> Self
    where
        F: Fn(&MetricObserver<'_, i64>) + Send + Sync + 'static,
    {
        Self::with_metadata(scope, name, "", "", callback)
    }

    pub fn with_metadata<F>(
        scope: &'static str,
        name: &'static str,
        unit: &'static str,
        description: &'static str,
        callback: F,
    ) -> Self
    where
        F: Fn(&MetricObserver<'_, i64>) + Send + Sync + 'static,
    {
        let m = meter(scope);
        let mut b = m.i64_observable_up_down_counter(name);
        if !unit.is_empty() {
            b = b.with_unit(unit);
        }
        if !description.is_empty() {
            b = b.with_description(description);
        }
        let inner = b
            .with_callback(move |inst| callback(&MetricObserver(inst)))
            .build();
        Self { _inner: inner }
    }
}
