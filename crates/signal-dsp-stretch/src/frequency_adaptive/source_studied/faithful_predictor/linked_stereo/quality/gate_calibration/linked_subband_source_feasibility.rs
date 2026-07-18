mod inputs;
mod measurement;
mod report;
mod specimen;
#[cfg(test)]
mod tests;

use std::{fs, path::PathBuf};

use inputs::PreparedInputs;
use specimen::{ResourceSummary, Specimen};

use super::{
    external::replace_directory,
    metrics::{self, evaluate, Metrics},
    relation_repair::transform::local_evidence,
    CALIBRATED_IMAGE_CORRELATION, CALIBRATED_IMAGE_MID_SIDE_DB, CALIBRATED_IMAGE_RELATION_RESIDUAL,
    CALIBRATED_TONE_IPD_RADIANS, SAMPLE_RATE,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    coherent_representation, render,
};

const PINNED_REVISION: &str = "e99cd7e6c6367e476577be34d2fdbe2023904d7e";
const ADAPTER_VERSION: &str =
    "sbsms-specimen-adapter 2.3.0 e99cd7e6c6367e476577be34d2fdbe2023904d7e";
const MONO_SAMPLE_RATE: usize = 44_100;
const MONO_FRAMES: usize = 44_100;
const MONO_RATIOS: [f64; 4] = [0.75, 1.0, 1.5, 2.0];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum LinkedSubbandFeasibilityDirection {
    CleanRoomProof,
    CloseCandidate,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LinkedSubbandFeasibilityReview {
    pub(in crate::frequency_adaptive) revision: String,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) stereo_rows: usize,
    pub(in crate::frequency_adaptive) stereo_failures: usize,
    pub(in crate::frequency_adaptive) local_consistency_failures: usize,
    pub(in crate::frequency_adaptive) mechanics_errors: [f64; 6],
    pub(in crate::frequency_adaptive) mono_hard_failures: usize,
    pub(in crate::frequency_adaptive) mono_row_complete_regressions: usize,
    pub(in crate::frequency_adaptive) metrics_worse_than_both_controls: usize,
    pub(in crate::frequency_adaptive) maximum_tracks_per_time: u64,
    pub(in crate::frequency_adaptive) maximum_track_visits_per_output_read: u64,
    pub(in crate::frequency_adaptive) maximum_peak_rss_bytes: u64,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) direction: LinkedSubbandFeasibilityDirection,
}

#[derive(Clone, Debug, PartialEq)]
struct StereoRow {
    ratio: f64,
    source_frames: usize,
    phase: f64,
    bin_aligned: bool,
    control: &'static str,
    current: [Metrics; 2],
    candidate: [Metrics; 2],
    structural_failures: usize,
    local_windows_improved: usize,
    maximum_local_residuals: [f64; 2],
    output_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    stereo: Vec<StereoRow>,
    mechanics_errors: [f64; 6],
    mono_hard_failures: usize,
    mono_row_complete_regressions: usize,
    metrics_worse_than_both_controls: usize,
    mono_report: String,
    resources: ResourceSummary,
    evidence_hash: u64,
}

pub(in crate::frequency_adaptive) fn review() -> LinkedSubbandFeasibilityReview {
    let specimen = Specimen::discover();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-sbsms-source-feasibility");
    replace_directory(&root);
    let prepared = inputs::prepare(&root);
    let first = run(&specimen, &root.join("first"), &prepared);
    let second = run(&specimen, &root.join("second"), &prepared);
    let repeated = first.evidence_hash == second.evidence_hash
        && first.stereo == second.stereo
        && first.mechanics_errors == second.mechanics_errors
        && first.mono_hard_failures == second.mono_hard_failures
        && first.mono_row_complete_regressions == second.mono_row_complete_regressions
        && first.metrics_worse_than_both_controls == second.metrics_worse_than_both_controls
        && first.resources.deterministic_fields() == second.resources.deterministic_fields();
    let stereo_failures = first
        .stereo
        .iter()
        .filter(|row| !stereo_gate(row.control, row.candidate))
        .count();
    let local_consistency_failures = first
        .stereo
        .iter()
        .filter(|row| {
            row.maximum_local_residuals[1] > row.maximum_local_residuals[0] + 1.0e-12
                || row.local_windows_improved < 4
        })
        .count();
    let quality_pass = repeated
        && stereo_failures == 0
        && local_consistency_failures == 0
        && first.mechanics_errors.iter().all(|error| *error <= 1.0e-6)
        && first.mono_hard_failures == 0
        && first.mono_row_complete_regressions == 0
        && first.metrics_worse_than_both_controls == 0;
    let direction = if quality_pass {
        LinkedSubbandFeasibilityDirection::CleanRoomProof
    } else {
        LinkedSubbandFeasibilityDirection::CloseCandidate
    };
    report::write(
        &root,
        &specimen,
        &first,
        repeated,
        stereo_failures,
        local_consistency_failures,
        direction,
    );
    LinkedSubbandFeasibilityReview {
        revision: specimen.revision,
        repeated,
        stereo_rows: first.stereo.len(),
        stereo_failures,
        local_consistency_failures,
        mechanics_errors: first.mechanics_errors,
        mono_hard_failures: first.mono_hard_failures,
        mono_row_complete_regressions: first.mono_row_complete_regressions,
        metrics_worse_than_both_controls: first.metrics_worse_than_both_controls,
        maximum_tracks_per_time: first.resources.maximum_tracks_per_time,
        maximum_track_visits_per_output_read: first.resources.maximum_track_visits_per_output_read,
        maximum_peak_rss_bytes: first.resources.maximum_peak_rss_bytes,
        evidence_hash: first.evidence_hash,
        direction,
    }
}

