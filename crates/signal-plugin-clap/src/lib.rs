//! CLAP plugin discovery for Signal.
//!
//! Performs real `clap-sys`/`libloading` FFI against CLAP shared libraries on
//! disk: factory enumeration by default, with explicitly opt-in in-process
//! capability probing. No hosting, lifecycle, or processing surfaces live
//! here — those belong to the future sandboxed hosting program.

#![warn(missing_docs)]
// The discovery module is dense FFI where nearly every line is an unsafe
// operation inside already-unsafe fns; per-operation unsafe blocks are
// deferred to the CLAP hosting rebuild rather than churned mechanically here.
#![allow(unsafe_op_in_unsafe_fn)]

mod adapter;
mod discovery;

pub use adapter::{ClapDiscoveredPluginType, ClapHostExtension, ClapPluginHostAdapter};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
