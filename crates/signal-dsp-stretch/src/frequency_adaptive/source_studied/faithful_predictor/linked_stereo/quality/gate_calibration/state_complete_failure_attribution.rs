use std::{fs, path::PathBuf};

use super::{
    external::{read_stereo, replace_directory, write_stereo},
    metrics::{self, control, ControlKind},
    relation_repair::transform::local_evidence,
    ALIGNMENTS, CALIBRATED_TONE_IPD_RADIANS, LENGTHS, PHASES, RATIOS, SAMPLE_RATE,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    coherent_representation,
    render::{self, SynthesisTraceSpec},
    state_complete_linked_phase_vocoder::{self, TraceRender},
};

const MATERIAL_ERROR: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum FirstDivergence {
    Coefficient,
    FullInverseFrame,
    SupportCrop,
    OverlapWindow,
    Normalization,
    NoMaterialExcess,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct StageEvidence {
    pub(in crate::frequency_adaptive) coefficient_relation_error: f64,
    pub(in crate::frequency_adaptive) full_support_inverse_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) inverse_ipd_error: [f64; 2],
    pub(in crate::frequency_adaptive) accumulated_window_ipd_error: [f64; 8],
    pub(in crate::frequency_adaptive) normalized_window_ipd_error: [f64; 8],
    pub(in crate::frequency_adaptive) hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AttributionRow {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) source_frames: usize,
    pub(in crate::frequency_adaptive) phase: f64,
    pub(in crate::frequency_adaptive) bin_aligned: bool,
    pub(in crate::frequency_adaptive) calibrated_failure: bool,
    pub(in crate::frequency_adaptive) local_windows_improved: usize,
    pub(in crate::frequency_adaptive) local_residuals: [f64; 2],
    pub(in crate::frequency_adaptive) traces: [StageEvidence; 3],
    pub(in crate::frequency_adaptive) first_divergence: FirstDivergence,
    pub(in crate::frequency_adaptive) first_inverse_frame: Option<isize>,
    pub(in crate::frequency_adaptive) state_changed_first_divergence: Option<FirstDivergence>,
    pub(in crate::frequency_adaptive) state_changed_first_inverse_frame: Option<isize>,
    pub(in crate::frequency_adaptive) first_overlap_window: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AttributionReview {
    pub(in crate::frequency_adaptive) rows: Vec<AttributionRow>,
    pub(in crate::frequency_adaptive) calibrated_failures: usize,
    pub(in crate::frequency_adaptive) divergence_counts: [usize; 6],
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
}

pub(in crate::frequency_adaptive) fn review() -> AttributionReview {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-state-complete-failure-attribution");
    replace_directory(&root);
    let first = run(&root.join("first"));
    let second = run(&root.join("second"));
    let repeated = first == second;
    let calibrated_failures = first
        .rows
        .iter()
        .filter(|row| row.calibrated_failure)
        .count();
    let divergence_counts = first.rows.iter().fold([0; 6], |mut counts, row| {
        counts[divergence_index(row.first_divergence)] += 1;
        counts
    });
    let review = AttributionReview {
        rows: first.rows,
        calibrated_failures,
        divergence_counts,
        repeated,
        evidence_hash: first.evidence_hash,
    };
    write_report(&root, &review);
    review
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<AttributionRow>,
    evidence_hash: u64,
}

fn run(root: &std::path::Path) -> Run {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let trim = geometry[0];
    let spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
    let policies = state_complete_linked_phase_vocoder::candidates();
    let mut rows = Vec::new();
    let mut evidence_hash = 0xcbf2_9ce4_8422_2325;

    for source_frames in LENGTHS {
        for phase in PHASES {
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * spacing;
                let source = control(ControlKind::Tone, source_frames, frequency, phase);
                for ratio in RATIOS {
                    let stem = format!("tone-{source_frames}-{phase:.2}-{bin_aligned}-{ratio:.2}");
                    let path = root.join(format!("{stem}.wav"));
                    write_stereo(&path, &source, SAMPLE_RATE as u32);
                    let input = read_stereo(&path, source_frames, SAMPLE_RATE as u32);
                    let coherent = render::linked([&input[0], &input[1]], ratio, SAMPLE_RATE);
                    let candidate = state_complete_linked_phase_vocoder::render(
                        [&input[0], &input[1]],
                        ratio,
                        SAMPLE_RATE,
                        policies[0],
                    );
                    let local = local_evidence(&input, &coherent.channels, &candidate.channels);
                    if local.2[1] <= local.2[0] + 1.0e-12 && local.0 >= 4 {
                        continue;
                    }
                    let source_trim =
                        ((trim as f64 / ratio).ceil() as usize).min(source_frames / 3);
                    let expected_ipd = [
                        metrics::ipd(&input, frequency, SAMPLE_RATE),
                        metrics::ipd(&metrics::crop(&input, source_trim), frequency, SAMPLE_RATE),
                    ];
                    let spec = SynthesisTraceSpec {
                        frequency,
                        inverse_expected_ipd: expected_ipd[0],
                        output_expected_ipd: expected_ipd,
                        sample_rate: SAMPLE_RATE,
                        interior_trim: trim,
                    };
                    let coherent = render::linked_with_synthesis_trace(
                        [&input[0], &input[1]],
                        ratio,
                        SAMPLE_RATE,
                        frequency,
                        expected_ipd[0],
                        expected_ipd,
                        None,
                    );
                    let candidate = state_complete_linked_phase_vocoder::render_with_trace(
                        [&input[0], &input[1]],
                        ratio,
                        SAMPLE_RATE,
                        policies[0],
                        spec,
                    );
                    let changed = state_complete_linked_phase_vocoder::render_with_trace(
                        [&input[0], &input[1]],
                        ratio,
                        SAMPLE_RATE,
                        policies[17],
                        spec,
                    );
                    let coherent_synthesis = coherent
                        .synthesis_relation_trace
                        .as_ref()
                        .expect("coherent trace");
                    let first_inverse =
                        first_inverse_excess(&candidate.synthesis, coherent_synthesis);
                    let state_changed_first_inverse =
                        first_inverse_excess(&changed.synthesis, coherent_synthesis);
                    let traces = [
                        StageEvidence {
                            coefficient_relation_error: coherent.maximum_constrained_relation_error,
                            full_support_inverse_ipd_error: coherent_synthesis
                                .full_support_inverse_ipd_error,
                            inverse_ipd_error: coherent_synthesis.inverse_ipd_error,
                            accumulated_window_ipd_error: coherent_synthesis
                                .accumulated_window_ipd_error,
                            normalized_window_ipd_error: coherent_synthesis
                                .normalized_window_ipd_error,
                            hash: coherent.hash,
                        },
                        stage_evidence(&candidate),
                        stage_evidence(&changed),
                    ];
                    let (first_divergence, first_overlap_window) =
                        first_divergence(&traces, first_inverse);
                    let candidate_metrics = metrics::evaluate(
                        &input,
                        &candidate.render.channels,
                        frequency,
                        SAMPLE_RATE,
                    );
                    let row = AttributionRow {
                        ratio,
                        source_frames,
                        phase,
                        bin_aligned,
                        calibrated_failure: candidate_metrics.ipd_error_radians
                            > CALIBRATED_TONE_IPD_RADIANS,
                        local_windows_improved: local.0,
                        local_residuals: local.2,
                        traces,
                        first_divergence,
                        first_inverse_frame: first_inverse.map(|(_, center)| center),
                        state_changed_first_divergence: state_changed_first_inverse
                            .map(|(divergence, _)| divergence),
                        state_changed_first_inverse_frame: state_changed_first_inverse
                            .map(|(_, center)| center),
                        first_overlap_window,
                    };
                    hash_row(&mut evidence_hash, &row);
                    rows.push(row);
                }
            }
        }
    }
    Run {
        rows,
        evidence_hash,
    }
}

