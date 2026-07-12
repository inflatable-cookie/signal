use super::*;

const SAMPLE_RATE: SampleRate = SampleRate(48_000);
const CONTROL_LEN: usize = 4_096;

#[test]
fn frequency_adaptive_reconstruction_controls_pass() {
    let mut controls = vec![
        sine(55.0),
        sine(440.0),
        sine(4_000.0),
        sine(19_500.0),
        sine(23_500.0),
        deterministic_noise(),
        mixed_control(),
        vec![0.0; CONTROL_LEN],
    ];
    let mut impulse = vec![0.0; CONTROL_LEN];
    impulse[192] = 1.0;
    controls.push(impulse);
    for input in controls {
        assert_reconstruction_gate(&input);
    }
}

#[test]
fn frequency_adaptive_reconstruction_is_deterministic() {
    let input = mixed_control();
    let first = frequency_adaptive_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = frequency_adaptive_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
    eprintln!(
        "frequency_adaptive fft={} bands={} coefficients={} frame=[{:.9},{:.9}] condition={:.9} overlap={} peak={:.9e} rms={:.9e} filters={:016x} coefficients_hash={:016x} reconstruction={:016x}",
        first.evidence.fft_frames,
        first.evidence.band_count,
        first.evidence.coefficient_count,
        first.evidence.frame_operator_min,
        first.evidence.frame_operator_max,
        first.evidence.frame_condition_ratio,
        first.evidence.multiply_covered_frequency_bins,
        first.evidence.reconstruction_peak_error,
        first.evidence.reconstruction_rms_error,
        first.evidence.filter_hash,
        first.evidence.coefficient_hash,
        first.evidence.reconstruction_hash,
    );
}

#[test]
fn frequency_adaptive_reconstruction_empty_input_is_exact() {
    let review = frequency_adaptive_reconstruction_review_mono(&[], SAMPLE_RATE);
    assert!(review.samples.is_empty());
    assert_eq!(review.evidence.source_frames, 0);
    assert_eq!(review.evidence.output_frames, 0);
    assert_eq!(review.evidence.reconstruction_peak_error, 0.0);
}

#[test]
fn common_grid_wavelet_reconstruction_meets_frame_and_dual_gate() {
    let input = mixed_control();
    let review = common_grid_wavelet_reconstruction_review_mono(&input, SAMPLE_RATE);
    let evidence = &review.evidence;
    assert_eq!(review.samples.len(), input.len());
    assert_eq!(evidence.channel_count, 1_536);
    assert_eq!(evidence.lowpass_channel_count, 16);
    assert_eq!(evidence.hop_frames, 384);
    assert_eq!(evidence.redundancy, 8.0);
    assert!(evidence.frame_condition_ratio <= 1.25, "{evidence:?}");
    assert!(evidence.canonical_dual_residual <= 1.0e-8, "{evidence:?}");
    assert!(evidence.reconstruction_peak_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_rms_error <= 1.0e-6, "{evidence:?}");
    assert_eq!(evidence.non_finite_values, 0);
    eprintln!("common_grid_wavelet {evidence:?}");
}

#[test]
fn common_grid_wavelet_reconstruction_is_deterministic() {
    let input = sine(440.0)[..384].to_vec();
    let first = common_grid_wavelet_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = common_grid_wavelet_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
}

#[test]
fn common_grid_wavelet_reconstruction_controls_pass() {
    let mut controls = vec![
        sine(55.0),
        sine(440.0),
        sine(4_000.0),
        sine(19_500.0),
        sine(23_500.0),
        deterministic_noise(),
        mixed_control(),
        vec![0.0; CONTROL_LEN],
    ];
    let mut impulse = vec![0.0; CONTROL_LEN];
    impulse[192] = 1.0;
    controls.push(impulse);
    controls.push(Vec::new());
    for input in controls {
        let short = &input[..input.len().min(384)];
        let review = common_grid_wavelet_reconstruction_review_mono(short, SAMPLE_RATE);
        assert_eq!(review.samples.len(), short.len());
        assert!(review.evidence.frame_condition_ratio <= 1.25);
        assert!(review.evidence.canonical_dual_residual <= 1.0e-8);
        assert!(review.evidence.reconstruction_peak_error <= 1.0e-5);
        assert!(review.evidence.reconstruction_rms_error <= 1.0e-6);
        assert!(review.evidence.reconstruction_head_error <= 1.0e-5);
        assert!(review.evidence.reconstruction_tail_error <= 1.0e-5);
        assert_eq!(review.evidence.non_finite_values, 0);
    }
}

#[test]
fn common_grid_boundary_candidate_rejects_frame_conditioning() {
    let input = mixed_control();
    let first = common_grid_boundary_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = common_grid_boundary_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
    let evidence = &first.reconstruction.evidence;
    assert_eq!(first.reconstruction.samples.len(), input.len());
    assert!(evidence.frame_condition_ratio > 1.25, "{evidence:?}");
    assert!(evidence.canonical_dual_residual <= 1.0e-8, "{evidence:?}");
    assert!(evidence.reconstruction_peak_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_rms_error <= 1.0e-6, "{evidence:?}");
    assert!(evidence.reconstruction_head_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_tail_error <= 1.0e-5, "{evidence:?}");
    assert_eq!(evidence.non_finite_values, 0);
    assert_ne!(first.preserved_filter_hash, 0);
    assert_ne!(first.nyquist_completion_hash, 0);
    assert_ne!(first.raw_filter_hash, 0);
}

#[test]
fn common_grid_preconditioned_candidate_rejects_frame_conditioning() {
    let input = mixed_control();
    let raw = common_grid_boundary_reconstruction_review_mono(&input, SAMPLE_RATE);
    let first = common_grid_preconditioned_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = common_grid_preconditioned_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
    let evidence = &first.reconstruction.evidence;
    assert_eq!(first.reconstruction.samples.len(), input.len());
    assert!(evidence.frame_condition_ratio > 1.25, "{evidence:?}");
    assert!(evidence.canonical_dual_residual <= 1.0e-8, "{evidence:?}");
    assert!(evidence.reconstruction_peak_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_rms_error <= 1.0e-6, "{evidence:?}");
    assert!(evidence.reconstruction_head_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_tail_error <= 1.0e-5, "{evidence:?}");
    assert_eq!(evidence.non_finite_values, 0);
    assert_eq!(first.raw_filter_hash, raw.raw_filter_hash);
    assert_ne!(first.multiplier_hash, 0);
}

