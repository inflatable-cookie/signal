//! RealtimePreview unit tests.

use super::{
    plan_realtime_preview_stream, project_realtime_preview_fixed_ratio_source_advance,
    RealtimePreviewCallbackProcessError, RealtimePreviewCallbackState,
    RealtimePreviewCallbackTimelineMode, RealtimePreviewIntegrationMode, RealtimePreviewPlanError,
    RealtimePreviewStreamConfig, RealtimePreviewUnsupportedMode,
};
use crate::benchmark::{
    compare_synthetic_realtime_preview_backends, generate_synthetic_stretch_audio,
    measure_dynamic_segment_seam_click, StretchBenchmarkBackend, StretchBenchmarkPath,
    StretchCorpusFamily, StretchMetric,
};
use crate::{
    RealtimePreviewStretcher, Sample, StretchQuality, StretchRatioPoint, TimeStretcher,
    REALTIME_PREVIEW_ANALYSIS_HOP, REALTIME_PREVIEW_WINDOW_SIZE,
};
use signal_primitives::SampleRate;

fn sine(frequency_hz: f32, sample_rate_hz: f32, len: usize) -> Vec<Sample> {
    (0..len)
        .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate_hz).sin())
        .collect()
}

fn rms(samples: &[Sample]) -> f32 {
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

/// Dominant frequency estimate by zero-crossing count over a trimmed
/// interior span (skips windup/tail edges).
fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
    let margin = samples.len() / 8;
    let interior = &samples[margin..samples.len() - margin];
    let crossings = interior
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count();
    crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
}

mod backend_comparison;
mod callback_state;
mod contract;
mod source_projection;
mod stretcher;
