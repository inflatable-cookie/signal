pub(in crate::frequency_adaptive) mod quality;
mod render;

use super::{coherent_representation, HASH_OFFSET};

const SAMPLE_RATE: usize = 8_000;
const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum LinkedStereoMechanicsDirection {
    QualityGate,
    MechanicsAttribution,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LinkedStereoRatioEvidence {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) target_frames: usize,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 4],
    pub(in crate::frequency_adaptive) duplicate_mismatches: usize,
    pub(in crate::frequency_adaptive) hard_pan_mismatches: usize,
    pub(in crate::frequency_adaptive) silent_channel_peak: f64,
    pub(in crate::frequency_adaptive) swap_mismatches: usize,
    pub(in crate::frequency_adaptive) polarity_mismatches: usize,
    pub(in crate::frequency_adaptive) gain_parity_mismatches: usize,
    pub(in crate::frequency_adaptive) shared_corrected: usize,
    pub(in crate::frequency_adaptive) shared_fallback: usize,
    pub(in crate::frequency_adaptive) unilateral_non_silent_completions: usize,
    pub(in crate::frequency_adaptive) audio_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LinkedStereoMechanicsReview {
    pub(in crate::frequency_adaptive) geometry: [usize; 4],
    pub(in crate::frequency_adaptive) identity_mismatches: usize,
    pub(in crate::frequency_adaptive) ratios: Vec<LinkedStereoRatioEvidence>,
    pub(in crate::frequency_adaptive) audio_hash: u64,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: LinkedStereoMechanicsDirection,
}