#[test]
#[cfg(not(debug_assertions))]
fn common_grid_conditioning_attribution_selects_boundary_geometry() {
    let first = common_grid_conditioning_attribution_review();
    let repeated = common_grid_conditioning_attribution_review();
    assert_eq!(first, repeated);
    assert_eq!(first.residues.len(), 33);
    assert_eq!(first.modes.len(), 6);
    assert!(
        first.maximum_errors[0] <= 1.0e-6,
        "{:?}",
        first.maximum_errors
    );
    assert!(first.maximum_errors[1] <= 1.0e-8, "{first:?}");
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    assert_eq!(
        first.direction,
        StretchCommonGridConditioningDirection::BoundaryGeometry
    );
    assert!(first.modes.iter().all(|mode| mode.top_bins.len() == 16));
    assert!(first
        .modes
        .iter()
        .all(|mode| mode.top_total_channels.len() == 16));
    assert!(first
        .modes
        .iter()
        .all(|mode| mode.top_cross_channels.len() == 16));
    let exact = &first.residues[11..22];
    let condition = exact
        .iter()
        .map(|row| row.eigenvalues[1])
        .fold(0.0, f64::max)
        / exact
            .iter()
            .map(|row| row.eigenvalues[0])
            .fold(f64::INFINITY, f64::min);
    assert!(condition > 1.25);
    assert!(first.modes[4].region_mass[0] + first.modes[4].region_mass[2] >= 0.9);
    assert!(first.modes[5].region_mass[0] + first.modes[5].region_mass[2] >= 0.9);
}

#[test]
#[cfg(not(debug_assertions))]
fn common_grid_hermitian_jacobi_proof_passes() {
    let first = common_grid_hermitian_jacobi_review();
    let repeated = common_grid_hermitian_jacobi_review();
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 6);
    assert_eq!(first.alias_blocks.len(), 33);
    assert!(first.passed, "{first:?}");
    assert!(first.evidence_hash != 0);
}

