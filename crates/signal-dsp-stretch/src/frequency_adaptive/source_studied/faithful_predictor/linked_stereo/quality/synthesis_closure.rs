use super::super::super::{coherent_representation, HASH_OFFSET};
use super::super::{hash_values, render};
use super::{controls::*, measure, projection_attribution};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum SynthesisClosureDirection {
    MeasurementFloor,
    InverseSupportSynthesis,
    OverlapAccumulation,
    Normalization,
    Exact,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SynthesisClosureRow {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) ideal_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) current_inverse_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) oracle_inverse_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) current_accumulated_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) oracle_accumulated_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) current_normalized_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) oracle_normalized_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) current_audio_hash: u64,
    pub(in crate::frequency_adaptive) oracle_audio_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SynthesisClosureReview {
    pub(in crate::frequency_adaptive) predecessor_evidence_hash: u64,
    pub(in crate::frequency_adaptive) rows: Vec<SynthesisClosureRow>,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: SynthesisClosureDirection,
}

pub(in crate::frequency_adaptive) fn synthesis_closure_review() -> SynthesisClosureReview {
    let predecessor = projection_attribution::projection_residual_review();
    let first = run();
    let second = run();
    let repeated = first == second;
    let direction = select_direction(&first.rows);
    SynthesisClosureReview {
        predecessor_evidence_hash: predecessor.evidence_hash,
        rows: first.rows,
        evidence_hash: first.evidence_hash,
        repeated,
        direction,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<SynthesisClosureRow>,
    evidence_hash: u64,
}

fn run() -> Run {
    let output_trim = coherent_representation::source_geometry(SAMPLE_RATE)[0];
    let expected_ipd = std::f64::consts::FRAC_PI_2;
    let mut evidence_hash = HASH_OFFSET;
    let mut rows = Vec::with_capacity(RATIOS.len());

    for ratio in RATIOS {
        let target_length = (SAMPLE_RATE as f64 * ratio).round() as usize;
        let mut row = SynthesisClosureRow {
            ratio,
            ideal_ipd_error: [0.0; 2],
            current_inverse_ipd_error: [0.0; 2],
            oracle_inverse_ipd_error: [0.0; 2],
            current_accumulated_ipd_error: [0.0; 2],
            oracle_accumulated_ipd_error: [0.0; 2],
            current_normalized_ipd_error: [0.0; 2],
            oracle_normalized_ipd_error: [0.0; 2],
            current_audio_hash: HASH_OFFSET,
            oracle_audio_hash: HASH_OFFSET,
        };

        for frequency in TONE_FREQUENCIES {
            let input = tone_control(expected_ipd, frequency);
            let ideal = tone(expected_ipd, frequency, target_length);
            let ideal_interior = crop(&ideal, output_trim);
            row.ideal_ipd_error[0] =
                row.ideal_ipd_error[0].max(measure::maximum_expected_ipd_error(
                    &ideal,
                    expected_ipd,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            row.ideal_ipd_error[1] =
                row.ideal_ipd_error[1].max(measure::maximum_expected_ipd_error(
                    &ideal_interior,
                    expected_ipd,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            let ideal_ipd = [
                measure::ipd(&ideal, frequency, SAMPLE_RATE),
                measure::ipd(&ideal_interior, frequency, SAMPLE_RATE),
            ];

            let current = render::linked_with_synthesis_trace(
                [&input[0], &input[1]],
                ratio,
                SAMPLE_RATE,
                frequency,
                expected_ipd,
                ideal_ipd,
                None,
            );
            let oracle = render::linked_with_synthesis_trace(
                [&input[0], &input[1]],
                ratio,
                SAMPLE_RATE,
                frequency,
                expected_ipd,
                ideal_ipd,
                Some(expected_ipd),
            );
            let current_trace = current.synthesis_relation_trace.expect("current trace");
            let oracle_trace = oracle.synthesis_relation_trace.expect("oracle trace");
            maximize(
                &mut row.current_inverse_ipd_error,
                current_trace.inverse_ipd_error,
            );
            maximize(
                &mut row.oracle_inverse_ipd_error,
                oracle_trace.inverse_ipd_error,
            );
            maximize(
                &mut row.current_accumulated_ipd_error,
                current_trace.accumulated_ipd_error,
            );
            maximize(
                &mut row.oracle_accumulated_ipd_error,
                oracle_trace.accumulated_ipd_error,
            );
            maximize(
                &mut row.current_normalized_ipd_error,
                current_trace.normalized_ipd_error,
            );
            maximize(
                &mut row.oracle_normalized_ipd_error,
                oracle_trace.normalized_ipd_error,
            );
            hash_values(&mut row.current_audio_hash, &[current.hash]);
            hash_values(&mut row.oracle_audio_hash, &[oracle.hash]);
        }
        hash_row(&mut evidence_hash, &row);
        rows.push(row);
    }
    Run {
        rows,
        evidence_hash,
    }
}

fn select_direction(rows: &[SynthesisClosureRow]) -> SynthesisClosureDirection {
    let tolerance = 1.0e-9;
    if rows.iter().any(|row| {
        row.current_inverse_ipd_error
            .into_iter()
            .chain(row.oracle_inverse_ipd_error)
            .any(|error| error > tolerance)
    }) {
        SynthesisClosureDirection::InverseSupportSynthesis
    } else if rows.iter().any(|row| {
        row.current_accumulated_ipd_error
            .into_iter()
            .chain(row.oracle_accumulated_ipd_error)
            .any(|error| error > tolerance)
    }) {
        SynthesisClosureDirection::OverlapAccumulation
    } else if rows.iter().any(|row| {
        row.current_normalized_ipd_error
            .into_iter()
            .chain(row.oracle_normalized_ipd_error)
            .any(|error| error > tolerance)
    }) {
        SynthesisClosureDirection::Normalization
    } else if rows.iter().any(|row| {
        row.ideal_ipd_error
            .into_iter()
            .any(|error| error > tolerance)
    }) {
        SynthesisClosureDirection::MeasurementFloor
    } else {
        SynthesisClosureDirection::Exact
    }
}

fn tone(phase_offset: f64, frequency: f64, length: usize) -> [Vec<f64>; 2] {
    std::array::from_fn(|channel| {
        (0..length)
            .map(|index| {
                let time = index as f64 / SAMPLE_RATE as f64;
                let phase = if channel == 0 { 0.0 } else { phase_offset };
                0.3 * (std::f64::consts::TAU * frequency * time + phase).sin()
            })
            .collect()
    })
}

fn crop(channels: &[Vec<f64>; 2], trim: usize) -> [Vec<f64>; 2] {
    std::array::from_fn(|channel| {
        let end = channels[channel].len().saturating_sub(trim);
        channels[channel][trim.min(end)..end].to_vec()
    })
}

fn maximize(target: &mut [f64; 2], value: [f64; 2]) {
    for index in 0..2 {
        target[index] = target[index].max(value[index]);
    }
}

fn hash_row(hash: &mut u64, row: &SynthesisClosureRow) {
    hash_values(hash, &[row.ratio.to_bits()]);
    for values in [
        row.ideal_ipd_error,
        row.current_inverse_ipd_error,
        row.oracle_inverse_ipd_error,
        row.current_accumulated_ipd_error,
        row.oracle_accumulated_ipd_error,
        row.current_normalized_ipd_error,
        row.oracle_normalized_ipd_error,
    ] {
        hash_values(hash, &values.map(f64::to_bits));
    }
    hash_values(hash, &[row.current_audio_hash, row.oracle_audio_hash]);
}
