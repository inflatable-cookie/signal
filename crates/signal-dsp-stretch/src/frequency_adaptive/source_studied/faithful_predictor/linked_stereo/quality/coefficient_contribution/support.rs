use super::*;

pub(super) fn add_trace(
    lifecycle: &mut [CoefficientClassEvidence; 3],
    energy: &mut [CoefficientClassEvidence; 2],
    trace: render::CoefficientContributionTrace,
) {
    for (target, source) in lifecycle.iter_mut().zip(trace.lifecycle) {
        add_class(target, source);
    }
    for (target, source) in energy.iter_mut().zip(trace.energy) {
        add_class(target, source);
    }
}

fn add_class(target: &mut CoefficientClassEvidence, source: CoefficientClassEvidence) {
    target.count += source.count;
    target.synthesized_energy += source.synthesized_energy;
    target.measurable_relations += source.measurable_relations;
    target.maximum_relation_error = target
        .maximum_relation_error
        .max(source.maximum_relation_error);
}

pub(super) fn add_structural_failures(
    failures: &mut [usize; 4],
    output: &render::StereoRender,
    target_length: usize,
) {
    failures[0] += usize::from(output.channels.iter().any(|c| c.len() != target_length));
    failures[1] += output.uncovered;
    failures[2] += output.non_finite;
    failures[3] += output.boundary_failures;
}

pub(super) fn image_delta(input: &[Vec<f64>; 2], output: &[Vec<f64>; 2]) -> [f64; 2] {
    let delta = measure::image_delta(input, output);
    [delta.mid_side_ratio_db, delta.correlation]
}

pub(super) fn closes(evidence: CoefficientAblationEvidence) -> bool {
    evidence.structural_failures == [0; 4]
        && evidence.maximum_ipd_error <= 1.0e-9
        && evidence.image_delta[0] <= 0.25
        && evidence.image_delta[1] <= 0.02
}

pub(super) fn hash_row(hash: &mut u64, row: &CoefficientContributionRow) {
    hash_values(
        hash,
        &[
            row.ratio.to_bits(),
            row.current_maximum_ipd_error.to_bits(),
            row.current_image_delta[0].to_bits(),
            row.current_image_delta[1].to_bits(),
            row.current_tone_hash,
            row.current_image_hash,
        ],
    );
    for class in row.lifecycle.into_iter().chain(row.energy) {
        hash_class(hash, class);
    }
    for ablation in row.ablations {
        hash_values(
            hash,
            &[
                ablation.maximum_ipd_error.to_bits(),
                ablation.image_delta[0].to_bits(),
                ablation.image_delta[1].to_bits(),
                ablation.tone_hash,
                ablation.image_hash,
            ],
        );
        hash_values(
            hash,
            &ablation.structural_failures.map(|value| value as u64),
        );
    }
}

fn hash_class(hash: &mut u64, class: CoefficientClassEvidence) {
    hash_values(
        hash,
        &[
            class.count as u64,
            class.synthesized_energy.to_bits(),
            class.measurable_relations as u64,
            class.maximum_relation_error.to_bits(),
        ],
    );
}
