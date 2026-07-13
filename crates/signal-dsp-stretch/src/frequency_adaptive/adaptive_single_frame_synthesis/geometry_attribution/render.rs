use crate::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::super::super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::render::{
    render_ordinary_geometry_factor, FftGridFactor, FrameGeometryFactor, Render,
};

pub(super) fn render_modes(source: &[f32], ratio: f64) -> [Vec<f32>; 5] {
    let channels = vec![source
        .iter()
        .map(|sample| f64::from(*sample))
        .collect::<Vec<_>>()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let current = OfflineHighQualityStretcher::with_path(ratio, OfflineHighQualityPath::Default)
        .stretch_mono(source);
    let factors = [
        (
            4_096,
            FftGridFactor::Shared4096,
            FrameGeometryFactor::CenteredReflected,
        ),
        (
            2_048,
            FftGridFactor::Shared4096,
            FrameGeometryFactor::CenteredReflected,
        ),
        (
            2_048,
            FftGridFactor::Native,
            FrameGeometryFactor::CenteredReflected,
        ),
        (
            2_048,
            FftGridFactor::Native,
            FrameGeometryFactor::StartAlignedPadded,
        ),
    ];
    let outputs = factors.map(|(length, fft_grid, frame_geometry)| {
        samples(&render_ordinary_geometry_factor(
            &channels,
            ratio,
            &points,
            &schedule,
            length,
            fft_grid,
            frame_geometry,
        ))
    });
    [
        current,
        outputs[0].clone(),
        outputs[1].clone(),
        outputs[2].clone(),
        outputs[3].clone(),
    ]
}

fn samples(render: &Render) -> Vec<f32> {
    render.samples[0]
        .iter()
        .map(|sample| *sample as f32)
        .collect()
}