fn stage_evidence(render: &TraceRender) -> StageEvidence {
    StageEvidence {
        coefficient_relation_error: render.maximum_coefficient_relation_error,
        full_support_inverse_ipd_error: render.synthesis.full_support_inverse_ipd_error,
        inverse_ipd_error: render.synthesis.inverse_ipd_error,
        accumulated_window_ipd_error: render.synthesis.accumulated_window_ipd_error,
        normalized_window_ipd_error: render.synthesis.normalized_window_ipd_error,
        hash: render.render.hash,
    }
}

fn first_divergence(
    traces: &[StageEvidence; 3],
    first_inverse: Option<(FirstDivergence, isize)>,
) -> (FirstDivergence, Option<usize>) {
    let control = traces[0];
    let candidate = traces[1];
    if candidate.coefficient_relation_error > control.coefficient_relation_error + MATERIAL_ERROR {
        return (FirstDivergence::Coefficient, None);
    }
    if let Some((divergence, _)) = first_inverse {
        return (divergence, None);
    }
    if let Some(window) = first_excess(
        &candidate.accumulated_window_ipd_error,
        &control.accumulated_window_ipd_error,
    ) {
        return (FirstDivergence::OverlapWindow, Some(window));
    }
    if let Some(window) = first_excess(
        &candidate.normalized_window_ipd_error,
        &candidate.accumulated_window_ipd_error,
    ) {
        return (FirstDivergence::Normalization, Some(window));
    }
    (FirstDivergence::NoMaterialExcess, None)
}

