//! Audio-thread LV2 process session.

use std::ffi::c_void;
use std::sync::Arc;

use signal_plugin::{PluginParamChange, PluginParamChangeQueue};

use super::support::*;

// ── Raw process session (audio thread) ──────────────────────────────────────

/// Raw, movable process handle for one activated LV2 instance: the plugin
/// handle, its `run` entry point, and raw pointers into the planar buffers
/// the audio ports were connected to at activate. The sandbox moves this
/// onto its audio thread; the owning `Lv2HostedInstance` must outlive it
/// and must not run lifecycle transitions while the session is live.
/// Ports stay connected from activation, so a block is exactly: copy in,
/// `run(n)`, copy out — alloc-free.
#[derive(Debug)]
pub struct Lv2ProcessSession {
    pub(crate) handle: *mut c_void,
    pub(crate) run: unsafe extern "C" fn(*mut c_void, u32),
    pub(crate) input_left: *mut f32,
    pub(crate) input_right: *mut f32,
    pub(crate) output_left: *mut f32,
    pub(crate) output_right: *mut f32,
    pub(crate) max_frames: usize,
    pub(crate) processing: bool,
    /// Pending param writes shared with the owning instance (g12.023).
    pub(crate) param_changes: Arc<PluginParamChangeQueue>,
    /// Drain scratch (preallocated; audio thread never allocates).
    pub(crate) param_scratch: Vec<PluginParamChange>,
    /// `(port_index, slot)` for every control INPUT port; the slots live
    /// in the owning instance until after the session stops.
    pub(crate) control_inputs: Vec<(u32, *mut f32)>,
}

// Safety: the session is handed to exactly one audio thread; LV2's `run`
// is the audio-class function, and the owner serializes lifecycle against
// the session per the type contract above (the buffers behind the raw
// pointers live in the owning instance until after the session stops).
unsafe impl Send for Lv2ProcessSession {}

impl Lv2ProcessSession {
    /// Mark the session processing. LV2 has no start-processing handshake
    /// (push model); this exists for surface parity with the other
    /// formats' sessions and always succeeds.
    pub fn start(&mut self) -> Result<(), Lv2HostingError> {
        self.processing = true;
        Ok(())
    }

    /// Mark the session stopped (no LV2 call to make — push model).
    pub fn stop(&mut self) {
        self.processing = false;
    }

    /// Whether `start()` has run and `stop()` has not yet.
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Drain pending param writes into the connected control-input slots
    /// (g12.023, block-boundary application). Audio thread only —
    /// alloc-free; the slot pointers stay valid per the session contract.
    fn apply_param_changes(&mut self) {
        if self.param_changes.is_empty() {
            return;
        }
        self.param_changes.drain_coalesced(&mut self.param_scratch);
        for change in &self.param_scratch {
            if let Some((_, slot)) = self
                .control_inputs
                .iter()
                .find(|(index, _)| *index == change.parameter_id)
            {
                // Safety: control slots are instance-owned boxes that
                // outlive the session; only this thread writes them while
                // processing runs.
                unsafe { **slot = change.value as f32 };
            }
        }
    }

    /// Run one block through the connected planar buffers.
    ///
    /// # Safety
    /// `frames` must be within the activated max block size (callers
    /// clamp) and the owning instance must still be active.
    unsafe fn run_block(&mut self, frames: usize) {
        unsafe { (self.run)(self.handle, frames as u32) };
    }

    /// Process one block: interleaved stereo in, interleaved stereo out.
    /// Alloc-free (ports stay connected to the activate-time buffers).
    /// Returns `false` only when the handle is dead (input passes through
    /// unchanged) — LV2 `run` itself cannot report failure.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        let frames = frame_count
            .min(self.max_frames)
            .min(input.len() / 2)
            .min(output.len() / 2);
        if self.handle.is_null() {
            output[..frames * 2].copy_from_slice(&input[..frames * 2]);
            return false;
        }
        self.apply_param_changes();
        // Safety: pointers target the instance-owned boxed buffers sized
        // at max_frames; frames is clamped above.
        unsafe {
            for frame in 0..frames {
                *self.input_left.add(frame) = input[frame * 2];
                *self.input_right.add(frame) = input[frame * 2 + 1];
            }
            self.run_block(frames);
            for frame in 0..frames {
                output[frame * 2] = *self.output_left.add(frame);
                output[frame * 2 + 1] = *self.output_right.add(frame);
            }
        }
        true
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY
    /// on success; on a dead handle the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        let frames = frame_count.min(self.max_frames).min(io.len() / 2);
        if self.handle.is_null() {
            return false;
        }
        self.apply_param_changes();
        // Safety: as in `process_interleaved_stereo`.
        unsafe {
            for frame in 0..frames {
                *self.input_left.add(frame) = io[frame * 2];
                *self.input_right.add(frame) = io[frame * 2 + 1];
            }
            self.run_block(frames);
            for frame in 0..frames {
                io[frame * 2] = *self.output_left.add(frame);
                io[frame * 2 + 1] = *self.output_right.add(frame);
            }
        }
        true
    }
}
