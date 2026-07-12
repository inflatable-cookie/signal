use super::super::types::{
    StretchRenyiAttributionAnchorEvidence as AnchorEvidence,
    StretchRenyiAttributionControlEvidence as ControlEvidence,
    StretchRenyiAttributionDirection as Direction, StretchRenyiAttributionReview as Review,
    StretchRenyiRegionRemovalEvidence as RemovalEvidence,
};
use super::controls::FRAMES;
use super::{
    controls, hash_f64, hash_u64, longest_minimum, renyi_time_resolution_selection_review, ALPHA,
    ANCHOR_HOP, FFT, HASH_OFFSET, LENGTHS,
};

mod measure;
use measure::{measure, Frame};

const REGIONS: usize = 8;

pub(crate) fn renyi_selector_failure_attribution_review() -> Review {
    let baseline = renyi_time_resolution_selection_review();
    let sources = controls();
    let mut controls = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().enumerate() {
        controls.push(attribute_control(
            index,
            source.samples.as_slice(),
            &baseline.controls[index],
        ));
    }

    let isolated = &baseline.controls[5];
    let applicable = isolated
        .selected_levels
        .iter()
        .enumerate()
        .filter(|(index, level)| {
            let anchor = index * ANCHOR_HOP;
            anchor.abs_diff(FRAMES / 2) > 2_048 && **level != 3
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let mixed_event = ((FRAMES / 2 / ANCHOR_HOP) - 2)..=((FRAMES / 2 / ANCHOR_HOP) + 2);
    let mixed_negative = (0..16).chain(48..64).collect::<Vec<_>>();

    let restored_isolated = applicable
        .iter()
        .filter(|index| {
            let region = if **index * ANCHOR_HOP < FRAMES / 2 {
                REGIONS - 1
            } else {
                0
            };
            controls[5].anchors[**index].time_removals[region].raw_winner == 3
        })
        .count();
    let changed_mixed_negatives = mixed_negative
        .iter()
        .filter(|index| {
            let region = if **index * ANCHOR_HOP < FRAMES / 2 {
                REGIONS - 1
            } else {
                0
            };
            controls[11].anchors[**index].time_removals[region].raw_winner
                != baseline.controls[11].raw_winners[**index]
        })
        .count();
    let geometry_passes = !applicable.is_empty()
        && restored_isolated == applicable.len()
        && changed_mixed_negatives == 0;
    let frequency_event_restorations = std::array::from_fn(|region| {
        mixed_event
            .clone()
            .filter(|index| controls[11].anchors[*index].frequency_removals[region].raw_winner == 0)
            .count()
    });
    let frequency_negative_changes = std::array::from_fn(|region| {
        mixed_negative
            .iter()
            .filter(|index| {
                controls[11].anchors[**index].frequency_removals[region].raw_winner
                    != baseline.controls[11].raw_winners[**index]
            })
            .count()
    });
    let frequency_candidates = (0..REGIONS)
        .filter(|region| {
            frequency_event_restorations[*region] == mixed_event.clone().count()
                && frequency_negative_changes[*region] == 0
        })
        .collect::<Vec<_>>();
    let linear_chirp_changes = std::array::from_fn(|axis| {
        std::array::from_fn(|region| {
            controls[8]
                .anchors
                .iter()
                .enumerate()
                .filter(|(index, anchor)| {
                    let removal = if axis == 0 {
                        &anchor.time_removals[region]
                    } else {
                        &anchor.frequency_removals[region]
                    };
                    removal.raw_winner != baseline.controls[8].raw_winners[*index]
                })
                .count()
        })
    });
    let direction = match (geometry_passes, frequency_candidates.len()) {
        (true, 0) => Direction::ComparisonRegionContract,
        (false, 1) => Direction::FrequencyEvidenceContract,
        _ => Direction::Inconclusive,
    };
    let mut review = Review {
        baseline,
        controls,
        diagnostic_counts: [applicable.len(), mixed_event.count(), mixed_negative.len()],
        candidate_counts: [usize::from(geometry_passes), frequency_candidates.len()],
        geometry_effects: [restored_isolated, changed_mixed_negatives],
        frequency_event_restorations,
        frequency_negative_changes,
        linear_chirp_changes,
        evidence_hash: 0,
        direction,
    };
    review.evidence_hash = review_hash(&review);
    review
}

fn attribute_control(
    control: usize,
    samples: &[f64],
    baseline: &super::super::types::StretchRenyiControlEvidence,
) -> ControlEvidence {
    let anchors = (0..FRAMES).step_by(ANCHOR_HOP).collect::<Vec<_>>();
    let grids = LENGTHS
        .into_iter()
        .map(|length| measure(samples, length, length / 4))
        .collect::<Vec<_>>();
    let mut evidence = Vec::with_capacity(anchors.len());
    let mut closure = [0.0_f64; 4];
    let mut structural = [0_usize; 2];
    let mut baseline_drift = 0;
    for (anchor_index, anchor) in anchors.into_iter().enumerate() {
        evidence.push(attribute_anchor(
            anchor,
            &grids,
            &baseline.energies[anchor_index],
            &baseline.entropies[anchor_index],
            &mut closure,
            &mut structural,
            &mut baseline_drift,
        ));
    }
    let mut result = ControlEvidence {
        control,
        anchors: evidence,
        closure_errors: closure,
        structural_failures: structural,
        baseline_drift,
        evidence_hash: 0,
    };
    result.evidence_hash = control_hash(&result);
    result
}

fn attribute_anchor(
    anchor: usize,
    grids: &[Vec<Frame>],
    baseline_energies: &[f64; 4],
    baseline_entropies: &[f64; 4],
    closure: &mut [f64; 4],
    structural: &mut [usize; 2],
    baseline_drift: &mut usize,
) -> AnchorEvidence {
    let mut time_counts = [[0; REGIONS]; 4];
    let mut time_energies = [[0.0; REGIONS]; 4];
    let mut time_alpha = [[0.0; REGIONS]; 4];
    let mut frequency_counts = [[0; REGIONS]; 4];
    let mut frequency_energies = [[0.0; REGIONS]; 4];
    let mut frequency_alpha = [[0.0; REGIONS]; 4];
    let mut parents = [[0.0; 2]; 4];
    let start = anchor as isize - FFT as isize / 2;
    let end = anchor as isize + FFT as isize / 2;
    for level in 0..4 {
        for frame in grids[level]
            .iter()
            .filter(|frame| frame.center >= start && frame.center < end)
        {
            let slice = ((frame.center - start) as usize / (FFT / REGIONS)).min(REGIONS - 1);
            time_counts[level][slice] += FFT;
            time_energies[level][slice] += frame.energy;
            time_alpha[level][slice] += frame.alpha_sum;
            parents[level][0] += frame.energy;
            parents[level][1] += frame.alpha_sum;
            for region in 0..REGIONS {
                frequency_counts[level][region] += frame.frequency_counts[region];
                frequency_energies[level][region] += frame.frequency_energies[region];
                frequency_alpha[level][region] += frame.frequency_alpha_sums[region];
            }
        }
        let time_count = time_counts[level].iter().sum::<usize>();
        let frequency_count = frequency_counts[level].iter().sum::<usize>();
        closure[0] = closure[0].max(usize::from(time_count != frequency_count) as f64);
        closure[2] = closure[2].max(usize::from(frequency_count != time_count) as f64);
        closure[1] = closure[1].max(sum_closure(&time_energies[level], parents[level][0]));
        closure[1] = closure[1].max(sum_closure(&time_alpha[level], parents[level][1]));
        closure[3] = closure[3].max(sum_closure(&frequency_energies[level], parents[level][0]));
        closure[3] = closure[3].max(sum_closure(&frequency_alpha[level], parents[level][1]));
        let entropy = entropy(parents[level][0], parents[level][1], LENGTHS[level] / 4);
        *baseline_drift +=
            usize::from(parents[level][0].to_bits() != baseline_energies[level].to_bits());
        *baseline_drift += usize::from(entropy.to_bits() != baseline_entropies[level].to_bits());
        structural[0] += usize::from(!entropy.is_finite());
    }
    let time_removals = std::array::from_fn(|region| {
        removal(region, &time_energies, &time_alpha, &parents, structural)
    });
    let frequency_removals = std::array::from_fn(|region| {
        removal(
            region,
            &frequency_energies,
            &frequency_alpha,
            &parents,
            structural,
        )
    });
    AnchorEvidence {
        anchor,
        time_counts,
        time_energies,
        time_alpha_sums: time_alpha,
        frequency_counts,
        frequency_energies,
        frequency_alpha_sums: frequency_alpha,
        time_removals,
        frequency_removals,
    }
}

fn removal(
    region: usize,
    energies: &[[f64; REGIONS]; 4],
    alpha_sums: &[[f64; REGIONS]; 4],
    parents: &[[f64; 2]; 4],
    structural: &mut [usize; 2],
) -> RemovalEvidence {
    let mut removed_entropies = [0.0; 4];
    let mut deltas = [0.0; 4];
    let mut energy_fractions = [0.0; 4];
    let mut alpha_fractions = [0.0; 4];
    for level in 0..4 {
        let parent_energy = parents[level][0];
        let parent_alpha = parents[level][1];
        let remaining_energy = parent_energy - energies[level][region];
        let remaining_alpha = parent_alpha - alpha_sums[level][region];
        let tolerance = parent_energy.abs().max(parent_alpha.abs()) * 1.0e-12;
        structural[1] += usize::from(remaining_energy < -tolerance || remaining_alpha < -tolerance);
        removed_entropies[level] = entropy(
            remaining_energy.max(0.0),
            remaining_alpha.max(0.0),
            LENGTHS[level] / 4,
        );
        deltas[level] =
            removed_entropies[level] - entropy(parent_energy, parent_alpha, LENGTHS[level] / 4);
        if parent_energy > 0.0 {
            energy_fractions[level] = energies[level][region] / parent_energy;
            alpha_fractions[level] = alpha_sums[level][region] / parent_alpha;
        }
        structural[0] += usize::from(
            !removed_entropies[level].is_finite()
                || !energy_fractions[level].is_finite()
                || !alpha_fractions[level].is_finite(),
        );
    }
    RemovalEvidence {
        entropy_deltas: deltas,
        energy_fractions,
        alpha_fractions,
        raw_winner: longest_minimum(&removed_entropies),
    }
}

fn entropy(energy: f64, alpha_sum: f64, hop: usize) -> f64 {
    if energy == 0.0 {
        0.0
    } else {
        (alpha_sum / energy.powf(ALPHA)).log2() / (1.0 - ALPHA) + (hop as f64 / FFT as f64).log2()
    }
}

fn sum_closure(values: &[f64; REGIONS], parent: f64) -> f64 {
    (values.iter().sum::<f64>() - parent).abs() / parent.abs().max(f64::MIN_POSITIVE)
}

fn control_hash(control: &ControlEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, control.control as u64);
    for anchor in &control.anchors {
        hash_u64(&mut hash, anchor.anchor as u64);
        for rows in [
            &anchor.time_energies,
            &anchor.time_alpha_sums,
            &anchor.frequency_energies,
            &anchor.frequency_alpha_sums,
        ] {
            for row in rows {
                for value in row {
                    hash_f64(&mut hash, *value);
                }
            }
        }
        for removals in [&anchor.time_removals, &anchor.frequency_removals] {
            for removal in removals {
                hash_u64(&mut hash, u64::from(removal.raw_winner));
                for value in removal.entropy_deltas {
                    hash_f64(&mut hash, value);
                }
            }
        }
    }
    hash
}

fn review_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, review.baseline.evidence_hash);
    for control in &review.controls {
        hash_u64(&mut hash, control.evidence_hash);
    }
    for value in review
        .diagnostic_counts
        .into_iter()
        .chain(review.candidate_counts)
        .chain(review.geometry_effects)
        .chain(review.frequency_event_restorations)
        .chain(review.frequency_negative_changes)
        .chain(review.linear_chirp_changes.into_iter().flatten())
    {
        hash_u64(&mut hash, value as u64);
    }
    hash
}
