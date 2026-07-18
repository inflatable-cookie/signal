use std::{fs, path::PathBuf};

use super::{
    external::{read_stereo, replace_directory, write_stereo},
    metrics::{self, control, evaluate, ControlKind, Metrics},
    relation_repair::transform::local_evidence,
    ALIGNMENTS, CALIBRATED_IMAGE_CORRELATION, CALIBRATED_IMAGE_MID_SIDE_DB,
    CALIBRATED_IMAGE_RELATION_RESIDUAL, CALIBRATED_TONE_IPD_RADIANS, LENGTHS, PHASES, RATIOS,
    SAMPLE_RATE,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    coherent_representation, mechanics, render,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum PeakRegionDirection {
    Accept,
    Reject,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PeakRegionRow {
    pub ratio: f64,
    pub source_frames: usize,
    pub phase: f64,
    pub bin_aligned: bool,
    pub control: &'static str,
    pub current: [Metrics; 2],
    pub candidate: [Metrics; 2],
    pub structural_failures: usize,
    pub local_windows_improved: usize,
    pub maximum_local_residuals: [f64; 2],
    pub peak_region_counts: [usize; 4],
    pub relation_states: [usize; 5],
    pub maximum_relation_error: f64,
    pub hashes: [u64; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PeakRegionReview {
    pub rows: Vec<PeakRegionRow>,
    pub current_failures: usize,
    pub candidate_failures: usize,
    pub row_complete_improvements: usize,
    pub metric_regressions: usize,
    pub local_consistency_failures: usize,
    pub mechanics_errors: [f64; 5],
    pub silent_peer_peak: f64,
    pub peak_region_counts: [usize; 4],
    pub relation_states: [usize; 5],
    pub maximum_relation_error: f64,
    pub evidence_hash: u64,
    pub repeated: bool,
    pub direction: PeakRegionDirection,
}

pub(in crate::frequency_adaptive) fn review() -> PeakRegionReview {
    review_candidate(
        "stretch-linked-stereo-peak-region",
        render::linked_peak_regions,
    )
}

pub(in crate::frequency_adaptive) fn review_candidate(
    directory: &str,
    candidate: impl Fn([&[f64]; 2], f64, usize) -> render::StereoRender + Copy,
) -> PeakRegionReview {
    review_candidate_inner(directory, candidate, false)
}

pub(in crate::frequency_adaptive) fn replay_development_rows(
    directory: &str,
    mut visit: impl FnMut(&str, usize, f64, bool, f64, [&[f64]; 2]),
) -> usize {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(directory);
    replace_directory(&root);
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
    let mut rows = 0;
    for source_frames in LENGTHS {
        for phase in PHASES {
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * spacing;
                for kind in [ControlKind::Tone, ControlKind::Image] {
                    let source = control(kind, source_frames, frequency, phase);
                    for ratio in RATIOS {
                        let stem = format!(
                            "{}-{source_frames}-{phase:.2}-{bin_aligned}-{ratio:.2}",
                            kind.name()
                        );
                        let input_path = root.join(format!("{stem}-input.wav"));
                        write_stereo(&input_path, &source, SAMPLE_RATE as u32);
                        let input = read_stereo(&input_path, source_frames, SAMPLE_RATE as u32);
                        visit(
                            kind.name(),
                            source_frames,
                            phase,
                            bin_aligned,
                            ratio,
                            [&input[0], &input[1]],
                        );
                        rows += 1;
                    }
                }
            }
        }
    }
    assert_eq!(rows, 48);
    rows
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PeakRegionScreen {
    pub(in crate::frequency_adaptive) rows: usize,
    pub(in crate::frequency_adaptive) structural_failures: usize,
    pub(in crate::frequency_adaptive) candidate_failures: usize,
    pub(in crate::frequency_adaptive) metric_regressions: usize,
    pub(in crate::frequency_adaptive) local_consistency_failures: usize,
    pub(in crate::frequency_adaptive) peak_region_counts: [usize; 4],
    pub(in crate::frequency_adaptive) evidence_hash: u64,
}

pub(in crate::frequency_adaptive) fn screen_candidate(
    directory: &str,
    candidate: impl Fn([&[f64]; 2], f64, usize) -> render::StereoRender + Copy,
) -> PeakRegionScreen {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(directory);
    replace_directory(&root);
    let run = run(&root, candidate, true, false);
    summarize_screen(&run)
}

fn review_candidate_inner(
    directory: &str,
    candidate: impl Fn([&[f64]; 2], f64, usize) -> render::StereoRender + Copy,
    short: bool,
) -> PeakRegionReview {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(directory);
    replace_directory(&root);
    let first = run(&root.join("first"), candidate, short, true);
    let second = run(&root.join("second"), candidate, short, true);
    let repeated = first == second;
    let current_failures = first
        .rows
        .iter()
        .filter(|row| !gate(row.control, row.current))
        .count();
    let candidate_failures = first
        .rows
        .iter()
        .filter(|row| !gate(row.control, row.candidate))
        .count();
    let row_complete_improvements = first
        .rows
        .iter()
        .filter(|row| improvement(row).0 && improvement(row).1)
        .count();
    let metric_regressions = first.rows.iter().filter(|row| !improvement(row).0).count();
    let local_consistency_failures = first
        .rows
        .iter()
        .filter(|row| {
            row.maximum_local_residuals[1] > row.maximum_local_residuals[0] + 1.0e-12
                || row.local_windows_improved < 4
        })
        .count();
    let relation_states = first.rows.iter().fold([0_usize; 5], |mut total, row| {
        for (slot, count) in total.iter_mut().zip(row.relation_states) {
            *slot += count;
        }
        total
    });
    let maximum_relation_error = first
        .rows
        .iter()
        .map(|row| row.maximum_relation_error)
        .fold(0.0_f64, f64::max);
    let direction = if repeated
        && candidate_failures == 0
        && row_complete_improvements == first.rows.len()
        && metric_regressions == 0
        && local_consistency_failures == 0
        && first.mechanics_errors.iter().all(|error| *error <= 1.0e-12)
        && first.silent_peer_peak == 0.0
        && first.peak_region_counts.iter().all(|count| *count > 0)
        && relation_states[2] == 0
        && maximum_relation_error <= 1.0e-12
    {
        PeakRegionDirection::Accept
    } else {
        PeakRegionDirection::Reject
    };
    write_report(
        &root,
        &first,
        repeated,
        current_failures,
        candidate_failures,
        row_complete_improvements,
        metric_regressions,
        local_consistency_failures,
        direction,
    );
    PeakRegionReview {
        rows: first.rows,
        current_failures,
        candidate_failures,
        row_complete_improvements,
        metric_regressions,
        local_consistency_failures,
        mechanics_errors: first.mechanics_errors,
        silent_peer_peak: first.silent_peer_peak,
        peak_region_counts: first.peak_region_counts,
        relation_states,
        maximum_relation_error,
        evidence_hash: first.evidence_hash,
        repeated,
        direction,
    }
}

fn summarize_screen(run: &Run) -> PeakRegionScreen {
    PeakRegionScreen {
        rows: run.rows.len(),
        structural_failures: run.rows.iter().map(|row| row.structural_failures).sum(),
        candidate_failures: run
            .rows
            .iter()
            .filter(|row| !gate(row.control, row.candidate))
            .count(),
        metric_regressions: run.rows.iter().filter(|row| !improvement(row).0).count(),
        local_consistency_failures: run
            .rows
            .iter()
            .filter(|row| {
                row.maximum_local_residuals[1] > row.maximum_local_residuals[0] + 1.0e-12
                    || row.local_windows_improved < 4
            })
            .count(),
        peak_region_counts: run.peak_region_counts,
        evidence_hash: run.evidence_hash,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<PeakRegionRow>,
    mechanics_errors: [f64; 5],
    silent_peer_peak: f64,
    peak_region_counts: [usize; 4],
    evidence_hash: u64,
}

fn run(
    root: &std::path::Path,
    candidate: impl Fn([&[f64]; 2], f64, usize) -> render::StereoRender + Copy,
    short: bool,
    include_mechanics: bool,
) -> Run {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let trim = geometry[0];
    let spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
    let mut rows = Vec::new();
    let mut evidence_hash = 0xcbf2_9ce4_8422_2325;
    let mut peak_region_counts = [0; 4];
    for source_frames in LENGTHS {
        for phase in PHASES {
            if short && (source_frames != LENGTHS[0] || phase != PHASES[0]) {
                continue;
            }
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * spacing;
                for kind in [ControlKind::Tone, ControlKind::Image] {
                    let source = control(kind, source_frames, frequency, phase);
                    for ratio in RATIOS {
                        let stem = format!(
                            "{}-{source_frames}-{phase:.2}-{bin_aligned}-{ratio:.2}",
                            kind.name()
                        );
                        let input_path = root.join(format!("{stem}-input.wav"));
                        write_stereo(&input_path, &source, SAMPLE_RATE as u32);
                        let input = read_stereo(&input_path, source_frames, SAMPLE_RATE as u32);
                        let current = render::linked([&input[0], &input[1]], ratio, SAMPLE_RATE);
                        let candidate = candidate([&input[0], &input[1]], ratio, SAMPLE_RATE);
                        let row = measure(
                            kind,
                            ratio,
                            source_frames,
                            phase,
                            bin_aligned,
                            frequency,
                            trim,
                            &input,
                            current,
                            candidate,
                        );
                        for value in row
                            .hashes
                            .into_iter()
                            .chain(row.peak_region_counts.map(|count| count as u64))
                        {
                            evidence_hash = (evidence_hash ^ value).wrapping_mul(0x100_0000_01b3);
                        }
                        if row.relation_states != [0; 5] || row.maximum_relation_error != 0.0 {
                            for value in row
                                .relation_states
                                .map(|count| count as u64)
                                .into_iter()
                                .chain([row.maximum_relation_error.to_bits()])
                            {
                                evidence_hash =
                                    (evidence_hash ^ value).wrapping_mul(0x100_0000_01b3);
                            }
                        }
                        for (total, count) in
                            peak_region_counts.iter_mut().zip(row.peak_region_counts)
                        {
                            *total += count;
                        }
                        rows.push(row);
                    }
                }
            }
        }
    }
    let (mechanics_errors, silent_peer_peak) = if include_mechanics {
        mechanics_review(candidate)
    } else {
        ([0.0; 5], 0.0)
    };
    for value in mechanics_errors
        .map(f64::to_bits)
        .into_iter()
        .chain([silent_peer_peak.to_bits(), evidence_hash])
    {
        evidence_hash = (evidence_hash ^ value).wrapping_mul(0x100_0000_01b3);
    }
    Run {
        rows,
        mechanics_errors,
        silent_peer_peak,
        peak_region_counts,
        evidence_hash,
    }
}

#[allow(clippy::too_many_arguments)]
fn measure(
    kind: ControlKind,
    ratio: f64,
    source_frames: usize,
    phase: f64,
    bin_aligned: bool,
    frequency: f64,
    trim: usize,
    input: &[Vec<f64>; 2],
    current: render::StereoRender,
    candidate: render::StereoRender,
) -> PeakRegionRow {
    let source_trim = ((trim as f64 / ratio).ceil() as usize).min(source_frames / 3);
    let output_trim = trim.min(candidate.channels[0].len() / 3);
    let pair = |output: &[Vec<f64>; 2]| {
        [
            evaluate(input, output, frequency, SAMPLE_RATE),
            evaluate(
                &metrics::crop(input, source_trim),
                &metrics::crop(output, output_trim),
                frequency,
                SAMPLE_RATE,
            ),
        ]
    };
    let (local_windows_improved, _, maximum_local_residuals) =
        local_evidence(input, &current.channels, &candidate.channels);
    let relation_states = [
        candidate.shared_corrected,
        candidate.shared_fallback,
        candidate.unilateral_non_silent_completions,
        candidate.reference_bins[0],
        candidate.reference_bins[1],
    ];
    let maximum_relation_error = candidate.maximum_constrained_relation_error;
    PeakRegionRow {
        ratio,
        source_frames,
        phase,
        bin_aligned,
        control: kind.name(),
        current: pair(&current.channels),
        candidate: pair(&candidate.channels),
        structural_failures: candidate.uncovered
            + candidate.non_finite
            + candidate.boundary_failures
            + usize::from(
                candidate.channels.iter().any(|channel| {
                    channel.len() != (source_frames as f64 * ratio).round() as usize
                }),
            ),
        local_windows_improved,
        maximum_local_residuals,
        peak_region_counts: candidate.peak_region_counts,
        relation_states,
        maximum_relation_error,
        hashes: [current.hash, candidate.hash],
    }
}

fn gate(control: &str, metrics: [Metrics; 2]) -> bool {
    metrics.into_iter().all(|metrics| {
        if control == "tone" {
            metrics.ipd_error_radians <= CALIBRATED_TONE_IPD_RADIANS
        } else {
            metrics.mid_side_delta_db <= CALIBRATED_IMAGE_MID_SIDE_DB
                && metrics.correlation_delta <= CALIBRATED_IMAGE_CORRELATION
                && metrics.relation_residual <= CALIBRATED_IMAGE_RELATION_RESIDUAL
        }
    })
}

fn improvement(row: &PeakRegionRow) -> (bool, bool) {
    let pairs = row.current.into_iter().zip(row.candidate);
    if row.control == "tone" {
        let values = pairs
            .map(|(current, candidate)| (current.ipd_error_radians, candidate.ipd_error_radians))
            .collect::<Vec<_>>();
        (
            values
                .iter()
                .all(|(before, after)| *after <= *before + 1.0e-12),
            values.iter().any(|(before, after)| *after < *before),
        )
    } else {
        let values = pairs
            .flat_map(|(current, candidate)| {
                [
                    (current.mid_side_delta_db, candidate.mid_side_delta_db),
                    (current.correlation_delta, candidate.correlation_delta),
                    (current.relation_residual, candidate.relation_residual),
                ]
            })
            .collect::<Vec<_>>();
        (
            values
                .iter()
                .all(|(before, after)| *after <= *before + 1.0e-12),
            values.iter().any(|(before, after)| *after < *before),
        )
    }
}

fn mechanics_review(
    candidate: impl Fn([&[f64]; 2], f64, usize) -> render::StereoRender + Copy,
) -> ([f64; 5], f64) {
    let primary = mechanics::primary_control(SAMPLE_RATE);
    let secondary = mechanics::secondary_control(SAMPLE_RATE);
    let silence = vec![0.0; primary.len()];
    let mut errors = [0.0_f64; 5];
    let mut silent_peer_peak = 0.0_f64;
    for ratio in RATIOS {
        let ordinary = candidate([&primary, &secondary], ratio, SAMPLE_RATE).channels;
        let duplicate = candidate([&primary, &primary], ratio, SAMPLE_RATE).channels;
        let duplicate_expected = render::linked([&primary, &primary], ratio, SAMPLE_RATE).channels;
        errors[0] = errors[0].max(maximum_error(&duplicate, &duplicate_expected, 1.0));

        let hard_pan = candidate([&primary, &silence], ratio, SAMPLE_RATE).channels;
        let hard_pan_expected = render::linked([&primary, &silence], ratio, SAMPLE_RATE).channels;
        errors[1] = errors[1].max(maximum_error(&hard_pan, &hard_pan_expected, 1.0));
        silent_peer_peak = silent_peer_peak.max(
            hard_pan[1]
                .iter()
                .map(|sample| sample.abs())
                .fold(0.0, f64::max),
        );

        let swapped = candidate([&secondary, &primary], ratio, SAMPLE_RATE).channels;
        errors[2] = errors[2].max(maximum_error(
            &swapped,
            &[ordinary[1].clone(), ordinary[0].clone()],
            1.0,
        ));

        let negative_primary = mechanics::scaled(&primary, -1.0);
        let negative_secondary = mechanics::scaled(&secondary, -1.0);
        let negative =
            candidate([&negative_primary, &negative_secondary], ratio, SAMPLE_RATE).channels;
        errors[3] = errors[3].max(maximum_error(&negative, &ordinary, -1.0));

        for gain in [0.25, 4.0] {
            let gained = mechanics::scaled(&primary, gain);
            let actual = candidate([&gained, &gained], ratio, SAMPLE_RATE).channels;
            let expected = render::linked([&gained, &gained], ratio, SAMPLE_RATE).channels;
            errors[4] = errors[4].max(maximum_error(&actual, &expected, 1.0));
        }
    }
    (errors, silent_peer_peak)
}

fn maximum_error(actual: &[Vec<f64>; 2], expected: &[Vec<f64>; 2], gain: f64) -> f64 {
    actual
        .iter()
        .zip(expected)
        .flat_map(|(actual, expected)| actual.iter().zip(expected))
        .map(|(actual, expected)| (actual - expected * gain).abs())
        .fold(0.0, f64::max)
}

#[allow(clippy::too_many_arguments)]
fn write_report(
    root: &std::path::Path,
    run: &Run,
    repeated: bool,
    current_failures: usize,
    candidate_failures: usize,
    row_complete_improvements: usize,
    metric_regressions: usize,
    local_consistency_failures: usize,
    direction: PeakRegionDirection,
) {
    let mut report = format!(
        "repeated\t{repeated}\ncurrent_failures\t{current_failures}\ncandidate_failures\t{candidate_failures}\nrow_complete_improvements\t{row_complete_improvements}\nmetric_regressions\t{metric_regressions}\nlocal_consistency_failures\t{local_consistency_failures}\nmechanics_errors\t{:e},{:e},{:e},{:e},{:e}\nsilent_peer_peak\t{:e}\npeak_region_counts\t{},{},{},{}\nevidence_hash\t{:016x}\ndirection\t{direction:?}\nratio\tframes\tphase\tbin_aligned\tcontrol\tscope\tcurrent_ipd\tcandidate_ipd\tcurrent_mid_side\tcandidate_mid_side\tcurrent_correlation\tcandidate_correlation\tcurrent_relation\tcandidate_relation\tstructural_failures\tlocal_improved\tlocal_before\tlocal_after\tregions\teligible\tshared_bins\tindependent_bins\ttwo_defined\tone_defined\tundefined\tzero_peer\tsilent\tmaximum_relation_error\tcurrent_hash\tcandidate_hash\n",
        run.mechanics_errors[0],
        run.mechanics_errors[1],
        run.mechanics_errors[2],
        run.mechanics_errors[3],
        run.mechanics_errors[4],
        run.silent_peer_peak,
        run.peak_region_counts[0],
        run.peak_region_counts[1],
        run.peak_region_counts[2],
        run.peak_region_counts[3],
        run.evidence_hash,
    );
    for row in &run.rows {
        for scope in 0..2 {
            let current = row.current[scope];
            let candidate = row.candidate[scope];
            report.push_str(&format!("{:.2}\t{}\t{:.2}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{:.12e}\t{:.12e}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.12e}\t{:016x}\t{:016x}\n", row.ratio, row.source_frames, row.phase, row.bin_aligned, row.control, ["whole", "interior"][scope], current.ipd_error_radians, candidate.ipd_error_radians, current.mid_side_delta_db, candidate.mid_side_delta_db, current.correlation_delta, candidate.correlation_delta, current.relation_residual, candidate.relation_residual, row.structural_failures, row.local_windows_improved, row.maximum_local_residuals[0], row.maximum_local_residuals[1], row.peak_region_counts[0], row.peak_region_counts[1], row.peak_region_counts[2], row.peak_region_counts[3], row.relation_states[0], row.relation_states[1], row.relation_states[2], row.relation_states[3], row.relation_states[4], row.maximum_relation_error, row.hashes[0], row.hashes[1]));
        }
    }
    fs::write(root.join("peak-region-feasibility.tsv"), report)
        .expect("write peak-region feasibility report");
}
