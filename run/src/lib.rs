//! One run, one directory, one format — shared by whatever writes runs (`cargo airbug test`,
//! benches) and whatever reads them (the hub).
//!
//! [`model`] is the on-disk contract; [`store`] finds, reads, writes and prunes run
//! directories under `<root>/.airbug/runs/`.
pub mod hub;
pub mod model;
pub mod store;
