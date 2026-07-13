use std::{fs, path::PathBuf};

use crate::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::super::{
    complete_system_tuning::listening_export::manifest::{render_root, rows, source_root},
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
    render::render_successor_owned,
};

const CURRENT: &str = "current-signal";
const SUCCESSOR: &str = "event-owned-successor";
const EXTERNAL: &str = "rubber-band-r3";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum DevelopmentDirection {
    ConcealedDevelopmentComparison,
    OwningMechanism,
    SpectralSynthesisAttribution,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct DevelopmentObjectiveReview {
    pub rows: usize,
    pub modes: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub hard_failures: usize,
    pub candidate_hard_failures: usize,
    pub candidate_changed_rows: usize,
    pub event_fallback_renders: usize,
    pub candidate_regression_rows: [usize; 4],
    pub hashes: [u64; 4],
    pub direction: DevelopmentDirection,
}

pub(in crate::frequency_adaptive) fn development_objective_review() -> DevelopmentObjectiveReview {
    let mut evidence = Vec::with_capacity(27);
    let mut manifest_hash = HASH_OFFSET;
    let mut render_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut candidate_changed_rows = 0;

    for row in rows() {
        let source = read_mono(&source_root().join(&row.source), SOURCE_FRAMES);
        let expected = (SOURCE_FRAMES as f64 * row.ratio).round() as usize;
        let current = render_current(&source, row.ratio);
        let successor = render_successor(&source, row.ratio);
        let external = read_mono(&render_root().join(&row.rubber_band), expected);
        candidate_changed_rows += usize::from(!same_samples(&successor, &current));

        hash_bytes(&mut manifest_hash, row.id.as_bytes());
        hash(&mut manifest_hash, row.ratio.to_bits());
        hash_bytes(&mut manifest_hash, row.source.as_bytes());
        hash_bytes(&mut manifest_hash, row.rubber_band.as_bytes());

        for (mode, output) in [
            (CURRENT, current),
            (SUCCESSOR, successor),
            (EXTERNAL, external),
        ] {
            let item = measure(row.id, row.ratio, mode, &source, &output);
            hash(&mut render_hash, item.render_hash);
            hash(&mut measurement_hash, item.measurement_hash);
            evidence.push(item);
        }
    }

    let report = report(&evidence);
    let path = report_path();
    fs::create_dir_all(path.parent().expect("development report parent"))
        .expect("create development report directory");
    fs::write(&path, &report).expect("write development objective report");
    let mut aggregate_hash = HASH_OFFSET;
    for value in [manifest_hash, render_hash, measurement_hash] {
        hash(&mut aggregate_hash, value);
    }
    hash_bytes(&mut aggregate_hash, report.as_bytes());

    let hard_failures = evidence.iter().filter(|item| !hard_pass(item)).count();
    let candidate_hard_failures = evidence
        .iter()
        .filter(|item| item.mode == SUCCESSOR && !hard_pass(item))
        .count();
    let event_fallback_renders = evidence.iter().filter(|item| item.event_fallback).count();
    let mut candidate_regression_rows = [0; 4];
    for modes in evidence.chunks_exact(3) {
        let current = &modes[0];
        let candidate = &modes[1];
        candidate_regression_rows[0] +=
            usize::from(candidate.mean_event_offset > current.mean_event_offset);
        candidate_regression_rows[1] +=
            usize::from(candidate.replica_ratio > current.replica_ratio);
        candidate_regression_rows[2] +=
            usize::from(candidate.static_residual > current.static_residual);
        candidate_regression_rows[3] +=
            usize::from(candidate.formant_residual > current.formant_residual);
    }
    let broad_objective_regression = candidate_regression_rows[0] >= 5
        && candidate_regression_rows[1] >= 5
        && candidate_regression_rows[2] == 9
        && candidate_regression_rows[3] == 9;
    DevelopmentObjectiveReview {
        rows: 9,
        modes: 3,
        renders: evidence.len(),
        holdout_reads: 0,
        hard_failures,
        candidate_hard_failures,
        candidate_changed_rows,
        event_fallback_renders,
        candidate_regression_rows,
        hashes: [manifest_hash, render_hash, measurement_hash, aggregate_hash],
        direction: if candidate_hard_failures != 0 {
            DevelopmentDirection::OwningMechanism
        } else if broad_objective_regression {
            DevelopmentDirection::SpectralSynthesisAttribution
        } else {
            DevelopmentDirection::ConcealedDevelopmentComparison
        },
    }
}

fn render_current(source: &[f32], ratio: f64) -> Vec<f32> {
    OfflineHighQualityStretcher::with_path(ratio, OfflineHighQualityPath::Default)
        .stretch_mono(source)
}

fn render_successor(source: &[f32], ratio: f64) -> Vec<f32> {
    let channels = vec![source
        .iter()
        .map(|sample| f64::from(*sample))
        .collect::<Vec<_>>()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let anchors = detect(&channels, SOURCE_FRAMES);
    render_successor_owned(&channels, ratio, &points, &anchors.positions, &schedule)
        .samples
        .remove(0)
        .into_iter()
        .map(|sample| sample as f32)
        .collect()
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-by-development-objective.tsv")
}
