use super::SharedRotationMechanicsReview;
use crate::frequency_adaptive::{
    source_studied::faithful_predictor::linked_stereo::{
        mechanics,
        shared_rotation_region_locked::{self, StateCounts},
    },
    HASH_OFFSET,
};

const SAMPLE_RATE: usize = 8_000;
const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];

pub(in crate::frequency_adaptive) fn review(
    renderer: fn([&[f64]; 2], f64, usize) -> shared_rotation_region_locked::SharedRotationRender,
) -> SharedRotationMechanicsReview {
    let first = run(renderer);
    let second = run(renderer);
    SharedRotationMechanicsReview {
        repeated: first == second,
        structural_failures: first.structural_failures,
        identity_mismatches: first.identity_mismatches,
        errors: first.errors,
        silent_peer_peak: first.silent_peer_peak,
        states: first.states,
        trajectory_break_resets: first.trajectory_break_resets,
        hash: first.hash,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    structural_failures: usize,
    identity_mismatches: usize,
    errors: [f64; 5],
    silent_peer_peak: f64,
    states: StateCounts,
    trajectory_break_resets: usize,
    hash: u64,
}

fn run(
    renderer: fn([&[f64]; 2], f64, usize) -> shared_rotation_region_locked::SharedRotationRender,
) -> Run {
    let primary = mechanics::primary_control(SAMPLE_RATE);
    let secondary = mechanics::secondary_control(SAMPLE_RATE);
    let silence = vec![0.0; SAMPLE_RATE];
    let identity = renderer([&primary, &secondary], 1.0, SAMPLE_RATE);
    let identity_mismatches = mismatch(&identity.channels[0], &primary, 1.0)
        + mismatch(&identity.channels[1], &secondary, 1.0);
    let mut structural_failures = 0;
    let mut errors = [0.0_f64; 5];
    let mut silent_peer_peak = 0.0_f64;
    let mut states = StateCounts::default();
    let mut hash = HASH_OFFSET;

    for ratio in RATIOS {
        let ordinary = renderer([&primary, &secondary], ratio, SAMPLE_RATE);
        let duplicate = renderer([&primary, &primary], ratio, SAMPLE_RATE);
        let hard_pan = renderer([&primary, &silence], ratio, SAMPLE_RATE);
        let swapped = renderer([&secondary, &primary], ratio, SAMPLE_RATE);
        let negative_primary = mechanics::scaled(&primary, -1.0);
        let negative_secondary = mechanics::scaled(&secondary, -1.0);
        let negative = renderer([&negative_primary, &negative_secondary], ratio, SAMPLE_RATE);
        let gained_primary = mechanics::scaled(&primary, 4.0);
        let gained = renderer([&gained_primary, &gained_primary], ratio, SAMPLE_RATE);
        let silent = renderer([&silence, &silence], ratio, SAMPLE_RATE);
        let crossing = mechanics::ownership_crossing_control(SAMPLE_RATE);
        let crossing = renderer([&crossing[0], &crossing[1]], ratio, SAMPLE_RATE);

        for render in [
            &ordinary, &duplicate, &hard_pan, &swapped, &negative, &gained, &silent, &crossing,
        ] {
            structural_failures += usize::from(
                render
                    .channels
                    .iter()
                    .any(|channel| channel.len() != render.target_length),
            ) + render.uncovered
                + render.non_finite
                + render.boundary_failures;
            add_states(&mut states, render.states);
            mix(&mut hash, render.hash);
        }
        errors[0] = errors[0]
            .max(maximum_error(
                &duplicate.channels[0],
                &hard_pan.channels[0],
                1.0,
            ))
            .max(maximum_error(
                &duplicate.channels[1],
                &hard_pan.channels[0],
                1.0,
            ));
        errors[1] = errors[1].max(maximum_error(
            &hard_pan.channels[1],
            &silent.channels[1],
            1.0,
        ));
        errors[2] = errors[2]
            .max(maximum_error(
                &swapped.channels[0],
                &ordinary.channels[1],
                1.0,
            ))
            .max(maximum_error(
                &swapped.channels[1],
                &ordinary.channels[0],
                1.0,
            ));
        errors[3] = errors[3]
            .max(maximum_error(
                &negative.channels[0],
                &ordinary.channels[0],
                -1.0,
            ))
            .max(maximum_error(
                &negative.channels[1],
                &ordinary.channels[1],
                -1.0,
            ));
        errors[4] = errors[4]
            .max(maximum_error(
                &gained.channels[0],
                &duplicate.channels[0],
                4.0,
            ))
            .max(maximum_error(
                &gained.channels[1],
                &duplicate.channels[1],
                4.0,
            ));
        silent_peer_peak = silent_peer_peak.max(
            hard_pan.channels[1]
                .iter()
                .chain(&silent.channels[0])
                .chain(&silent.channels[1])
                .map(|sample| sample.abs())
                .fold(0.0, f64::max),
        );
    }

    let mut broken = primary.clone();
    broken[SAMPLE_RATE / 3..SAMPLE_RATE * 2 / 3].fill(0.0);
    let break_render = renderer([&broken, &broken], 1.5, SAMPLE_RATE);
    add_states(&mut states, break_render.states);
    mix(&mut hash, break_render.hash);
    Run {
        structural_failures,
        identity_mismatches,
        errors,
        silent_peer_peak,
        states,
        trajectory_break_resets: break_render.states.reset,
        hash,
    }
}

fn add_states(target: &mut StateCounts, source: StateCounts) {
    target.tracked += source.tracked;
    target.reset += source.reset;
    target.silent += source.silent;
    target.regions += source.regions;
    target.owner_switches += source.owner_switches;
    target.shoulder += source.shoulder;
    target.locked += source.locked;
    target.diffuse += source.diffuse;
}

fn mismatch(actual: &[f64], expected: &[f64], gain: f64) -> usize {
    actual
        .iter()
        .zip(expected)
        .filter(|(actual, expected)| actual.to_bits() != (**expected * gain).to_bits())
        .count()
        + actual.len().abs_diff(expected.len())
}

fn maximum_error(actual: &[f64], expected: &[f64], gain: f64) -> f64 {
    actual
        .iter()
        .zip(expected)
        .map(|(actual, expected)| (actual - expected * gain).abs())
        .fold(0.0, f64::max)
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