fn run(specimen: &Specimen, root: &std::path::Path, prepared: &PreparedInputs) -> Run {
    fs::create_dir_all(root).expect("create SBSMS run root");
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let trim = geometry[0];
    let mut stereo = Vec::with_capacity(prepared.stereo.len());
    let mut resources = ResourceSummary::default();
    let mut evidence_hash = 0xcbf2_9ce4_8422_2325_u64;
    for (index, item) in prepared.stereo.iter().enumerate() {
        let rendered = specimen.render(
            root,
            &format!("stereo-{index:02}"),
            &[&item.source[0], &item.source[1]],
            item.ratio,
        );
        resources.add(rendered.stats);
        let candidate: [Vec<f64>; 2] = rendered.channels.try_into().expect("stereo result");
        let current = render::linked([&item.source[0], &item.source[1]], item.ratio, SAMPLE_RATE);
        let source_trim = ((trim as f64 / item.ratio).ceil() as usize).min(item.source_frames / 3);
        let output_trim = trim.min(candidate[0].len() / 3);
        let pair = |output: &[Vec<f64>; 2]| {
            [
                evaluate(&item.source, output, item.frequency, SAMPLE_RATE),
                evaluate(
                    &metrics::crop(&item.source, source_trim),
                    &metrics::crop(output, output_trim),
                    item.frequency,
                    SAMPLE_RATE,
                ),
            ]
        };
        let (local_windows_improved, _, maximum_local_residuals) =
            local_evidence(&item.source, &current.channels, &candidate);
        let structural_failures = candidate
            .iter()
            .flatten()
            .filter(|sample| !sample.is_finite())
            .count()
            + usize::from(candidate.iter().any(|channel| {
                channel.len() != (item.source_frames as f64 * item.ratio).round() as usize
            }));
        mix(&mut evidence_hash, rendered.output_hash);
        stereo.push(StereoRow {
            ratio: item.ratio,
            source_frames: item.source_frames,
            phase: item.phase,
            bin_aligned: item.bin_aligned,
            control: item.kind.name(),
            current: pair(&current.channels),
            candidate: pair(&candidate),
            structural_failures,
            local_windows_improved,
            maximum_local_residuals,
            output_hash: rendered.output_hash,
        });
    }

    let mechanics_errors =
        measurement::mechanics(specimen, root, &mut resources, &mut evidence_hash);
    let (
        mono_hard_failures,
        mono_row_complete_regressions,
        metrics_worse_than_both_controls,
        mono_report,
    ) = measurement::mono(
        specimen,
        root,
        &prepared.mono,
        &mut resources,
        &mut evidence_hash,
    );
    for field in resources.deterministic_fields() {
        mix(&mut evidence_hash, field);
    }
    for error in mechanics_errors {
        mix(&mut evidence_hash, error.to_bits());
    }
    Run {
        stereo,
        mechanics_errors,
        mono_hard_failures,
        mono_row_complete_regressions,
        metrics_worse_than_both_controls,
        mono_report,
        resources,
        evidence_hash,
    }
}

fn stereo_gate(control: &str, values: [Metrics; 2]) -> bool {
    values.into_iter().all(|metrics| {
        if control == "tone" {
            metrics.ipd_error_radians <= CALIBRATED_TONE_IPD_RADIANS
        } else {
            metrics.mid_side_delta_db <= CALIBRATED_IMAGE_MID_SIDE_DB
                && metrics.correlation_delta <= CALIBRATED_IMAGE_CORRELATION
                && metrics.relation_residual <= CALIBRATED_IMAGE_RELATION_RESIDUAL
        }
    })
}

fn mix(hash: &mut u64, value: u64) {
    *hash = (*hash ^ value).wrapping_mul(0x100_0000_01b3);
}
