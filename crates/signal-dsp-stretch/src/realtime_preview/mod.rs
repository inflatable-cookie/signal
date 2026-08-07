//! Callback-facing RealtimePreview tier.
//!
//! This tier has no consumer outside the crate and its callback path is not
//! render-plane usable: `process` is quantum-locked, so at any ratio other
//! than `1.0` it stalls analysis or drops source frames while returning `Ok`.
//! `g10.040` decides whether the tier is completed or closed; `g10.038`
//! deliberately left it intact and only moved it out of `lib.rs`.

mod callback;
mod contract;

pub use contract::{
    plan_realtime_preview_stream, project_realtime_preview_fixed_ratio_source_advance,
    RealtimePreviewCallbackProcessError, RealtimePreviewCallbackProcessReport,
    RealtimePreviewCallbackState, RealtimePreviewCallbackTimelineMode,
    RealtimePreviewDynamicSourceProjectionReport, RealtimePreviewIntegrationMode,
    RealtimePreviewPlanError, RealtimePreviewSourceProjectionReport, RealtimePreviewStreamConfig,
    RealtimePreviewStreamingContract, RealtimePreviewUnsupportedMode,
};

#[cfg(test)]
mod tests;
