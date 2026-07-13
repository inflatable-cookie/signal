use std::f64::consts::TAU;

use crate::measure_tonal_texture;

use super::super::study_local_schedule::{
    schedule::{build_schedule, Schedule},
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::HASH_OFFSET;
use super::render::{render, Mode, Render};

const SAMPLE_RATE: f64 = 48_000.0;
const RATIOS: [f64; 4] = [1.0, 0.75, 1.5, 2.0];
const TIMING_SEARCH: usize = 512;
const DENSE_EXCLUSION: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum Control {
    LowTone,
    MidTone,
    HighTone,
    TwoTone,
    LinearChirp,
    ExponentialChirp,
    IsolatedImpulse,
    DenseEvent,
    Boundary,
    Noise,
    Mixed,
    Silence,
}

impl Control {
    fn tone_hz(self) -> Option<f64> {
        match self {
            Self::LowTone => Some(55.0),
            Self::MidTone => Some(440.0),
            Self::HighTone => Some(8_000.0),
            _ => None,
        }
    }

    fn texture(self) -> bool {
        matches!(
            self,
            Self::LowTone
                | Self::MidTone
                | Self::HighTone
                | Self::TwoTone
                | Self::LinearChirp
                | Self::ExponentialChirp
                | Self::Mixed
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum QualityDirection {
    FrozenMonoDevelopmentObjective,
    MeasuredPhaseEventVerticalOrSynthesisStage,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ModeEvidence {
    pub hard_failures: [usize; 10],
    pub lengths: [usize; 2],
    pub coverage: [usize; 2],
    pub assembly_actions: [usize; 2],
    pub frame_condition: f64,
    pub symmetry_error: f64,
    pub imaginary_residue: f64,
    pub non_finite_values: usize,
    pub identity_error: [f64; 4],
    pub endpoint_rms: [f64; 2],
    pub tone_angular_error: f64,
    pub isolated_error: usize,
    pub dense_errors: [usize; 2],
    pub dense_unmatched: usize,
    pub impulse_crest_db: f64,
    pub replica_ratio: f64,
    pub texture: [f64; 6],
    pub silence_peak: f64,
    pub phase_assignments: [usize; 2],
    pub hashes: [u64; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CaseEvidence {
    pub control: Control,
    pub ratio: f64,
    pub selected_points: usize,
    pub modes: [ModeEvidence; 2],
    pub mode_deltas: [f64; 6],
    pub ownership_failures: usize,
    pub combined_regressions: usize,
    pub hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct QualityReview {
    pub cases: Vec<CaseEvidence>,
    pub hard_failures: usize,
    pub combined_regressions: usize,
    pub evidence_hash: u64,
    pub direction: QualityDirection,
}

pub(in crate::frequency_adaptive) fn quality_review() -> QualityReview {
    let mut cases = Vec::with_capacity(controls().len() * RATIOS.len());
    for (control, input) in controls() {
        for ratio in RATIOS {
            cases.push(review_case(control, &input, ratio));
        }
    }
    let hard_failures = cases
        .iter()
        .map(|case| {
            case.ownership_failures
                + case
                    .modes
                    .iter()
                    .flat_map(|mode| mode.hard_failures)
                    .sum::<usize>()
        })
        .sum();
    let combined_regressions = cases.iter().map(|case| case.combined_regressions).sum();
    let direction = if hard_failures == 0 && combined_regressions == 0 {
        QualityDirection::FrozenMonoDevelopmentObjective
    } else {
        QualityDirection::MeasuredPhaseEventVerticalOrSynthesisStage
    };
    let mut evidence_hash = HASH_OFFSET;
    for case in &cases {
        hash(&mut evidence_hash, case.hash);
    }
    QualityReview {
        cases,
        hard_failures,
        combined_regressions,
        evidence_hash,
        direction,
    }
}

fn review_case(control: Control, input: &[f64], ratio: f64) -> CaseEvidence {
    let channels = [input.to_vec()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let renders = [
        render(&channels, ratio, &points, &schedule, Mode::Ordinary),
        render(&channels, ratio, &points, &schedule, Mode::Both),
    ];
    let modes = [
        measure(control, input, ratio, &schedule, &renders[0]),
        measure(control, input, ratio, &schedule, &renders[1]),
    ];
    let combined_regressions = modes[0]
        .hard_failures
        .iter()
        .zip(modes[1].hard_failures)
        .filter(|(ordinary, combined)| **ordinary == 0 && *combined != 0)
        .count();
    let ownership_failures = usize::from(
        renders[0].schedule_hash != renders[1].schedule_hash
            || renders[0].frame_hash != renders[1].frame_hash
            || renders[0].coefficient_hash != renders[1].coefficient_hash
            || renders[0].magnitude_hash != renders[1].magnitude_hash,
    );
    let mode_deltas = [
        modes[1].impulse_crest_db - modes[0].impulse_crest_db,
        modes[1].replica_ratio - modes[0].replica_ratio,
        modes[1].texture[0] - modes[0].texture[0],
        modes[1].texture[1] - modes[0].texture[1],
        modes[1].texture[4] - modes[0].texture[4],
        modes[1].texture[5] - modes[0].texture[5],
    ];
    let mut result = CaseEvidence {
        control,
        ratio,
        selected_points: points.len(),
        modes,
        mode_deltas,
        ownership_failures,
        combined_regressions,
        hash: 0,
    };
    result.hash = case_hash(&result);
    result
}

fn measure(
    control: Control,
    input: &[f64],
    ratio: f64,
    schedule: &Schedule,
    render: &Render,
) -> ModeEvidence {
    let output = &render.samples[0];
    let identity_error = if ratio == 1.0 {
        error(input, output)
    } else {
        [0.0; 4]
    };
    let tone_angular_error = control
        .tone_hz()
        .map(|hz| angular_frequency_error(output, hz))
        .unwrap_or(0.0);
    let isolated_error = if control == Control::IsolatedImpulse {
        let expected = projected(schedule, SOURCE_FRAMES / 2);
        peak_index(output, expected, TIMING_SEARCH).abs_diff(expected)
    } else {
        0
    };
    let (dense_errors, dense_unmatched) = if control == Control::DenseEvent {
        dense_event_errors(
            output,
            [
                projected(schedule, SOURCE_FRAMES / 2 - 128),
                projected(schedule, SOURCE_FRAMES / 2 + 128),
            ],
        )
    } else {
        ([0; 2], 0)
    };
    let impulse_center = match control {
        Control::IsolatedImpulse | Control::Mixed => Some(projected(schedule, SOURCE_FRAMES / 2)),
        _ => None,
    };
    let impulse_crest_db = impulse_center
        .map(|center| crest_db(output, center, 256))
        .unwrap_or(0.0);
    let replica_ratio = impulse_center
        .map(|center| replica_ratio(output, center))
        .unwrap_or(0.0);
    let texture = if control.texture() {
        texture(input, output, ratio)
    } else {
        [0.0; 6]
    };
    let silence_peak = if control == Control::Silence {
        peak(output)
    } else {
        0.0
    };
    let endpoint_rms = [rms_prefix(output, 256), rms_suffix(output, 256)];
    let target_len = (SOURCE_FRAMES as f64 * ratio).round() as usize;
    let hard_failures = [
        usize::from(render.target_len != target_len || output.len() != target_len),
        render.uncovered,
        render.boundary_failures,
        render.non_finite,
        usize::from(render.symmetry_error > 1.0e-9),
        usize::from(render.imaginary_residue > 1.0e-9),
        usize::from(
            ratio == 1.0
                && (identity_error[0] > 1.0e-5
                    || identity_error[1] > 1.0e-6
                    || identity_error[2] > 1.0e-5
                    || identity_error[3] > 1.0e-5),
        ),
        usize::from(control.tone_hz().is_some() && tone_angular_error > 1.0e-6),
        usize::from(control == Control::IsolatedImpulse && isolated_error > 1),
        usize::from(
            (control == Control::DenseEvent
                && (dense_unmatched != 0 || dense_errors.into_iter().any(|error| error > 256)))
                || (control == Control::Silence && silence_peak > 1.0e-12),
        ),
    ];
    let mut hashes = [
        render.coefficient_hash,
        render.magnitude_hash,
        render.phase_hash,
        render.output_hash,
    ];
    // The zero-fill and post-fade counts are structurally zero: this renderer
    // crops the covered diagonal-dual sum directly and has no such assembly path.
    hash(&mut hashes[2], 0);
    hash(&mut hashes[2], 0);
    ModeEvidence {
        hard_failures,
        lengths: [output.len(), target_len],
        coverage: [render.uncovered, render.covered],
        assembly_actions: [0, 0],
        frame_condition: render.frame_values[2],
        symmetry_error: render.symmetry_error,
        imaginary_residue: render.imaginary_residue,
        non_finite_values: render.non_finite,
        identity_error,
        endpoint_rms,
        tone_angular_error,
        isolated_error,
        dense_errors,
        dense_unmatched,
        impulse_crest_db,
        replica_ratio,
        texture,
        silence_peak,
        phase_assignments: [render.event_phase_changes, render.vertical_phase_changes],
        hashes,
    }
}

fn controls() -> Vec<(Control, Vec<f64>)> {
    vec![
        (Control::LowTone, tone(55.0)),
        (Control::MidTone, tone(440.0)),
        (Control::HighTone, tone(8_000.0)),
        (
            Control::TwoTone,
            (0..SOURCE_FRAMES)
                .map(|index| 0.6 * sinusoid(220.0, index) + 0.4 * sinusoid(3_000.0, index))
                .collect(),
        ),
        (
            Control::LinearChirp,
            chirp(|position| 55.0 + (8_000.0 - 55.0) * position),
        ),
        (
            Control::ExponentialChirp,
            chirp(|position| 55.0 * (8_000.0_f64 / 55.0).powf(position)),
        ),
        (
            Control::IsolatedImpulse,
            impulses(&[(SOURCE_FRAMES / 2, 1.0)]),
        ),
        (
            Control::DenseEvent,
            impulses(&[
                (SOURCE_FRAMES / 2 - 128, 1.0),
                (SOURCE_FRAMES / 2 + 128, 0.75),
            ]),
        ),
        (
            Control::Boundary,
            impulses(&[(0, 1.0), (SOURCE_FRAMES - 1, -0.75)]),
        ),
        (Control::Noise, noise()),
        (
            Control::Mixed,
            noise()
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    0.45 * sinusoid(220.0, index)
                        + 0.08 * value
                        + if index == SOURCE_FRAMES / 2 { 1.0 } else { 0.0 }
                })
                .collect(),
        ),
        (Control::Silence, vec![0.0; SOURCE_FRAMES]),
    ]
}

fn tone(hz: f64) -> Vec<f64> {
    (0..SOURCE_FRAMES)
        .map(|index| sinusoid(hz, index))
        .collect()
}

fn sinusoid(hz: f64, index: usize) -> f64 {
    (TAU * hz * index as f64 / SAMPLE_RATE).sin()
}

fn chirp(frequency: impl Fn(f64) -> f64) -> Vec<f64> {
    let mut phase = 0.0_f64;
    (0..SOURCE_FRAMES)
        .map(|index| {
            let sample = phase.sin();
            phase += TAU * frequency(index as f64 / SOURCE_FRAMES as f64) / SAMPLE_RATE;
            sample
        })
        .collect()
}

fn impulses(events: &[(usize, f64)]) -> Vec<f64> {
    let mut result = vec![0.0; SOURCE_FRAMES];
    for (index, amplitude) in events {
        result[*index] = *amplitude;
    }
    result
}

fn noise() -> Vec<f64> {
    let mut state = 0x8f3d_9b17_u32;
    (0..SOURCE_FRAMES)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as f64 / u32::MAX as f64 * 2.0 - 1.0
        })
        .collect()
}

fn error(input: &[f64], output: &[f64]) -> [f64; 4] {
    let differences = input
        .iter()
        .zip(output)
        .map(|(input, output)| (input - output).abs())
        .collect::<Vec<_>>();
    let peak = differences.iter().copied().fold(0.0_f64, f64::max);
    let rms = (differences.iter().map(|value| value * value).sum::<f64>()
        / differences.len() as f64)
        .sqrt();
    [
        peak,
        rms,
        differences[0],
        differences[differences.len() - 1],
    ]
}

fn angular_frequency_error(samples: &[f64], hz: f64) -> f64 {
    let window = 4_096;
    let first = samples.len() / 3;
    let second = samples.len() * 2 / 3;
    let first_start = first.saturating_sub(window / 2);
    let second_start = second.saturating_sub(window / 2);
    let omega = TAU * hz / SAMPLE_RATE;
    let coefficient = |start: usize| {
        samples[start..start + window].iter().enumerate().fold(
            (0.0_f64, 0.0_f64),
            |sum, (index, sample)| {
                let weight = 0.5 - 0.5 * (TAU * index as f64 / window as f64).cos();
                let phase = omega * index as f64;
                (
                    sum.0 + weight * sample * phase.cos(),
                    sum.1 - weight * sample * phase.sin(),
                )
            },
        )
    };
    let left = coefficient(first_start);
    let right = coefficient(second_start);
    let phase_delta = (right.1.atan2(right.0)
        - left.1.atan2(left.0)
        - omega * (second_start - first_start) as f64)
        .rem_euclid(TAU);
    let wrapped = if phase_delta > std::f64::consts::PI {
        phase_delta - TAU
    } else {
        phase_delta
    };
    (wrapped / (second_start - first_start) as f64).abs()
}

fn projected(schedule: &Schedule, source: usize) -> usize {
    let base = source / BASE_HOP;
    let remainder = source % BASE_HOP;
    if remainder == 0 {
        return schedule.positions[base];
    }
    let left = schedule.positions[base] as f64;
    let right = schedule.positions[base + 1] as f64;
    (left + (right - left) * remainder as f64 / BASE_HOP as f64).round() as usize
}

fn peak_index(samples: &[f64], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    (start..end)
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start)
}

fn dense_event_errors(samples: &[f64], expected: [usize; 2]) -> ([usize; 2], usize) {
    let start = expected[0].saturating_sub(TIMING_SEARCH);
    let end = (expected[1] + TIMING_SEARCH + 1).min(samples.len());
    let first = peak_index(samples, (start + end) / 2, (end - start) / 2);
    let mut masked = samples[start..end]
        .iter()
        .enumerate()
        .filter(|(index, _)| (start + *index).abs_diff(first) > DENSE_EXCLUSION)
        .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))
        .map(|(index, _)| start + index);
    let Some(second) = masked.take() else {
        return ([usize::MAX; 2], 2);
    };
    let mut actual = [first, second];
    actual.sort_unstable();
    let unmatched = actual
        .iter()
        .zip(expected)
        .filter(|(actual, expected)| actual.abs_diff(*expected) > TIMING_SEARCH)
        .count();
    (
        [
            actual[0].abs_diff(expected[0]),
            actual[1].abs_diff(expected[1]),
        ],
        unmatched,
    )
}

fn crest_db(samples: &[f64], center: usize, radius: usize) -> f64 {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    let slice = &samples[start..end];
    let rms = (slice.iter().map(|sample| sample * sample).sum::<f64>() / slice.len() as f64).sqrt();
    20.0 * (peak(slice) / (rms + 1.0e-15)).log10()
}

fn replica_ratio(samples: &[f64], center: usize) -> f64 {
    let primary_start = center.saturating_sub(64);
    let primary_end = (center + 65).min(samples.len());
    let secondary_end = (center + 513).min(samples.len());
    let primary = peak(&samples[primary_start..primary_end]);
    let secondary = if primary_end < secondary_end {
        peak(&samples[primary_end..secondary_end])
    } else {
        0.0
    };
    secondary / (primary + 1.0e-15)
}

fn texture(source: &[f64], output: &[f64], ratio: f64) -> [f64; 6] {
    let source = source
        .iter()
        .map(|sample| *sample as f32)
        .collect::<Vec<_>>();
    let output = output
        .iter()
        .map(|sample| *sample as f32)
        .collect::<Vec<_>>();
    let evidence = measure_tonal_texture(&source, &output, ratio);
    [
        evidence.mean_spectral_residual_ratio,
        evidence.mean_added_sideband_ratio,
        evidence.spectral_modulation_delta,
        evidence.envelope_modulation_delta_db,
        evidence.max_spectral_residual_ratio,
        evidence.max_added_sideband_ratio,
    ]
}

fn peak(samples: &[f64]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f64, f64::max)
}

fn rms_prefix(samples: &[f64], count: usize) -> f64 {
    rms(&samples[..count.min(samples.len())])
}

fn rms_suffix(samples: &[f64], count: usize) -> f64 {
    rms(&samples[samples.len().saturating_sub(count)..])
}

fn rms(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f64>() / samples.len() as f64).sqrt()
}

