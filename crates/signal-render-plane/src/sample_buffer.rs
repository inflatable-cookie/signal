//! Shared sample buffers and render-plane capacity constants.

use std::sync::Arc;

/// Largest callback quantum the plan's scratch buffers are sized for. Every
/// stage owns `MAX_BLOCK_FRAMES × channels` samples of scratch, preallocated
/// at compile; `render_block` clamps (and debug-asserts) the frame count.
pub const MAX_BLOCK_FRAMES: usize = 4096;

/// Fixed capacity of the shared per-stage meter table. Plans with more
/// stages than this render normally but stages past the capacity are
/// silently unmetered (slot i meters topological stage i; there is no
/// overflow signalling — meters are cosmetic).
pub const METER_SLOT_CAPACITY: usize = 256;

/// Callback-interval factor past which a missed deadline is inferred: an
/// interval since the previous callback longer than `1.5 ×` the block
/// duration at the plan rate counts as an xrun.
pub(crate) const XRUN_INTERVAL_FACTOR: f64 = 1.5;

/// Shared immutable sample data: interleaved f32 at a source rate. `channels`
/// gives the interleaving stride — 1 (mono), 2 (stereo), or more. When a clip's
/// source channel count differs from its stage's format, the render adapts it
/// with the standard up/down-mix coefficients (`signal_dsp::default_adapter_matrix`).
///
/// Equality is pointer-based so plan specs containing large buffers compare
/// cheaply and a cached buffer keeps reinstalls idempotent.
#[derive(Clone, Debug)]
pub struct RenderSampleBuffer {
    /// Source sample rate of the buffer.
    pub sample_rate_hz: u32,
    /// Interleaved channel count (1 = mono, 2 = stereo, …). `frames.len()` must
    /// be a multiple of this.
    pub channels: u16,
    /// Interleaved frames (`frame_count = frames.len() / channels`).
    pub frames: Arc<[f32]>,
}

impl PartialEq for RenderSampleBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.sample_rate_hz == other.sample_rate_hz
            && self.channels == other.channels
            && Arc::ptr_eq(&self.frames, &other.frames)
    }
}

impl RenderSampleBuffer {
    /// Build a sample buffer of the given interleaved channel count.
    pub fn new(sample_rate_hz: u32, channels: u16, frames: Arc<[f32]>) -> Self {
        Self {
            sample_rate_hz,
            channels: channels.max(1),
            frames,
        }
    }

    /// Build a mono sample buffer.
    pub fn mono(sample_rate_hz: u32, frames: Arc<[f32]>) -> Self {
        Self::new(sample_rate_hz, 1, frames)
    }

    /// Build a stereo (interleaved L/R) sample buffer.
    pub fn stereo(sample_rate_hz: u32, frames: Arc<[f32]>) -> Self {
        Self::new(sample_rate_hz, 2, frames)
    }

    /// Number of frames in the buffer (`frames.len() / channels`).
    pub fn frame_count(&self) -> usize {
        self.frames.len() / self.channels.max(1) as usize
    }
}
