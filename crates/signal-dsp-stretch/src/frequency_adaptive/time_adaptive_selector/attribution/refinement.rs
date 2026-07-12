use super::super::super::types::{
    StretchRenyiReassessmentDirection as Direction, StretchRenyiReassessmentReview as Review,
    StretchRenyiRefinedAnchorEvidence as AnchorEvidence,
    StretchRenyiRefinedControlEvidence as ControlEvidence,
};
use super::super::{
    controls, hash_f64, hash_u64, longest_minimum, ALPHA, ANCHOR_HOP, FFT, HASH_OFFSET, LENGTHS,
};
use super::measure::{measure, Frame};
use super::{renyi_selector_failure_attribution_review, FRAMES, REGIONS};

pub(crate) fn renyi_attribution_reassessment_review() -> Review {
    let prior = renyi_selector_failure_attribution_review();
    let sources = controls();
    let controls = [5, 8, 11]
        .into_iter()
        .map(|index| {
            refine_control(
                index,
                &sources[index].samples,
                (index != 8).then_some(FRAMES / 2),
                &prior.baseline.controls[index],
            )
        })
        .collect::<Vec<_>>();
    let isolated = &prior.baseline.controls[5];
    let applicable = isolated
        .selected_levels
        .iter()
        .enumerate()
        .filter(|(index, level)| (index * ANCHOR_HOP).abs_diff(FRAMES / 2) > 2_048 && **level != 3)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let mixed_event = ((FRAMES / 2 / ANCHOR_HOP) - 2)..=((FRAMES / 2 / ANCHOR_HOP) + 2);
    let mixed_negative = (0..16).chain(48..64).collect::<Vec<_>>();
    let restored_support = applicable
        .iter()
        .filter(|index| controls[0].anchors[**index].support_removed_winner == 3)
        .count();
    let changed_support_negatives = mixed_negative
        .iter()
        .filter(|index| {
            controls[2].anchors[**index].support_removed_winner
                != prior.baseline.controls[11].raw_winners[**index]
        })
        .count();
    let support_passes = restored_support == applicable.len() && changed_support_negatives == 0;
    let low_event_restorations = std::array::from_fn(|region| {
        mixed_event
            .clone()
            .filter(|index| controls[2].anchors[*index].low_removed_winners[region] == 0)
            .count()
    });
    let low_negative_changes = std::array::from_fn(|region| {
        mixed_negative
            .iter()
            .filter(|index| {
                controls[2].anchors[**index].low_removed_winners[region]
                    != prior.baseline.controls[11].raw_winners[**index]
            })
            .count()
    });
    let low_candidates = (0..REGIONS)
        .filter(|region| low_event_restorations[*region] == 5 && low_negative_changes[*region] == 0)
        .collect::<Vec<_>>();
    let linear_chirp_changes = std::array::from_fn(|region| {
        controls[1]
            .anchors
            .iter()
            .enumerate()
            .filter(|(index, anchor)| {
                anchor.low_removed_winners[region] != prior.baseline.controls[8].raw_winners[*index]
            })
            .count()
    });
    let structural_failure = controls.iter().any(|control| {
        control.structural_failures != [0; 3]
            || control.closure_errors[0] != 0.0
            || control.closure_errors[1] > 1.0e-12
            || control.closure_errors[2] != 0.0
            || control.closure_errors[3] > 1.0e-12
    });
    let direction = if structural_failure || low_candidates.len() > 1 {
        Direction::OperatorReview
    } else {
        match (support_passes, low_candidates.len()) {
            (true, 0) => Direction::ComparisonRegionContract,
            (false, 1) => Direction::FrequencyEvidenceContract,
            (true, 1) => Direction::LocalizedTimeFrequencyContract,
            _ => Direction::OperatorReview,
        }
    };
    let mut review = Review {
        prior,
        controls,
        support_effects: [restored_support, changed_support_negatives],
        low_event_restorations,
        low_negative_changes,
        linear_chirp_changes,
        candidate_counts: [usize::from(support_passes), low_candidates.len()],
        evidence_hash: 0,
        direction,
    };
    review.evidence_hash = review_hash(&review);
    review
}

