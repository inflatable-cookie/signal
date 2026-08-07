//! Real compiled VST3 fixture for tests: source generator + rustc harness
//! (the VST3 mirror of `signal_plugin_clap::fixture`).
//!
//! The fixture is an actual VST3 bundle: a rustc-compiled cdylib laid out at
//! the platform module path (`Contents/MacOS/<name>` on macOS,
//! `Contents/<arch>-linux/<name>.so` on Linux) plus `Contents/Info.plist`
//! and `Contents/Resources/moduleinfo.json`, so discovery and hosting tests
//! exercise the genuine bundle-resolution/dlopen/COM path. The module
//! exports `bundleEntry`/`ModuleEntry`/`InitDll` (+ exits) and
//! `GetPluginFactory`; the single class is a single-component effect
//! implementing `IComponent`, `IAudioProcessor`, and `IEditController`
//! facets on one static object. Its `process()` is real: a fixed-gain
//! effect (output = input × [`VST3_FIXTURE_GAIN`]) with two controller
//! parameters (Gain id 4096 default 0.5, Bypass id 0) matching the CLAP
//! fixture's inventory shape.
//!
//! Shared across crates (the sandbox broker's integration tests compile the
//! same fixture), hence public but hidden from the crate's documented API.

mod compile;
mod source;

pub use compile::{
    compile_vst3_fixture, compile_vst3_fixture_with_default_bus_channels, rustc_available,
};
pub use source::vst3_fixture_source;

/// Linear gain the fixture's `process()` applies until a param write lands
/// (the Gain param's default; g12.023 makes the param live via the block's
/// input `IParameterChanges`).
pub const VST3_FIXTURE_GAIN: f32 = 0.5;

/// Param id of the fixture's live Gain parameter (normalized == plain).
pub const VST3_FIXTURE_GAIN_PARAM_ID: u32 = 4096;

/// MIDI controller number the fixture's `IMidiMapping` assigns to the Gain
/// parameter (bus 0, channel 0) — the CC → param delivery proof.
pub const VST3_FIXTURE_GAIN_CC: u8 = 7;

/// Initial editor content size the fixture's `IPlugView::getSize` reports.
pub const VST3_FIXTURE_VIEW_INITIAL_SIZE: (u32, u32) = (400, 300);

/// The resize the fixture's view requests from the host `IPlugFrame` on
/// `attached` (exercises the resizeView callback path without any real
/// window system).
pub const VST3_FIXTURE_VIEW_REQUESTED_SIZE: (u32, u32) = (500, 320);

/// Canonical component-class ID hex of the fixture (the catalog load key on
/// non-Windows platforms; hosting's hex decoder applies the COM swap on
/// Windows). Must stay in sync with the four UID fields in the generated
/// source below.
pub const VST3_FIXTURE_CLASS_ID_HEX: &str = "51F1C7A15E0C4B3D9A2F41D67B3C55E2";
