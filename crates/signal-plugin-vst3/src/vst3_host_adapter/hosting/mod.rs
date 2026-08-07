//! In-child VST3 instance hosting: module/factory loading, instance
//! lifecycle (create/initialize/activate/setProcessing), parameter inventory
//! via `IEditController`, and a raw process session for the sandbox audio
//! thread — the VST3 mirror of `signal-plugin-clap`'s hosting module.
//!
//! # FFI design
//!
//! The COM surface is handwritten, extending the introspection module's
//! factory-enumeration FFI (no `vst3-sys`, no Steinberg SDK code). Each
//! interface is a `#[repr(C)]` vtable whose first three slots are the
//! `FUnknown` methods (`queryInterface`/`addRef`/`release`); base-interface
//! methods precede derived methods in declaration order. On macOS and Linux
//! VST3 uses the plain C calling convention (`extern "C"`); the historical
//! thiscall concern is Windows/x86-only and out of scope here.
//!
//! # TUID byte order
//!
//! Interface and class IDs are 16-byte TUIDs. Steinberg's `INLINE_UID`
//! stores the four canonical `u32` fields big-endian on non-Windows
//! platforms, but COM-compatible (first field and the two 16-bit halves of
//! the second byte-swapped little-endian) on Windows. [`tuid_from_uid`]
//! encodes that per-platform. Catalog load keys are the *raw in-memory*
//! TUID hex exactly as the introspection module reports `PClassInfo` CIDs
//! (and as conforming `moduleinfo.json` files carry them on non-Windows), so
//! [`tuid_from_class_id_hex`] is a straight hex decode on macOS/Linux and
//! applies the COM swap only on Windows.

#![allow(unsafe_op_in_unsafe_fn)]
// The COM vtables mirror Steinberg's interface layouts, which are wider
// than clippy's default argument budget for a few methods.
#![allow(clippy::too_many_arguments)]

mod instance;
mod process;
mod wire;

use crate::vst3_host_adapter::Vst3HostPlatform;

/// Error surface for VST3 hosting operations; carries a stable snake_case
/// token suitable for broker receipt details (mirrors `ClapHostingError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3HostingError {
    /// Stable snake_case failure token (e.g. `module_open_failed`).
    pub token: String,
}

impl Vst3HostingError {
    pub(crate) fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for Vst3HostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for Vst3HostingError {}

/// The build-target VST3 platform (module layout + entry symbol names).
pub const fn current_vst3_platform() -> Vst3HostPlatform {
    if cfg!(target_os = "macos") {
        Vst3HostPlatform::MacOs
    } else if cfg!(target_os = "windows") {
        Vst3HostPlatform::Windows
    } else {
        Vst3HostPlatform::Linux
    }
}

pub use instance::{Vst3HostedInstance, Vst3HostedPortLayout};
pub use process::Vst3ProcessSession;
pub(crate) use wire::*;
pub use wire::{VST3_RESTART_IO_CHANGED, VST3_RESTART_LATENCY_CHANGED};
