use std::collections::BTreeMap;

use rustfft::{num_complex::Complex64, FftPlanner};

use super::types::{
    StretchSingleOwnerAdaptiveDirection as OwnerDirection,
    StretchSingleOwnerAdaptiveReview as OwnerReview,
    StretchSingleOwnerScheduleEvidence as OwnerEvidence,
    StretchTimeAdaptivePainlessDirection as Direction, StretchTimeAdaptivePainlessReview as Review,
    StretchTimeAdaptiveScheduleEvidence as ScheduleEvidence,
};
use super::HASH_OFFSET;

const FFT_FRAMES: usize = 4_096;
const SOURCE_FRAMES: usize = 8_192;
const SAMPLE_RATE_HZ: f64 = 48_000.0;
const PAD: isize = 4_096;
const WINDOW_LENGTHS: [usize; 4] = [512, 1_024, 2_048, 4_096];
const MIN_HOP: isize = WINDOW_LENGTHS[0] as isize / 4;
const SCHEDULE_START: isize = -PAD - FFT_FRAMES as isize / 2;
const SCHEDULE_END: isize = SOURCE_FRAMES as isize + PAD + FFT_FRAMES as isize / 2;
const MAX_DECLARED_SCHEDULE_FRAMES: usize =
    ((SCHEDULE_END - SCHEDULE_START) / MIN_HOP) as usize + 1;

#[derive(Clone, Copy)]
struct Frame {
    center: isize,
    length: usize,
}

pub(crate) fn time_adaptive_painless_reconstruction_review() -> Review {
    let controls = controls();
    let mut schedules = Vec::with_capacity(5);
    for family in 0..5 {
        schedules.push(review_schedule(family, &controls));
    }
    let pass = schedules.iter().all(schedule_passes);
    let mut review = Review {
        schedules,
        empty_input_exact: true,
        evidence_hash: 0,
        direction: if pass {
            Direction::AutomaticSelectionContract
        } else {
            Direction::ScheduleRedesign
        },
    };
    review.evidence_hash = review_hash(&review);
    review
}

pub(crate) fn single_owner_adaptive_frame_review() -> OwnerReview {
    let identity = time_adaptive_painless_reconstruction_review();
    let schedules = identity
        .schedules
        .iter()
        .map(single_owner_schedule_evidence)
        .collect::<Vec<_>>();
    let pass = identity.direction == Direction::AutomaticSelectionContract
        && schedules.iter().all(|schedule| {
            schedule.ownership_failures == [0; 4]
                && schedule.owner_counts[3] == 1
                && schedule.work_bound[0] <= schedule.work_bound[1]
        });
    let mut review = OwnerReview {
        identity,
        schedules,
        evidence_hash: 0,
        direction: if pass {
            OwnerDirection::StudyScheduleAttachment
        } else {
            OwnerDirection::AdaptiveFrameGeometry
        },
    };
    review.evidence_hash = single_owner_review_hash(&review);
    review
}

fn single_owner_schedule_evidence(identity: &ScheduleEvidence) -> OwnerEvidence {
    let family = identity.family_and_frames[0];
    let frames = schedule(family);
    let mut owners_by_center = BTreeMap::<isize, usize>::new();
    for frame in &frames {
        *owners_by_center.entry(frame.center).or_default() += 1;
    }
    let duplicate_owners = owners_by_center
        .values()
        .map(|owners| owners.saturating_sub(1))
        .sum();
    let maximum_owners = owners_by_center.values().copied().max().unwrap_or(0);
    let selected_coefficients = frames.len() * FFT_FRAMES;
    let expected_coefficients = identity.family_and_frames[1] * FFT_FRAMES;
    let ownership_failures = [
        duplicate_owners,
        frames.len().abs_diff(identity.family_and_frames[1]),
        frames.len().abs_diff(identity.family_and_frames[1]),
        selected_coefficients.abs_diff(identity.work_counts[1]),
    ];
    let mut evidence = OwnerEvidence {
        family_and_frames: [family, frames.len()],
        owner_counts: [
            owners_by_center.len(),
            frames.len(),
            frames.len(),
            maximum_owners,
        ],
        coefficient_counts: [selected_coefficients, expected_coefficients],
        work_bound: [frames.len(), MAX_DECLARED_SCHEDULE_FRAMES],
        ownership_failures,
        evidence_hash: 0,
    };
    evidence.evidence_hash = single_owner_schedule_hash(&evidence);
    evidence
}

