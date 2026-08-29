//! In-process VST3 plugin processor.

mod block;
mod editor;
mod processor;

pub use editor::InProcessVst3Editor;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
use std::sync::{Arc, Mutex};

use signal_plugin::{PluginEvent, PluginParameterDescriptor};
use signal_plugin_vst3::{Vst3HostedInstance, Vst3ProcessSession};

/// In-process VST3 processing backend (g11.031): the exact mirror of
/// `InProcessClapProcessor` over the VST3 COM hosting FFI.
///
/// Owns the hosted instance (module, component/processor/controller,
/// activation) for its whole lifetime. The process session sits behind a
/// `Mutex` taken with `try_lock` only — the audio thread never blocks; a
/// contended lock (teardown racing a callback) bypasses that block.
///
/// `setProcessing(true)` runs lazily on the first processed block, which is
/// the audio thread — matching VST3's processing-thread contract.
pub struct InProcessVst3Processor {
    /// Field order matters: the session must drop before the instance.
    session: Mutex<Vst3ProcessSession>,
    instance: Mutex<Vst3HostedInstance>,
    /// Preallocated conversion scratch for per-block note/CC delivery
    /// (taken with `try_lock` on the audio thread, like the session).
    events_scratch: Mutex<Vec<PluginEvent>>,
    /// Whether the controller exposed an `IMidiMapping` at load (CC events
    /// deliver as mapped parameter changes; without one they drop).
    midi_cc_mapping: bool,
    midi_cc_mappings: [bool; 128],
    pitch_bend_mapping: bool,
    channel_pressure_mapping: bool,
    parameters: Vec<PluginParameterDescriptor>,
    latency_frames: AtomicU32,
    latency_revision: AtomicU64,
    observed_latency_changes: AtomicU64,
    pub(crate) pending_restart_flags: Arc<AtomicU32>,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, plugin error, teardown race).
    misses: AtomicU64,
    unsupported_events: AtomicU64,
}
