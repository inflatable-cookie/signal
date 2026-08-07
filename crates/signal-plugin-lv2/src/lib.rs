//! LV2 plugin format adapter for Signal: real Turtle-manifest discovery
//! and dlopen-based hosting (g11.033).
//!
//! Discovery is pure file parsing over a handwritten Turtle subset
//! ([`turtle`]) — no lilv/serd/RDF dependencies and no plugin binary is
//! opened at scan time. Hosting ([`Lv2HostedInstance`]) re-parses the
//! bundle TTL at load (library path = the `.lv2` bundle directory, load
//! key = the bare plugin URI) and drives the plain LV2 C ABI: dlopen +
//! `lv2_descriptor(index)` walk, instantiate at activate, connected ports,
//! `run(n)` per block.

#![warn(missing_docs)]

#[doc(hidden)]
pub mod fixture;
mod lv2_host_adapter;

pub use lv2_host_adapter::*;

#[cfg(test)]
mod tests;