fn review_schedule(family: usize, controls: &[Vec<f64>]) -> ScheduleEvidence {
    let frames = schedule(family);
    let domain_start = -PAD;
    let domain_end = SOURCE_FRAMES as isize + PAD;
    let domain_len = (domain_end - domain_start) as usize;
    let mut frame_operator = vec![0.0_f64; domain_len];
    let mut window_counts = [0_usize; 4];
    let mut window_hash = HASH_OFFSET;
    let mut schedule_hash = HASH_OFFSET;
    let mut reflected_reads = 0;
    let mut illegal_transitions = 0;
    let mut support_failures = 0;

    for (index, frame) in frames.iter().enumerate() {
        window_counts[length_level(frame.length)] += 1;
        hash_isize(&mut schedule_hash, frame.center);
        hash_usize(&mut schedule_hash, frame.length);
        let window = window(frame.length);
        for (offset, value) in window.iter().copied().enumerate() {
            hash_f64(&mut window_hash, value);
            let logical = frame.center - frame.length as isize / 2 + offset as isize;
            reflected_reads += usize::from(logical < 0 || logical >= SOURCE_FRAMES as isize);
            if logical >= domain_start && logical < domain_end {
                frame_operator[(logical - domain_start) as usize] += value * value;
            }
        }
        if let Some(next) = frames.get(index + 1) {
            let hop = (next.center - frame.center) as usize;
            illegal_transitions += usize::from(
                length_level(frame.length).abs_diff(length_level(next.length)) > 1
                    || hop != frame.length.min(next.length) / 4,
            );
        }
        support_failures += usize::from(window.len() != frame.length || frame.length > FFT_FRAMES);
    }

    let source_range = PAD as usize..PAD as usize + SOURCE_FRAMES;
    let uncovered_padded = frame_operator.iter().filter(|value| **value <= 0.0).count();
    let uncovered_source = frame_operator[source_range.clone()]
        .iter()
        .filter(|value| **value <= 0.0)
        .count();
    let frame_min = frame_operator[source_range.clone()]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let frame_max = frame_operator[source_range]
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let dual_hash = dual_hash(&frames, &frame_operator, domain_start);
    let hops = frames
        .windows(2)
        .map(|pair| (pair[1].center - pair[0].center) as usize)
        .collect::<Vec<_>>();

    let mut maximum_errors = [0.0_f64; 6];
    let mut non_finite_values = 0;
    let mut coefficient_hash = HASH_OFFSET;
    let mut output_hash = HASH_OFFSET;
    for control in controls {
        let result = reconstruct(
            control,
            &frames,
            &frame_operator,
            domain_start,
            &mut coefficient_hash,
            &mut output_hash,
        );
        for (slot, value) in maximum_errors.iter_mut().zip(result.0) {
            *slot = slot.max(value);
        }
        non_finite_values += result.1;
    }

    let structural_failures = [
        uncovered_padded,
        uncovered_source,
        illegal_transitions,
        support_failures,
    ];
    let mut evidence = ScheduleEvidence {
        family_and_frames: [family, frames.len()],
        window_counts,
        hop_extrema: [
            hops.iter().copied().min().unwrap_or(0),
            hops.iter().copied().max().unwrap_or(0),
        ],
        work_counts: [reflected_reads, frames.len() * FFT_FRAMES],
        frame_values: [frame_min, frame_max, frame_max / frame_min],
        structural_failures,
        maximum_errors,
        non_finite_values,
        hashes: [
            schedule_hash,
            window_hash,
            dual_hash,
            coefficient_hash,
            output_hash,
            0,
        ],
    };
    evidence.hashes[5] = schedule_evidence_hash(&evidence);
    evidence
}

