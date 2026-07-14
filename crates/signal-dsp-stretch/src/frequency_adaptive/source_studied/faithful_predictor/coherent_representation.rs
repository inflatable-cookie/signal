use super::{
    analysis_interaction, review_with_grid_and_window, stage_trace, Direction, MechanismCounts,
    Review,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum CoherentRepresentationDirection {
    ExactInputRealSourceConfirmation,
    RepresentationResearch,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CoherentRepresentationReview {
    pub(in crate::frequency_adaptive) geometry: [usize; 4],
    pub(in crate::frequency_adaptive) structural_failures: [usize; 5],
    pub(in crate::frequency_adaptive) maximum_bass_error_hz: f64,
    pub(in crate::frequency_adaptive) octave_failures: usize,
    pub(in crate::frequency_adaptive) maximum_chord_peak_error_hz: f64,
    pub(in crate::frequency_adaptive) chord_input_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) maximum_event_error_frames: usize,
    pub(in crate::frequency_adaptive) replica_failures: usize,
    pub(in crate::frequency_adaptive) silence_peak: f64,
    pub(in crate::frequency_adaptive) mechanisms: MechanismCounts,
    pub(in crate::frequency_adaptive) output_hash: u64,
    pub(in crate::frequency_adaptive) source_relative_failures: [usize; 2],
    pub(in crate::frequency_adaptive) source_parity_hashes: [u64; 5],
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: CoherentRepresentationDirection,
}

pub(in crate::frequency_adaptive) fn review() -> CoherentRepresentationReview {
    let window = stage_trace::pinned_window();
    let full = review_with_grid_and_window(&window.analysis);
    let parity = analysis_interaction::review();
    let passed = full.direction == Direction::PinnedSourceParity
        && parity.direction
            == analysis_interaction::AnalysisInteractionDirection::SourceParityClosed;
    from_reviews(
        full,
        parity,
        if passed {
            CoherentRepresentationDirection::ExactInputRealSourceConfirmation
        } else {
            CoherentRepresentationDirection::RepresentationResearch
        },
    )
}

fn from_reviews(
    full: Review,
    parity: analysis_interaction::AnalysisInteractionReview,
    direction: CoherentRepresentationDirection,
) -> CoherentRepresentationReview {
    CoherentRepresentationReview {
        geometry: [960, 240, 1_024, 512],
        structural_failures: full.structural_failures,
        maximum_bass_error_hz: full.maximum_bass_error_hz,
        octave_failures: full.octave_failures,
        maximum_chord_peak_error_hz: full.maximum_chord_peak_error_hz,
        chord_input_out_of_band_db: full.chord_input_out_of_band_db,
        chord_out_of_band_db: full.chord_out_of_band_db,
        maximum_event_error_frames: full.maximum_event_error_frames,
        replica_failures: full.replica_failures,
        silence_peak: full.silence_peak,
        mechanisms: full.mechanisms,
        output_hash: full.output_hash,
        source_relative_failures: parity.source_relative_failures,
        source_parity_hashes: [
            parity.tones[0].hash,
            parity.tones[1].hash,
            parity.tones[2].hash,
            parity.tones[3].hash,
            parity.chord.hash,
        ],
        repeated: full.repeated && parity.repeated,
        direction,
    }
}
