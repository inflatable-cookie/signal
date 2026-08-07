//! Abstract time-stretcher contract.

use signal_primitives::Sample;

use crate::stretch_engine::StretchRenderError;

use super::types::StretchQuality;

/// Abstract time-stretcher contract (memo 013): stretch audio in time while
/// preserving pitch. `ratio` is the OUTPUT/INPUT duration factor — 2.0 makes
/// the audio twice as long (half speed), 0.5 twice as fast.
///
/// v1 scope is offline/control-side whole-buffer processing; the direct
/// streaming/RT surface (bounded latency, PDC reporting, variable ratio
/// mid-stream) extends this trait when a production callback-safe backend
/// lands.
pub trait TimeStretcher {
    /// Quality tier this backend provides — consumers must be able to make
    /// an honest offline/RT routing decision from this.
    fn quality(&self) -> StretchQuality;

    /// Current output/input duration ratio.
    fn ratio(&self) -> f64;

    /// Set the output/input duration ratio. Non-finite or non-positive
    /// values are clamped to 1.0 (identity).
    fn set_ratio(&mut self, ratio: f64);

    /// Stretch one mono buffer offline. Output length contract:
    /// `round(input.len() as f64 * ratio)` frames (identity ratio returns the
    /// input verbatim).
    ///
    /// Renders larger than [`MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES`] are refused
    /// rather than attempted.
    fn stretch_mono(&mut self, input: &[Sample]) -> Result<Vec<Sample>, StretchRenderError>;
}
