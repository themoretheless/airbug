//! Cross-domain dashboard server, symbol store, streaming endpoints, and stores
//! for the airbug monorepo (`unit` / `bench` / `mon` / `otel` / `err`).

mod analytics;
pub mod app;
mod assets;
pub mod bench_launch;
mod benchcmp;
pub mod collector;
pub mod config;
pub mod error;
pub mod event_model;
pub mod http;
pub mod issues;
pub mod otlp;
mod rerun;
pub mod resources;
pub mod rundetail;
pub mod runs;
pub mod scan;
pub mod sentry;
pub mod serve;
pub mod store;
pub mod stream;
pub mod symbols;
pub mod test_runs;