fn first_inverse_excess(
    candidate: &render::SynthesisRelationTrace,
    control: &render::SynthesisRelationTrace,
) -> Option<(FirstDivergence, isize)> {
    candidate
        .inverse_frames
        .iter()
        .zip(&control.inverse_frames)
        .find_map(|(candidate, control)| {
            if candidate.output_center != control.output_center {
                return None;
            }
            if candidate.full_support_ipd_error > control.full_support_ipd_error + MATERIAL_ERROR {
                return Some((FirstDivergence::FullInverseFrame, candidate.output_center));
            }
            if candidate.cropped_support_ipd_error
                > control.cropped_support_ipd_error + MATERIAL_ERROR
            {
                return Some((FirstDivergence::SupportCrop, candidate.output_center));
            }
            None
        })
}

fn first_excess(actual: &[f64; 8], control: &[f64; 8]) -> Option<usize> {
    actual
        .iter()
        .zip(control)
        .position(|(actual, control)| *actual > *control + MATERIAL_ERROR)
}

fn divergence_index(divergence: FirstDivergence) -> usize {
    match divergence {
        FirstDivergence::Coefficient => 0,
        FirstDivergence::FullInverseFrame => 1,
        FirstDivergence::SupportCrop => 2,
        FirstDivergence::OverlapWindow => 3,
        FirstDivergence::Normalization => 4,
        FirstDivergence::NoMaterialExcess => 5,
    }
}

fn hash_row(hash: &mut u64, row: &AttributionRow) {
    for value in [
        row.ratio.to_bits(),
        row.source_frames as u64,
        row.phase.to_bits(),
        row.bin_aligned as u64,
        row.calibrated_failure as u64,
        row.local_windows_improved as u64,
        row.local_residuals[0].to_bits(),
        row.local_residuals[1].to_bits(),
        divergence_index(row.first_divergence) as u64,
        row.first_inverse_frame.unwrap_or(isize::MIN) as u64,
        row.state_changed_first_divergence
            .map_or(u64::MAX, |divergence| divergence_index(divergence) as u64),
        row.state_changed_first_inverse_frame.unwrap_or(isize::MIN) as u64,
        row.first_overlap_window.unwrap_or(usize::MAX) as u64,
    ]
    .into_iter()
    .chain(row.traces.iter().flat_map(|trace| {
        [
            trace.coefficient_relation_error.to_bits(),
            trace.full_support_inverse_ipd_error[0].to_bits(),
            trace.full_support_inverse_ipd_error[1].to_bits(),
            trace.inverse_ipd_error[0].to_bits(),
            trace.inverse_ipd_error[1].to_bits(),
            trace.hash,
        ]
        .into_iter()
        .chain(trace.accumulated_window_ipd_error.map(f64::to_bits))
        .chain(trace.normalized_window_ipd_error.map(f64::to_bits))
    })) {
        *hash = (*hash ^ value).wrapping_mul(0x100_0000_01b3);
    }
}

