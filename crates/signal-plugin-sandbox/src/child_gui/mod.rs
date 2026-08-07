//! Child-owned plugin editor windows for the sandboxed tier (g13.027
//! Batch 1).
//!
//! macOS requires AppKit on the process main thread, so the sandbox child
//! runs a GUI SERVICE LOOP on its main thread while the stdio protocol
//! (`broker::SandboxBrokerProcess::serve`) moves to a dedicated control
//! thread (`main.rs`). The RT audio thread is untouched: it still
//! spin/yield-waits on the shared-memory request stamp and never touches
//! AppKit — the isolation invariant of the packet.
//!
//! The control thread marshals editor lifecycle calls onto the main thread
//! through [`ChildGuiHandle`] (blocking request/reply channels — the
//! control thread waits, so instance access never overlaps). AppKit is
//! initialized LAZILY on the first `open-editor`: a child that never opens
//! an editor behaves exactly as before (no window-server connection).
//!
//! Editor windows are CHILD-OWNED floating `NSWindow`s titled by instance
//! (no cross-process view parenting — the packet's authority decision).
//! The user closing a window emits a spontaneous `editor_closed` receipt
//! line with `reason=user_closed` through the shared writer; child death
//! implies every window dies with the process.
//!
//! The Objective-C surface is the house-style handwritten FFI (typed
//! `objc_msgSend` casts, no objc crate — see `signal-plugin-au`'s gui).

mod handle;
mod service;
mod types;

#[cfg(target_os = "macos")]
mod editor;
#[cfg(target_os = "macos")]
mod macos;

pub use handle::{channel, ChildGuiHandle};
pub use service::run_gui_service;
pub use types::{ChildEditorSpec, SharedLineWriter};
