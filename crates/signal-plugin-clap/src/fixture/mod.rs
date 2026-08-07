//! Real compiled CLAP fixture for tests: source generator + rustc harness.
//!
//! The fixture is an actual CLAP cdylib (entry, factory, descriptor, audio
//! ports, note ports, params) compiled with `rustc` at test time, so
//! discovery and hosting tests exercise the genuine dlopen/FFI path. Its
//! `process()` is real too: a fixed-gain effect (output = input ×
//! [`CLAP_FIXTURE_GAIN`]), which gives hosting round-trip tests an audible,
//! exactly-checkable transform.
//!
//! Shared across crates (the sandbox broker's integration tests compile the
//! same fixture), hence public but hidden from the crate's documented API.

mod compile;
mod source;

pub use compile::{
    compile_clap_fixture, compile_clap_instrument_fixture,
    compile_clap_multi_output_instrument_fixture, rustc_available,
};
pub use source::clap_fixture_source;
/// Linear gain the fixture's `process()` applies until a param write lands
/// (the Gain param's default; g12.023 makes the param live via
/// `CLAP_EVENT_PARAM_VALUE` in-events).
pub const CLAP_FIXTURE_GAIN: f32 = 0.5;

/// Param id of the fixture's live Gain parameter (plain range 0..1).
pub const CLAP_FIXTURE_GAIN_PARAM_ID: u32 = 4096;

/// Initial `clap.gui` content size the fixture reports from `get_size`.
pub const CLAP_FIXTURE_GUI_INITIAL_SIZE: (u32, u32) = (400, 300);

/// The resize the fixture's gui requests from the host on `show` (exercises
/// the host-callback path without any real window system).
pub const CLAP_FIXTURE_GUI_REQUESTED_SIZE: (u32, u32) = (500, 320);

/// The PLAIN Gain value the fixture's gui "tweaks" on `show`: pushed as a
/// `CLAP_EVENT_PARAM_VALUE` OUT-event at the top of the next processed
/// block (g12.024 plugin→host param sync proof; the Gain range is 0..1 so
/// plain == normalized).
pub const CLAP_FIXTURE_GUI_PARAM_OUT_VALUE: f64 = 0.75;