#[test]
#[cfg(not(debug_assertions))]
fn common_grid_nyquist_alias_coupling_ablation_selects_geometry() {
    let first = common_grid_nyquist_alias_coupling_review();
    let repeated = common_grid_nyquist_alias_coupling_review();
    assert_eq!(first, repeated);
    assert_eq!(first.residues.len(), 33);
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    assert!(first.maximum_errors[0] <= 1.0e-8, "{first:?}");
    assert!(first.maximum_errors[1] <= 1.0e-10, "{first:?}");
    assert!(first.maximum_errors[2] <= 1.0e-12, "{first:?}");
    assert!(first.maximum_errors[3] <= 1.0e-10, "{first:?}");
    assert!(first.maximum_errors[4] <= 1.0e-8, "{first:?}");
    assert_eq!(
        first.globals.each_ref().map(|row| row.operator),
        [
            StretchCommonGridNyquistAblationOperator::Full,
            StretchCommonGridNyquistAblationOperator::CompletionRemoved,
            StretchCommonGridNyquistAblationOperator::CompletionDiagonalized,
        ]
    );
    assert!(
        first.globals[0].condition_ratio > 1.25,
        "{:?}",
        first.globals
    );
    assert!(first.globals[1].condition_ratio > 1.25, "{first:?}");
    assert!(first.globals[2].condition_ratio <= 1.25, "{first:?}");
    assert_eq!(
        first.direction,
        StretchCommonGridNyquistAblationDirection::OrthogonalOrMultiRowCompletion
    );
    assert_eq!(first.modes[0].residue, 0);
    assert_eq!(first.modes[1].residue, 0);
    eprintln!(
        "common_grid_nyquist_alias_coupling globals={:?} modes={:?} errors={:?} hashes={:x?} direction={:?}",
        first.globals, first.modes, first.maximum_errors, first.hashes, first.direction
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn common_grid_three_row_nyquist_completion_rejects_conditioning() {
    let first = common_grid_three_row_nyquist_review();
    let repeated = common_grid_three_row_nyquist_review();
    assert_eq!(first, repeated);
    assert_eq!(first.row_count, 1_538);
    assert_eq!(first.hop_frames, 384);
    assert_eq!(first.completion_delays, [-128, 0, 128]);
    assert_ne!(first.preserved_hash, 0);
    assert!(first.completion_hashes.iter().all(|hash| *hash != 0));
    assert!(
        first
            .construction_errors
            .iter()
            .all(|error| *error <= 1.0e-12),
        "{first:?}"
    );
    assert_eq!(first.residues.len(), 11);
    assert!(first.maximum_proof_errors[0] <= 1.0e-8, "{first:?}");
    assert!(first.maximum_proof_errors[1] <= 1.0e-10, "{first:?}");
    assert!(first.maximum_proof_errors[2] <= 1.0e-12, "{first:?}");
    assert!(first.maximum_proof_errors[3] <= 1.0e-10, "{first:?}");
    assert!(first.condition_ratio > 1.25, "{first:?}");
    assert_eq!(
        first.direction,
        StretchCommonGridThreeRowNyquistDirection::BoundaryGeometry
    );
    assert_ne!(first.evidence_hash, 0);
    eprintln!(
        "common_grid_three_row_nyquist eigenvalues={:?} residues={:?} condition={:.12} construction={:?} proof={:?} preserved={:016x} completions={:x?} evidence={:016x} direction={:?}",
        first.eigenvalues,
        first.limiting_residues,
        first.condition_ratio,
        first.construction_errors,
        first.maximum_proof_errors,
        first.preserved_hash,
        first.completion_hashes,
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn common_grid_residual_boundary_attribution_selects_direction() {
    let first = common_grid_residual_boundary_attribution_review();
    let repeated = common_grid_residual_boundary_attribution_review();
    assert_eq!(first, repeated);
    assert_eq!(first.residues.len(), 44);
    assert!(first.maximum_errors[0] <= 1.0e-8, "{first:?}");
    assert!(first.maximum_errors[1] <= 1.0e-10, "{first:?}");
    assert!(first.maximum_errors[2] <= 1.0e-12, "{first:?}");
    assert!(first.maximum_errors[3] <= 1.0e-10, "{first:?}");
    assert!(first.maximum_errors[4] <= 1.0e-8, "{first:?}");
    assert_eq!(first.modes[0].residue, 3);
    assert_eq!(first.modes[1].residue, 8);
    assert!(first.modes.iter().all(|mode| mode.top_bins.len() == 16));
    assert!(first
        .modes
        .iter()
        .all(|mode| mode.top_total_channels.len() == 16));
    assert!(first
        .modes
        .iter()
        .all(|mode| mode.top_cross_channels.len() == 16));
    assert_eq!(
        first.direction,
        StretchCommonGridResidualBoundaryDirection::CompleteRawBank
    );
    assert_ne!(first.evidence_hash, 0);
    eprintln!(
        "common_grid_residual_boundary conditions={:?} modes={:?} errors={:?} evidence={:016x} direction={:?}",
        first.conditions, first.modes, first.maximum_errors, first.evidence_hash, first.direction
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn common_grid_canonical_tightener_selects_localization_direction() {
    let first = common_grid_canonical_tightener_review();
    let repeated = common_grid_canonical_tightener_review();
    assert_eq!(first, repeated);
    assert!(first.frame_values[2] <= 1.0 + 1.0e-10, "{first:?}");
    assert!(first.maximum_proof_errors[0] <= 1.0e-8, "{first:?}");
    assert!(first.maximum_proof_errors[1] <= 1.0e-10, "{first:?}");
    assert!(first.maximum_proof_errors[2] <= 1.0e-12, "{first:?}");
    assert!(first.maximum_proof_errors[3] <= 1.0e-10, "{first:?}");
    assert!(first.maximum_proof_errors[4] <= 1.0e-10, "{first:?}");
    assert!(first.evaluated_rows > 0);
    assert_eq!(
        first.direction,
        StretchCommonGridCanonicalTightenerDirection::TransformFamilyReassessment
    );
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    eprintln!("common_grid_canonical_tightener {first:?}");
}

#[test]
#[cfg(not(debug_assertions))]
fn dense_painless_common_lattice_selects_feasibility_direction() {
    let first = dense_painless_common_lattice_review();
    let repeated = dense_painless_common_lattice_review();
    assert_eq!(first, repeated);
    assert_eq!(first.geometry[0], 65_536);
    assert_eq!(first.hashes[0], first.hashes[1], "{first:?}");
    assert_eq!(first.hashes[2], first.hashes[3], "{first:?}");
    assert_eq!(first.hashes[4], first.hashes[5], "{first:?}");
    assert_eq!(first.structural_failures, [0; 3], "{first:?}");
    assert!(first.frame_values[2] <= 1.0 + 1.0e-6, "{first:?}");
    assert!(first.reconstruction_errors[0] > 1.0e-12);
    assert!(first.reconstruction_errors[1] <= 1.0e-5, "{first:?}");
    assert!(first.reconstruction_errors[2] <= 1.0e-6, "{first:?}");
    assert!(first.reconstruction_errors[3] <= 1.0e-5, "{first:?}");
    assert!(first.reconstruction_errors[4] <= 1.0e-5, "{first:?}");
    assert_eq!(first.required_radii, [usize::MAX; 2]);
    assert_eq!(
        first.direction,
        StretchDensePainlessDirection::OperatorReview
    );
    assert_ne!(first.hashes[6], 0);
    eprintln!(
        "dense_painless_common_lattice geometry={:?} coefficients={:?} cost={:?} frame={:?} structural={:?} reconstruction={:?} cap={:?} required={:?} limiting={:?} hashes={:016x?} direction={:?}",
        first.geometry,
        first.coefficient_counts,
        first.coefficient_cost,
        first.frame_values,
        first.structural_failures,
        first.reconstruction_errors,
        first.localization_curves.last().copied().unwrap_or([f64::INFINITY; 2]),
        first.required_radii,
        first.limiting_bands,
        first.hashes,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn time_adaptive_painless_reconstruction_selects_identity_direction() {
    let first = time_adaptive_painless_reconstruction_review();
    let repeated = time_adaptive_painless_reconstruction_review();
    assert_eq!(first, repeated);
    assert_eq!(first.schedules.len(), 5);
    assert!(first.empty_input_exact);
    assert!(first.schedules.iter().all(|schedule| {
        schedule.frame_values[2] <= 4.0
            && schedule.structural_failures == [0; 4]
            && schedule.maximum_errors[0] <= 1.0e-12
            && schedule.maximum_errors[1] <= 1.0e-12
            && schedule.maximum_errors[2] <= 1.0e-5
            && schedule.maximum_errors[3] <= 1.0e-6
            && schedule.maximum_errors[4] <= 1.0e-5
            && schedule.maximum_errors[5] <= 1.0e-5
            && schedule.non_finite_values == 0
            && schedule.hashes.iter().all(|hash| *hash != 0)
    }));
    assert_eq!(
        first.direction,
        StretchTimeAdaptivePainlessDirection::AutomaticSelectionContract
    );
    assert_ne!(first.evidence_hash, 0);
    eprintln!(
        "time_adaptive_painless schedules={:?} evidence={:016x} direction={:?}",
        first
            .schedules
            .iter()
            .map(|schedule| (
                schedule.family_and_frames,
                schedule.window_counts,
                schedule.hop_extrema,
                schedule.frame_values,
                schedule.maximum_errors,
                schedule.hashes,
            ))
            .collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn simultaneous_multi_window_union_selects_study_direction() {
    use super::simultaneous_multi_window::{review, Direction};

    let first = review();
    let repeated = review();
    assert_eq!(first, repeated);
    assert_eq!(first.layer_lengths, [512, 2_048, 8_192]);
    assert!(first.empty_input_exact);
    assert_eq!(first.structural_failures, [0; 3]);
    assert!(first.frame_bounds[0] > 0.0, "{first:?}");
    assert!(first.frame_bounds[2] <= 1.000_001, "{first:?}");
    assert!(first.maximum_errors[0] <= 2.0e-12, "{first:?}");
    assert!(first.maximum_errors[1] <= 2.0e-12, "{first:?}");
    assert!(first.maximum_errors[2] <= 2.0e-10, "{first:?}");
    assert!(first.maximum_errors[3] <= 2.0e-10, "{first:?}");
    assert_eq!(first.non_finite_values, 0);
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    assert_eq!(first.direction, Direction::StudyAndScheduleProof);
    eprintln!("simultaneous_multi_window {first:?}");
}

#[test]
#[cfg(not(debug_assertions))]
fn linked_study_local_schedule_selects_phase_proof_direction() {
    use super::study_local_schedule::{review, Direction};

    let first = review();
    let repeated = review();
    eprintln!("study_local_schedule {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 3);
    assert!(first.controls.iter().all(|control| {
        control.evidence_parity
            && control.linked_decision_equivalence
            && control.selected_points[0] >= 2
            && control.dense_points_retained >= 2
            && control.local_unity_improvement > 0.0
            && control.schedule_failures == [0; 5]
            && control.hashes.iter().all(|hash| *hash != 0)
    }));
    assert!(first
        .controls
        .iter()
        .any(|control| control.selected_points[1] >= 2));
    assert_eq!(first.direction, Direction::SyntheticPhaseAndSynthesisProof);
    assert_ne!(first.evidence_hash, 0);
}

#[test]
#[cfg(not(debug_assertions))]
fn complete_phase_synthesis_selects_bounded_tuning_direction() {
    use super::complete_phase_synthesis::{review, Direction};

    let first = review();
    let repeated = review();
    eprintln!("complete_phase_synthesis {first:?}");
    assert_eq!(first, repeated);
    assert!(first.identity_peak_error <= 5.0e-12, "{first:?}");
    assert_eq!(first.structural_failures, [0; 7], "{first:?}");
    assert!(first.event_phase_changes > 0, "{first:?}");
    assert!(first.vertical_phase_changes > 0, "{first:?}");
    assert!(first.tone_frequency_error_hz <= 2.0, "{first:?}");
    assert!(first.maximum_event_error <= 256, "{first:?}");
    assert!(first.maximum_symmetry_error <= 2.0e-10, "{first:?}");
    assert!(first.maximum_imaginary_residue <= 2.0e-10, "{first:?}");
    assert_eq!(first.non_finite_values, 0);
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    assert_eq!(first.direction, Direction::BoundedCompleteSystemTuning);
}

#[test]
#[cfg(not(debug_assertions))]
fn complete_system_tuning_grid_is_bounded_and_frozen() {
    use super::complete_system_tuning::{review, Direction};

    let first = review();
    let repeated = review();
    assert_eq!(first, repeated);
    assert_eq!(first.configuration_count, 108);
    assert_eq!(first.unique_configuration_count, 108);
    assert_eq!(first.dimension_counts, [3, 2, 3, 3, 2]);
    assert_eq!(first.development_rows.len(), 9);
    assert_eq!(first.holdout_rows.len(), 6);
    assert_eq!(first.family_counts, [[2, 2, 2, 1, 2], [1, 1, 1, 2, 1]]);
    assert!(first
        .development_rows
        .iter()
        .all(|row| !first.holdout_rows.contains(row)));
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    assert_eq!(first.direction, Direction::ExecuteObjectiveGrid);
}

#[test]
#[cfg(not(debug_assertions))]
fn complete_system_tuning_dimensions_reach_renderer() {
    let first = super::complete_system_tuning::reachability_review();
    let repeated = super::complete_system_tuning::reachability_review();
    assert_eq!(first, repeated);
    assert_eq!(first.dimension_changes, [2, 1, 2, 2, 1]);
    assert_eq!(first.structural_failures, [0; 6], "{first:?}");
    assert!(
        first.event_resets_by_scope.iter().all(|count| *count > 0),
        "{first:?}"
    );
    assert!(first.hashes.iter().all(|hash| *hash != 0));
}

#[test]
#[cfg(not(debug_assertions))]
fn complete_system_objective_grid_exports_pareto_candidates() {
    use super::complete_system_tuning::objective_grid_review;

    let review = objective_grid_review();
    eprintln!("complete_system_objective_grid {review:?}");
    assert_eq!(review.configuration_count, 108);
    assert_eq!(review.development_render_count, 972);
    assert!(review.passing_count > 0, "{review:?}");
    assert!(review.frontier_count > 0, "{review:?}");
    assert!((1..=3).contains(&review.candidates.len()), "{review:?}");
    assert_eq!(review.holdout_reads, 0);
    assert!(review.hashes.iter().all(|hash| *hash != 0));
}

#[test]
#[cfg(not(debug_assertions))]
fn renyi_time_resolution_selection_selects_schedule_direction() {
    let first = renyi_time_resolution_selection_review();
    let repeated = renyi_time_resolution_selection_review();
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 12);
    assert!(first.controls.iter().all(|control| {
        control.structural_counts[1] == 0
            && control.channel_energy_closure <= 1.0e-12
            && control
                .selected_levels
                .windows(2)
                .all(|pair| pair[0].abs_diff(pair[1]) <= 1)
            && control.hashes.iter().all(|hash| *hash != 0)
    }));
    assert_eq!(first.gate_failures, [0, 1, 0, 0, 2, 0, 0]);
    assert_eq!(
        first.direction,
        StretchRenyiSelectorDirection::SelectorResearch
    );
    assert_ne!(first.evidence_hash, 0);
    eprintln!(
        "renyi_time_resolution gate_failures={:?} perturbation={} paths={:?} evidence={:016x} direction={:?}",
        first.gate_failures,
        first.maximum_perturbation_change,
        first
            .controls
            .iter()
            .map(|control| (control.control, control.level_counts, control.path_shape, control.path_cost))
            .collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn renyi_selector_failure_attribution_stops_inconclusive() {
    let first = renyi_selector_failure_attribution_review();
    let repeated = renyi_selector_failure_attribution_review();
    assert_eq!(first, repeated);
    assert_eq!(first.baseline.gate_failures, [0, 1, 0, 0, 2, 0, 0]);
    assert_eq!(first.baseline.evidence_hash, 0x5568_f0a3_8f67_9a40);
    assert_eq!(first.controls.len(), 12);
    eprintln!(
        "renyi_attribution controls={:?}",
        first
            .controls
            .iter()
            .map(|control| (
                control.control,
                control.closure_errors,
                control.structural_failures,
                control.baseline_drift,
            ))
            .collect::<Vec<_>>()
    );
    assert!(first.controls.iter().all(|control| {
        control.anchors.len() == 64
            && control.closure_errors[0] == 0.0
            && control.closure_errors[1] <= 1.0e-12
            && control.closure_errors[2] == 0.0
            && control.closure_errors[3] <= 1.0e-12
            && control.structural_failures == [0, 0]
            && control.baseline_drift == 0
            && control.evidence_hash != 0
    }));
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(first.diagnostic_counts, [15, 5, 32]);
    assert_eq!(first.candidate_counts, [0, 0]);
    assert_eq!(first.geometry_effects, [8, 5]);
    assert_eq!(first.frequency_event_restorations, [5, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(first.frequency_negative_changes, [1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(first.linear_chirp_changes[0], [0; 8]);
    assert_eq!(first.linear_chirp_changes[1], [39, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        first.direction,
        StretchRenyiAttributionDirection::Inconclusive
    );
    eprintln!(
        "renyi_attribution diagnostics={:?} candidates={:?} geometry={:?} frequency_event={:?} frequency_negative={:?} chirp={:?} evidence={:016x} direction={:?}",
        first.diagnostic_counts,
        first.candidate_counts,
        first.geometry_effects,
        first.frequency_event_restorations,
        first.frequency_negative_changes,
        first.linear_chirp_changes,
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn renyi_attribution_reassessment_selects_terminal_direction() {
    let first = renyi_attribution_reassessment_review();
    let repeated = renyi_attribution_reassessment_review();
    assert_eq!(first, repeated);
    assert_eq!(first.prior.baseline.evidence_hash, 0x5568_f0a3_8f67_9a40);
    assert_eq!(first.prior.evidence_hash, 0xe0b4_4210_3849_2480);
    assert_eq!(first.controls.len(), 3);
    assert!(first.controls.iter().all(|control| {
        control.anchors.len() == 64
            && control.closure_errors[0] == 0.0
            && control.closure_errors[1] <= 1.0e-12
            && control.closure_errors[2] == 0.0
            && control.closure_errors[3] <= 1.0e-12
            && control.structural_failures == [0; 3]
            && control.evidence_hash != 0
    }));
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(first.support_effects, [15, 0]);
    assert_eq!(first.low_event_restorations, [5, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(first.low_negative_changes, [32, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(first.linear_chirp_changes, [0; 8]);
    assert_eq!(first.candidate_counts, [1, 0]);
    assert_eq!(first.evidence_hash, 0x009a_37d3_55b9_d6fe);
    assert_eq!(
        first.direction,
        StretchRenyiReassessmentDirection::ComparisonRegionContract
    );
    eprintln!(
        "renyi_reassessment closure={:?} support={:?} low_event={:?} low_negative={:?} chirp={:?} candidates={:?} evidence={:016x} direction={:?}",
        first.controls.iter().map(|control| control.closure_errors).collect::<Vec<_>>(),
        first.support_effects,
        first.low_event_restorations,
        first.low_negative_changes,
        first.linear_chirp_changes,
        first.candidate_counts,
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn renyi_anchor_local_geometry_selects_terminal_direction() {
    let first = renyi_anchor_local_geometry_review();
    let repeated = renyi_anchor_local_geometry_review();
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 12);
    assert_eq!(
        first.support_extrema,
        [[-1792, 1792], [-1536, 1536], [-1024, 1024], [0, 0]]
    );
    assert_eq!(first.geometry_failures, [0, 0]);
    assert!(first.controls.iter().all(|control| {
        control.structural_counts[1] == 0
            && control.channel_energy_closure <= 1.0e-12
            && control
                .selected_levels
                .windows(2)
                .all(|pair| pair[0].abs_diff(pair[1]) <= 1)
            && control.hashes.iter().all(|hash| *hash != 0)
    }));
    assert_ne!(first.membership_hash, 0);
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(first.gate_failures, [0, 1, 0, 0, 1, 1, 0]);
    assert_eq!(
        first.direction,
        StretchRenyiGeometryDirection::OperatorReview
    );
    assert_eq!(
        first.perturbation_changes,
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.125, 0.125, 0.125, 0.0, 0.0, 0.0, 0.0]
    );
    assert_eq!(first.equivalence_failures, 0);
    assert_eq!(first.membership_hash, 0x13ee_bb72_76ee_283d);
    assert_eq!(first.evidence_hash, 0x8e6e_86b6_830b_fa3e);
    eprintln!(
        "renyi_anchor_local gates={:?} perturbation={:?} equivalence={} paths={:?} failed_paths={:?} membership={:016x} evidence={:016x} direction={:?}",
        first.gate_failures,
        first.perturbation_changes,
        first.equivalence_failures,
        first.controls.iter().map(|control| (control.control, control.level_counts, control.path_shape, control.path_cost)).collect::<Vec<_>>(),
        [5, 11].map(|index| (index, first.controls[index].selected_levels.clone())),
        first.membership_hash,
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn transient_evidence_measurement_selects_terminal_direction() {
    let first = transient_evidence_measurement_review();
    let repeated = transient_evidence_measurement_review();
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 12);
    assert!(first.controls.iter().all(|control| {
        control.anchors.len() == 64
            && control.structural_counts[1] == 0
            && control.hashes.iter().all(|hash| *hash != 0)
    }));
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(first.gate_failures, [7, 3, 1, 1, 1, 3, 0]);
    assert_eq!(first.unmatched_perturbation_peaks, 2);
    assert_eq!(first.equivalence_peak_failures, 0);
    assert_eq!(
        first.direction,
        StretchTransientEvidenceDirection::OperatorReview
    );
    assert_eq!(first.evidence_hash, 0x6f67_33bd_a803_16a9);
    eprintln!(
        "transient_evidence gates={:?} perturbation={:?} displacement={:?} unmatched={} equivalence={:?} peak_equivalence={} controls={:?} evidence={:016x} direction={:?}",
        first.gate_failures,
        first.perturbation_changes,
        first.peak_displacements,
        first.unmatched_perturbation_peaks,
        first.equivalence_errors,
        first.equivalence_peak_failures,
        first.controls.iter().map(|control| (control.control, control.peaks.clone(), control.event_offsets.clone(), control.anchors.iter().map(|anchor| anchor.occupancy).fold(0.0_f64, f64::max))).collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn mixed_phase_distribution_audit_selects_terminal_direction() {
    let first = mixed_phase_distribution_review();
    let repeated = mixed_phase_distribution_review();
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 24);
    assert_eq!(first.audit_pairs.len(), 25);
    eprintln!(
        "mixed_phase_distribution structural={:?} equivalence={:.12e} controls={:?} pairs={:?} evidence={:016x} direction={:?}",
        first.structural_failures,
        first.maximum_equivalence_error,
        first.equivalence_errors,
        first
            .audit_pairs
            .iter()
            .map(|pair| (
                pair.magnitude_cutoff,
                pair.mixed_phase_radius,
                pair.event_recall,
                pair.negative_leakage,
                pair.separates,
            ))
            .collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first.structural_failures, [0, 0, 0, 1]);
    assert!((first.maximum_equivalence_error - 2.656292390935e-5).abs() <= 1.0e-15);
    assert_eq!(
        first
            .equivalence_errors
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .map(|(index, _)| index),
        Some(7)
    );
    assert!(first.audit_pairs.iter().all(|pair| !pair.separates));
    assert_eq!(
        first.direction,
        StretchMixedPhaseDistributionDirection::StructuralFailure
    );
    assert!(first
        .controls
        .iter()
        .all(|control| control.bands.len() == 10
            && control.structural_counts[0] == control.structural_counts[1]
            && control.structural_counts[3] == 0
            && control.hashes.iter().all(|hash| *hash != 0)));
    assert_eq!(first.evidence_hash, 0x5b3b_ecee_9074_5c1f);
}

#[test]
#[cfg(not(debug_assertions))]
fn median_hpss_evidence_selects_terminal_direction() {
    let first = median_hpss_evidence_review();
    let repeated = median_hpss_evidence_review();
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 12);
    assert!(first.controls.iter().all(|control| {
        control.anchors.len() == 64
            && control.structural_counts[2] == 0
            && control.hashes.iter().all(|hash| *hash != 0)
    }));
    assert_ne!(first.evidence_hash, 0);
    eprintln!(
        "median_hpss gates={:?} perturbation={:?} displacement={:?} unmatched={} equivalence={:?} peak_equivalence={} controls={:?} evidence={:016x} direction={:?}",
        first.gate_failures,
        first.perturbation_changes,
        first.peak_displacements,
        first.unmatched_perturbation_peaks,
        first.equivalence_errors,
        first.equivalence_peak_failures,
        first.controls.iter().map(|control| (control.control, control.peaks.clone(), control.event_offsets.clone(), control.anchors.iter().map(|anchor| anchor.occupancy).fold(0.0_f64, f64::max))).collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first.gate_failures, [7, 3, 1, 1, 0, 3, 0]);
    assert_eq!(first.unmatched_perturbation_peaks, 2);
    assert_eq!(first.equivalence_peak_failures, 0);
    assert!(first.maximum_equivalence_error <= 2.0e-15);
    assert_eq!(first.direction, StretchMedianHpssDirection::OperatorReview);
    assert_eq!(first.evidence_hash, 0xb481_2090_f561_ea14);
}

#[test]
fn common_grid_phase_transport_rejects_high_band_phase_aliasing() {
    for frequency in [312.5_f32, 1_000.0] {
        let input = (0..24_576)
            .map(|index| {
                0.5 * (std::f32::consts::TAU * frequency * index as f32 / SAMPLE_RATE.0 as f32)
                    .sin()
            })
            .collect::<Vec<_>>();
        let evidence =
            common_grid_tone_phase_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
        assert!(evidence.horizontal_measurements > 0, "{evidence:?}");
        assert!(evidence.vertical_measurements > 0, "{evidence:?}");
        assert!(
            evidence.max_angular_frequency_error <= 1.0e-6,
            "{evidence:?}"
        );
        assert!(
            evidence.max_compensated_phase_residual <= 2.0e-5,
            "{evidence:?}"
        );
        assert!(evidence.all_values_finite);
        eprintln!("common_grid_phase frequency={frequency} {evidence:?}");
    }
    let frequency = 8_000.0_f32;
    let input = (0..24_576)
        .map(|index| {
            (0.5 * (std::f64::consts::TAU * f64::from(frequency) * index as f64
                / f64::from(SAMPLE_RATE.0))
            .sin()) as f32
        })
        .collect::<Vec<_>>();
    let evidence = common_grid_tone_phase_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
    assert!(evidence.max_angular_frequency_error > 1.0e-3);
    assert!(evidence.max_compensated_phase_residual > 0.1);
}

#[test]
fn common_grid_derivative_estimator_is_alias_free_and_deterministic() {
    for frequency in [312.5_f32, 1_000.0, 8_000.0, 19_500.0] {
        let input = periodic_tone(frequency);
        let first =
            common_grid_derivative_tone_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
        let repeated =
            common_grid_derivative_tone_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
        assert_eq!(first, repeated);
        assert!(first.horizontal_measurements > 0, "{first:?}");
        assert!(first.vertical_measurements > 0, "{first:?}");
        assert!(first.max_angular_frequency_error <= 1.0e-6, "{first:?}");
        assert!(first.max_compensated_phase_residual <= 2.0e-5, "{first:?}");
        assert!(first.all_values_finite);
        eprintln!("derivative frequency={frequency} {first:?}");
    }
}

#[test]
fn common_grid_derivative_estimator_handles_silence_and_noise() {
    let silence = common_grid_derivative_tone_review_mono(&vec![0.0; 384], SAMPLE_RATE, 0.0);
    assert_eq!(silence.horizontal_measurements, 0);
    assert_eq!(silence.vertical_measurements, 0);
    assert!(silence.zero_energy_skips > 0);
    assert!(silence.all_values_finite);

    let noise =
        common_grid_derivative_tone_review_mono(&deterministic_noise()[..384], SAMPLE_RATE, 0.0);
    assert!(noise.horizontal_measurements > 0);
    assert!(noise.all_values_finite);
}

#[test]
fn common_grid_projected_phase_fields_are_exact_finite_and_deterministic() {
    let input = mixed_control()[..768].to_vec();
    for ratio in [0.75, 1.0, 1.5] {
        let first = common_grid_projected_phase_review_mono(&input, ratio);
        let repeated = common_grid_projected_phase_review_mono(&input, ratio);
        assert_eq!(first, repeated);
        assert_eq!(
            first.target_frames,
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first.output_columns, first.target_frames.div_ceil(384) + 1);
        assert_eq!(
            first.projected_field_values,
            first.output_columns * 1_536 * 3
        );
        assert!(first.max_coordinate_error <= 1.0e-9, "{first:?}");
        assert!(first.coordinates_monotonic);
        assert!(first.boundary_pad_reads > 0, "{first:?}");
        assert_eq!(first.missing_assignments, 0, "{first:?}");
        assert_eq!(first.duplicate_assignments, 0, "{first:?}");
        assert!(first.heap_high_water <= first.heap_capacity, "{first:?}");
        assert_eq!(first.non_finite_values, 0, "{first:?}");
        if ratio != 1.0 {
            assert!(first.fractional_columns > 0, "{first:?}");
        }
        eprintln!("projected ratio={ratio} {first:?}");
    }
}

#[test]
fn common_grid_projected_phase_heap_uses_both_directions_and_handles_silence() {
    let mixed = common_grid_projected_phase_review_mono(&mixed_control()[..1_536], 1.5);
    assert!(mixed.seed_assignments > 0, "{mixed:?}");
    assert!(mixed.horizontal_assignments > 0, "{mixed:?}");
    assert!(mixed.vertical_assignments > 0, "{mixed:?}");
    assert_eq!(mixed.missing_assignments, 0, "{mixed:?}");
    assert!(mixed.heap_high_water <= mixed.heap_capacity, "{mixed:?}");

    let silence = common_grid_projected_phase_review_mono(&vec![0.0; 768], 0.75);
    assert_eq!(silence.seed_assignments, 0);
    assert_eq!(silence.horizontal_assignments, 0);
    assert_eq!(silence.vertical_assignments, 0);
    assert_eq!(silence.missing_assignments, 0);
    assert_eq!(silence.non_finite_values, 0);
}

#[test]
fn common_grid_projected_phase_contract_controls_pass() {
    let mut impulse = vec![0.0; 768];
    impulse[192] = 1.0;
    let controls = [
        sine(312.5)[..768].to_vec(),
        sine(1_000.0)[..768].to_vec(),
        sine(8_000.0)[..768].to_vec(),
        two_tone_control(),
        chirp_control(false),
        chirp_control(true),
        impulse,
        deterministic_noise()[..768].to_vec(),
        mixed_control()[..768].to_vec(),
        vec![0.0; 768],
    ];
    let mut horizontal = 0;
    let mut vertical = 0;
    let mut max_heap_high_water = 0;
    for input in controls {
        for ratio in [0.75, 1.0, 1.5] {
            let evidence = common_grid_projected_phase_review_mono(&input, ratio);
            assert!(evidence.max_coordinate_error <= 1.0e-9, "{evidence:?}");
            assert!(evidence.coordinates_monotonic);
            assert_eq!(evidence.missing_assignments, 0, "{evidence:?}");
            assert_eq!(evidence.duplicate_assignments, 0, "{evidence:?}");
            assert!(
                evidence.heap_high_water <= evidence.heap_capacity,
                "{evidence:?}"
            );
            assert_eq!(evidence.non_finite_values, 0, "{evidence:?}");
            horizontal += evidence.horizontal_assignments;
            vertical += evidence.vertical_assignments;
            max_heap_high_water = max_heap_high_water.max(evidence.heap_high_water);
        }
    }
    assert!(horizontal > 0);
    assert!(vertical > 0);
    eprintln!(
        "projected controls horizontal={horizontal} vertical={vertical} heap={max_heap_high_water}/3072"
    );
}

#[test]
fn common_grid_dual_guard_is_exact_bounded_and_deterministic() {
    let first = common_grid_dual_guard_review(384);
    let repeated = common_grid_dual_guard_review(384);
    assert_eq!(first, repeated);
    assert!(first.evaluated_channels > 0, "{first:?}");
    assert!(first.max_dual_residual <= 1.0e-8, "{first:?}");
    assert_eq!(first.non_finite_values, 0, "{first:?}");
    if first.passed {
        assert_eq!(first.evaluated_channels, first.channel_count);
        assert!(first.required_guard_lower_bound_frames <= first.guard_cap_frames);
        assert!(first.max_tail_energy_ratio <= 1.0e-12, "{first:?}");
    } else {
        assert!(first.required_guard_lower_bound_frames > first.guard_cap_frames);
    }
    eprintln!("dual_guard {first:?}");
}

#[test]
fn common_grid_tail_attribution_matrix_is_complete_and_deterministic() {
    let first = common_grid_tail_attribution_review();
    let repeated = common_grid_tail_attribution_review();
    assert_eq!(first, repeated);
    assert_eq!(first.probe_fft_frames, 34_176);
    assert_eq!(
        first.radii_frames,
        [384, 1_536, 4_096, 8_192, 12_288, 16_000]
    );
    assert_eq!(first.thresholds, [1.0e-6, 1.0e-8, 1.0e-10, 1.0e-12]);
    assert_eq!(first.atoms.len(), 30);
    assert_eq!(first.tightening_ratios.len(), 5);
    assert_eq!(first.dualization_ratios.len(), 5);
    assert_eq!(first.mirroring_ratios.len(), 15);
    assert!(first.max_dual_residual <= 1.0e-8, "{first:?}");
    assert_eq!(first.non_finite_values, 0, "{first:?}");
    assert!(first.atoms.iter().all(|atom| {
        atom.total_energy.is_finite()
            && atom.total_energy > 0.0
            && atom.tail_energy_ratios.len() == 6
            && atom.guard_lower_bounds.len() == 4
            && atom
                .tail_energy_ratios
                .iter()
                .all(|value| value.is_finite())
    }));
    assert!(first
        .tightening_ratios
        .iter()
        .chain(&first.dualization_ratios)
        .chain(&first.mirroring_ratios)
        .all(|value| value.is_finite() || value.is_infinite()));
    eprintln!("tail_attribution {first:?}");
}

fn periodic_tone(frequency: f32) -> Vec<Sample> {
    (0..24_576)
        .map(|index| {
            (0.5 * (std::f64::consts::TAU * f64::from(frequency) * index as f64
                / f64::from(SAMPLE_RATE.0))
            .sin()) as f32
        })
        .collect()
}

fn two_tone_control() -> Vec<Sample> {
    (0..768)
        .map(|index| {
            let time = index as f64 / f64::from(SAMPLE_RATE.0);
            (0.3 * (std::f64::consts::TAU * 440.0 * time).sin()
                + 0.2 * (std::f64::consts::TAU * 4_000.0 * time).sin()) as f32
        })
        .collect()
}

fn chirp_control(exponential: bool) -> Vec<Sample> {
    let mut phase = 0.0_f64;
    (0..768)
        .map(|index| {
            let position = index as f64 / 767.0;
            let frequency = if exponential {
                200.0_f64 * (8_000.0_f64 / 200.0).powf(position)
            } else {
                200.0 + (8_000.0 - 200.0) * position
            };
            phase += std::f64::consts::TAU * frequency / f64::from(SAMPLE_RATE.0);
            (0.5 * phase.sin()) as f32
        })
        .collect()
}

fn assert_reconstruction_gate(input: &[Sample]) {
    let review = frequency_adaptive_reconstruction_review_mono(input, SAMPLE_RATE);
    let evidence = &review.evidence;
    assert_eq!(review.samples.len(), input.len());
    assert_eq!(evidence.source_frames, input.len());
    assert_eq!(evidence.output_frames, input.len());
    assert!(evidence.band_count > 2);
    assert!(evidence.coefficient_count > 0);
    assert!(evidence.frame_operator_min.is_finite());
    assert!(evidence.frame_operator_min > 0.0);
    assert!(evidence.frame_operator_max.is_finite());
    assert!(evidence.frame_condition_ratio.is_finite());
    assert_eq!(evidence.uncovered_frequency_bins, 0);
    assert!(evidence.multiply_covered_frequency_bins > 0);
    assert_eq!(evidence.painless_support_violations, 0);
    assert!(evidence.reconstruction_peak_error <= 1.0e-5);
    assert!(evidence.reconstruction_rms_error <= 1.0e-6);
    assert!(evidence.reconstruction_head_error <= 1.0e-5);
    assert!(evidence.reconstruction_tail_error <= 1.0e-5);
    assert_eq!(evidence.non_finite_coefficients, 0);
    assert_eq!(evidence.non_finite_output_samples, 0);
    assert!(evidence.max_band_impulse_delay_frames <= 1);
    assert!(evidence.bands.iter().all(|band| {
        band.support_bins <= band.coefficient_count
            && band.decimation_frames > 0
            && band.impulse_peak_frame == 0
    }));
}

fn sine(frequency_hz: f32) -> Vec<Sample> {
    (0..CONTROL_LEN)
        .map(|index| {
            0.5 * (std::f32::consts::TAU * frequency_hz * index as f32 / SAMPLE_RATE.0 as f32).sin()
        })
        .collect()
}

fn deterministic_noise() -> Vec<Sample> {
    let mut state = 0x1234_5678_u32;
    (0..CONTROL_LEN)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state as f32 / u32::MAX as f32 - 0.5) * 0.5
        })
        .collect()
}

fn mixed_control() -> Vec<Sample> {
    let mut samples = (0..CONTROL_LEN)
        .map(|index| {
            let time = index as f32 / SAMPLE_RATE.0 as f32;
            0.3 * (std::f32::consts::TAU * 110.0 * time).sin()
                + 0.2 * (std::f32::consts::TAU * 3_200.0 * time).sin()
        })
        .collect::<Vec<_>>();
    samples[CONTROL_LEN / 3] += 0.5;
    samples
}

#[test]
#[cfg(not(debug_assertions))]
fn oracle_adaptive_synthesis_selects_terminal_direction() {
    let review = super::oracle_adaptive::oracle_adaptive_synthesis_review();
    assert_eq!(review.impulse_errors, [0, 0, 0, -127]);
    assert_eq!(
        review.direction,
        super::oracle_adaptive::Direction::RetireTimeAdaptiveSynthesis
    );
    eprintln!("oracle_adaptive {review:?}");
}
