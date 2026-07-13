use crate::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::super::super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::render::{
    render_ordinary_factor, LatticeFactor, OverlapFactor, PhaseFactor, Render,
};

pub(super) fn render_modes(source: &[f32], ratio: f64) -> [Vec<f32>; 9] {
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
            LatticeFactor::EventWarped,
            PhaseFactor::Transport,
            OverlapFactor::DiagonalDual,
        ),
        (
            LatticeFactor::EventWarped,
            PhaseFactor::Transport,
            OverlapFactor::AnalysisPartition,
        ),
        (
            LatticeFactor::EventWarped,
            PhaseFactor::AnalysisPassthrough,
            OverlapFactor::DiagonalDual,
        ),
        (
            LatticeFactor::EventWarped,
            PhaseFactor::AnalysisPassthrough,
            OverlapFactor::AnalysisPartition,
        ),
        (
            LatticeFactor::GlobalLinear,
            PhaseFactor::Transport,
            OverlapFactor::DiagonalDual,
        ),
        (
            LatticeFactor::GlobalLinear,
            PhaseFactor::Transport,
            OverlapFactor::AnalysisPartition,
        ),
        (
            LatticeFactor::GlobalLinear,
            PhaseFactor::AnalysisPassthrough,
            OverlapFactor::DiagonalDual,
        ),
        (
            LatticeFactor::GlobalLinear,
            PhaseFactor::AnalysisPassthrough,
            OverlapFactor::AnalysisPartition,
        ),
    ];
    let outputs = factors.map(|(lattice, phase, overlap)| {
        samples(&render_ordinary_factor(
            &channels, ratio, &points, &schedule, lattice, phase, overlap,
        ))
    });
    [
        current,
        outputs[0].clone(),
        outputs[1].clone(),
        outputs[2].clone(),
        outputs[3].clone(),
        outputs[4].clone(),
        outputs[5].clone(),
        outputs[6].clone(),
        outputs[7].clone(),
    ]
}

fn samples(render: &Render) -> Vec<f32> {
    render.samples[0]
        .iter()
        .map(|sample| *sample as f32)
        .collect()
}
