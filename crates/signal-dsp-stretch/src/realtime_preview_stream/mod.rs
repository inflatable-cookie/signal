//! Source-owning RealtimePreview streaming kernel (`g10.040` Batch 40.3).
//!
//! Isolated candidate per Contract `084` Rule 2: nothing in the workspace
//! constructs this yet, and [`crate::realtime_preview`] is untouched.
//!
//! The difference from the shipped callback state is the whole point of the
//! lane. That one is quantum-locked — it takes `n` input frames and returns `n`
//! output frames whatever the ratio, so the analysis and synthesis cursors
//! diverge until a ring guard silently discards unanalysed source and returns
//! `Ok`. This one has no input parameter at all. The caller pushes source
//! frames ahead of time from a non-realtime thread, and [`RealtimePreviewStreamState::render`]
//! pulls however many source frames the active ratio demands.
//!
//! Frozen by `g10.040` Batch 40.2: ratio range, memory ceiling, one scheduler,
//! the underrun contract, and the latency report.

mod constants;
mod state_accessors;
mod state_io;
mod state_new;
mod state_render;
mod state_spectral;
mod types;

pub use constants::{
    REALTIME_PREVIEW_STREAM_MAX_RATIO, REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES,
    REALTIME_PREVIEW_STREAM_MIN_RATIO,
};
pub use types::{
    RealtimePreviewStreamError, RealtimePreviewStreamRenderReport, RealtimePreviewStreamState,
};
