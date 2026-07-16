use super::super::super::{coherent_representation, HASH_OFFSET};
use super::super::{hash_values, render};
use super::{controls::*, measure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum ProjectionResidualDirection {
    CoefficientProjection,
    RealEdgeConstraint,
    OverlapSynthesisOrBoundary,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ProjectionResidualRow {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) projected_relation_error: f64,
    pub(in crate::frequency_adaptive) constrained_relation_error: f64,
    pub(in crate::frequency_adaptive) current_whole_ipd_error: f64,
    pub(in crate::frequency_adaptive) current_interior_ipd_error: f64,
    pub(in crate::frequency_adaptive) oracle_whole_ipd_error: f64,
    pub(in crate::frequency_adaptive) oracle_interior_ipd_error: f64,
    pub(in crate::frequency_adaptive) current_whole_image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) current_interior_image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) current_audio_hash: u64,
    pub(in crate::frequency_adaptive) oracle_audio_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ProjectionResidualReview {
    pub(in crate::frequency_adaptive) rows: Vec<ProjectionResidualRow>,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: ProjectionResidualDirection,
}

pub(in crate::frequency_adaptive) fn projection_residual_review() -> ProjectionResidualReview {
    let first = run();
    let second = run();
    let repeated = first == second;
    let coefficient_exact = first
        .rows
        .iter()
        .all(|row| row.projected_relation_error <= 1.0e-12);
    let constrained_exact = first
        .rows
        .iter()
        .all(|row| row.constrained_relation_error <= 1.0e-12);
    ProjectionResidualReview {
        rows: first.rows,
        evidence_hash: first.evidence_hash,
        repeated,
        direction: if !coefficient_exact {
            ProjectionResidualDirection::CoefficientProjection
        } else if !constrained_exact {
            ProjectionResidualDirection::RealEdgeConstraint
        } else {
            ProjectionResidualDirection::OverlapSynthesisOrBoundary
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<ProjectionResidualRow>,
    evidence_hash: u64,
}

fn run() -> Run {
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let output_trim = geometry[0];
    let correlated = correlated_control();
    let mut evidence_hash = HASH_OFFSET;
    let mut rows = Vec::with_capacity(RATIOS.len());

    for ratio in RATIOS {
        let mut projected_relation_error = 0.0_f64;
        let mut constrained_relation_error = 0.0_f64;
        let mut current_whole_ipd_error = 0.0_f64;
        let mut current_interior_ipd_error = 0.0_f64;
        let mut oracle_whole_ipd_error = 0.0_f64;
        let mut oracle_interior_ipd_error = 0.0_f64;
        let mut current_audio_hash = HASH_OFFSET;
        let mut oracle_audio_hash = HASH_OFFSET;

        for frequency in TONE_FREQUENCIES {
            let input = tone_control(std::f64::consts::FRAC_PI_2, frequency);
            let current = render::linked([&input[0], &input[1]], ratio, SAMPLE_RATE);
            let oracle = render::linked_with_relation_oracle(
                [&input[0], &input[1]],
                ratio,
                SAMPLE_RATE,
                std::f64::consts::FRAC_PI_2,
            );
            let current_interior = crop(&current.channels, output_trim);
            let oracle_interior = crop(&oracle.channels, output_trim);
            projected_relation_error =
                projected_relation_error.max(current.maximum_projected_relation_error);
            constrained_relation_error =
                constrained_relation_error.max(current.maximum_constrained_relation_error);
            current_whole_ipd_error =
                current_whole_ipd_error.max(measure::maximum_expected_ipd_error(
                    &current.channels,
                    std::f64::consts::FRAC_PI_2,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            current_interior_ipd_error =
                current_interior_ipd_error.max(measure::maximum_expected_ipd_error(
                    &current_interior,
                    std::f64::consts::FRAC_PI_2,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            oracle_whole_ipd_error =
                oracle_whole_ipd_error.max(measure::maximum_expected_ipd_error(
                    &oracle.channels,
                    std::f64::consts::FRAC_PI_2,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            oracle_interior_ipd_error =
                oracle_interior_ipd_error.max(measure::maximum_expected_ipd_error(
                    &oracle_interior,
                    std::f64::consts::FRAC_PI_2,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            hash_values(&mut current_audio_hash, &[current.hash]);
            hash_values(&mut oracle_audio_hash, &[oracle.hash]);
        }

        let current_image = render::linked([&correlated[0], &correlated[1]], ratio, SAMPLE_RATE);
        let source_trim = (output_trim as f64 / ratio).ceil() as usize;
        let current_whole_image = measure::image_delta(&correlated, &current_image.channels);
        let current_interior_image = measure::image_delta(
            &crop(&correlated, source_trim),
            &crop(&current_image.channels, output_trim),
        );
        let current_whole_image_delta = [
            current_whole_image.mid_side_ratio_db,
            current_whole_image.correlation,
        ];
        let current_interior_image_delta = [
            current_interior_image.mid_side_ratio_db,
            current_interior_image.correlation,
        ];
        let row = ProjectionResidualRow {
            ratio,
            projected_relation_error,
            constrained_relation_error,
            current_whole_ipd_error,
            current_interior_ipd_error,
            oracle_whole_ipd_error,
            oracle_interior_ipd_error,
            current_whole_image_delta,
            current_interior_image_delta,
            current_audio_hash,
            oracle_audio_hash,
        };
        hash_row(&mut evidence_hash, &row);
        rows.push(row);
    }
    Run {
        rows,
        evidence_hash,
    }
}

fn crop(channels: &[Vec<f64>; 2], trim: usize) -> [Vec<f64>; 2] {
    std::array::from_fn(|channel| {
        let end = channels[channel].len().saturating_sub(trim);
        channels[channel][trim.min(end)..end].to_vec()
    })
}

fn hash_row(hash: &mut u64, row: &ProjectionResidualRow) {
    hash_values(
        hash,
        &[
            row.ratio.to_bits(),
            row.projected_relation_error.to_bits(),
            row.constrained_relation_error.to_bits(),
            row.current_whole_ipd_error.to_bits(),
            row.current_interior_ipd_error.to_bits(),
            row.oracle_whole_ipd_error.to_bits(),
            row.oracle_interior_ipd_error.to_bits(),
            row.current_whole_image_delta[0].to_bits(),
            row.current_whole_image_delta[1].to_bits(),
            row.current_interior_image_delta[0].to_bits(),
            row.current_interior_image_delta[1].to_bits(),
            row.current_audio_hash,
            row.oracle_audio_hash,
        ],
    );
}
