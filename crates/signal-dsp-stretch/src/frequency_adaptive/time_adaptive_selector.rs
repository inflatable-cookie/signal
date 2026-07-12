use rustfft::{num_complex::Complex64, FftPlanner};

use super::types::{
    StretchRenyiControlEvidence as ControlEvidence, StretchRenyiSelectorDirection as Direction,
    StretchRenyiSelectorReview as Review,
};
use super::HASH_OFFSET;

mod attribution;
mod controls;
mod geometry;
mod path;
mod transient;
use controls::{controls, perturbed, Kind, FRAMES};

pub(crate) use attribution::renyi_attribution_reassessment_review;
pub(crate) use attribution::renyi_selector_failure_attribution_review;
pub(crate) use geometry::renyi_anchor_local_geometry_review;
pub(crate) use transient::median_hpss_evidence_review;
pub(crate) use transient::mixed_phase_distribution_review;
pub(crate) use transient::transient_evidence_measurement_review;

const FFT: usize = 4_096;
const ANCHOR_HOP: usize = 128;
const ALPHA: f64 = 0.7;
const LENGTHS: [usize; 4] = [512, 1_024, 2_048, 4_096];

struct Selection {
    evidence: ControlEvidence,
}

pub(crate) fn renyi_time_resolution_selection_review() -> Review {
    let controls = controls();
    let mut evidence = Vec::with_capacity(controls.len());
    let mut perturbation_max = 0.0_f64;
    let mut equivalence_failures = 0;
    for (index, control) in controls.iter().enumerate() {
        let base = select(index, &[control.samples.as_slice()]);
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
            perturbation_max = perturbation_max.max(path_change(
                &base.evidence.selected_levels,
                &changed.evidence.selected_levels,
            ));
        }
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
    let mut review = Review {
        controls: evidence,
        gate_failures,
        maximum_perturbation_change: perturbation_max,
        evidence_hash: 0,
        direction: if gate_failures == [0; 7] {
            Direction::VariableHopPhaseContract
        } else {
            Direction::SelectorResearch
        },
    };
    review.evidence_hash = review_hash(&review);
    review
}

fn select(control: usize, channels: &[&[f64]]) -> Selection {
    let anchors = (0..FRAMES).step_by(ANCHOR_HOP).collect::<Vec<_>>();
    let mut energies = vec![[0.0; 4]; anchors.len()];
    let mut entropies = vec![[0.0; 4]; anchors.len()];
    let mut reflected_reads = 0;
    let mut non_finite = 0;
    let mut entropy_hash = HASH_OFFSET;
    let mut channel_closure = 0.0_f64;

    for (level, length) in LENGTHS.into_iter().enumerate() {
        let hop = length / 4;
        let (frames, reads, closure, invalid) = spectral_frames(channels, length, hop);
        reflected_reads += reads;
        channel_closure = channel_closure.max(closure);
        non_finite += invalid;
        for (anchor_index, anchor) in anchors.iter().copied().enumerate() {
            let start = anchor as isize - FFT as isize / 2;
            let end = anchor as isize + FFT as isize / 2;
            let mut energy = 0.0;
            let mut alpha_sum = 0.0;
            for frame in frames
                .iter()
                .filter(|frame| frame.0 >= start && frame.0 < end)
            {
                energy += frame.1;
                alpha_sum += frame.2;
            }
            energies[anchor_index][level] = energy;
            entropies[anchor_index][level] = if energy == 0.0 {
                0.0
            } else {
                (alpha_sum / energy.powf(ALPHA)).log2() / (1.0 - ALPHA)
                    + (hop as f64 / FFT as f64).log2()
            };
            hash_f64(&mut entropy_hash, energies[anchor_index][level]);
            hash_f64(&mut entropy_hash, entropies[anchor_index][level]);
            non_finite += usize::from(!entropies[anchor_index][level].is_finite());
        }
    }
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
    let input_hash = input_hash(channels);
    let path_hash = byte_hash(&selected_levels);
    let mut result = ControlEvidence {
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
        hashes: [input_hash, entropy_hash, path_hash, 0],
    };
    result.hashes[3] = control_hash(&result);
    Selection { evidence: result }
}

fn spectral_frames(
    channels: &[&[f64]],
    length: usize,
    hop: usize,
) -> (Vec<(isize, f64, f64)>, usize, f64, usize) {
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(FFT);
    let window = window(length);
    let mut result = Vec::new();
    let mut reflected_reads = 0;
    let mut closure = 0.0_f64;
    let mut invalid = 0;
    for center in ((-(FFT as isize) / 2)..(FRAMES as isize + FFT as isize / 2)).step_by(hop) {
        let mut spectra = Vec::with_capacity(channels.len());
        for channel in channels {
            let mut buffer = vec![Complex64::new(0.0, 0.0); FFT];
            let offset = (FFT - length) / 2;
            for (index, weight) in window.iter().copied().enumerate() {
                let logical = center - length as isize / 2 + index as isize;
                reflected_reads += usize::from(logical < 0 || logical >= FRAMES as isize);
                buffer[offset + index].re = reflected(channel, logical) * weight;
            }
            fft.process(&mut buffer);
            spectra.push(buffer);
        }
        let mut energy = 0.0;
        let mut alpha_sum = 0.0;
        for bin in 0..FFT {
            let combined = spectra
                .iter()
                .map(|spectrum| spectrum[bin].norm_sqr())
                .sum::<f64>();
            let separate = spectra
                .iter()
                .map(|spectrum| spectrum[bin].norm_sqr())
                .sum::<f64>();
            closure = closure.max((combined - separate).abs() / combined.max(f64::MIN_POSITIVE));
            energy += combined;
            alpha_sum += combined.powf(ALPHA);
            invalid += usize::from(!combined.is_finite());
        }
        result.push((center, energy, alpha_sum));
    }
    (result, reflected_reads, closure, invalid)
}

