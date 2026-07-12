use super::super::types::{
    StretchRenyiControlEvidence as ControlEvidence, StretchRenyiGeometryDirection as Direction,
    StretchRenyiGeometryReview as Review,
};
use super::controls::{controls, perturbed, Kind, FRAMES};
use super::{
    byte_hash, control_hash, gates, hash_f64, hash_u64, input_hash, longest_minimum, path,
    path_change, ALPHA, ANCHOR_HOP, FFT, HASH_OFFSET, LENGTHS,
};

mod measure;
use measure::measure;

const EXPECTED_COUNTS: [usize; 4] = [29, 13, 5, 1];

struct Selection {
    evidence: ControlEvidence,
    geometry_failures: [usize; 2],
    membership_hash: u64,
}

pub(crate) fn renyi_anchor_local_geometry_review() -> Review {
    let controls = controls();
    let mut evidence = Vec::with_capacity(controls.len());
    let mut geometry_failures = [0; 2];
    let mut membership_hash = HASH_OFFSET;
    let mut perturbation_max = 0.0_f64;
    let mut perturbation_changes = Vec::with_capacity(controls.len());
    let mut equivalence_failures = 0;
    for (index, control) in controls.iter().enumerate() {
        let base = select(index, &[control.samples.as_slice()]);
        for slot in 0..2 {
            geometry_failures[slot] += base.geometry_failures[slot];
        }
        hash_u64(&mut membership_hash, base.membership_hash);
        let mut perturbation_change = 0.0;
        if control.kind != Kind::Silence {
            for scale in [0.25, 4.0, -1.0] {
                let variant = control
                    .samples
                    .iter()
                    .map(|sample| sample * scale)
                    .collect::<Vec<_>>();
                equivalence_failures += usize::from(
                    select(index, &[variant.as_slice()])
                        .evidence
                        .selected_levels
                        != base.evidence.selected_levels,
                );
            }
            let changed = select(index, &[perturbed(&control.samples).as_slice()]);
            perturbation_change = path_change(
                &base.evidence.selected_levels,
                &changed.evidence.selected_levels,
            );
            perturbation_max = perturbation_max.max(perturbation_change);
        }
        perturbation_changes.push(perturbation_change);
        let silent = vec![0.0; FRAMES];
        let stereo = select(index, &[control.samples.as_slice(), silent.as_slice()]);
        let swapped = select(index, &[silent.as_slice(), control.samples.as_slice()]);
        let split = control
            .samples
            .iter()
            .map(|sample| sample * std::f64::consts::FRAC_1_SQRT_2)
            .collect::<Vec<_>>();
        let centered = select(index, &[split.as_slice(), split.as_slice()]);
        equivalence_failures +=
            usize::from(stereo.evidence.selected_levels != base.evidence.selected_levels);
        equivalence_failures +=
            usize::from(swapped.evidence.selected_levels != base.evidence.selected_levels);
        equivalence_failures +=
            usize::from(centered.evidence.selected_levels != base.evidence.selected_levels);
        evidence.push(base.evidence);
    }
    let gate_failures = gates(&controls, &evidence, equivalence_failures, perturbation_max);
    let direction = if geometry_failures == [0; 2] && gate_failures == [0; 7] {
        Direction::VariableHopPhaseContract
    } else {
        Direction::OperatorReview
    };
    let mut review = Review {
        controls: evidence,
        support_extrema: std::array::from_fn(|level| {
            let radius = (FFT - LENGTHS[level]) as isize / 2;
            [-radius, radius]
        }),
        geometry_failures,
        gate_failures,
        maximum_perturbation_change: perturbation_max,
        perturbation_changes,
        equivalence_failures,
        membership_hash,
        evidence_hash: 0,
        direction,
    };
    review.evidence_hash = review_hash(&review);
    review
}

