use super::{
    chord_control, chord_control_frames, chord_spectrum_metrics, hash_samples, render_stage,
    TraceStage, SAMPLE_RATE,
};

const STAGES: [TraceStage; 6] = [
    TraceStage::Horizontal,
    TraceStage::ShortLower,
    TraceStage::ShortUpper,
    TraceStage::LongLower,
    TraceStage::LongUpper,
    TraceStage::Complete,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct StageEvidence {
    pub(in crate::frequency_adaptive) stage: TraceStage,
    pub(in crate::frequency_adaptive) out_of_band_db: f64,
    pub(in crate::frequency_adaptive) strongest_sideband_hz: f64,
    pub(in crate::frequency_adaptive) strongest_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) frame_grid_error_hz: f64,
    pub(in crate::frequency_adaptive) output_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AttributionReview {
    pub(in crate::frequency_adaptive) stages: [StageEvidence; 6],
    pub(in crate::frequency_adaptive) overlap_oracle_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) maximum_normalization_phase_delta: f64,
    pub(in crate::frequency_adaptive) significant_fallback: usize,
    pub(in crate::frequency_adaptive) earliest_failure: TraceStage,
    pub(in crate::frequency_adaptive) repeated: bool,
}

pub(in crate::frequency_adaptive) fn review() -> AttributionReview {
    let first = run();
    let second = run();
    AttributionReview {
        repeated: first == second,
        ..first
    }
}

fn run() -> AttributionReview {
    let input = chord_control();
    let frame_rate = SAMPLE_RATE as f64 / (SAMPLE_RATE as f64 * 0.03).round();
    let stages = STAGES.map(|stage| {
        let render = render_stage(&input, 2.0, SAMPLE_RATE, stage);
        let metrics = chord_spectrum_metrics(&render.samples[SAMPLE_RATE..SAMPLE_RATE * 3]);
        let multiple = (metrics.strongest_sideband_offset_hz / frame_rate).round();
        StageEvidence {
            stage,
            out_of_band_db: metrics.out_of_band_db,
            strongest_sideband_hz: metrics.strongest_sideband_hz,
            strongest_sideband_offset_hz: metrics.strongest_sideband_offset_hz,
            frame_grid_error_hz: (metrics.strongest_sideband_offset_hz - multiple * frame_rate)
                .abs(),
            output_hash: hash_samples(&render.samples),
        }
    });
    let earliest_failure = stages
        .iter()
        .find(|stage| stage.out_of_band_db > -60.0)
        .map(|stage| stage.stage)
        .unwrap_or(TraceStage::Complete);
    let ideal_output = chord_control_frames(SAMPLE_RATE * 4);
    let overlap_oracle = render_stage(&ideal_output, 1.0, SAMPLE_RATE, TraceStage::Analysis);
    let overlap_oracle_metrics =
        chord_spectrum_metrics(&overlap_oracle.samples[SAMPLE_RATE..SAMPLE_RATE * 3]);
    let complete = render_stage(&input, 2.0, SAMPLE_RATE, TraceStage::Complete);
    AttributionReview {
        stages,
        overlap_oracle_out_of_band_db: overlap_oracle_metrics.out_of_band_db,
        maximum_normalization_phase_delta: complete.maximum_normalization_phase_delta,
        significant_fallback: complete.significant_fallback,
        earliest_failure,
        repeated: false,
    }
}
