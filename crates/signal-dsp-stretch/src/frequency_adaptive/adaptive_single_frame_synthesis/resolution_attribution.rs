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
    development_measurement::{hard_pass, hash, hash_bytes, measure, read_mono, report},
    render::{render, render_ordinary_fixed, Mode, Render},
};

const MODES: [&str; 6] = [
    "current-signal",
    "ordinary-fixed-512",
    "ordinary-fixed-1024",
    "ordinary-fixed-2048",
    "ordinary-fixed-4096",
    "ordinary-adaptive",
];
const LENGTHS: [usize; 4] = [512, 1_024, 2_048, 4_096];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum ResolutionAttributionDirection {
    SplitResolutionTransitionAndSharedMechanism,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ResolutionAttributionReview {
    pub rows: usize,
    pub modes: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub hard_failures: usize,
    pub hard_failures_by_mode: [usize; 6],
    pub event_fallback_renders: usize,
    pub resolution_changes: [usize; 5],
    pub changed_from_current: [usize; 5],
    pub changed_from_adaptive: [usize; 4],
    pub regression_from_current: [[usize; 4]; 5],
    pub mean_delta_from_current: [[f64; 4]; 5],
    pub adaptive_regression_from_fixed: [[usize; 4]; 4],
    pub adaptive_mean_delta_from_fixed: [[f64; 4]; 4],
    pub hashes: [u64; 4],
    pub direction: ResolutionAttributionDirection,
}

pub(in crate::frequency_adaptive) fn resolution_attribution_review() -> ResolutionAttributionReview
{
    let mut evidence = Vec::with_capacity(54);
    let mut manifest_hash = HASH_OFFSET;
    let mut render_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut resolution_changes = [0; 5];
    let mut changed_from_current = [0; 5];
    let mut changed_from_adaptive = [0; 4];

    for row in rows() {
        let source = read_mono(&source_root().join(&row.source), SOURCE_FRAMES);
        let outputs = render_modes(&source, row.ratio);
        hash_bytes(&mut manifest_hash, row.id.as_bytes());
        hash(&mut manifest_hash, row.ratio.to_bits());
        hash_bytes(&mut manifest_hash, row.source.as_bytes());
        for mode in MODES {
            hash_bytes(&mut manifest_hash, mode.as_bytes());
        }
        for mode in 0..5 {
            resolution_changes[mode] += outputs[mode + 1].1;
            changed_from_current[mode] +=
                usize::from(!same_samples(&outputs[0].0, &outputs[mode + 1].0));
        }
        for fixed in 0..4 {
            changed_from_adaptive[fixed] +=
                usize::from(!same_samples(&outputs[fixed + 1].0, &outputs[5].0));
        }
        for (mode, (output, _)) in MODES.into_iter().zip(outputs) {
            let item = measure(row.id, row.ratio, mode, &source, &output);
            hash(&mut render_hash, item.render_hash);
            hash(&mut measurement_hash, item.measurement_hash);
            evidence.push(item);
        }
    }

    let report = report(&evidence);
    let path = report_path();
    fs::create_dir_all(path.parent().expect("resolution report parent"))
        .expect("create resolution report directory");
    fs::write(&path, &report).expect("write resolution attribution report");
    let mut aggregate_hash = HASH_OFFSET;
    for value in [manifest_hash, render_hash, measurement_hash] {
        hash(&mut aggregate_hash, value);
    }
    hash_bytes(&mut aggregate_hash, report.as_bytes());

    let hard_failures = evidence.iter().filter(|item| !hard_pass(item)).count();
    let mut hard_failures_by_mode = [0; 6];
    let mut regression_from_current = [[0; 4]; 5];
    let mut mean_delta_from_current = [[0.0; 4]; 5];
    let mut adaptive_regression_from_fixed = [[0; 4]; 4];
    let mut adaptive_mean_delta_from_fixed = [[0.0; 4]; 4];
    for modes in evidence.chunks_exact(6) {
        for (mode, item) in modes.iter().enumerate() {
            hard_failures_by_mode[mode] += usize::from(!hard_pass(item));
        }
        let current = metrics(&modes[0]);
        let adaptive = metrics(&modes[5]);
        for mode in 0..5 {
            let candidate = metrics(&modes[mode + 1]);
            accumulate(
                current,
                candidate,
                &mut regression_from_current[mode],
                &mut mean_delta_from_current[mode],
            );
        }
        for fixed in 0..4 {
            accumulate(
                metrics(&modes[fixed + 1]),
                adaptive,
                &mut adaptive_regression_from_fixed[fixed],
                &mut adaptive_mean_delta_from_fixed[fixed],
            );
        }
    }
    let direction = classify(
        &hard_failures_by_mode,
        &regression_from_current,
        &adaptive_regression_from_fixed,
    );
    ResolutionAttributionReview {
        rows: 9,
        modes: 6,
        renders: evidence.len(),
        holdout_reads: 0,
        hard_failures,
        hard_failures_by_mode,
        event_fallback_renders: evidence.iter().filter(|item| item.event_fallback).count(),
        resolution_changes,
        changed_from_current,
        changed_from_adaptive,
        regression_from_current,
        mean_delta_from_current,
        adaptive_regression_from_fixed,
        adaptive_mean_delta_from_fixed,
        hashes: [manifest_hash, render_hash, measurement_hash, aggregate_hash],
        direction,
    }
}

fn render_modes(source: &[f32], ratio: f64) -> [(Vec<f32>, usize); 6] {
    let channels = vec![source
        .iter()
        .map(|sample| f64::from(*sample))
        .collect::<Vec<_>>()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let current = OfflineHighQualityStretcher::with_path(ratio, OfflineHighQualityPath::Default)
        .stretch_mono(source);
    let fixed = LENGTHS.map(|length| {
        let render = render_ordinary_fixed(&channels, ratio, &points, &schedule, length);
        (samples(&render), render.resolution_changes)
    });
    let adaptive = render(&channels, ratio, &points, &schedule, Mode::Ordinary);
    [
        (current, 0),
        fixed[0].clone(),
        fixed[1].clone(),
        fixed[2].clone(),
        fixed[3].clone(),
        (samples(&adaptive), adaptive.resolution_changes),
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

fn classify(
    hard_failures: &[usize; 6],
    from_current: &[[usize; 4]; 5],
    adaptive_from_fixed: &[[usize; 4]; 4],
) -> ResolutionAttributionDirection {
    let shared_spectral = from_current
        .iter()
        .all(|counts| counts[2] == 9 && counts[3] == 9);
    let transition_timing = adaptive_from_fixed.iter().all(|counts| counts[0] >= 5);
    let resolution_integrity = hard_failures[1] == 9
        && hard_failures[2] == 9
        && hard_failures[3] > 0
        && hard_failures[4] == 0
        && hard_failures[5] > 0;
    if shared_spectral && transition_timing && resolution_integrity {
        ResolutionAttributionDirection::SplitResolutionTransitionAndSharedMechanism
    } else {
        ResolutionAttributionDirection::Unresolved
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

fn same_samples(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.to_bits() == right.to_bits())
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-ca-resolution-attribution.tsv")
}