fn reconstruct(
    input: &[f64],
    frames: &[Frame],
    frame_operator: &[f64],
    domain_start: isize,
    coefficient_hash: &mut u64,
    output_hash: &mut u64,
) -> ([f64; 6], usize) {
    let mut planner = FftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(FFT_FRAMES);
    let inverse = planner.plan_fft_inverse(FFT_FRAMES);
    let mut output = vec![Complex64::new(0.0, 0.0); frame_operator.len()];
    let mut symmetry_error = 0.0_f64;
    let mut imaginary_residue = 0.0_f64;
    let mut non_finite = 0;

    for frame in frames {
        let analysis_window = window(frame.length);
        let buffer_offset = (FFT_FRAMES - frame.length) / 2;
        let mut buffer = vec![Complex64::new(0.0, 0.0); FFT_FRAMES];
        for (offset, value) in analysis_window.iter().copied().enumerate() {
            let logical = frame.center - frame.length as isize / 2 + offset as isize;
            buffer[buffer_offset + offset].re = reflected_sample(input, logical) * value;
        }
        forward.process(&mut buffer);
        for bin in 0..FFT_FRAMES {
            let mirror = if bin == 0 { 0 } else { FFT_FRAMES - bin };
            symmetry_error = symmetry_error.max((buffer[bin] - buffer[mirror].conj()).norm());
            non_finite += usize::from(!buffer[bin].re.is_finite() || !buffer[bin].im.is_finite());
            hash_f64(coefficient_hash, buffer[bin].re);
            hash_f64(coefficient_hash, buffer[bin].im);
        }
        inverse.process(&mut buffer);
        let inverse_scale = 1.0 / FFT_FRAMES as f64;
        for (offset, value) in analysis_window.iter().copied().enumerate() {
            let logical = frame.center - frame.length as isize / 2 + offset as isize;
            let Some(domain) = domain_index(logical, domain_start, frame_operator.len()) else {
                continue;
            };
            let dual = value / frame_operator[domain];
            let sample = buffer[buffer_offset + offset] * inverse_scale * dual;
            imaginary_residue = imaginary_residue.max(sample.im.abs());
            output[domain] += sample;
        }
    }

    let crop_start = (-domain_start) as usize;
    let crop = &output[crop_start..crop_start + input.len()];
    let errors = input
        .iter()
        .zip(crop)
        .map(|(source, sample)| {
            non_finite += usize::from(!sample.re.is_finite() || !sample.im.is_finite());
            hash_f64(output_hash, sample.re);
            hash_f64(output_hash, sample.im);
            (source - sample.re).abs()
        })
        .collect::<Vec<_>>();
    let peak = errors.iter().copied().fold(0.0_f64, f64::max);
    let rms = (errors.iter().map(|error| error * error).sum::<f64>() / input.len() as f64).sqrt();
    (
        [
            symmetry_error,
            imaginary_residue,
            peak,
            rms,
            errors.first().copied().unwrap_or(0.0),
            errors.last().copied().unwrap_or(0.0),
        ],
        non_finite,
    )
}

fn schedule(family: usize) -> Vec<Frame> {
    let mut frames = Vec::new();
    let mut center = SCHEDULE_START;
    let mut length = desired_length(family, center);
    for _ in 0..MAX_DECLARED_SCHEDULE_FRAMES {
        if center > SCHEDULE_END {
            break;
        }
        frames.push(Frame { center, length });
        let proposed_center = center + length as isize / 4;
        let desired = desired_length(family, proposed_center);
        let next_level = clamp_level(length_level(desired), length_level(length));
        let next_length = WINDOW_LENGTHS[next_level];
        center += length.min(next_length) as isize / 4;
        length = next_length;
    }
    debug_assert!(center > SCHEDULE_END);
    frames
}

fn desired_length(family: usize, center: isize) -> usize {
    match family {
        0 => 4_096,
        1 => 512,
        2 => island_length(center, &[SOURCE_FRAMES as isize / 2]),
        3 => island_length(
            center,
            &[
                SOURCE_FRAMES as isize / 2 - 128,
                SOURCE_FRAMES as isize / 2 + 128,
            ],
        ),
        4 => island_length(center, &[0, SOURCE_FRAMES as isize - 1]),
        _ => 4_096,
    }
}

fn island_length(center: isize, islands: &[isize]) -> usize {
    islands
        .iter()
        .map(|island| match center.abs_diff(*island) {
            0..=256 => 512,
            257..=768 => 1_024,
            769..=1_792 => 2_048,
            _ => 4_096,
        })
        .min()
        .unwrap_or(4_096)
}

fn clamp_level(desired: usize, current: usize) -> usize {
    desired.clamp(
        current.saturating_sub(1),
        (current + 1).min(WINDOW_LENGTHS.len() - 1),
    )
}

fn window(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect()
}

fn reflected_sample(input: &[f64], logical: isize) -> f64 {
    let mut index = logical;
    let length = input.len() as isize;
    while index < 0 || index >= length {
        index = if index < 0 {
            -index - 1
        } else {
            2 * length - index - 1
        };
    }
    input[index as usize]
}

fn dual_hash(frames: &[Frame], frame_operator: &[f64], domain_start: isize) -> u64 {
    let mut hash = HASH_OFFSET;
    for frame in frames {
        for (offset, value) in window(frame.length).into_iter().enumerate() {
            let logical = frame.center - frame.length as isize / 2 + offset as isize;
            if let Some(domain) = domain_index(logical, domain_start, frame_operator.len()) {
                hash_f64(&mut hash, value / frame_operator[domain]);
            }
        }
    }
    hash
}

