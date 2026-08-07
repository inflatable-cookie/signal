//! In-child Audio Unit (AUv2) instance hosting: system-registry component
//! resolution, instance lifecycle (new/initialize/uninitialize/dispose),
//! parameter inventory via the AudioUnit property API, and a raw process
//! session for the sandbox audio thread — the AU mirror of
//! `signal-plugin-vst3`'s hosting module.
//!
//! # FFI design
//!
//! The AudioToolbox + CoreFoundation surface is handwritten C externs with
//! `#[link(name = "...", kind = "framework")]` on `cfg(target_os = "macos")`
//! blocks — no build script, no objc bridge, no SDK crates. Public types are
//! unconditional; only the FFI internals are platform-gated, and an
//! off-macOS `load` fails with the stable `unsupported_platform` token so
//! the sandbox/bridge/host layers stay cfg-free.
//!
//! # Pull-model rendering
//!
//! Unlike CLAP/VST3's push-model `process()`, an Audio Unit PULLS its input:
//! the host installs a render callback on the input scope and then asks the
//! unit to render via `AudioUnitRender`; mid-render the unit calls back into
//! the host to fetch the dry block. [`AuProcessSession`] stashes each
//! incoming block in preallocated planar buffers and serves it from an
//! `extern "C"` trampoline (boxed session state as `inRefCon`), handling
//! both callback buffer conventions: non-null `mData` (copy into the unit's
//! buffer) and null `mData` (point the unit at our stash). The render
//! timestamp advances monotonically (`mSampleTime` = session frame counter)
//! because time-based units (delays) misbehave on a static timestamp. Zero
//! allocation in the render path.

#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(target_os = "macos")]
pub(crate) mod ffi;

mod instance;
mod process;
mod types;

pub use instance::{AuHostedInstance, AuHostedPortLayout};
pub use process::AuProcessSession;
pub use types::{current_au_platform, AuHostingError, AU_REGISTRY_COMPONENT_PATH};
pub(crate) use types::{fourcc_from_str, fourcc_to_string};

#[cfg(test)]
mod tests;
