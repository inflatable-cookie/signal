mod ambiguity;
mod frame_segments;

use signal_dsp_spectral::Spectrogram;

use crate::{KeyDetectorConfig, LocalTonalTrackingSummary};

use ambiguity::summarize_changes_and_ambiguities;
use frame_segments::build_tonal_segments;

pub(crate) fn analyze_local_tonal_tracking(
    spectrogram: &Spectrogram,
    config: KeyDetectorConfig,
    reference_hz: f32,
) -> LocalTonalTrackingSummary {
    let segments = build_tonal_segments(spectrogram, config, reference_hz);
    summarize_changes_and_ambiguities(config, segments)
}