fn case_hash(case: &CaseEvidence) -> u64 {
    let mut state = HASH_OFFSET;
    hash(&mut state, case.control as u64);
    hash(&mut state, case.ratio.to_bits());
    hash(&mut state, case.selected_points as u64);
    hash(&mut state, case.ownership_failures as u64);
    hash(&mut state, case.combined_regressions as u64);
    for mode in &case.modes {
        for value in mode.hard_failures {
            hash(&mut state, value as u64);
        }
        for value in mode
            .lengths
            .into_iter()
            .chain(mode.coverage)
            .chain(mode.assembly_actions)
            .chain([mode.non_finite_values])
            .chain(mode.phase_assignments)
            .chain(mode.dense_errors)
            .chain([mode.dense_unmatched, mode.isolated_error])
        {
            hash(&mut state, value as u64);
        }
        for value in mode
            .identity_error
            .into_iter()
            .chain(mode.endpoint_rms)
            .chain([
                mode.frame_condition,
                mode.symmetry_error,
                mode.imaginary_residue,
                mode.tone_angular_error,
                mode.impulse_crest_db,
                mode.replica_ratio,
            ])
            .chain(mode.texture)
            .chain([mode.silence_peak])
        {
            hash(&mut state, value.to_bits());
        }
        for value in mode.hashes {
            hash(&mut state, value);
        }
    }
    for value in case.mode_deltas {
        hash(&mut state, value.to_bits());
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}

#[test]
fn synthetic_tone_frequency_measurement_resolves_rule_30n_limit() {
    for hz in [55.0, 440.0, 8_000.0] {
        for ratio in RATIOS {
            let length = (SOURCE_FRAMES as f64 * ratio).round() as usize;
            let samples = (0..length)
                .map(|index| sinusoid(hz, index))
                .collect::<Vec<_>>();
            assert!(
                angular_frequency_error(&samples, hz) <= 2.5e-7,
                "hz={hz} ratio={ratio} error={}",
                angular_frequency_error(&samples, hz),
            );
        }
    }
}