fn refine_control(
    control: usize,
    samples: &[f64],
    event: Option<usize>,
    baseline: &super::super::super::types::StretchRenyiControlEvidence,
) -> ControlEvidence {
    let grids = LENGTHS
        .into_iter()
        .map(|length| measure(samples, length, length / 4))
        .collect::<Vec<_>>();
    let mut closure = [0.0_f64; 4];
    let mut structural = [0_usize; 3];
    let anchors = (0..FRAMES)
        .step_by(ANCHOR_HOP)
        .enumerate()
        .map(|(index, anchor)| {
            refine_anchor(
                anchor,
                event,
                &grids,
                &baseline.energies[index],
                &baseline.entropies[index],
                &mut closure,
                &mut structural,
            )
        })
        .collect::<Vec<_>>();
    let mut evidence = ControlEvidence {
        control,
        anchors,
        closure_errors: closure,
        structural_failures: structural,
        evidence_hash: 0,
    };
    evidence.evidence_hash = control_hash(&evidence);
    evidence
}

fn refine_anchor(
    anchor: usize,
    event: Option<usize>,
    grids: &[Vec<Frame>],
    baseline_energies: &[f64; 4],
    baseline_entropies: &[f64; 4],
    closure: &mut [f64; 4],
    structural: &mut [usize; 3],
) -> AnchorEvidence {
    let mut support_counts = [[0; 2]; 4];
    let mut support_energies = [[0.0; 2]; 4];
    let mut support_alpha = [[0.0; 2]; 4];
    let mut low_counts = [[0; REGIONS]; 4];
    let mut low_energies = [[0.0; REGIONS]; 4];
    let mut low_alpha = [[0.0; REGIONS]; 4];
    let mut complement_counts = [0; 4];
    let mut complement_energies = [0.0; 4];
    let mut complement_alpha = [0.0; 4];
    let mut parents = [[0.0; 2]; 4];
    let start = anchor as isize - FFT as isize / 2;
    let end = anchor as isize + FFT as isize / 2;
    for level in 0..4 {
        let length = LENGTHS[level];
        let mut included_frames = 0;
        for frame in grids[level]
            .iter()
            .filter(|frame| frame.center >= start && frame.center < end)
        {
            included_frames += 1;
            let owns = event.is_some_and(|event| {
                let event = event as isize;
                frame.center - length as isize / 2 <= event
                    && event < frame.center + length as isize / 2
            }) as usize;
            support_counts[level][owns] += FFT;
            support_energies[level][owns] += frame.energy;
            support_alpha[level][owns] += frame.alpha_sum;
            parents[level][0] += frame.energy;
            parents[level][1] += frame.alpha_sum;
            for region in 0..REGIONS {
                low_counts[level][region] += frame.low_counts[region];
                low_energies[level][region] += frame.low_energies[region];
                low_alpha[level][region] += frame.low_alpha_sums[region];
            }
            complement_counts[level] += frame.complement_count;
            complement_energies[level] += frame.complement_energy;
            complement_alpha[level] += frame.complement_alpha_sum;
        }
        let parent_count = support_counts[level].iter().sum::<usize>();
        closure[0] = closure[0].max(usize::from(parent_count != included_frames * FFT) as f64);
        closure[1] = closure[1].max(partition_closure(
            &support_energies[level],
            parents[level][0],
        ));
        closure[1] = closure[1].max(partition_closure(&support_alpha[level], parents[level][1]));
        let low_count = low_counts[level].iter().sum::<usize>() + complement_counts[level];
        closure[2] = closure[2].max(usize::from(low_count != parent_count) as f64);
        closure[3] = closure[3].max(low_closure(
            &low_energies[level],
            complement_energies[level],
            parents[level][0],
        ));
        closure[3] = closure[3].max(low_closure(
            &low_alpha[level],
            complement_alpha[level],
            parents[level][1],
        ));
        let full_entropy = entropy(parents[level][0], parents[level][1], length / 4);
        structural[2] +=
            usize::from(parents[level][0].to_bits() != baseline_energies[level].to_bits());
        structural[2] += usize::from(full_entropy.to_bits() != baseline_entropies[level].to_bits());
    }
    let support_removed_winner = removed_winner(
        std::array::from_fn(|level| support_energies[level][1]),
        std::array::from_fn(|level| support_alpha[level][1]),
        &parents,
        structural,
    );
    let low_removed_winners = std::array::from_fn(|region| {
        removed_winner(
            std::array::from_fn(|level| low_energies[level][region]),
            std::array::from_fn(|level| low_alpha[level][region]),
            &parents,
            structural,
        )
    });
    AnchorEvidence {
        anchor,
        support_counts,
        support_energies,
        support_alpha_sums: support_alpha,
        support_removed_winner,
        low_counts,
        low_energies,
        low_alpha_sums: low_alpha,
        complement_counts,
        complement_energies,
        complement_alpha_sums: complement_alpha,
        low_removed_winners,
    }
}