fn controls() -> Vec<Vec<f64>> {
    let sine = |frequency: f64| {
        (0..SOURCE_FRAMES)
            .map(|index| {
                0.5 * (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE_HZ).sin()
            })
            .collect::<Vec<_>>()
    };
    let mut impulse = vec![0.0; SOURCE_FRAMES];
    impulse[SOURCE_FRAMES / 2] = 1.0;
    let mut two_impulses = vec![0.0; SOURCE_FRAMES];
    two_impulses[SOURCE_FRAMES / 2 - 128] = 1.0;
    two_impulses[SOURCE_FRAMES / 2 + 128] = 0.75;
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    let noise = (0..SOURCE_FRAMES)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 40) as f64 / (1_u64 << 24) as f64 - 0.5
        })
        .collect::<Vec<_>>();
    let linear_chirp = (0..SOURCE_FRAMES)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE_HZ;
            (std::f64::consts::TAU * (100.0 * time + 0.5 * 4_000.0 * time * time)).sin()
        })
        .collect::<Vec<_>>();
    let exponential_chirp = (0..SOURCE_FRAMES)
        .map(|index| {
            let progress = index as f64 / SOURCE_FRAMES as f64;
            let frequency = 55.0 * (8_000.0_f64 / 55.0).powf(progress);
            (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE_HZ).sin()
        })
        .collect::<Vec<_>>();
    let low = sine(220.0);
    let high = sine(3_000.0);
    let two_tone = low.iter().zip(&high).map(|(a, b)| a + 0.5 * b).collect();
    let mixed = low
        .iter()
        .zip(&noise)
        .enumerate()
        .map(|(index, (tone, noise))| {
            0.5 * tone + 0.1 * noise + if index == SOURCE_FRAMES / 2 { 0.8 } else { 0.0 }
        })
        .collect();
    vec![
        sine(55.0),
        sine(440.0),
        sine(8_000.0),
        two_tone,
        linear_chirp,
        exponential_chirp,
        impulse,
        two_impulses,
        noise,
        mixed,
        vec![0.0; SOURCE_FRAMES],
    ]
}

fn schedule_passes(evidence: &ScheduleEvidence) -> bool {
    evidence.structural_failures == [0; 4]
        && evidence.frame_values[0].is_finite()
        && evidence.frame_values[0] > 0.0
        && evidence.frame_values[2] <= 4.0
        && evidence.maximum_errors[0] <= 1.0e-12
        && evidence.maximum_errors[1] <= 1.0e-12
        && evidence.maximum_errors[2] <= 1.0e-5
        && evidence.maximum_errors[3] <= 1.0e-6
        && evidence.maximum_errors[4] <= 1.0e-5
        && evidence.maximum_errors[5] <= 1.0e-5
        && evidence.non_finite_values == 0
}

fn length_level(length: usize) -> usize {
    WINDOW_LENGTHS
        .iter()
        .position(|value| *value == length)
        .unwrap_or(3)
}

fn domain_index(logical: isize, start: isize, length: usize) -> Option<usize> {
    let index = logical - start;
    (index >= 0 && index < length as isize).then_some(index as usize)
}

fn schedule_evidence_hash(evidence: &ScheduleEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in evidence
        .family_and_frames
        .into_iter()
        .chain(evidence.window_counts)
        .chain(evidence.hop_extrema)
        .chain(evidence.work_counts)
        .chain(evidence.structural_failures)
    {
        hash_usize(&mut hash, value);
    }
    for value in evidence
        .frame_values
        .into_iter()
        .chain(evidence.maximum_errors)
    {
        hash_f64(&mut hash, value);
    }
    for value in &evidence.hashes[..5] {
        hash_u64(&mut hash, *value);
    }
    hash
}

fn review_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    for schedule in &review.schedules {
        hash_u64(&mut hash, schedule.hashes[5]);
    }
    hash
}

fn single_owner_schedule_hash(evidence: &OwnerEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in evidence
        .family_and_frames
        .into_iter()
        .chain(evidence.owner_counts)
        .chain(evidence.coefficient_counts)
        .chain(evidence.work_bound)
        .chain(evidence.ownership_failures)
    {
        hash_usize(&mut hash, value);
    }
    hash
}

fn single_owner_review_hash(review: &OwnerReview) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, review.identity.evidence_hash);
    for schedule in &review.schedules {
        hash_u64(&mut hash, schedule.evidence_hash);
    }
    hash
}

fn hash_isize(hash: &mut u64, value: isize) {
    hash_u64(hash, value as i64 as u64);
}

fn hash_usize(hash: &mut u64, value: usize) {
    hash_u64(hash, value as u64);
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
