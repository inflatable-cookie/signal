use std::path::PathBuf;

use super::{SharedRotationCorpusReview, SharedRotationCorpusRow};
use crate::frequency_adaptive::{
    adaptive_single_frame_synthesis::development_measurement,
    source_studied::{
        confirmation,
        faithful_predictor::{
            coherent_representation, linked_stereo::shared_rotation_region_locked,
        },
        long_form,
    },
    HASH_OFFSET,
};

const SAMPLE_RATE: usize = 44_100;
const SOURCE_FRAMES: usize = 220_500;

pub(in super::super) fn review(
    renderer: fn([&[f64]; 2], f64, usize) -> shared_rotation_region_locked::SharedRotationRender,
    label: &'static str,
) -> SharedRotationCorpusReview {
    let first = run(renderer, label);
    let second = run(renderer, label);
    SharedRotationCorpusReview {
        candidate_hard_failures: first.rows.iter().filter(|row| !row.hard_passes[1]).count(),
        row_complete_regressions: first
            .rows
            .iter()
            .filter(|row| row.candidate_regressions == row.current.len())
            .count(),
        repeated: first == second,
        rows: first.rows,
        hash: first.hash,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<SharedRotationCorpusRow>,
    hash: u64,
}

fn run(
    renderer: fn([&[f64]; 2], f64, usize) -> shared_rotation_region_locked::SharedRotationRender,
    label: &'static str,
) -> Run {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-db-exact-source");
    let mut rows = Vec::new();
    let mut hash = HASH_OFFSET;
    for case in long_form::cases() {
        let source = confirmation::read_exact(
            &root.join("inputs").join(format!("{}.wav", case.id)),
            SOURCE_FRAMES,
            16,
        );
        let target = (SOURCE_FRAMES as f64 * case.ratio).round() as usize;
        let rubber_band = confirmation::read_exact(
            &root.join("rubber-band-r3").join(format!("{}.wav", case.id)),
            target,
            0,
        );
        let current = coherent_representation::render(&source, case.ratio, SAMPLE_RATE);
        let silence = vec![0.0; source.len()];
        let candidate = renderer([&source, &silence], case.ratio, SAMPLE_RATE);
        let source_f32 = as_f32(&source);
        let current_evidence = development_measurement::measure(
            case.id,
            case.ratio,
            "coherent-control",
            &source_f32,
            &as_f32(&current.samples),
        );
        let candidate_evidence = development_measurement::measure(
            case.id,
            case.ratio,
            label,
            &source_f32,
            &as_f32(&candidate.channels[0]),
        );
        let rubber_evidence = development_measurement::measure(
            case.id,
            case.ratio,
            "rubber-band-r3",
            &source_f32,
            &as_f32(&rubber_band),
        );
        let current_metrics = quality_fields(&current_evidence);
        let candidate_metrics = quality_fields(&candidate_evidence);
        let rubber_metrics = quality_fields(&rubber_evidence);
        let candidate_regressions = current_metrics
            .iter()
            .zip(candidate_metrics)
            .filter(|(current, candidate)| candidate > *current)
            .count();
        let hashes = [
            current_evidence.render_hash,
            candidate_evidence.render_hash,
            rubber_evidence.render_hash,
        ];
        for value in hashes {
            mix(&mut hash, value);
        }
        rows.push(SharedRotationCorpusRow {
            id: case.id,
            ratio: case.ratio,
            current: current_metrics,
            candidate: candidate_metrics,
            rubber_band: rubber_metrics,
            candidate_regressions,
            hard_passes: [
                development_measurement::hard_pass(&current_evidence),
                development_measurement::hard_pass(&candidate_evidence),
                development_measurement::hard_pass(&rubber_evidence),
            ],
            hashes,
        });
    }
    Run { rows, hash }
}

fn quality_fields(evidence: &development_measurement::Evidence) -> [f64; 7] {
    [
        evidence.mean_event_offset,
        evidence.replica_ratio,
        evidence.static_residual,
        evidence.unsupported_mass,
        evidence.formant_residual,
        evidence.formant_shift_hz,
        evidence.boundary_growth_db,
    ]
}

fn as_f32(samples: &[f64]) -> Vec<f32> {
    samples.iter().map(|sample| *sample as f32).collect()
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
