use super::{
    chord_control, chord_control_frames, chord_spectrum_metrics, hash_samples, render_stage,
    spectrum_metrics, TraceStage, CHORD_FREQUENCIES, SAMPLE_RATE,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum MixtureDirection {
    ObservationGeometry,
    PredictorEquation,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ToneMixtureEvidence {
    pub(in crate::frequency_adaptive) frequency_hz: f64,
    pub(in crate::frequency_adaptive) isolated_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) isolated_ratio_variance: f64,
    pub(in crate::frequency_adaptive) mixed_ratio_variance: f64,
    pub(in crate::frequency_adaptive) isolated_output_phase_variance: f64,
    pub(in crate::frequency_adaptive) mixed_output_phase_variance: f64,
    pub(in crate::frequency_adaptive) isolated_maximum_ratio_error: f64,
    pub(in crate::frequency_adaptive) mixed_maximum_ratio_error: f64,
    pub(in crate::frequency_adaptive) isolated_strongest_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) isolated_frame_grid_error_hz: f64,
    pub(in crate::frequency_adaptive) isolated_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct MixtureAttributionReview {
    pub(in crate::frequency_adaptive) tones: [ToneMixtureEvidence; 4],
    pub(in crate::frequency_adaptive) mixed_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) mixed_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: MixtureDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum StateLineageDirection {
    DirectHorizontalRecurrence,
    VerticalStateFeedback,
    NoFailure,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ToneStateLineageEvidence {
    pub(in crate::frequency_adaptive) frequency_hz: f64,
    pub(in crate::frequency_adaptive) corrected_feedback_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) horizontal_phase_recurrence_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) corrected_feedback_phase_variance: f64,
    pub(in crate::frequency_adaptive) horizontal_phase_recurrence_variance: f64,
    pub(in crate::frequency_adaptive) corrected_feedback_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) horizontal_phase_recurrence_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) corrected_feedback_hash: u64,
    pub(in crate::frequency_adaptive) horizontal_phase_recurrence_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct StateLineageReview {
    pub(in crate::frequency_adaptive) tones: [ToneStateLineageEvidence; 4],
    pub(in crate::frequency_adaptive) mixed_corrected_feedback_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) mixed_horizontal_phase_recurrence_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) mixed_corrected_feedback_phase_variance: [f64; 4],
    pub(in crate::frequency_adaptive) mixed_horizontal_phase_recurrence_variance: [f64; 4],
    pub(in crate::frequency_adaptive) mixed_corrected_feedback_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) mixed_horizontal_phase_recurrence_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) mixed_corrected_feedback_hash: u64,
    pub(in crate::frequency_adaptive) mixed_horizontal_phase_recurrence_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: StateLineageDirection,
}

pub(in crate::frequency_adaptive) fn review() -> AttributionReview {
    let first = run();
    let second = run();
    AttributionReview {
        repeated: first == second,
        ..first
    }
}

pub(in crate::frequency_adaptive) fn mixture_review() -> MixtureAttributionReview {
    let first = run_mixture();
    let second = run_mixture();
    MixtureAttributionReview {
        repeated: first == second,
        ..first
    }
}

