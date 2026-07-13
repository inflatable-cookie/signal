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
    anchors::detect,
    development_measurement::{
        hard_pass, hash, hash_bytes, measure, read_mono, report, same_samples,
    },
    render::{render, render_successor, render_successor_owned, Mode},
};

const MODES: [&str; 5] = [
    "current-signal",
    "ordinary-adaptive",
    "tracked-no-anchor",
    "tracked-anchor",
    "event-owned",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum StageAttributionDirection {
    OrdinaryAdaptiveSynthesis,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct StageAttributionReview {
    pub rows: usize,
    pub modes: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub hard_failures: usize,
    pub hard_failures_by_mode: [usize; 5],
    pub changed_rows: [usize; 4],
    pub event_fallback_renders: usize,
    pub stage_regression_rows: [[usize; 4]; 4],
    pub stage_mean_deltas: [[f64; 4]; 4],
    pub hashes: [u64; 4],
    pub direction: StageAttributionDirection,
}

pub(in crate::frequency_adaptive) fn stage_attribution_review() -> StageAttributionReview {
    let mut evidence = Vec::with_capacity(45);
    let mut manifest_hash = HASH_OFFSET;
    let mut render_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut changed_rows = [0; 4];

    for row in rows() {
        let source = read_mono(&source_root().join(&row.source), SOURCE_FRAMES);
        let outputs = render_stages(&source, row.ratio);
        hash_bytes(&mut manifest_hash, row.id.as_bytes());
        hash(&mut manifest_hash, row.ratio.to_bits());
        hash_bytes(&mut manifest_hash, row.source.as_bytes());
        for mode in MODES {
            hash_bytes(&mut manifest_hash, mode.as_bytes());
        }
        for stage in 0..4 {
            changed_rows[stage] += usize::from(!same_samples(&outputs[stage], &outputs[stage + 1]));
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
    fs::create_dir_all(path.parent().expect("stage report parent"))
        .expect("create stage report directory");
    fs::write(&path, &report).expect("write stage attribution report");
    let mut aggregate_hash = HASH_OFFSET;
    for value in [manifest_hash, render_hash, measurement_hash] {
        hash(&mut aggregate_hash, value);
    }
    hash_bytes(&mut aggregate_hash, report.as_bytes());

    let hard_failures = evidence.iter().filter(|item| !hard_pass(item)).count();
    let mut hard_failures_by_mode = [0; 5];
    for modes in evidence.chunks_exact(5) {
        for (mode, item) in modes.iter().enumerate() {
            hard_failures_by_mode[mode] += usize::from(!hard_pass(item));
        }
    }
    let event_fallback_renders = evidence.iter().filter(|item| item.event_fallback).count();
    let mut stage_regression_rows = [[0; 4]; 4];
    let mut stage_mean_deltas = [[0.0; 4]; 4];
    for modes in evidence.chunks_exact(5) {
        for stage in 0..4 {
            let before = metrics(&modes[stage]);
            let after = metrics(&modes[stage + 1]);
            for metric in 0..4 {
                let delta = after[metric] - before[metric];
                stage_mean_deltas[stage][metric] += delta / 9.0;
                stage_regression_rows[stage][metric] += usize::from(delta > 0.0);
            }
        }
    }
    StageAttributionReview {
        rows: 9,
        modes: 5,
        renders: evidence.len(),
        holdout_reads: 0,
        hard_failures,
        hard_failures_by_mode,
        changed_rows,
        event_fallback_renders,
        stage_regression_rows,
        stage_mean_deltas,
        hashes: [manifest_hash, render_hash, measurement_hash, aggregate_hash],
        direction: if hard_failures_by_mode == [0, 7, 0, 0, 0]
            && stage_regression_rows[0] == [8, 7, 9, 9]
            && changed_rows[3] == 0
        {
            StageAttributionDirection::OrdinaryAdaptiveSynthesis
        } else {
            StageAttributionDirection::Unresolved
        },
    }
}

fn render_stages(source: &[f32], ratio: f64) -> [Vec<f32>; 5] {
    let channels = vec![source
        .iter()
        .map(|sample| f64::from(*sample))
        .collect::<Vec<_>>()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let anchors = detect(&channels, SOURCE_FRAMES);
    [
        OfflineHighQualityStretcher::with_path(ratio, OfflineHighQualityPath::Default)
            .stretch_mono(source),
        samples(render(&channels, ratio, &points, &schedule, Mode::Ordinary)),
        samples(render_successor(&channels, ratio, &points, &[], &schedule)),
        samples(render_successor(
            &channels,
            ratio,
            &points,
            &anchors.positions,
            &schedule,
        )),
        samples(render_successor_owned(
            &channels,
            ratio,
            &points,
            &anchors.positions,
            &schedule,
        )),
    ]
}

fn samples(mut render: super::render::Render) -> Vec<f32> {
    render
        .samples
        .remove(0)
        .into_iter()
        .map(|sample| sample as f32)
        .collect()
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
        .join("../../target/stretch-successor-bz-stage-attribution.tsv")
}
