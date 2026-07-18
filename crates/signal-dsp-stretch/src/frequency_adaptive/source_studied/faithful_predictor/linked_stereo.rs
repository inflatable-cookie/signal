mod mechanics;
pub(in crate::frequency_adaptive) mod quality;
pub(in crate::frequency_adaptive) mod render;
pub(in crate::frequency_adaptive) mod shared_rotation_finite_support_reset;
pub(in crate::frequency_adaptive) mod shared_rotation_region_locked;
pub(in crate::frequency_adaptive) mod state_complete_linked_phase_vocoder;

use super::{coherent_representation, HASH_OFFSET};
use mechanics::{
    mismatch_count, primary_control, scaled, secondary_control, signed_mismatch_count,
};

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
    pub(in crate::frequency_adaptive) reference_bins: [usize; 2],
    pub(in crate::frequency_adaptive) active_reference_ties: usize,
    pub(in crate::frequency_adaptive) reference_switches: usize,
    pub(in crate::frequency_adaptive) switch_step_growth_db: f64,
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
                && row.reference_bins.iter().all(|count| *count > 0)
                && row.active_reference_ties > 0
                && row.reference_switches > 0
                && row.switch_step_growth_db <= 0.0
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
    let primary = primary_control(SAMPLE_RATE);
    let secondary = secondary_control(SAMPLE_RATE);
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
        let crossing_input = mechanics::ownership_crossing_control(SAMPLE_RATE);
        let crossing = render::linked([&crossing_input[0], &crossing_input[1]], ratio, SAMPLE_RATE);
        let switch_step_growth_db = mechanics::switch_step_growth_db(
            &crossing.channels,
            ratio,
            primary.len(),
            coherent_representation::source_geometry(SAMPLE_RATE)[1],
        );

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
                ordinary.reference_bins[0] as u64,
                ordinary.reference_bins[1] as u64,
                duplicate.active_reference_ties as u64,
                crossing.reference_switches as u64,
                switch_step_growth_db.to_bits(),
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
            reference_bins: ordinary.reference_bins,
            active_reference_ties: duplicate.active_reference_ties,
            reference_switches: crossing.reference_switches,
            switch_step_growth_db,
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

fn hash_values(state: &mut u64, values: &[u64]) {
    for value in values {
        *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
    }
}
