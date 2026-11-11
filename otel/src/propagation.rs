//! W3C TraceContext (`traceparent`, `tracestate`) and W3C Baggage (`baggage`) propagation.

use opentelemetry::{
    Context, KeyValue,
    baggage::{Baggage, BaggageExt},
    global,
    propagation::{Extractor, Injector, TextMapCompositePropagator},
    trace::{FutureExt, TraceContextExt},
};
use opentelemetry_sdk::propagation::{BaggagePropagator, TraceContextPropagator};
use std::{collections::HashMap, future::Future, sync::OnceLock};

static PROPAGATOR_INSTALLED: OnceLock<()> = OnceLock::new();

/// Install the composite W3C TraceContext + Baggage propagator globally.
pub fn install_w3c_propagator() {
    let propagator = TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]);
    global::set_text_map_propagator(propagator);
    let _ = PROPAGATOR_INSTALLED.set(());
}

fn ensure_propagator() {
    PROPAGATOR_INSTALLED.get_or_init(install_w3c_propagator);
}

struct CaseInsensitiveMapExtractor<'a>(&'a HashMap<String, String>);

impl Extractor for CaseInsensitiveMapExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        if let Some(val) = self.0.get(key) {
            return Some(val.as_str());
        }
        self.0
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect()
    }
}

/// Inject the current [`Context`] (`traceparent`, `tracestate`, `baggage`) into a mutable `HashMap`.
pub fn inject_map(headers: &mut HashMap<String, String>) {
    Context::map_current(|cx| inject_context_map(cx, headers));
}

/// Inject an explicit [`Context`] into a mutable `HashMap`.
pub fn inject_context_map(cx: &Context, headers: &mut HashMap<String, String>) {
    inject_with(cx, headers);
}

/// Inject an explicit [`Context`] into any [`Injector`] carrier.
pub fn inject_with(cx: &Context, injector: &mut impl Injector) {
    ensure_propagator();
    global::get_text_map_propagator(|p| p.inject_context(cx, injector));
}

/// Extract a [`Context`] (remote `SpanContext` + `Baggage`) from HTTP/RPC headers (case-insensitive).
pub fn extract_map(headers: &HashMap<String, String>) -> Context {
    extract_with(&CaseInsensitiveMapExtractor(headers))
}

/// Extract a [`Context`] from any [`Extractor`] carrier.
pub fn extract_with(extractor: &impl Extractor) -> Context {
    ensure_propagator();
    global::get_text_map_propagator(|p| p.extract(extractor))
}

/// Run synchronous closure `f` with the [`Context`] extracted from `headers` attached.
pub fn with_extracted_headers<T>(headers: &HashMap<String, String>, f: impl FnOnce() -> T) -> T {
    let cx = extract_map(headers);
    let _guard = cx.attach();
    f()
}

/// Run `future` with the [`Context`] extracted from `headers` attached across `.await` points.
pub async fn with_extracted_headers_async<F: Future>(
    headers: &HashMap<String, String>,
    future: F,
) -> F::Output {
    let cx = extract_map(headers);
    future.with_context(cx).await
}

/// Return the 32-hex W3C `trace_id` of the currently active span, if valid.
pub fn current_trace_id() -> Option<String> {
    Context::map_current(|cx| {
        let sc = cx.span().span_context().clone();
        sc.is_valid().then(|| sc.trace_id().to_string())
    })
}

/// Return the 16-hex W3C `span_id` of the currently active span, if valid.
pub fn current_span_id() -> Option<String> {
    Context::map_current(|cx| {
        let sc = cx.span().span_context().clone();
        sc.is_valid().then(|| sc.span_id().to_string())
    })
}

/// Format the W3C `traceparent` (`00-<trace_id>-<span_id>-<flags>`) of the currently active span, if valid.
pub fn current_traceparent() -> Option<String> {
    Context::map_current(|cx| {
        let sc = cx.span().span_context().clone();
        sc.is_valid().then(|| {
            format!(
                "00-{}-{}-{:02x}",
                sc.trace_id(),
                sc.span_id(),
                sc.trace_flags().to_u8()
            )
        })
    })
}

fn merged_baggage_context(entries: &[KeyValue]) -> Context {
    Context::map_current(|cx| {
        let mut bag = Baggage::new();
        for (k, (v, meta)) in cx.baggage() {
            let _ = bag.insert_with_metadata(k.clone(), v.clone(), meta.clone());
        }
        for kv in entries {
            let _ = bag.insert(kv.key.clone(), kv.value.to_string());
        }
        cx.with_baggage(bag)
    })
}

/// Run synchronous closure `f` with additional W3C Baggage key-values merged into the active [`Context`].
pub fn with_baggage<T>(entries: &[KeyValue], f: impl FnOnce() -> T) -> T {
    let cx = merged_baggage_context(entries);
    let _guard = cx.attach();
    f()
}

/// Run `future` with additional W3C Baggage key-values merged into the active [`Context`].
pub async fn with_baggage_async<F: Future>(entries: &[KeyValue], future: F) -> F::Output {
    let cx = merged_baggage_context(entries);
    future.with_context(cx).await
}

/// Look up a W3C Baggage entry by key in the current [`Context`].
pub fn baggage_value(key: &str) -> Option<String> {
    Context::map_current(|cx| cx.baggage().get(key).map(|v| v.as_str().to_string()))
}
