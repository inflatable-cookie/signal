use super::super::super::HASH_OFFSET;
use super::super::{
    hash_values,
    render::{self, CoefficientAblation, CoefficientClassEvidence},
};
use super::{controls::*, measure};

mod support;
use support::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum CoefficientContributionDirection {
    InitialFrameRepair,
    ReferenceFallbackRepair,
    WeakCoefficientRepair,
    GateDefinitionReassessment,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CoefficientAblationEvidence {
    pub(in crate::frequency_adaptive) maximum_ipd_error: f64,
    pub(in crate::frequency_adaptive) image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) structural_failures: [usize; 4],
    pub(in crate::frequency_adaptive) tone_hash: u64,
    pub(in crate::frequency_adaptive) image_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CoefficientContributionRow {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) lifecycle: [CoefficientClassEvidence; 3],
    pub(in crate::frequency_adaptive) energy: [CoefficientClassEvidence; 2],
    pub(in crate::frequency_adaptive) current_maximum_ipd_error: f64,
    pub(in crate::frequency_adaptive) current_image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) current_tone_hash: u64,
    pub(in crate::frequency_adaptive) current_image_hash: u64,
    pub(in crate::frequency_adaptive) ablations: [CoefficientAblationEvidence; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CoefficientContributionReview {
    pub(in crate::frequency_adaptive) rows: Vec<CoefficientContributionRow>,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: CoefficientContributionDirection,
}

pub(in crate::frequency_adaptive) fn coefficient_contribution_review(
) -> CoefficientContributionReview {
    let first = run();
    let second = run();
    let repeated = first == second;
    let direction = [
        CoefficientContributionDirection::InitialFrameRepair,
        CoefficientContributionDirection::ReferenceFallbackRepair,
        CoefficientContributionDirection::WeakCoefficientRepair,
    ]
    .into_iter()
    .enumerate()
    .find(|(index, _)| first.rows.iter().all(|row| closes(row.ablations[*index])))
    .map(|(_, direction)| direction)
    .unwrap_or(CoefficientContributionDirection::GateDefinitionReassessment);
    CoefficientContributionReview {
        rows: first.rows,
        evidence_hash: first.evidence_hash,
        repeated,
        direction,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<CoefficientContributionRow>,
    evidence_hash: u64,
}

fn run() -> Run {
    let correlated = correlated_control();
    let ablations = [
        CoefficientAblation::Initial,
        CoefficientAblation::Fallback,
        CoefficientAblation::Weak,
    ];
    let mut rows = Vec::with_capacity(RATIOS.len());
    let mut evidence_hash = HASH_OFFSET;

    for ratio in RATIOS {
        let mut lifecycle = [CoefficientClassEvidence::default(); 3];
        let mut energy = [CoefficientClassEvidence::default(); 2];
        let mut current_maximum_ipd_error = 0.0_f64;
        let mut current_tone_hash = HASH_OFFSET;
        let mut ablation_ipd = [0.0_f64; 3];
        let mut ablation_tone_hash = [HASH_OFFSET; 3];
        let mut structural_failures = [[0; 4]; 3];

        for frequency in TONE_FREQUENCIES {
            let input = tone_control(std::f64::consts::FRAC_PI_2, frequency);
            let current = render::linked_with_coefficient_trace(
                [&input[0], &input[1]],
                ratio,
                SAMPLE_RATE,
                Some(std::f64::consts::FRAC_PI_2),
                None,
            );
            add_trace(
                &mut lifecycle,
                &mut energy,
                current
                    .coefficient_contribution_trace
                    .expect("coefficient trace"),
            );
            current_maximum_ipd_error =
                current_maximum_ipd_error.max(measure::maximum_expected_ipd_error(
                    &current.channels,
                    std::f64::consts::FRAC_PI_2,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            hash_values(&mut current_tone_hash, &[current.hash]);
            for (index, ablation) in ablations.into_iter().enumerate() {
                let output = render::linked_with_coefficient_trace(
                    [&input[0], &input[1]],
                    ratio,
                    SAMPLE_RATE,
                    Some(std::f64::consts::FRAC_PI_2),
                    Some(ablation),
                );
                ablation_ipd[index] = ablation_ipd[index].max(measure::maximum_expected_ipd_error(
                    &output.channels,
                    std::f64::consts::FRAC_PI_2,
                    &[frequency],
                    SAMPLE_RATE,
                ));
                hash_values(&mut ablation_tone_hash[index], &[output.hash]);
                add_structural_failures(
                    &mut structural_failures[index],
                    &output,
                    (input[0].len() as f64 * ratio).round() as usize,
                );
            }
        }

        let current_image = render::linked_with_coefficient_trace(
            [&correlated[0], &correlated[1]],
            ratio,
            SAMPLE_RATE,
            None,
            None,
        );
        add_trace(
            &mut lifecycle,
            &mut energy,
            current_image
                .coefficient_contribution_trace
                .expect("coefficient trace"),
        );
        let current_image_delta = image_delta(&correlated, &current_image.channels);
        let mut ablation_evidence = [CoefficientAblationEvidence {
            maximum_ipd_error: 0.0,
            image_delta: [0.0; 2],
            structural_failures: [0; 4],
            tone_hash: 0,
            image_hash: 0,
        }; 3];
        for (index, ablation) in ablations.into_iter().enumerate() {
            let output = render::linked_with_coefficient_trace(
                [&correlated[0], &correlated[1]],
                ratio,
                SAMPLE_RATE,
                None,
                Some(ablation),
            );
            add_structural_failures(
                &mut structural_failures[index],
                &output,
                (correlated[0].len() as f64 * ratio).round() as usize,
            );
            ablation_evidence[index] = CoefficientAblationEvidence {
                maximum_ipd_error: ablation_ipd[index],
                image_delta: image_delta(&correlated, &output.channels),
                structural_failures: structural_failures[index],
                tone_hash: ablation_tone_hash[index],
                image_hash: output.hash,
            };
        }
        let row = CoefficientContributionRow {
            ratio,
            lifecycle,
            energy,
            current_maximum_ipd_error,
            current_image_delta,
            current_tone_hash,
            current_image_hash: current_image.hash,
            ablations: ablation_evidence,
        };
        hash_row(&mut evidence_hash, &row);
        rows.push(row);
    }
    Run {
        rows,
        evidence_hash,
    }
}
