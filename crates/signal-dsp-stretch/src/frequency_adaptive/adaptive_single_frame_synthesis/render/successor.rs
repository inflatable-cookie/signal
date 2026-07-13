use super::super::super::study_local_schedule::schedule::Schedule;
use super::{render_frames, schedule, Mode, Render};

pub(in crate::frequency_adaptive::adaptive_single_frame_synthesis) fn render_successor(
    channels: &[Vec<f64>],
    ratio: f64,
    resolution_points: &[usize],
    anchors: &[usize],
    schedule: &Schedule,
) -> Render {
    render_successor_mode(
        channels,
        ratio,
        resolution_points,
        anchors,
        &[],
        schedule,
        Mode::Successor,
    )
}

pub(in crate::frequency_adaptive::adaptive_single_frame_synthesis) fn render_successor_traced(
    channels: &[Vec<f64>],
    ratio: f64,
    resolution_points: &[usize],
    anchors: &[usize],
    trace_outputs: &[isize],
    schedule: &Schedule,
) -> Render {
    render_successor_mode(
        channels,
        ratio,
        resolution_points,
        anchors,
        trace_outputs,
        schedule,
        Mode::Successor,
    )
}

pub(in crate::frequency_adaptive::adaptive_single_frame_synthesis) fn render_successor_owned(
    channels: &[Vec<f64>],
    ratio: f64,
    resolution_points: &[usize],
    anchors: &[usize],
    schedule: &Schedule,
) -> Render {
    render_successor_mode(
        channels,
        ratio,
        resolution_points,
        anchors,
        &[],
        schedule,
        Mode::SuccessorOwned,
    )
}

pub(in crate::frequency_adaptive::adaptive_single_frame_synthesis) fn render_successor_owned_traced(
    channels: &[Vec<f64>],
    ratio: f64,
    resolution_points: &[usize],
    anchors: &[usize],
    trace_outputs: &[isize],
    schedule: &Schedule,
) -> Render {
    render_successor_mode(
        channels,
        ratio,
        resolution_points,
        anchors,
        trace_outputs,
        schedule,
        Mode::SuccessorOwned,
    )
}

fn render_successor_mode(
    channels: &[Vec<f64>],
    ratio: f64,
    resolution_points: &[usize],
    anchors: &[usize],
    trace_outputs: &[isize],
    schedule: &Schedule,
    mode: Mode,
) -> Render {
    render_frames(
        channels,
        ratio,
        anchors,
        anchors,
        trace_outputs,
        schedule,
        mode,
        schedule::successor(ratio, resolution_points, anchors, schedule),
    )
}
