use std::f64::consts::TAU;

use super::super::super::study_local_schedule::{schedule::Schedule, SOURCE_FRAMES};
use super::super::anchors::projected;
use super::super::quality::{
    control::SAMPLE_RATE,
    measurement::{angular_frequency_error, dense_event_errors, peak_index},
    Control,
};
use super::super::render::Render;

pub(super) fn tone_errors(control: Control, render: &Render) -> [f64; 2] {
    let Some(hz) = control.tone_hz() else {
        return [0.0; 2];
    };
    let expected = TAU * hz / SAMPLE_RATE;
    let worst = render
        .phase_trace
        .iter()
        .filter(|frame| {
            frame.phase.trace_owner_matched
                && frame.source >= 4_096
                && frame.source + 4_096 < SOURCE_FRAMES as isize
        })
        .max_by(|left, right| {
            (left.phase.estimated_frequency - expected)
                .abs()
                .total_cmp(&(right.phase.estimated_frequency - expected).abs())
        });
    let frame = worst
        .map(|frame| (frame.phase.estimated_frequency - expected).abs())
        .unwrap_or(0.0);
    [angular_frequency_error(&render.samples[0], hz), frame]
}

pub(super) fn event_errors(
    control: Control,
    expected: &[usize],
    schedule: &Schedule,
    render: &Render,
) -> [usize; 3] {
    match control {
        Control::IsolatedImpulse => {
            let target = projected(schedule, expected[0]) as usize;
            [
                peak_index(&render.samples[0], target, 512).abs_diff(target),
                0,
                0,
            ]
        }
        Control::DenseEvent => {
            let targets = [
                projected(schedule, expected[0]) as usize,
                projected(schedule, expected[1]) as usize,
            ];
            let (errors, unmatched) = dense_event_errors(&render.samples[0], targets);
            [errors[0], errors[1], unmatched]
        }
        _ => [0; 3],
    }
}