fn write_report(root: &std::path::Path, review: &AttributionReview) {
    let mut report = format!(
        "repeated\t{}\nrows\t{}\ncalibrated_failures\t{}\ndivergence_counts\t{},{},{},{},{},{}\nevidence_hash\t{:016x}\nratio\tframes\tphase\tbin_aligned\tcalibrated_failure\tlocal_improved\tlocal_before\tlocal_after\tfirst_divergence\tfirst_inverse_frame\tstate_changed_first_divergence\tstate_changed_first_inverse_frame\tfirst_overlap_window\tengine\tcoefficient_error\tfull_inverse_whole\tfull_inverse_interior\tcropped_inverse_whole\tcropped_inverse_interior\taccumulated_windows\tnormalized_windows\thash\n",
        review.repeated,
        review.rows.len(),
        review.calibrated_failures,
        review.divergence_counts[0],
        review.divergence_counts[1],
        review.divergence_counts[2],
        review.divergence_counts[3],
        review.divergence_counts[4],
        review.divergence_counts[5],
        review.evidence_hash,
    );
    for row in &review.rows {
        for (engine, trace) in ["coherent", "candidate-0", "candidate-17"]
            .into_iter()
            .zip(row.traces)
        {
            report.push_str(&format!(
                "{:.2}\t{}\t{:.2}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{:016x}\n",
                row.ratio,
                row.source_frames,
                row.phase,
                row.bin_aligned,
                row.calibrated_failure,
                row.local_windows_improved,
                row.local_residuals[0],
                row.local_residuals[1],
                row.first_divergence,
                row.first_inverse_frame.map_or_else(|| "none".to_owned(), |center| center.to_string()),
                row.state_changed_first_divergence.map_or_else(|| "none".to_owned(), |divergence| format!("{divergence:?}")),
                row.state_changed_first_inverse_frame.map_or_else(|| "none".to_owned(), |center| center.to_string()),
                row.first_overlap_window.map_or_else(|| "none".to_owned(), |window| window.to_string()),
                engine,
                trace.coefficient_relation_error,
                trace.full_support_inverse_ipd_error[0],
                trace.full_support_inverse_ipd_error[1],
                trace.inverse_ipd_error[0],
                trace.inverse_ipd_error[1],
                format_values(trace.accumulated_window_ipd_error),
                format_values(trace.normalized_window_ipd_error),
                trace.hash,
            ));
        }
    }
    fs::write(root.join("attribution.tsv"), report)
        .expect("write state-complete failure attribution");
}

fn format_values(values: [f64; 8]) -> String {
    values
        .into_iter()
        .map(|value| format!("{value:.12e}"))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires release-only source-studied renderer"]
    fn state_complete_failure_attribution_names_first_operation() {
        let result = review();
        assert!(result.repeated);
        assert_eq!(result.rows.len(), 11);
        assert_eq!(result.calibrated_failures, 1);
        assert_eq!(result.divergence_counts, [0, 4, 7, 0, 0, 0]);
        assert_eq!(result.evidence_hash, 0xfc10_cd64_42d5_5e4a);
        assert!(result.rows.iter().all(|row| {
            row.state_changed_first_divergence == Some(row.first_divergence)
                && row
                    .traces
                    .iter()
                    .all(|trace| trace.coefficient_relation_error <= 2.0e-15)
        }));
        assert!(result
            .rows
            .iter()
            .filter(|row| row.calibrated_failure)
            .all(|row| row.first_divergence == FirstDivergence::FullInverseFrame));
    }
}
