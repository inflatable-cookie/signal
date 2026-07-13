use std::{fs, path::PathBuf};

use super::super::{
    complete_system_tuning::listening_export::manifest::{rows, source_root},
    study_local_schedule::SOURCE_FRAMES,
    HASH_OFFSET,
};
use super::development_measurement::{
    hard_pass, hash, hash_bytes, measure, read_mono, report, same_samples,
};

mod factor_render;
use factor_render::render_modes;

const MODES: [&str; 9] = [
    "current-signal",
    "event-transport-dual",
    "event-transport-partition",
    "event-analysis-dual",
    "event-analysis-partition",
    "linear-transport-dual",
    "linear-transport-partition",
    "linear-analysis-dual",
    "linear-analysis-partition",
];
const LATTICE_PAIRS: [(usize, usize); 4] = [(0, 4), (1, 5), (2, 6), (3, 7)];
const PHASE_PAIRS: [(usize, usize); 4] = [(0, 2), (1, 3), (4, 6), (5, 7)];
const OVERLAP_PAIRS: [(usize, usize); 4] = [(0, 1), (2, 3), (4, 5), (6, 7)];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum MechanismAttributionDirection {
    WindowedCoefficientRepresentation,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct MechanismAttributionReview {
    pub rows: usize,
    pub modes: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub hard_failures: usize,
    pub hard_failures_by_mode: [usize; 9],
    pub event_fallback_renders: usize,
    pub changed_from_current: [usize; 8],
    pub regression_from_current: [[usize; 4]; 8],
    pub mean_delta_from_current: [[f64; 4]; 8],
    pub lattice_regressions: [[usize; 4]; 4],
    pub lattice_mean_deltas: [[f64; 4]; 4],
    pub phase_regressions: [[usize; 4]; 4],
    pub phase_mean_deltas: [[f64; 4]; 4],
    pub overlap_regressions: [[usize; 4]; 4],
    pub overlap_mean_deltas: [[f64; 4]; 4],
    pub hashes: [u64; 4],
    pub direction: MechanismAttributionDirection,
}

pub(in crate::frequency_adaptive) fn mechanism_attribution_review() -> MechanismAttributionReview {
    let mut evidence = Vec::with_capacity(81);
    let mut manifest_hash = HASH_OFFSET;
    let mut render_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut changed_from_current = [0; 8];

    for row in rows() {
        let source = read_mono(&source_root().join(&row.source), SOURCE_FRAMES);
        let outputs = render_modes(&source, row.ratio);
        hash_bytes(&mut manifest_hash, row.id.as_bytes());
        hash(&mut manifest_hash, row.ratio.to_bits());
        hash_bytes(&mut manifest_hash, row.source.as_bytes());
        for mode in MODES {
            hash_bytes(&mut manifest_hash, mode.as_bytes());
        }
        for mode in 0..8 {
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
    fs::create_dir_all(path.parent().expect("mechanism report parent"))
        .expect("create mechanism report directory");
    fs::write(&path, &report).expect("write mechanism attribution report");
    let mut aggregate_hash = HASH_OFFSET;
    for value in [manifest_hash, render_hash, measurement_hash] {
        hash(&mut aggregate_hash, value);
    }
    hash_bytes(&mut aggregate_hash, report.as_bytes());

    let mut hard_failures_by_mode = [0; 9];
    let mut regression_from_current = [[0; 4]; 8];
    let mut mean_delta_from_current = [[0.0; 4]; 8];
    let mut lattice_regressions = [[0; 4]; 4];
    let mut lattice_mean_deltas = [[0.0; 4]; 4];
    let mut phase_regressions = [[0; 4]; 4];
    let mut phase_mean_deltas = [[0.0; 4]; 4];
    let mut overlap_regressions = [[0; 4]; 4];
    let mut overlap_mean_deltas = [[0.0; 4]; 4];
    for modes in evidence.chunks_exact(9) {
        for (mode, item) in modes.iter().enumerate() {
            hard_failures_by_mode[mode] += usize::from(!hard_pass(item));
        }
        let current = metrics(&modes[0]);
        for mode in 0..8 {
            accumulate(
                current,
                metrics(&modes[mode + 1]),
                &mut regression_from_current[mode],
                &mut mean_delta_from_current[mode],
            );
        }
        accumulate_pairs(
            modes,
            &LATTICE_PAIRS,
            &mut lattice_regressions,
            &mut lattice_mean_deltas,
        );
        accumulate_pairs(
            modes,
            &PHASE_PAIRS,
            &mut phase_regressions,
            &mut phase_mean_deltas,
        );
        accumulate_pairs(
            modes,
            &OVERLAP_PAIRS,
            &mut overlap_regressions,
            &mut overlap_mean_deltas,
        );
    }
    let direction = if regression_from_current
        .iter()
        .all(|counts| counts[2] == 9 && counts[3] == 9)
        && lattice_mean_deltas[0][2].abs() < 0.002
        && lattice_mean_deltas[0][3].abs() < 0.002
        && phase_regressions[0][2] == 9
        && phase_regressions[2][2] == 9
        && overlap_regressions[0][2..] == [9, 9]
        && overlap_regressions[2][2..] == [9, 9]
    {
        MechanismAttributionDirection::WindowedCoefficientRepresentation
    } else {
        MechanismAttributionDirection::Unresolved
    };
    MechanismAttributionReview {
        rows: 9,
        modes: 9,
        renders: evidence.len(),
        holdout_reads: 0,
        hard_failures: evidence.iter().filter(|item| !hard_pass(item)).count(),
        hard_failures_by_mode,
        event_fallback_renders: evidence.iter().filter(|item| item.event_fallback).count(),
        changed_from_current,
        regression_from_current,
        mean_delta_from_current,
        lattice_regressions,
        lattice_mean_deltas,
        phase_regressions,
        phase_mean_deltas,
        overlap_regressions,
        overlap_mean_deltas,
        hashes: [manifest_hash, render_hash, measurement_hash, aggregate_hash],
        direction,
    }
}

fn accumulate_pairs(
    modes: &[super::development_measurement::Evidence],
    pairs: &[(usize, usize); 4],
    regressions: &mut [[usize; 4]; 4],
    means: &mut [[f64; 4]; 4],
) {
    for (index, (before, after)) in pairs.iter().copied().enumerate() {
        accumulate(
            metrics(&modes[before + 1]),
            metrics(&modes[after + 1]),
            &mut regressions[index],
            &mut means[index],
        );
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

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-cb-mechanism-attribution.tsv")
}