fn gates(
    controls: &[controls::Control],
    evidence: &[ControlEvidence],
    equivalence: usize,
    perturbation: f64,
) -> [usize; 7] {
    let mut failures = [0; 7];
    for (control, report) in controls.iter().zip(evidence) {
        match control.kind {
            Kind::Silence | Kind::Steady => {
                failures[0] += usize::from(report.selected_levels.iter().any(|level| *level != 3));
            }
            Kind::Impulse => {
                failures[1] += usize::from(!event_gate(&report.selected_levels, FRAMES / 2));
            }
            Kind::DenseImpulses => {
                let left = (FRAMES / 2 - 128) / ANCHOR_HOP;
                let right = (FRAMES / 2 + 128) / ANCHOR_HOP;
                failures[2] += usize::from(report.selected_levels[left..=right].contains(&3));
            }
            Kind::BoundaryImpulses => {
                failures[3] += usize::from(
                    !report.selected_levels[..=2].contains(&0)
                        || !report.selected_levels[report.selected_levels.len() - 3..].contains(&0)
                        || report.structural_counts[0] == 0,
                );
            }
            Kind::Chirp => {
                failures[4] += usize::from(
                    report
                        .level_counts
                        .iter()
                        .filter(|count| **count > 0)
                        .count()
                        < 2,
                );
            }
            Kind::Noise => failures[4] += usize::from(report.level_counts[0] > 0),
            Kind::Mixed => {
                failures[4] += usize::from(
                    !event_gate(&report.selected_levels, FRAMES / 2)
                        || report.selected_levels[..16].iter().any(|level| *level != 3)
                        || report.selected_levels[48..].iter().any(|level| *level != 3),
                );
            }
        }
        failures[6] += usize::from(
            report.structural_counts[1] != 0
                || report.channel_energy_closure > 1.0e-12
                || report
                    .selected_levels
                    .windows(2)
                    .any(|pair| pair[0].abs_diff(pair[1]) > 1),
        );
    }
    failures[5] = equivalence + usize::from(perturbation > 0.05);
    failures
}

fn event_gate(path: &[u8], event: usize) -> bool {
    let anchor = event / ANCHOR_HOP;
    path[anchor.saturating_sub(2)..=(anchor + 2).min(path.len() - 1)].contains(&0)
        && path
            .iter()
            .enumerate()
            .all(|(index, level)| index.abs_diff(anchor) * ANCHOR_HOP <= 2_048 || *level == 3)
}

fn longest_minimum(row: &[f64; 4]) -> u8 {
    (0..4)
        .rev()
        .min_by(|left, right| row[*left].total_cmp(&row[*right]))
        .unwrap_or(3) as u8
}

fn window(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect()
}

fn reflected(input: &[f64], logical: isize) -> f64 {
    let mut index = logical;
    while index < 0 || index >= input.len() as isize {
        index = if index < 0 {
            -index - 1
        } else {
            2 * input.len() as isize - index - 1
        };
    }
    input[index as usize]
}

fn path_change(left: &[u8], right: &[u8]) -> f64 {
    left.iter().zip(right).filter(|(a, b)| a != b).count() as f64 / left.len() as f64
}

fn review_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    for control in &review.controls {
        hash_u64(&mut hash, control.hashes[3]);
    }
    for failure in review.gate_failures {
        hash_u64(&mut hash, failure as u64);
    }
    hash_f64(&mut hash, review.maximum_perturbation_change);
    hash
}

fn control_hash(control: &ControlEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in &control.hashes[..3] {
        hash_u64(&mut hash, *value);
    }
    for value in control.level_counts.into_iter().chain(control.path_shape) {
        hash_u64(&mut hash, value as u64);
    }
    hash_f64(&mut hash, control.path_cost);
    hash
}

fn input_hash(channels: &[&[f64]]) -> u64 {
    let mut hash = HASH_OFFSET;
    for channel in channels {
        for sample in *channel {
            hash_f64(&mut hash, *sample);
        }
    }
    hash
}

fn byte_hash(values: &[u8]) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in values {
        hash_u64(&mut hash, u64::from(*value));
    }
    hash
}

fn hash_f64(hash: &mut u64, value: f64) {
    hash_u64(hash, value.to_bits());
}
fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
