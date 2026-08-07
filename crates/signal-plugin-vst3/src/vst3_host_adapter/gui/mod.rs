//! VST3 `IPlugView` hosting (g12.024, GUI phase 2).
//!
//! Embedded (parented) editor support for the IN-PROCESS tier: the host
//! process owns a native window and hands its content view to the plugin
//! via `IPlugView::attached(parent, "NSView")` — the VST3 mirror of the
//! CLAP `clap.gui` session in `signal-plugin-clap`. The COM surface is
//! handwritten per the g11.031 vtable idiom (`FUnknown` prefix, base
//! methods before derived, `extern "C"` on macOS/Linux).
//!
//! All view methods are UI-THREAD functions per the VST3 threading model;
//! the embedding host must dispatch every call here onto the application
//! main thread (Tauri `run_on_main_thread`) — this module can only
//! document that contract, not enforce it.
//!
//! Plugin-initiated resizes arrive through the host's `IPlugFrame` object
//! (`resizeView`), which queues a [`Vst3GuiEvent`] for the owner to drain
//! and apply to its window (then grant via
//! [`Vst3GuiSession::accept_plugin_resize`], which calls `onSize` without
//! applying the host-only `checkSizeConstraint` step).

#![allow(unsafe_op_in_unsafe_fn)]

mod constants;
mod frame;
mod session;
#[cfg(test)]
mod tests;
mod types;
mod view;

pub(crate) use constants::{IPLUG_VIEW_IID, VIEW_TYPE_EDITOR};
pub use session::Vst3GuiSession;
pub use types::Vst3GuiEvent;
