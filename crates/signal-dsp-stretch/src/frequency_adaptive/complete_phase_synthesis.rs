mod controls;
mod render;

use controls::{controls, peak_index, tone_frequency};
use render::{render, Mode, Render};

use super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
};
use super::HASH_OFFSET;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    BoundedCompleteSystemTuning,
    PhaseOrSynthesisRedesign,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Review {
    pub identity_peak_error: f64,
    pub structural_failures: [usize; 7],
    pub event_phase_changes: usize,
    pub vertical_phase_changes: usize,
    pub tone_frequency_error_hz: f64,
    pub maximum_event_error: usize,
    pub maximum_symmetry_error: f64,
    pub maximum_imaginary_residue: f64,
    pub non_finite_values: usize,
    pub hashes: [u64; 5],
    pub direction: Direction,
}

pub(super) fn review() -> Review {
    let cases = controls();
    let identity = run_modes(&cases[0].channels, 1.0);
    let stretched = run_modes(&cases[1].channels, 1.5);
    let boundary = run_modes(&cases[2].channels, 0.75);
    let identity_peak_error = identity.0.samples[0]
        .iter()
        .zip(&cases[0].channels[0])
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0, f64::max);
    let mut structural_failures = [0; 7];
    let mut event_phase_changes = 0;
    let mut vertical_phase_changes = 0;
    let mut maximum_symmetry_error = 0.0_f64;
    let mut maximum_imaginary_residue = 0.0_f64;
    let mut non_finite_values = 0;
    let mut hashes = [HASH_OFFSET; 5];
    for modes in [&identity, &stretched, &boundary] {
        accumulate(
            modes,
            &mut structural_failures,
            &mut event_phase_changes,
            &mut vertical_phase_changes,
            &mut maximum_symmetry_error,
            &mut maximum_imaginary_residue,
            &mut non_finite_values,
            &mut hashes,
        );
    }
    let measured_tone = tone_frequency(&stretched.3.samples[0], 48_000.0);
    let tone_frequency_error_hz = (measured_tone - 997.0).abs();
    let maximum_event_error = cases[1]
        .events
        .iter()
        .map(|event| {
            let expected = (1.5 * *event as f64).round() as usize;
            peak_index(&stretched.3.samples[0], expected, 256).abs_diff(expected)
        })
        .max()
        .unwrap_or(0);
    let pass = identity_peak_error <= 5.0e-12
        && structural_failures == [0; 7]
        && event_phase_changes > 0
        && vertical_phase_changes > 0
        && tone_frequency_error_hz <= 2.0
        && maximum_event_error <= 256
        && maximum_symmetry_error <= 2.0e-10
        && maximum_imaginary_residue <= 2.0e-10
        && non_finite_values == 0;
    Review {
        identity_peak_error,
        structural_failures,
        event_phase_changes,
        vertical_phase_changes,
        tone_frequency_error_hz,
        maximum_event_error,
        maximum_symmetry_error,
        maximum_imaginary_residue,
        non_finite_values,
        hashes,
        direction: if pass {
            Direction::BoundedCompleteSystemTuning
        } else {
            Direction::PhaseOrSynthesisRedesign
        },
    }
}

type Modes = (Render, Render, Render, Render);

fn run_modes(channels: &[Vec<f64>], ratio: f64) -> Modes {
    let study = analyze(channels, channels[0].len());
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(channels[0].len(), 128, ratio, &points);
    let reversed = channels.iter().cloned().rev().collect::<Vec<_>>();
    let reversed_points = select(&analyze(&reversed, channels[0].len()), 3.0, 2);
    assert_eq!(points, reversed_points);
    (
        render(channels, ratio, &points, &schedule, Mode::Ordinary),
        render(channels, ratio, &points, &schedule, Mode::Event),
        render(channels, ratio, &points, &schedule, Mode::Vertical),
        render(channels, ratio, &points, &schedule, Mode::Both),
    )
}

fn accumulate(
    modes: &Modes,
    failures: &mut [usize; 7],
    event_changes: &mut usize,
    vertical_changes: &mut usize,
    symmetry: &mut f64,
    imaginary: &mut f64,
    non_finite: &mut usize,
    hashes: &mut [u64; 5],
) {
    let renders = [&modes.0, &modes.1, &modes.2, &modes.3];
    for render in renders {
        failures[0] += usize::from(
            render
                .samples
                .iter()
                .any(|channel| channel.len() != render.target_len),
        );
        failures[1] += render.uncovered;
        failures[2] += usize::from(render.schedule_hash != modes.0.schedule_hash);
        failures[3] += usize::from(render.magnitude_hash != modes.0.magnitude_hash);
        failures[4] += render.boundary_failures;
        failures[5] += render.event_order_failures;
        failures[6] += usize::from(render.channel_decision_hash == 0);
        *symmetry = symmetry.max(render.symmetry_error);
        *imaginary = imaginary.max(render.imaginary_residue);
        *non_finite += render.non_finite;
        mix(&mut hashes[0], render.schedule_hash);
        mix(&mut hashes[1], render.magnitude_hash);
        mix(&mut hashes[2], render.phase_hash);
        mix(&mut hashes[3], render.output_hash);
        mix(&mut hashes[4], render.channel_decision_hash);
    }
    *event_changes += modes.1.event_resets + modes.3.event_resets;
    *vertical_changes += modes.2.vertical_alignments + modes.3.vertical_alignments;
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