pub(in crate::frequency_adaptive) fn state_lineage_review() -> StateLineageReview {
    let first = run_state_lineage();
    let second = run_state_lineage();
    StateLineageReview {
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

fn run_mixture() -> MixtureAttributionReview {
    let mixed_input = chord_control();
    let mixed = render_stage(&mixed_input, 2.0, SAMPLE_RATE, TraceStage::Horizontal);
    let mixed_metrics = chord_spectrum_metrics(&mixed.samples[SAMPLE_RATE..SAMPLE_RATE * 3]);
    let tones = CHORD_FREQUENCIES.map(|frequency| {
        let tone_index = CHORD_FREQUENCIES
            .iter()
            .position(|candidate| *candidate == frequency)
            .expect("frozen chord frequency");
        let amplitude = 0.16 - tone_index as f64 * 0.015;
        let input = (0..SAMPLE_RATE * 2)
            .map(|index| {
                amplitude
                    * (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64).sin()
            })
            .collect::<Vec<_>>();
        let isolated = render_stage(&input, 2.0, SAMPLE_RATE, TraceStage::Horizontal);
        let metrics = spectrum_metrics(
            &isolated.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
            std::slice::from_ref(&frequency),
        );
        let isolated_errors = isolated
            .horizontal_ratio_errors
            .iter()
            .map(|errors| errors[tone_index])
            .collect::<Vec<_>>();
        let mixed_errors = mixed
            .horizontal_ratio_errors
            .iter()
            .map(|errors| errors[tone_index])
            .collect::<Vec<_>>();
        let isolated_output_errors =
            phase_advance_errors(&isolated.horizontal_output_phases, tone_index, frequency);
        let mixed_output_errors =
            phase_advance_errors(&mixed.horizontal_output_phases, tone_index, frequency);
        let frame_rate = SAMPLE_RATE as f64 / (SAMPLE_RATE as f64 * 0.03).round();
        let multiple = (metrics.strongest_sideband_offset_hz / frame_rate).round();
        ToneMixtureEvidence {
            frequency_hz: frequency,
            isolated_out_of_band_db: metrics.out_of_band_db,
            isolated_ratio_variance: variance(&isolated_errors),
            mixed_ratio_variance: variance(&mixed_errors),
            isolated_output_phase_variance: variance(&isolated_output_errors),
            mixed_output_phase_variance: variance(&mixed_output_errors),
            isolated_maximum_ratio_error: maximum_abs(&isolated_errors),
            mixed_maximum_ratio_error: maximum_abs(&mixed_errors),
            isolated_strongest_sideband_offset_hz: metrics.strongest_sideband_offset_hz,
            isolated_frame_grid_error_hz: (metrics.strongest_sideband_offset_hz
                - multiple * frame_rate)
                .abs(),
            isolated_hash: hash_samples(&isolated.samples),
        }
    });
    let isolated_clean = tones
        .iter()
        .all(|tone| tone.isolated_out_of_band_db <= -60.0);
    MixtureAttributionReview {
        tones,
        mixed_out_of_band_db: mixed_metrics.out_of_band_db,
        mixed_hash: hash_samples(&mixed.samples),
        repeated: false,
        direction: if isolated_clean && mixed_metrics.out_of_band_db > -60.0 {
            MixtureDirection::ObservationGeometry
        } else {
            MixtureDirection::PredictorEquation
        },
    }
}

fn run_state_lineage() -> StateLineageReview {
    let mixed_input = chord_control();
    let mixed_corrected = render_stage(&mixed_input, 2.0, SAMPLE_RATE, TraceStage::Horizontal);
    let mixed_horizontal = render_stage(
        &mixed_input,
        2.0,
        SAMPLE_RATE,
        TraceStage::HorizontalPhaseRecurrence,
    );
    let mixed_corrected_metrics =
        chord_spectrum_metrics(&mixed_corrected.samples[SAMPLE_RATE..SAMPLE_RATE * 3]);
    let mixed_horizontal_metrics =
        chord_spectrum_metrics(&mixed_horizontal.samples[SAMPLE_RATE..SAMPLE_RATE * 3]);
    let tones = CHORD_FREQUENCIES.map(|frequency| {
        let tone_index = CHORD_FREQUENCIES
            .iter()
            .position(|candidate| *candidate == frequency)
            .expect("frozen chord frequency");
        let amplitude = 0.16 - tone_index as f64 * 0.015;
        let input = (0..SAMPLE_RATE * 2)
            .map(|index| {
                amplitude
                    * (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64).sin()
            })
            .collect::<Vec<_>>();
        let corrected = render_stage(&input, 2.0, SAMPLE_RATE, TraceStage::Horizontal);
        let horizontal = render_stage(
            &input,
            2.0,
            SAMPLE_RATE,
            TraceStage::HorizontalPhaseRecurrence,
        );
        let corrected_metrics = spectrum_metrics(
            &corrected.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
            std::slice::from_ref(&frequency),
        );
        let horizontal_metrics = spectrum_metrics(
            &horizontal.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
            std::slice::from_ref(&frequency),
        );
        ToneStateLineageEvidence {
            frequency_hz: frequency,
            corrected_feedback_out_of_band_db: corrected_metrics.out_of_band_db,
            horizontal_phase_recurrence_out_of_band_db: horizontal_metrics.out_of_band_db,
            corrected_feedback_phase_variance: variance(&phase_advance_errors(
                &corrected.horizontal_output_phases,
                tone_index,
                frequency,
            )),
            horizontal_phase_recurrence_variance: variance(&phase_advance_errors(
                &horizontal.horizontal_output_phases,
                tone_index,
                frequency,
            )),
            corrected_feedback_sideband_offset_hz: corrected_metrics.strongest_sideband_offset_hz,
            horizontal_phase_recurrence_sideband_offset_hz: horizontal_metrics
                .strongest_sideband_offset_hz,
            corrected_feedback_hash: hash_samples(&corrected.samples),
            horizontal_phase_recurrence_hash: hash_samples(&horizontal.samples),
        }
    });
    let direct_horizontal_failure = tones
        .iter()
        .any(|tone| tone.horizontal_phase_recurrence_out_of_band_db > -60.0);
    let corrected_feedback_failure = tones
        .iter()
        .any(|tone| tone.corrected_feedback_out_of_band_db > -60.0);
    StateLineageReview {
        tones,
        mixed_corrected_feedback_out_of_band_db: mixed_corrected_metrics.out_of_band_db,
        mixed_horizontal_phase_recurrence_out_of_band_db: mixed_horizontal_metrics.out_of_band_db,
        mixed_corrected_feedback_phase_variance: std::array::from_fn(|tone| {
            variance(&phase_advance_errors(
                &mixed_corrected.horizontal_output_phases,
                tone,
                CHORD_FREQUENCIES[tone],
            ))
        }),
        mixed_horizontal_phase_recurrence_variance: std::array::from_fn(|tone| {
            variance(&phase_advance_errors(
                &mixed_horizontal.horizontal_output_phases,
                tone,
                CHORD_FREQUENCIES[tone],
            ))
        }),
        mixed_corrected_feedback_sideband_offset_hz: mixed_corrected_metrics
            .strongest_sideband_offset_hz,
        mixed_horizontal_phase_recurrence_sideband_offset_hz: mixed_horizontal_metrics
            .strongest_sideband_offset_hz,
        mixed_corrected_feedback_hash: hash_samples(&mixed_corrected.samples),
        mixed_horizontal_phase_recurrence_hash: hash_samples(&mixed_horizontal.samples),
        repeated: false,
        direction: if direct_horizontal_failure {
            StateLineageDirection::DirectHorizontalRecurrence
        } else if corrected_feedback_failure {
            StateLineageDirection::VerticalStateFeedback
        } else {
            StateLineageDirection::NoFailure
        },
    }
}

fn phase_advance_errors(phases: &[[f64; 4]], tone: usize, frequency: f64) -> Vec<f64> {
    let hop = (SAMPLE_RATE as f64 * 0.03).round();
    let expected = std::f64::consts::TAU * frequency * hop / SAMPLE_RATE as f64;
    phases
        .windows(2)
        .map(|pair| wrap(pair[1][tone] - pair[0][tone] - expected))
        .collect()
}

fn variance(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64
}

fn maximum_abs(values: &[f64]) -> f64 {
    values.iter().map(|value| value.abs()).fold(0.0, f64::max)
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
