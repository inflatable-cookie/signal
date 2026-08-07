//! Resumable offline stretch renderer.
//!
//! Frozen by `g10.039` Batch 39.2. The renderer carries phase, detector, and
//! overlap-add state across calls, so a source rendered in any number of chunks
//! produces bit-identical output to the same source rendered in one call.
//!
//! Frame scheduling is the reason that holds: analysis frames sit on a fixed
//! grid measured from the source origin, never from a chunk boundary. A chunk
//! edge changes only *when* a frame is computed, never *which* frames exist or
//! what they see.

mod engine;
mod pipeline;
mod pitch;
mod spectral;
mod types;

pub use engine::ResumableOfflineStretch;
pub use types::{
    ResumableRenderReport, ResumableStretchConfig, MAX_RESUMABLE_WINDOW_SIZE,
    MAX_RESUMABLE_WORKING_BYTES,
};
