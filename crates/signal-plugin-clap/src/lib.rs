//! CLAP plugin discovery and child-side hosting for Signal.
//!
//! Performs real `clap-sys`/`libloading` FFI against CLAP shared libraries on
//! disk: factory enumeration by default, with explicitly opt-in in-process
//! capability probing. The hosting surfaces ([`ClapHostedInstance`]:
//! instance lifecycle + processing) are for the sandbox CHILD process only —
//! the parent host never loads plugin code in-process.

#![warn(missing_docs)]
// The discovery module is dense FFI where nearly every line is an unsafe
// operation inside already-unsafe fns; per-operation unsafe blocks are
// deferred to the CLAP hosting rebuild rather than churned mechanically here.
#![allow(unsafe_op_in_unsafe_fn)]

mod adapter;
mod discovery;
#[doc(hidden)]
pub mod fixture;
mod gui;
mod hosting;

pub use adapter::{ClapDiscoveredPluginType, ClapHostExtension, ClapPluginHostAdapter};
pub use gui::{ClapGuiEvent, ClapGuiSession};
pub use hosting::{
    ClapHostedInstance, ClapHostedPortLayout, ClapHostingError, ClapProcessSession, LoadedClapEntry,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
