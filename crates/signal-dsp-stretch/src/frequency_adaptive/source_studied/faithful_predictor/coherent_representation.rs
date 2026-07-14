use super::{
    analysis_interaction, hash_samples, modified_transform_length,
    render_stage_with_grid_and_window, review_with_grid_and_window, stage_trace, Direction,
    MechanismCounts, Render, Review, TraceStage, TransformGrid,
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
    pub(in crate::frequency_adaptive) window_hash: u64,
    pub(in crate::frequency_adaptive) pinned_window_maximum_delta: f64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: CoherentRepresentationDirection,
}

pub(in crate::frequency_adaptive) fn review() -> CoherentRepresentationReview {
    let pinned_window = stage_trace::pinned_window();
    let window = source_kaiser_window(960, 240);
    let pinned_window_maximum_delta = window
        .iter()
        .zip(&pinned_window.analysis)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0, f64::max);
    let full = review_with_grid_and_window(&pinned_window.analysis);
    let parity = analysis_interaction::review();
    let passed = full.direction == Direction::PinnedSourceParity
        && parity.direction
            == analysis_interaction::AnalysisInteractionDirection::SourceParityClosed;
    from_reviews(
        full,
        parity,
        hash_samples(&window),
        pinned_window_maximum_delta,
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
    window_hash: u64,
    pinned_window_maximum_delta: f64,
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
        window_hash,
        pinned_window_maximum_delta,
        repeated: full.repeated && parity.repeated,
        direction,
    }
}

pub(in crate::frequency_adaptive) fn source_geometry(sample_rate: usize) -> [usize; 4] {
    let block = (sample_rate as f64 * 0.12) as usize;
    let interval = ((sample_rate as f64 * 0.03) as usize).max(1);
    let transform = modified_transform_length(block);
    [block, interval, transform, transform / 2]
}

pub(super) fn render(input: &[f64], ratio: f64, sample_rate: usize) -> Render {
    let geometry = source_geometry(sample_rate);
    let window = source_kaiser_window(geometry[0], geometry[1]);
    render_stage_with_grid_and_window(
        input,
        ratio,
        sample_rate,
        TraceStage::Complete,
        TransformGrid::ModifiedHalfBin,
        &window,
    )
}

pub(super) fn source_kaiser_window(block_frames: usize, interval_frames: usize) -> Vec<f64> {
    let mut bandwidth = block_frames as f64 / interval_frames as f64;
    bandwidth += 8.0 / ((bandwidth + 3.0) * (bandwidth + 3.0));
    bandwidth += 0.25 * (3.0 - bandwidth).max(0.0);
    bandwidth = bandwidth.max(2.0);
    let beta = (bandwidth * bandwidth * 0.25 - 1.0).sqrt() * std::f64::consts::PI;
    let inverse_bessel = 1.0 / bessel_zero(beta);
    let mut window = (0..block_frames)
        .map(|index| {
            let radius = 2.0 * index as f64 / block_frames as f64 - 1.0;
            f64::from((bessel_zero(beta * (1.0 - radius * radius).sqrt()) * inverse_bessel) as f32)
        })
        .collect::<Vec<_>>();
    for residue in 0..interval_frames {
        let sum = (residue..block_frames)
            .step_by(interval_frames)
            .map(|index| {
                let value = window[index] as f32;
                f64::from(value * value)
            })
            .sum::<f64>();
        let scale = 1.0 / sum.sqrt();
        for index in (residue..block_frames).step_by(interval_frames) {
            window[index] = f64::from((window[index] * scale) as f32);
        }
    }
    window
}

fn bessel_zero(value: f64) -> f64 {
    let mut result = 0.0;
    let mut term = 1.0;
    let mut order = 0.0;
    while term > 1.0e-4 {
        result += term;
        order += 1.0;
        term *= value * value / (4.0 * order * order);
    }
    result
}