pub(in crate::frequency_adaptive) fn mechanics_review() -> LinkedStereoMechanicsReview {
    let first = run();
    let second = run();
    let repeated = first == second;
    let passed = repeated
        && first.identity_mismatches == 0
        && first.ratios.iter().all(|row| {
            row.structural_failures == [0; 4]
                && row.duplicate_mismatches == 0
                && row.hard_pan_mismatches == 0
                && row.silent_channel_peak == 0.0
                && row.swap_mismatches == 0
                && row.polarity_mismatches == 0
                && row.gain_parity_mismatches == 0
                && row.shared_corrected > 0
                && row.shared_fallback > 0
                && row.unilateral_non_silent_completions == 0
        });
    LinkedStereoMechanicsReview {
        geometry: first.geometry,
        identity_mismatches: first.identity_mismatches,
        ratios: first.ratios,
        audio_hash: first.audio_hash,
        evidence_hash: first.evidence_hash,
        repeated,
        direction: if passed {
            LinkedStereoMechanicsDirection::QualityGate
        } else {
            LinkedStereoMechanicsDirection::MechanicsAttribution
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    geometry: [usize; 4],
    identity_mismatches: usize,
    ratios: Vec<LinkedStereoRatioEvidence>,
    audio_hash: u64,
    evidence_hash: u64,
}

fn run() -> Run {
    let primary = primary_control();
    let secondary = secondary_control();
    let identity = render::linked([&primary, &secondary], 1.0, SAMPLE_RATE);
    let identity_mismatches = mismatch_count(&identity.channels[0], &primary)
        + mismatch_count(&identity.channels[1], &secondary);
    let mut audio_hash = HASH_OFFSET;
    let mut evidence_hash = HASH_OFFSET;
    let mut ratios = Vec::with_capacity(RATIOS.len());

    for ratio in RATIOS {
        let mono = coherent_representation::render(&primary, ratio, SAMPLE_RATE);
        let duplicate = render::linked([&primary, &primary], ratio, SAMPLE_RATE);
        let silence = vec![0.0; primary.len()];
        let hard_pan = render::linked([&primary, &silence], ratio, SAMPLE_RATE);
        let ordinary = render::linked([&primary, &secondary], ratio, SAMPLE_RATE);
        let swapped = render::linked([&secondary, &primary], ratio, SAMPLE_RATE);
        let negative_primary = scaled(&primary, -1.0);
        let negative_secondary = scaled(&secondary, -1.0);
        let polarity = render::linked([&negative_primary, &negative_secondary], ratio, SAMPLE_RATE);
        let low_primary = scaled(&primary, 0.25);
        let low = render::linked([&low_primary, &low_primary], ratio, SAMPLE_RATE);
        let low_primary_mono = coherent_representation::render(&low_primary, ratio, SAMPLE_RATE);
        let high_primary = scaled(&primary, 4.0);
        let high = render::linked([&high_primary, &high_primary], ratio, SAMPLE_RATE);
        let high_primary_mono = coherent_representation::render(&high_primary, ratio, SAMPLE_RATE);
        let silence_linked = render::linked([&silence, &silence], ratio, SAMPLE_RATE);

        let target_frames = mono.samples.len();
        let structural_failures = [
            usize::from(
                duplicate
                    .channels
                    .iter()
                    .any(|channel| channel.len() != target_frames)
                    || hard_pan
                        .channels
                        .iter()
                        .any(|channel| channel.len() != target_frames)
                    || ordinary
                        .channels
                        .iter()
                        .any(|channel| channel.len() != target_frames),
            ),
            duplicate.uncovered + hard_pan.uncovered + ordinary.uncovered,
            duplicate.non_finite + hard_pan.non_finite + ordinary.non_finite,
            duplicate.boundary_failures + hard_pan.boundary_failures + ordinary.boundary_failures,
        ];
        let duplicate_mismatches = mismatch_count(&duplicate.channels[0], &mono.samples)
            + mismatch_count(&duplicate.channels[1], &mono.samples);
        let hard_pan_mismatches = mismatch_count(&hard_pan.channels[0], &mono.samples);
        let silent_channel_peak = hard_pan.channels[1]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0, f64::max);
        let swap_mismatches = mismatch_count(&swapped.channels[0], &ordinary.channels[1])
            + mismatch_count(&swapped.channels[1], &ordinary.channels[0]);
        let polarity_mismatches =
            signed_mismatch_count(&polarity.channels[0], &ordinary.channels[0], -1.0)
                + signed_mismatch_count(&polarity.channels[1], &ordinary.channels[1], -1.0);
        let gain_parity_mismatches = mismatch_count(&low.channels[0], &low_primary_mono.samples)
            + mismatch_count(&low.channels[1], &low_primary_mono.samples)
            + mismatch_count(&high.channels[0], &high_primary_mono.samples)
            + mismatch_count(&high.channels[1], &high_primary_mono.samples);
        let row_audio_hash = [duplicate.hash, hard_pan.hash, ordinary.hash]
            .into_iter()
            .fold(HASH_OFFSET, |mut state, value| {
                hash_values(&mut state, &[value]);
                state
            });
        hash_values(&mut audio_hash, &[row_audio_hash]);
        hash_values(
            &mut evidence_hash,
            &[
                ratio.to_bits(),
                target_frames as u64,
                duplicate_mismatches as u64,
                hard_pan_mismatches as u64,
                silent_channel_peak.to_bits(),
                swap_mismatches as u64,
                polarity_mismatches as u64,
                gain_parity_mismatches as u64,
                ordinary.shared_corrected as u64,
                silence_linked.shared_fallback as u64,
                ordinary.unilateral_non_silent_completions as u64,
                row_audio_hash,
            ],
        );
        ratios.push(LinkedStereoRatioEvidence {
            ratio,
            target_frames,
            structural_failures,
            duplicate_mismatches,
            hard_pan_mismatches,
            silent_channel_peak,
            swap_mismatches,
            polarity_mismatches,
            gain_parity_mismatches,
            shared_corrected: ordinary.shared_corrected,
            shared_fallback: silence_linked.shared_fallback,
            unilateral_non_silent_completions: ordinary.unilateral_non_silent_completions,
            audio_hash: row_audio_hash,
        });
    }

    hash_values(
        &mut evidence_hash,
        &[identity_mismatches as u64, audio_hash],
    );
    Run {
        geometry: coherent_representation::source_geometry(SAMPLE_RATE),
        identity_mismatches,
        ratios,
        audio_hash,
        evidence_hash,
    }
}

fn primary_control() -> Vec<f64> {
    (0..SAMPLE_RATE)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE as f64;
            let mut sample = 0.28 * (std::f64::consts::TAU * 110.0 * time).sin()
                + 0.17 * (std::f64::consts::TAU * 523.251 * time).sin();
            if index == SAMPLE_RATE / 3 || index == SAMPLE_RATE * 2 / 3 {
                sample += 0.65;
            }
            sample
        })
        .collect()
}

fn secondary_control() -> Vec<f64> {
    (0..SAMPLE_RATE)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE as f64;
            let mut sample = 0.21 * (std::f64::consts::TAU * 164.8138 * time + 0.7).sin()
                + 0.13 * (std::f64::consts::TAU * 880.0 * time - 0.4).sin();
            if index == SAMPLE_RATE / 2 {
                sample -= 0.55;
            }
            sample
        })
        .collect()
}

fn scaled(input: &[f64], gain: f64) -> Vec<f64> {
    input.iter().map(|sample| sample * gain).collect()
}

fn mismatch_count(actual: &[f64], expected: &[f64]) -> usize {
    actual
        .iter()
        .zip(expected)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count()
        + actual.len().abs_diff(expected.len())
}

fn signed_mismatch_count(actual: &[f64], expected: &[f64], gain: f64) -> usize {
    actual
        .iter()
        .zip(expected)
        .filter(|(actual, expected)| actual.to_bits() != (**expected * gain).to_bits())
        .count()
        + actual.len().abs_diff(expected.len())
}

fn hash_values(state: &mut u64, values: &[u64]) {
    for value in values {
        *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
    }
}