fn select(control: usize, channels: &[&[f64]]) -> Selection {
    let anchors = (0..FRAMES).step_by(ANCHOR_HOP).collect::<Vec<_>>();
    let mut energies = vec![[0.0; 4]; anchors.len()];
    let mut entropies = vec![[0.0; 4]; anchors.len()];
    let mut membership = vec![[0; 4]; anchors.len()];
    let mut reflected_reads = 0;
    let mut non_finite = 0;
    let mut channel_closure = 0.0_f64;
    let mut entropy_hash = HASH_OFFSET;
    let mut membership_hash = HASH_OFFSET;
    let mut support_escapes = 0;
    for (level, length) in LENGTHS.into_iter().enumerate() {
        let hop = length / 4;
        let radius = (FFT - length) as isize / 2;
        let (frames, reads, closure, invalid) = measure(
            channels,
            length,
            -radius,
            (FRAMES - ANCHOR_HOP) as isize + radius,
        );
        reflected_reads += reads;
        channel_closure = channel_closure.max(closure);
        non_finite += invalid;
        for (anchor_index, anchor) in anchors.iter().copied().enumerate() {
            for frame in frames.iter().filter(|frame| {
                let offset = frame.center - anchor as isize;
                offset.abs() <= radius && offset.rem_euclid(hop as isize) == 0
            }) {
                let support_start = frame.center - length as isize / 2;
                let support_end = frame.center + length as isize / 2;
                let region_start = anchor as isize - FFT as isize / 2;
                let region_end = anchor as isize + FFT as isize / 2;
                support_escapes +=
                    usize::from(support_start < region_start || support_end > region_end);
                energies[anchor_index][level] += frame.energy;
                entropies[anchor_index][level] += frame.alpha_sum;
                membership[anchor_index][level] += 1;
            }
            let energy = energies[anchor_index][level];
            entropies[anchor_index][level] = if energy == 0.0 {
                0.0
            } else {
                (entropies[anchor_index][level] / energy.powf(ALPHA)).log2() / (1.0 - ALPHA)
                    + (hop as f64 / FFT as f64).log2()
            };
            hash_f64(&mut entropy_hash, energy);
            hash_f64(&mut entropy_hash, entropies[anchor_index][level]);
            hash_u64(&mut membership_hash, membership[anchor_index][level] as u64);
            non_finite += usize::from(!entropies[anchor_index][level].is_finite());
        }
    }
    let membership_failures = membership
        .iter()
        .filter(|counts| **counts != EXPECTED_COUNTS)
        .count();
    let raw_winners = entropies.iter().map(longest_minimum).collect::<Vec<_>>();
    let (selected_levels, path_cost) = path::solve(&entropies);
    let mut counts = [0; 4];
    for level in &selected_levels {
        counts[*level as usize] += 1;
    }
    let transitions = selected_levels
        .windows(2)
        .filter(|pair| pair[0] != pair[1])
        .count();
    let hops = selected_levels
        .windows(2)
        .map(|pair| LENGTHS[pair[0] as usize].min(LENGTHS[pair[1] as usize]) / 4)
        .collect::<Vec<_>>();
    let mut evidence = ControlEvidence {
        control,
        raw_winners,
        selected_levels,
        energies,
        entropies,
        level_counts: counts,
        path_shape: [
            transitions,
            hops.iter().copied().min().unwrap_or(0),
            hops.iter().copied().max().unwrap_or(0),
        ],
        structural_counts: [reflected_reads, non_finite],
        channel_energy_closure: channel_closure,
        path_cost,
        hashes: [input_hash(channels), entropy_hash, 0, 0],
    };
    evidence.hashes[2] = byte_hash(&evidence.selected_levels);
    evidence.hashes[3] = control_hash(&evidence);
    Selection {
        evidence,
        geometry_failures: [membership_failures, support_escapes],
        membership_hash,
    }
}

fn review_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    for control in &review.controls {
        hash_u64(&mut hash, control.hashes[3]);
    }
    for value in review
        .geometry_failures
        .into_iter()
        .chain(review.gate_failures)
    {
        hash_u64(&mut hash, value as u64);
    }
    hash_f64(&mut hash, review.maximum_perturbation_change);
    for value in &review.perturbation_changes {
        hash_f64(&mut hash, *value);
    }
    hash_u64(&mut hash, review.equivalence_failures as u64);
    hash_u64(&mut hash, review.membership_hash);
    hash
}
