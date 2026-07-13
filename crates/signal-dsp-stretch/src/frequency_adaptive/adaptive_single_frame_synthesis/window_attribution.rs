use std::{fs, path::PathBuf};

use crate::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::super::{
    complete_system_tuning::listening_export::manifest::{rows, source_root},
    study_local_schedule::{
        schedule::build_schedule,
        study::{analyze, select},
        BASE_HOP, SOURCE_FRAMES,
    },
    HASH_OFFSET,
};
use super::{
    development_measurement::{
        hard_pass, hash, hash_bytes, measure, read_mono, report, same_samples,
    },
    render::{render_ordinary_window_factor, Render, WindowFactor},
};

const MODES: [&str; 5] = [
    "current-signal",
    "root-analysis-root-synthesis",
    "root-analysis-hann-synthesis",
    "hann-analysis-root-synthesis",
    "hann-analysis-hann-synthesis",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum WindowAttributionDirection {
    WindowKernelsContributeButDoNotOwn,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct WindowAttributionReview {
    pub rows: usize,
    pub modes: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub hard_failures: usize,
    pub hard_failures_by_mode: [usize; 5],
    pub event_fallback_renders: usize,
    pub changed_from_current: [usize; 4],
    pub regression_from_current: [[usize; 4]; 4],
    pub mean_delta_from_current: [[f64; 4]; 4],
    pub analysis_regressions: [[usize; 4]; 2],
    pub analysis_mean_deltas: [[f64; 4]; 2],
    pub synthesis_regressions: [[usize; 4]; 2],
    pub synthesis_mean_deltas: [[f64; 4]; 2],
    pub hashes: [u64; 4],
    pub direction: WindowAttributionDirection,
}

pub(in crate::frequency_adaptive) fn window_attribution_review() -> WindowAttributionReview {
    let mut evidence = Vec::with_capacity(45);
    let mut manifest_hash = HASH_OFFSET;
    let mut render_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut changed_from_current = [0; 4];

    for row in rows() {
        let source = read_mono(&source_root().join(&row.source), SOURCE_FRAMES);
        let outputs = render_modes(&source, row.ratio);
        hash_bytes(&mut manifest_hash, row.id.as_bytes());
        hash(&mut manifest_hash, row.ratio.to_bits());
        hash_bytes(&mut manifest_hash, row.source.as_bytes());
        for mode in MODES {
            hash_bytes(&mut manifest_hash, mode.as_bytes());
        }
        for mode in 0..4 {
            changed_from_current[mode] +=
                usize::from(!same_samples(&outputs[0], &outputs[mode + 1]));
        }
        for (mode, output) in MODES.into_iter().zip(outputs) {
            let item = measure(row.id, row.ratio, mode, &source, &output);
            hash(&mut render_hash, item.render_hash);
            hash(&mut measurement_hash, item.measurement_hash);
            evidence.push(item);
        }
    }

    let report = report(&evidence);
    let path = report_path();
    fs::create_dir_all(path.parent().expect("window report parent"))
        .expect("create window report directory");
    fs::write(&path, &report).expect("write window attribution report");
    let mut aggregate_hash = HASH_OFFSET;
    for value in [manifest_hash, render_hash, measurement_hash] {
        hash(&mut aggregate_hash, value);
    }
    hash_bytes(&mut aggregate_hash, report.as_bytes());

    let mut hard_failures_by_mode = [0; 5];
    let mut regression_from_current = [[0; 4]; 4];
    let mut mean_delta_from_current = [[0.0; 4]; 4];
    let mut analysis_regressions = [[0; 4]; 2];
    let mut analysis_mean_deltas = [[0.0; 4]; 2];
    let mut synthesis_regressions = [[0; 4]; 2];
    let mut synthesis_mean_deltas = [[0.0; 4]; 2];
    for modes in evidence.chunks_exact(5) {
        for (mode, item) in modes.iter().enumerate() {
            hard_failures_by_mode[mode] += usize::from(!hard_pass(item));
        }
        let current = metrics(&modes[0]);
        for mode in 0..4 {
            accumulate(
                current,
                metrics(&modes[mode + 1]),
                &mut regression_from_current[mode],
                &mut mean_delta_from_current[mode],
            );
        }
        for (index, (before, after)) in [(1, 3), (2, 4)].into_iter().enumerate() {
            accumulate(
                metrics(&modes[before]),
                metrics(&modes[after]),
                &mut analysis_regressions[index],
                &mut analysis_mean_deltas[index],
            );
        }
        for (index, (before, after)) in [(1, 2), (3, 4)].into_iter().enumerate() {
            accumulate(
                metrics(&modes[before]),
                metrics(&modes[after]),
                &mut synthesis_regressions[index],
                &mut synthesis_mean_deltas[index],
            );
        }
    }
    let direction = if regression_from_current
        .iter()
        .all(|counts| counts[2] == 9 && counts[3] == 9)
        && mean_delta_from_current[3][2] < mean_delta_from_current[0][2]
        && mean_delta_from_current[3][3] < mean_delta_from_current[0][3]
    {
        WindowAttributionDirection::WindowKernelsContributeButDoNotOwn
    } else {
        WindowAttributionDirection::Unresolved
    };
    WindowAttributionReview {
        rows: 9,
        modes: 5,
        renders: evidence.len(),
        holdout_reads: 0,
        hard_failures: evidence.iter().filter(|item| !hard_pass(item)).count(),
        hard_failures_by_mode,
        event_fallback_renders: evidence.iter().filter(|item| item.event_fallback).count(),
        changed_from_current,
        regression_from_current,
        mean_delta_from_current,
        analysis_regressions,
        analysis_mean_deltas,
        synthesis_regressions,
        synthesis_mean_deltas,
        hashes: [manifest_hash, render_hash, measurement_hash, aggregate_hash],
        direction,
    }
}

fn render_modes(source: &[f32], ratio: f64) -> [Vec<f32>; 5] {
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
        (WindowFactor::RootHann, WindowFactor::RootHann),
        (WindowFactor::RootHann, WindowFactor::Hann),
        (WindowFactor::Hann, WindowFactor::RootHann),
        (WindowFactor::Hann, WindowFactor::Hann),
    ];
    let outputs = factors.map(|(analysis, synthesis)| {
        samples(&render_ordinary_window_factor(
            &channels, ratio, &points, &schedule, analysis, synthesis,
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

fn accumulate(
    before: [f64; 4],
    after: [f64; 4],
    regressions: &mut [usize; 4],
    means: &mut [f64; 4],
) {
    for metric in 0..4 {
        let delta = after[metric] - before[metric];
        regressions[metric] += usize::from(delta > 0.0);
        means[metric] += delta / 9.0;
    }
}

fn metrics(evidence: &super::development_measurement::Evidence) -> [f64; 4] {
    [
        evidence.mean_event_offset,
        evidence.replica_ratio,
        evidence.static_residual,
        evidence.formant_residual,
    ]
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-cc-window-attribution.tsv")
}