fn removed_winner(
    removed_energy: [f64; 4],
    removed_alpha: [f64; 4],
    parents: &[[f64; 2]; 4],
    structural: &mut [usize; 3],
) -> u8 {
    let entropies = std::array::from_fn(|level| {
        let energy = parents[level][0] - removed_energy[level];
        let alpha = parents[level][1] - removed_alpha[level];
        let tolerance = parents[level][0].abs().max(parents[level][1].abs()) * 1.0e-12;
        structural[1] += usize::from(energy < -tolerance || alpha < -tolerance);
        let value = entropy(energy.max(0.0), alpha.max(0.0), LENGTHS[level] / 4);
        structural[0] += usize::from(!value.is_finite());
        value
    });
    longest_minimum(&entropies)
}

fn entropy(energy: f64, alpha_sum: f64, hop: usize) -> f64 {
    if energy == 0.0 {
        0.0
    } else {
        (alpha_sum / energy.powf(ALPHA)).log2() / (1.0 - ALPHA) + (hop as f64 / FFT as f64).log2()
    }
}

fn partition_closure<const N: usize>(parts: &[f64; N], parent: f64) -> f64 {
    (parts.iter().sum::<f64>() - parent).abs() / parent.abs().max(f64::MIN_POSITIVE)
}

fn low_closure(parts: &[f64; REGIONS], complement: f64, parent: f64) -> f64 {
    (parts.iter().sum::<f64>() + complement - parent).abs() / parent.abs().max(f64::MIN_POSITIVE)
}

fn control_hash(control: &ControlEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, control.control as u64);
    for anchor in &control.anchors {
        hash_u64(&mut hash, anchor.anchor as u64);
        hash_u64(&mut hash, u64::from(anchor.support_removed_winner));
        for winner in anchor.low_removed_winners {
            hash_u64(&mut hash, u64::from(winner));
        }
        for rows in [&anchor.support_energies, &anchor.support_alpha_sums] {
            for row in rows {
                for value in row {
                    hash_f64(&mut hash, *value);
                }
            }
        }
        for rows in [&anchor.low_energies, &anchor.low_alpha_sums] {
            for row in rows {
                for value in row {
                    hash_f64(&mut hash, *value);
                }
            }
        }
    }
    hash
}

fn review_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, review.prior.evidence_hash);
    for control in &review.controls {
        hash_u64(&mut hash, control.evidence_hash);
    }
    for value in review
        .support_effects
        .into_iter()
        .chain(review.low_event_restorations)
        .chain(review.low_negative_changes)
        .chain(review.linear_chirp_changes)
        .chain(review.candidate_counts)
    {
        hash_u64(&mut hash, value as u64);
    }
    hash
}
