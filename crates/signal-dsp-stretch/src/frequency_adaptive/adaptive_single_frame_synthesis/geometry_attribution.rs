use std::{fs, path::PathBuf};

mod render;

use super::super::{
    complete_system_tuning::listening_export::manifest::{rows, source_root},
    study_local_schedule::SOURCE_FRAMES,
    HASH_OFFSET,
};
use super::development_measurement::{
    hard_pass, hash, hash_bytes, measure, read_mono, report, same_samples,
};
use render::render_modes;

const MODES: [&str; 5] = [
    "current-signal",
    "hann-hann-4096-centered-reflected-shared4096",
    "hann-hann-2048-centered-reflected-shared4096",
    "hann-hann-2048-centered-reflected-native2048",
    "hann-hann-2048-start-aligned-padded-native2048",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum GeometryAttributionDirection {
    SharedGridContributesRemainingPathOwns,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct GeometryAttributionReview {
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
    pub resolution_regressions: [usize; 4],
    pub resolution_mean_deltas: [f64; 4],
    pub fft_grid_regressions: [usize; 4],
    pub fft_grid_mean_deltas: [f64; 4],
    pub frame_geometry_regressions: [usize; 4],
    pub frame_geometry_mean_deltas: [f64; 4],
    pub hashes: [u64; 4],
    pub direction: GeometryAttributionDirection,
}

pub(in crate::frequency_adaptive) fn geometry_attribution_review() -> GeometryAttributionReview {
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
    fs::create_dir_all(path.parent().expect("geometry report parent"))
        .expect("create geometry report directory");
    fs::write(&path, &report).expect("write geometry attribution report");
    let mut aggregate_hash = HASH_OFFSET;
    for value in [manifest_hash, render_hash, measurement_hash] {
        hash(&mut aggregate_hash, value);
    }
    hash_bytes(&mut aggregate_hash, report.as_bytes());

    let mut hard_failures_by_mode = [0; 5];
    let mut regression_from_current = [[0; 4]; 4];
    let mut mean_delta_from_current = [[0.0; 4]; 4];
    let mut resolution_regressions = [0; 4];
    let mut resolution_mean_deltas = [0.0; 4];
    let mut fft_grid_regressions = [0; 4];
    let mut fft_grid_mean_deltas = [0.0; 4];
    let mut frame_geometry_regressions = [0; 4];
    let mut frame_geometry_mean_deltas = [0.0; 4];
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
        accumulate(
            metrics(&modes[1]),
            metrics(&modes[2]),
            &mut resolution_regressions,
            &mut resolution_mean_deltas,
        );
        accumulate(
            metrics(&modes[2]),
            metrics(&modes[3]),
            &mut fft_grid_regressions,
            &mut fft_grid_mean_deltas,
        );
        accumulate(
            metrics(&modes[3]),
            metrics(&modes[4]),
            &mut frame_geometry_regressions,
            &mut frame_geometry_mean_deltas,
        );
    }
    let direction = classify(
        &regression_from_current,
        &fft_grid_mean_deltas,
        &frame_geometry_mean_deltas,
    );
    GeometryAttributionReview {
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
        resolution_regressions,
        resolution_mean_deltas,
        fft_grid_regressions,
        fft_grid_mean_deltas,
        frame_geometry_regressions,
        frame_geometry_mean_deltas,
        hashes: [manifest_hash, render_hash, measurement_hash, aggregate_hash],
        direction,
    }
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

fn classify(
    from_current: &[[usize; 4]; 4],
    fft_grid: &[f64; 4],
    frame_geometry: &[f64; 4],
) -> GeometryAttributionDirection {
    let all_timbral_rows_regress = from_current
        .iter()
        .all(|counts| counts[2] == 9 && counts[3] == 9);
    let native_grid_improves_both = fft_grid[2] < 0.0 && fft_grid[3] < 0.0;
    let zero_padding_worsens_both = frame_geometry[2] > 0.0 && frame_geometry[3] > 0.0;
    if all_timbral_rows_regress && native_grid_improves_both && zero_padding_worsens_both {
        GeometryAttributionDirection::SharedGridContributesRemainingPathOwns
    } else {
        GeometryAttributionDirection::Unresolved
    }
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-cd-geometry-attribution.tsv")
}
