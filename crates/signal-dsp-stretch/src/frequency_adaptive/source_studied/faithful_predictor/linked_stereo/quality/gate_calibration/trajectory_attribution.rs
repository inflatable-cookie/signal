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
    coherent_representation, render,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum TrajectoryAttributionDirection {
    IndependentRecurrenceDominatesLoss,
    SharedRegionsDominateLoss,
    Mixed,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct TrajectoryAttributionRow {
    pub ratio: f64,
    pub source_frames: usize,
    pub phase: f64,
    pub bin_aligned: bool,
    pub control: &'static str,
    pub metrics: [[Metrics; 2]; 3],
    pub structural_failures: [usize; 2],
    pub local_windows_improved: [usize; 2],
    pub maximum_local_residuals: [f64; 3],
    pub peak_region_counts: [usize; 4],
    pub hashes: [u64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct TrajectoryAttributionReview {
    pub rows: Vec<TrajectoryAttributionRow>,
    pub failures: [usize; 3],
    pub baseline_to_independent: [usize; 2],
    pub independent_to_shared: [usize; 2],
    pub local_regressions: [usize; 2],
    pub peak_region_counts: [usize; 4],
    pub evidence_hash: u64,
    pub repeated: bool,
    pub direction: TrajectoryAttributionDirection,
}

pub(in crate::frequency_adaptive) fn review() -> TrajectoryAttributionReview {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-linked-stereo-trajectory-attribution");
    replace_directory(&root);
    let first = run(&root.join("first"));
    let second = run(&root.join("second"));
    let repeated = first == second;
    let failures = std::array::from_fn(|stage| {
        first
            .rows
            .iter()
            .filter(|row| !gate(row.control, row.metrics[stage]))
            .count()
    });
    let baseline_to_independent = comparisons(&first.rows, 0, 1);
    let independent_to_shared = comparisons(&first.rows, 1, 2);
    let local_regressions = std::array::from_fn(|stage| {
        first
            .rows
            .iter()
            .filter(|row| {
                row.maximum_local_residuals[stage + 1]
                    > row.maximum_local_residuals[stage] + 1.0e-12
                    || row.local_windows_improved[stage] < 4
            })
            .count()
    });
    let direction = if failures[1] > failures[2]
        && independent_to_shared[0] > independent_to_shared[1]
        && local_regressions[1] < local_regressions[0]
    {
        TrajectoryAttributionDirection::IndependentRecurrenceDominatesLoss
    } else if failures[2] > failures[1]
        && independent_to_shared[1] > independent_to_shared[0]
        && local_regressions[1] > local_regressions[0]
    {
        TrajectoryAttributionDirection::SharedRegionsDominateLoss
    } else {
        TrajectoryAttributionDirection::Mixed
    };
    write_report(
        &root,
        &first,
        repeated,
        failures,
        baseline_to_independent,
        independent_to_shared,
        local_regressions,
        direction,
    );
    TrajectoryAttributionReview {
        rows: first.rows,
        failures,
        baseline_to_independent,
        independent_to_shared,
        local_regressions,
        peak_region_counts: first.peak_region_counts,
        evidence_hash: first.evidence_hash,
        repeated,
        direction,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<TrajectoryAttributionRow>,
    peak_region_counts: [usize; 4],
    evidence_hash: u64,
}

fn run(root: &std::path::Path) -> Run {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let trim = geometry[0];
    let spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
    let mut rows = Vec::new();
    let mut evidence_hash = 0xcbf2_9ce4_8422_2325;
    let mut peak_region_counts = [0; 4];
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
                        let renders = [
                            render::linked([&input[0], &input[1]], ratio, SAMPLE_RATE),
                            render::linked_independent([&input[0], &input[1]], ratio, SAMPLE_RATE),
                            render::linked_peak_regions([&input[0], &input[1]], ratio, SAMPLE_RATE),
                        ];
                        let row = measure(
                            kind,
                            ratio,
                            source_frames,
                            phase,
                            bin_aligned,
                            frequency,
                            trim,
                            &input,
                            renders,
                        );
                        for value in row
                            .hashes
                            .into_iter()
                            .chain(row.peak_region_counts.map(|count| count as u64))
                        {
                            evidence_hash = (evidence_hash ^ value).wrapping_mul(0x100_0000_01b3);
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
    Run {
        rows,
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
    renders: [render::StereoRender; 3],
) -> TrajectoryAttributionRow {
    let source_trim = ((trim as f64 / ratio).ceil() as usize).min(source_frames / 3);
    let output_trim = trim.min(renders[0].channels[0].len() / 3);
    let metrics = std::array::from_fn(|stage| {
        let output = &renders[stage].channels;
        [
            evaluate(input, output, frequency, SAMPLE_RATE),
            evaluate(
                &metrics::crop(input, source_trim),
                &metrics::crop(output, output_trim),
                frequency,
                SAMPLE_RATE,
            ),
        ]
    });
    let independent_local = local_evidence(input, &renders[0].channels, &renders[1].channels);
    let shared_local = local_evidence(input, &renders[1].channels, &renders[2].channels);
    let target = (source_frames as f64 * ratio).round() as usize;
    TrajectoryAttributionRow {
        ratio,
        source_frames,
        phase,
        bin_aligned,
        control: kind.name(),
        metrics,
        structural_failures: [1, 2].map(|stage| {
            renders[stage].uncovered
                + renders[stage].non_finite
                + renders[stage].boundary_failures
                + usize::from(
                    renders[stage]
                        .channels
                        .iter()
                        .any(|channel| channel.len() != target),
                )
        }),
        local_windows_improved: [independent_local.0, shared_local.0],
        maximum_local_residuals: [
            independent_local.2[0],
            independent_local.2[1],
            shared_local.2[1],
        ],
        peak_region_counts: renders[2].peak_region_counts,
        hashes: renders.map(|render| render.hash),
    }
}

fn comparisons(rows: &[TrajectoryAttributionRow], before: usize, after: usize) -> [usize; 2] {
    let improved = rows
        .iter()
        .filter(|row| comparison(row, before, after).0 && comparison(row, before, after).1)
        .count();
    let regressed = rows
        .iter()
        .filter(|row| !comparison(row, before, after).0)
        .count();
    [improved, regressed]
}

fn comparison(row: &TrajectoryAttributionRow, before: usize, after: usize) -> (bool, bool) {
    let pairs = row.metrics[before].into_iter().zip(row.metrics[after]);
    let values = if row.control == "tone" {
        pairs
            .map(|(before, after)| (before.ipd_error_radians, after.ipd_error_radians))
            .collect::<Vec<_>>()
    } else {
        pairs
            .flat_map(|(before, after)| {
                [
                    (before.mid_side_delta_db, after.mid_side_delta_db),
                    (before.correlation_delta, after.correlation_delta),
                    (before.relation_residual, after.relation_residual),
                ]
            })
            .collect::<Vec<_>>()
    };
    (
        values
            .iter()
            .all(|(before, after)| *after <= *before + 1.0e-12),
        values.iter().any(|(before, after)| *after < *before),
    )
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

#[allow(clippy::too_many_arguments)]
fn write_report(
    root: &std::path::Path,
    run: &Run,
    repeated: bool,
    failures: [usize; 3],
    baseline_to_independent: [usize; 2],
    independent_to_shared: [usize; 2],
    local_regressions: [usize; 2],
    direction: TrajectoryAttributionDirection,
) {
    let mut report = format!(
        "repeated\t{repeated}\nfailures\t{},{},{}\nbaseline_to_independent\t{},{}\nindependent_to_shared\t{},{}\nlocal_regressions\t{},{}\npeak_region_counts\t{},{},{},{}\nevidence_hash\t{:016x}\ndirection\t{direction:?}\nratio\tframes\tphase\tbin_aligned\tcontrol\tscope\tbaseline_ipd\tindependent_ipd\tshared_ipd\tbaseline_mid_side\tindependent_mid_side\tshared_mid_side\tbaseline_correlation\tindependent_correlation\tshared_correlation\tbaseline_relation\tindependent_relation\tshared_relation\tindependent_structural\tshared_structural\tbaseline_to_independent_local\tindependent_to_shared_local\tbaseline_local\tindependent_local\tshared_local\tregions\teligible\tshared_bins\tindependent_bins\tbaseline_hash\tindependent_hash\tshared_hash\n",
        failures[0], failures[1], failures[2],
        baseline_to_independent[0], baseline_to_independent[1],
        independent_to_shared[0], independent_to_shared[1],
        local_regressions[0], local_regressions[1],
        run.peak_region_counts[0], run.peak_region_counts[1],
        run.peak_region_counts[2], run.peak_region_counts[3], run.evidence_hash,
    );
    for row in &run.rows {
        for scope in 0..2 {
            let metrics = row.metrics.map(|stage| stage[scope]);
            report.push_str(&format!(
                "{:.2}\t{}\t{:.2}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{}\t{}\t{:016x}\t{:016x}\t{:016x}\n",
                row.ratio, row.source_frames, row.phase, row.bin_aligned, row.control,
                ["whole", "interior"][scope], metrics[0].ipd_error_radians,
                metrics[1].ipd_error_radians, metrics[2].ipd_error_radians,
                metrics[0].mid_side_delta_db, metrics[1].mid_side_delta_db,
                metrics[2].mid_side_delta_db, metrics[0].correlation_delta,
                metrics[1].correlation_delta, metrics[2].correlation_delta,
                metrics[0].relation_residual, metrics[1].relation_residual,
                metrics[2].relation_residual, row.structural_failures[0],
                row.structural_failures[1], row.local_windows_improved[0],
                row.local_windows_improved[1], row.maximum_local_residuals[0],
                row.maximum_local_residuals[1], row.maximum_local_residuals[2],
                row.peak_region_counts[0], row.peak_region_counts[1],
                row.peak_region_counts[2], row.peak_region_counts[3], row.hashes[0],
                row.hashes[1], row.hashes[2],
            ));
        }
    }
    fs::write(root.join("trajectory-attribution.tsv"), report)
        .expect("write trajectory attribution report");
}
