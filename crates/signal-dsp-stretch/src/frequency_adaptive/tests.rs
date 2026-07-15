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
fn single_owner_adaptive_frame_selects_study_schedule_attachment() {
    let first = single_owner_adaptive_frame_review();
    let repeated = single_owner_adaptive_frame_review();
    assert_eq!(first, repeated);
    assert_eq!(first.identity.evidence_hash, 0x6987_080e_517f_1aec);
    assert_eq!(first.schedules.len(), 5);
    assert!(first.schedules.iter().all(|schedule| {
        schedule.owner_counts[0] == schedule.family_and_frames[1]
            && schedule.owner_counts[1] == schedule.family_and_frames[1]
            && schedule.owner_counts[2] == schedule.family_and_frames[1]
            && schedule.owner_counts[3] == 1
            && schedule.coefficient_counts[0] == schedule.coefficient_counts[1]
            && schedule.work_bound[0] <= schedule.work_bound[1]
            && schedule.ownership_failures == [0; 4]
            && schedule.evidence_hash != 0
    }));
    assert_eq!(
        first.direction,
        StretchSingleOwnerAdaptiveDirection::StudyScheduleAttachment
    );
    assert_ne!(first.evidence_hash, 0);
    eprintln!(
        "single_owner_adaptive_frame schedules={:?} identity={:016x} evidence={:016x} direction={:?}",
        first
            .schedules
            .iter()
            .map(|schedule| (
                schedule.family_and_frames,
                schedule.owner_counts,
                schedule.coefficient_counts,
                schedule.work_bound,
                schedule.ownership_failures,
                schedule.evidence_hash,
            ))
            .collect::<Vec<_>>(),
        first.identity.evidence_hash,
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
fn adaptive_study_time_map_selects_single_frame_phase_contract() {
    let first = adaptive_study_time_map_review();
    let repeated = adaptive_study_time_map_review();
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 3);
    assert_eq!(
        first
            .controls
            .iter()
            .map(|control| control.ratio)
            .collect::<Vec<_>>(),
        [0.75, 1.5, 2.0]
    );
    assert!(first.controls.iter().all(|control| {
        control.selected_points.len() >= 2
            && control.frame_counts[0] == control.source_centres.len()
            && control.frame_counts[0] == control.output_centres.len()
            && control.frame_counts[0] == control.frame_counts[1] + control.frame_counts[2]
            && control.window_counts.iter().sum::<usize>() == control.frame_counts[0]
            && control.hop_extrema.iter().all(|hop| *hop > 0)
            && control.level_mapping_failures == [0; 4]
            && control.structural_failures == [0; 8]
            && control.maximum_event_movement <= 256
            && control.non_finite_values == 0
            && control.hashes.iter().all(|hash| *hash != 0)
    }));
    assert_eq!(
        first.direction,
        StretchAdaptiveStudyMappingDirection::SingleFramePhaseContract
    );
    assert_ne!(first.evidence_hash, 0);
    eprintln!(
        "adaptive_study_time_map controls={:?} evidence={:016x} direction={:?}",
        first
            .controls
            .iter()
            .map(|control| (
                control.ratio,
                control.selected_points.len(),
                control.window_counts,
                control.frame_counts,
                control.hop_extrema,
                control.level_mapping_failures,
                control.structural_failures,
                control.maximum_event_movement,
                control.hashes,
            ))
            .collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_phase_synthesis_selects_mono_objective_gate() {
    use super::adaptive_single_frame_synthesis::{review, Direction};

    let first = review();
    let repeated = review();
    eprintln!("adaptive_single_frame_phase_synthesis {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.controls.len(), 4);
    assert_eq!(
        first
            .controls
            .iter()
            .map(|control| control.ratio)
            .collect::<Vec<_>>(),
        [1.0, 0.75, 1.5, 2.0]
    );
    assert!(first.controls.iter().all(|control| {
        control.selected_points >= 2
            && control.frame_counts[0] > 0
            && control.frame_counts[1] > 0
            && control.phase_state_counts[0] == control.frame_counts[1]
            && control.phase_state_counts[1] == 2
            && control.coverage[0] == 0
            && control.coverage[1] > 0
            && control.frame_values[0] > 0.0
            && control.frame_values[2].is_finite()
            && control.structural_failures == [0; 8]
            && control.identity_peak_error <= 5.0e-12
            && control.tone_frequency_error_hz <= 2.0
            && control.maximum_event_error <= 256
            && control.event_phase_changes > 0
            && control.vertical_phase_changes > 0
            && control.maximum_symmetry_error <= 2.0e-10
            && control.maximum_imaginary_residue <= 2.0e-10
            && control.non_finite_values == 0
            && control.hashes.iter().all(|hash| *hash != 0)
    }));
    assert_eq!(first.direction, Direction::FixedRatioMonoObjectiveGate);
    assert_ne!(first.evidence_hash, 0);
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_synthetic_quality_selects_measured_direction() {
    use super::adaptive_single_frame_synthesis::{quality_review, QualityDirection};

    let first = quality_review();
    let repeated = quality_review();
    let modes = first
        .cases
        .iter()
        .flat_map(|case| case.modes.iter())
        .collect::<Vec<_>>();
    let maximum =
        |field: fn(&super::adaptive_single_frame_synthesis::quality::ModeEvidence) -> f64| {
            modes.iter().map(|mode| field(mode)).fold(0.0_f64, f64::max)
        };
    eprintln!(
        "adaptive_single_frame_synthetic_quality failures={:?} evidence={:016x} direction={:?}",
        first
            .cases
            .iter()
            .filter(|case| {
                case.ownership_failures != 0
                    || case
                        .modes
                        .iter()
                        .flat_map(|mode| mode.hard_failures)
                        .sum::<usize>()
                        != 0
            })
            .map(|case| (
                case.control,
                case.ratio,
                case.ownership_failures,
                case.modes[0].hard_failures,
                case.modes[1].hard_failures,
                case.modes[0].tone_angular_error,
                case.modes[1].tone_angular_error,
                case.modes[0].isolated_error,
                case.modes[1].isolated_error,
                case.modes[0].dense_errors,
                case.modes[1].dense_errors,
            ))
            .collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
    eprintln!(
        "adaptive_single_frame_synthetic_quality summary hard={} regressions={} condition={:.6} symmetry={:.3e} residue={:.3e} identity_peak={:.3e} identity_rms={:.3e} tone={:.3e} isolated={} dense={} replica={:.6} texture_delta={:.6}",
        first.hard_failures,
        first.combined_regressions,
        maximum(|mode| mode.frame_condition),
        maximum(|mode| mode.symmetry_error),
        maximum(|mode| mode.imaginary_residue),
        maximum(|mode| mode.identity_error[0]),
        maximum(|mode| mode.identity_error[1]),
        maximum(|mode| mode.tone_angular_error),
        first
            .cases
            .iter()
            .flat_map(|case| case.modes.iter())
            .map(|mode| mode.isolated_error)
            .max()
            .unwrap_or(0),
        first
            .cases
            .iter()
            .flat_map(|case| case.modes.iter())
            .flat_map(|mode| mode.dense_errors)
            .max()
            .unwrap_or(0),
        maximum(|mode| mode.replica_ratio),
        first
            .cases
            .iter()
            .flat_map(|case| case.mode_deltas)
            .map(f64::abs)
            .fold(0.0_f64, f64::max),
    );
    assert_eq!(first, repeated);
    assert_eq!(first.cases.len(), 48);
    assert!(first.cases.iter().all(|case| {
        case.selected_points >= 2
            && case.modes.iter().all(|mode| {
                mode.frame_condition.is_finite()
                    && mode.frame_condition > 0.0
                    && mode.hashes.iter().all(|hash| *hash != 0)
                    && mode.texture.iter().all(|value| value.is_finite())
            })
    }));
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(
        first.direction,
        if first.hard_failures == 0 && first.combined_regressions == 0 {
            QualityDirection::FrozenMonoDevelopmentObjective
        } else {
            QualityDirection::MeasuredPhaseEventVerticalOrSynthesisStage
        }
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_successor_synthetic_quality_selects_direction() {
    use super::adaptive_single_frame_synthesis::{successor_quality_review, QualityDirection};

    let first = successor_quality_review();
    let repeated = successor_quality_review();
    let candidate_modes = first
        .cases
        .iter()
        .map(|case| &case.modes[1])
        .collect::<Vec<_>>();
    let maximum =
        |field: fn(&super::adaptive_single_frame_synthesis::quality::ModeEvidence) -> f64| {
            candidate_modes
                .iter()
                .map(|mode| field(mode))
                .fold(0.0_f64, f64::max)
        };
    let maximum_texture = std::array::from_fn::<_, 6, _>(|index| {
        candidate_modes
            .iter()
            .map(|mode| mode.texture[index])
            .fold(0.0_f64, f64::max)
    });
    let maximum_mode_delta = std::array::from_fn::<_, 6, _>(|index| {
        first
            .cases
            .iter()
            .map(|case| case.mode_deltas[index].abs())
            .fold(0.0_f64, f64::max)
    });
    eprintln!(
        "adaptive_single_frame_successor_synthetic_quality failures={:?} evidence={:016x} direction={:?}",
        first
            .cases
            .iter()
            .filter(|case| {
                case.ownership_failures != 0
                    || case.modes[1].hard_failures.into_iter().sum::<usize>() != 0
            })
            .map(|case| (
                case.control,
                case.ratio,
                case.ownership_failures,
                case.modes[0].hard_failures,
                case.modes[1].hard_failures,
                case.modes[0].tone_angular_error,
                case.modes[1].tone_angular_error,
                case.modes[0].isolated_error,
                case.modes[1].isolated_error,
                case.modes[0].dense_errors,
                case.modes[1].dense_errors,
            ))
            .collect::<Vec<_>>(),
        first.evidence_hash,
        first.direction,
    );
    eprintln!(
        "adaptive_single_frame_successor_synthetic_quality summary hard={} regressions={} condition={:.6} symmetry={:.3e} residue={:.3e} identity_peak={:.3e} identity_rms={:.3e} tone={:.3e} isolated={} dense={} crest={:.6} replica={:.6} texture={:?} mode_delta={:?}",
        first.hard_failures,
        first.combined_regressions,
        maximum(|mode| mode.frame_condition),
        maximum(|mode| mode.symmetry_error),
        maximum(|mode| mode.imaginary_residue),
        maximum(|mode| mode.identity_error[0]),
        maximum(|mode| mode.identity_error[1]),
        maximum(|mode| mode.tone_angular_error),
        first
            .cases
            .iter()
            .map(|case| case.modes[1].isolated_error)
            .max()
            .unwrap_or(0),
        first
            .cases
            .iter()
            .flat_map(|case| case.modes[1].dense_errors)
            .max()
            .unwrap_or(0),
        maximum(|mode| mode.impulse_crest_db),
        maximum(|mode| mode.replica_ratio),
        maximum_texture,
        maximum_mode_delta,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.cases.len(), 48);
    assert!(first.cases.iter().all(|case| {
        case.selected_points >= 2
            && case.modes.iter().all(|mode| {
                mode.frame_condition.is_finite()
                    && mode.frame_condition > 0.0
                    && mode.hashes.iter().all(|hash| *hash != 0)
                    && mode.texture.iter().all(|value| value.is_finite())
            })
    }));
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(
        first.direction,
        if first.hard_failures == 0 && first.combined_regressions == 0 {
            QualityDirection::SuccessorFrozenMonoDevelopmentObjective
        } else {
            QualityDirection::SuccessorOwningMechanism
        }
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_dense_event_attribution_selects_owner() {
    use super::adaptive_single_frame_synthesis::{
        dense_attribution_review, DenseAttributionDirection,
    };

    let first = dense_attribution_review();
    let repeated = dense_attribution_review();
    eprintln!(
        "adaptive_single_frame_dense_event_attribution rows={} failing={} stages={:?} errors={:?} row_errors={:?} anchors={} resets={} owners={} closure={:?} cancellation={:.6} contributions={} targets={:?} peaks={:?} target_values={:?} peak_values={:?} local={:?} target_contributions={:?} replica={:?} evidence={:016x} direction={:?}",
        first.row_count,
        first.failing_rows,
        first.stage_counts,
        first.maximum_errors,
        first.row_errors,
        first.anchor_failures,
        first.reset_failures,
        first.owner_failures,
        first.maximum_closure_error,
        first.maximum_cancellation_ratio,
        first.traced_contributions,
        first.failure_targets,
        first.failure_peaks,
        first.failure_target_values,
        first.failure_peak_values,
        first.failure_local_peaks,
        first.target_contributions,
        first.replica_contributions,
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.row_count, 6);
    assert_eq!(first.failing_rows, 1);
    assert_eq!(first.stage_counts, [0, 0, 0, 1, 0]);
    assert_eq!(first.maximum_errors, [896, 262]);
    assert_eq!(
        first.row_errors,
        [
            [[463, 401], [219, 351], [896, 509]],
            [[0, 0], [0, 0], [0, 262]],
        ]
    );
    assert_eq!(first.anchor_failures, 0);
    assert_eq!(first.reset_failures, 0);
    assert_eq!(first.owner_failures, 0);
    assert!(first.maximum_closure_error[0] <= 1.0e-12, "{first:?}");
    assert!(first.maximum_closure_error[1] <= 1.0e-9, "{first:?}");
    assert!(first.traced_contributions > 0);
    assert_eq!(first.failure_targets, [16126, 16644]);
    assert_eq!(first.failure_peaks, [16126, 16382]);
    assert_eq!(first.failure_target_values, [1.0, 0.75]);
    assert_eq!(first.evidence_hash, 0x2336_b977_3c32_b2ca);
    assert_eq!(
        first.direction,
        DenseAttributionDirection::OverlapSynthesisRedesign
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_event_overlap_ownership_passes() {
    use super::adaptive_single_frame_synthesis::{
        owned_successor_quality_review, QualityDirection,
    };

    let first = owned_successor_quality_review();
    let repeated = owned_successor_quality_review();
    let failures = first
        .cases
        .iter()
        .filter(|case| case.modes[1].hard_failures.iter().sum::<usize>() != 0)
        .map(|case| {
            (
                case.control,
                case.ratio,
                case.modes[1].hard_failures,
                case.modes[1].dense_errors,
                case.modes[1].identity_error,
            )
        })
        .collect::<Vec<_>>();
    eprintln!(
        "adaptive_single_frame_event_overlap_ownership failures={failures:?} hard={} regressions={} evidence={:016x} direction={:?}",
        first.hard_failures,
        first.combined_regressions,
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.cases.len(), 48);
    assert_eq!(first.hard_failures, 0);
    assert_eq!(first.combined_regressions, 0);
    assert!(first.cases.iter().all(|case| {
        case.ownership_failures == 0 && case.modes[1].hard_failures.into_iter().sum::<usize>() == 0
    }));
    assert_eq!(first.evidence_hash, 0xdec1_5b71_8aa2_7de9);
    assert_eq!(
        first.direction,
        QualityDirection::SuccessorFrozenMonoDevelopmentObjective
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_event_overlap_ownership_removes_replica() {
    use super::adaptive_single_frame_synthesis::overlap_ownership_review;

    let first = overlap_ownership_review();
    let repeated = overlap_ownership_review();
    eprintln!(
        "adaptive_single_frame_event_overlap_ownership pre={:?} post={:?} target_delta={:.3e} replica={} values={:?} contributors={:?} owned={:?} ownership={:x?} contribution={:x?} output={:x?} evidence={:016x}",
        first.pre_errors,
        first.post_errors,
        first.maximum_target_delta,
        first.replica_output,
        first.replica_values,
        first.replica_contributors,
        first.event_owned_samples,
        first.ownership_hashes,
        first.contribution_hashes,
        first.output_hashes,
        first.evidence_hash,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.pre_errors, [[0, 0], [0, 0], [0, 262]]);
    assert_eq!(first.post_errors, [[0, 0]; 3]);
    assert!(first.maximum_target_delta <= 1.0e-12, "{first:?}");
    assert_eq!(first.replica_output, 16382);
    assert_eq!(first.replica_contributors[0].len(), 1);
    assert!(first.replica_values[1].abs() <= 1.0e-12, "{first:?}");
    assert_eq!(first.event_owned_samples, [0, 0, 2]);
    assert_eq!(first.output_hashes[0][0], first.output_hashes[0][1]);
    assert_eq!(first.output_hashes[1][0], first.output_hashes[1][1]);
    assert_ne!(first.output_hashes[2][0], first.output_hashes[2][1]);
    assert_eq!(
        first.contribution_hashes,
        [0xb5fa_80b2_89fc_f1b4, 0x3a77_bac0_45f1_d468]
    );
    assert_eq!(first.evidence_hash, 0xadf3_7bdd_7201_2e19);
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_development_objective_is_frozen() {
    use super::adaptive_single_frame_synthesis::{
        development_objective_review, DevelopmentDirection,
    };

    let first = development_objective_review();
    let repeated = development_objective_review();
    eprintln!("adaptive_single_frame_development_objective {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.rows, 9);
    assert_eq!(first.modes, 3);
    assert_eq!(first.renders, 27);
    assert_eq!(first.holdout_reads, 0);
    assert_eq!(first.hard_failures, 0, "{first:?}");
    assert_eq!(first.candidate_hard_failures, 0, "{first:?}");
    assert_eq!(first.candidate_changed_rows, 9);
    assert_eq!(first.event_fallback_renders, 15);
    assert_eq!(first.candidate_regression_rows, [6, 7, 9, 9]);
    assert_eq!(
        first.hashes,
        [
            0x2abd_e0a1_0417_b469,
            0x4359_fd9e_43ff_6a9c,
            0x1882_3a80_9bb4_b2cc,
            0x10d2_5f84_0426_2480,
        ]
    );
    assert_eq!(
        first.direction,
        DevelopmentDirection::SpectralSynthesisAttribution
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_real_source_stage_attribution_is_frozen() {
    use super::adaptive_single_frame_synthesis::{
        stage_attribution_review, StageAttributionDirection,
    };

    let first = stage_attribution_review();
    let repeated = stage_attribution_review();
    eprintln!("adaptive_single_frame_stage_attribution {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.rows, 9);
    assert_eq!(first.modes, 5);
    assert_eq!(first.renders, 45);
    assert_eq!(first.holdout_reads, 0);
    assert_eq!(first.hard_failures, 7, "{first:?}");
    assert_eq!(first.hard_failures_by_mode, [0, 7, 0, 0, 0]);
    assert_eq!(first.changed_rows, [9, 9, 8, 0]);
    assert_eq!(first.event_fallback_renders, 26);
    assert_eq!(
        first.stage_regression_rows,
        [[8, 7, 9, 9], [2, 3, 1, 3], [3, 4, 7, 3], [0; 4]]
    );
    assert_eq!(
        first.hashes,
        [
            0x59fd_e9d5_897f_e070,
            0x4380_6ef3_d1b3_a311,
            0x30b2_9a8a_65b5_0861,
            0x557e_af8e_6c9e_e5c5,
        ]
    );
    assert_eq!(
        first.direction,
        StageAttributionDirection::OrdinaryAdaptiveSynthesis
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_real_source_resolution_attribution_is_frozen() {
    use super::adaptive_single_frame_synthesis::{
        resolution_attribution_review, ResolutionAttributionDirection,
    };

    let first = resolution_attribution_review();
    let repeated = resolution_attribution_review();
    eprintln!("adaptive_single_frame_resolution_attribution {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.rows, 9);
    assert_eq!(first.modes, 6);
    assert_eq!(first.renders, 54);
    assert_eq!(first.holdout_reads, 0);
    assert_eq!(first.hard_failures, 29, "{first:?}");
    assert_eq!(first.hard_failures_by_mode, [0, 9, 9, 4, 0, 7]);
    assert_eq!(first.event_fallback_renders, 35);
    assert_eq!(first.resolution_changes, [0, 0, 0, 0, 214]);
    assert_eq!(first.changed_from_current, [9; 5]);
    assert_eq!(first.changed_from_adaptive, [9; 4]);
    assert_eq!(
        first.regression_from_current,
        [
            [6, 7, 9, 9],
            [7, 6, 9, 9],
            [4, 4, 9, 9],
            [6, 4, 9, 9],
            [8, 7, 9, 9],
        ]
    );
    assert_eq!(
        first.adaptive_regression_from_fixed,
        [[6, 4, 2, 0], [5, 6, 8, 3], [7, 5, 7, 7], [6, 5, 6, 6]]
    );
    assert_eq!(
        first.hashes,
        [
            0xc4cd_e9a6_38c1_e36e,
            0x9a3f_f69d_dc1d_c765,
            0x3e4f_4a84_89a8_217d,
            0xc00d_6c13_0888_505a,
        ]
    );
    assert_eq!(
        first.direction,
        ResolutionAttributionDirection::SplitResolutionTransitionAndSharedMechanism
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_real_source_mechanism_attribution_is_frozen() {
    use super::adaptive_single_frame_synthesis::{
        mechanism_attribution_review, MechanismAttributionDirection,
    };

    let first = mechanism_attribution_review();
    let repeated = mechanism_attribution_review();
    eprintln!("adaptive_single_frame_mechanism_attribution {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.rows, 9);
    assert_eq!(first.modes, 9);
    assert_eq!(first.renders, 81);
    assert_eq!(first.holdout_reads, 0);
    assert_eq!(first.hard_failures, 19, "{first:?}");
    assert_eq!(first.hard_failures_by_mode, [0, 0, 0, 5, 5, 0, 0, 4, 5]);
    assert_eq!(first.event_fallback_renders, 55);
    assert_eq!(first.changed_from_current, [9; 8]);
    assert_eq!(
        first.regression_from_current,
        [
            [6, 4, 9, 9],
            [7, 3, 9, 9],
            [7, 5, 9, 9],
            [7, 6, 9, 9],
            [6, 4, 9, 9],
            [7, 4, 9, 9],
            [5, 3, 9, 9],
            [5, 4, 9, 9],
        ]
    );
    assert_eq!(
        first.lattice_regressions,
        [[1, 5, 5, 4], [2, 4, 8, 7], [5, 2, 9, 9], [4, 4, 9, 9]]
    );
    assert_eq!(
        first.phase_regressions,
        [[3, 5, 9, 3], [2, 7, 9, 2], [4, 4, 9, 7], [2, 7, 9, 8]]
    );
    assert_eq!(
        first.overlap_regressions,
        [[2, 4, 9, 9], [2, 8, 7, 8], [2, 4, 9, 9], [2, 6, 9, 9]]
    );
    assert_eq!(
        first.hashes,
        [
            0x63d6_4c56_e0e4_02bb,
            0x671b_feb4_1898_1df8,
            0xaaf1_1244_6dc0_f0a8,
            0x3c9f_3f66_ae65_d5c1,
        ]
    );
    assert_eq!(
        first.direction,
        MechanismAttributionDirection::WindowedCoefficientRepresentation
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_real_source_window_attribution_is_frozen() {
    use super::adaptive_single_frame_synthesis::{
        window_attribution_review, WindowAttributionDirection,
    };

    let first = window_attribution_review();
    let repeated = window_attribution_review();
    eprintln!("adaptive_single_frame_window_attribution {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.rows, 9);
    assert_eq!(first.modes, 5);
    assert_eq!(first.renders, 45);
    assert_eq!(first.holdout_reads, 0);
    assert_eq!(first.hard_failures, 1, "{first:?}");
    assert_eq!(first.hard_failures_by_mode, [0, 0, 1, 0, 0]);
    assert_eq!(first.event_fallback_renders, 30);
    assert_eq!(first.changed_from_current, [9; 4]);
    assert_eq!(
        first.regression_from_current,
        [[6, 4, 9, 9], [5, 4, 9, 9], [5, 5, 9, 9], [5, 5, 9, 9]]
    );
    assert_eq!(first.analysis_regressions, [[3, 5, 3, 4], [4, 5, 3, 4]]);
    assert_eq!(first.synthesis_regressions, [[0, 7, 0, 3], [1, 5, 1, 3]]);
    assert_eq!(
        first.hashes,
        [
            0x7d78_8640_2f66_2bc7,
            0x7629_8caf_c837_79af,
            0xa217_3e14_c6eb_7535,
            0x1f7a_6548_0074_cf7b,
        ]
    );
    assert_eq!(
        first.direction,
        WindowAttributionDirection::WindowKernelsContributeButDoNotOwn
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_real_source_geometry_attribution_is_frozen() {
    use super::adaptive_single_frame_synthesis::{
        geometry_attribution_review, GeometryAttributionDirection,
    };

    let first = geometry_attribution_review();
    let repeated = geometry_attribution_review();
    eprintln!("adaptive_single_frame_geometry_attribution {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.rows, 9);
    assert_eq!(first.modes, 5);
    assert_eq!(first.renders, 45);
    assert_eq!(first.holdout_reads, 0);
    assert_eq!(first.hard_failures, 8);
    assert_eq!(first.hard_failures_by_mode, [0, 0, 4, 2, 2]);
    assert_eq!(first.event_fallback_renders, 27);
    assert_eq!(first.changed_from_current, [9; 4]);
    assert_eq!(
        first.regression_from_current,
        [[5, 5, 9, 9], [5, 7, 9, 9], [3, 7, 9, 9], [7, 8, 9, 9]]
    );
    assert_eq!(first.resolution_regressions, [4, 6, 3, 4]);
    assert_eq!(first.fft_grid_regressions, [3, 6, 0, 0]);
    assert_eq!(first.frame_geometry_regressions, [5, 5, 9, 7]);
    assert_eq!(
        first.hashes,
        [
            0x5502_1268_ac0c_b16f,
            0xd788_ea76_42e1_6b09,
            0xb56a_87e8_49ff_3f5a,
            0xfcd4_2c86_7eef_4419,
        ]
    );
    assert_eq!(
        first.direction,
        GeometryAttributionDirection::SharedGridContributesRemainingPathOwns
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_failure_attribution_selects_bounded_redesign() {
    use super::adaptive_single_frame_synthesis::{attribution_review, AttributionDirection};

    let first = attribution_review();
    let repeated = attribution_review();
    eprintln!(
        "adaptive_single_frame_failure_attribution failing={} stages={:?} tone={:.3e} phase={:.3e} resolution={:?} owners={} isolated={} dense={} selected={} centred={} phase_frames={} contributions={} evidence={:016x} direction={:?}",
        first.failing_rows,
        first.stage_counts,
        first.maximum_tone_error,
        first.maximum_phase_frequency_error,
        first.resolution_frequency_error,
        first.peak_owner_changes,
        first.maximum_isolated_error,
        first.maximum_dense_error,
        first.selected_event_centres,
        first.exact_event_centres,
        first.traced_phase_frames,
        first.traced_contributions,
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.failing_rows, 25);
    assert_eq!(first.stage_counts.iter().sum::<usize>(), first.failing_rows);
    assert!(first.maximum_tone_error > 1.0e-6);
    assert!(first.maximum_isolated_error > 1);
    assert!(first.maximum_dense_error > 256);
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(
        first.direction,
        AttributionDirection::ActivePeakPhaseAndInjectedEventOwnershipContract
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_active_peak_and_anchor_ownership_passes() {
    use super::adaptive_single_frame_synthesis::{ownership_review, OwnershipDirection};

    let first = ownership_review();
    let repeated = ownership_review();
    eprintln!(
        "adaptive_single_frame_active_peak_and_anchor_ownership failures={:?} identity={:?} tones={:?} events={:?} owners={:?} anchors={}/{} evidence={:016x} direction={:?}",
        first.failure_counts,
        first.maximum_identity_error,
        first.maximum_tone_errors,
        first.maximum_event_errors,
        first.owner_counts,
        first.detected_anchors,
        first.expected_anchors,
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.failure_counts, [0; 8], "{first:?}");
    assert_eq!(first.detected_anchors, first.expected_anchors);
    assert!(first.owner_counts[0] > 0, "{first:?}");
    assert!(first.owner_counts[1] > 0, "{first:?}");
    assert!(first.owner_counts[3] > 0, "{first:?}");
    assert_ne!(first.evidence_hash, 0);
    assert_eq!(
        first.direction,
        OwnershipDirection::SuccessorSyntheticQualityGate
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_native_grid_active_owner_mechanism_selects_projection_owner() {
    use super::adaptive_single_frame_synthesis::{native_ownership_review, OwnershipDirection};

    let first = native_ownership_review();
    let repeated = native_ownership_review();
    eprintln!(
        "adaptive_single_frame_native_grid_active_owner_mechanism failures={:?} identity={:?} tones={:?} events={:?} owners={:?} transitions={}/{} anchors={}/{} evidence={:016x} direction={:?}",
        first.failure_counts,
        first.maximum_identity_error,
        first.maximum_tone_errors,
        first.maximum_event_errors,
        first.owner_counts,
        first.matched_resolution_transitions,
        first.resolution_transitions,
        first.detected_anchors,
        first.expected_anchors,
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.failure_counts, [0, 0, 3, 0, 0, 0, 0, 0]);
    assert_eq!(first.detected_anchors, first.expected_anchors);
    assert_eq!(
        first.matched_resolution_transitions,
        first.resolution_transitions
    );
    assert!(first.resolution_transitions > 0, "{first:?}");
    assert!(first.owner_counts[0] > 0, "{first:?}");
    assert!(first.owner_counts[1] > 0, "{first:?}");
    assert!(first.owner_counts[3] > 0, "{first:?}");
    assert_eq!(first.maximum_tone_errors[1], 1.263527633866765e-7);
    assert_eq!(first.evidence_hash, 0x19c5_548b_af4a_10c8);
    assert_eq!(
        first.direction,
        OwnershipDirection::ActivePeakOrTransientAnchorRedesign
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn adaptive_single_frame_native_grid_active_owner_synthetic_quality_stops() {
    use super::adaptive_single_frame_synthesis::{
        native_successor_quality_review, QualityDirection,
    };

    let first = native_successor_quality_review();
    let repeated = native_successor_quality_review();
    let failures = first
        .cases
        .iter()
        .filter(|case| {
            case.ownership_failures != 0
                || case.modes[1].hard_failures.into_iter().sum::<usize>() != 0
        })
        .map(|case| {
            (
                case.control,
                case.ratio,
                case.ownership_failures,
                case.modes[1].hard_failures,
                case.modes[1].tone_angular_error,
                case.modes[1].isolated_error,
                case.modes[1].dense_errors,
                case.modes[1].replica_ratio,
            )
        })
        .collect::<Vec<_>>();
    eprintln!(
        "adaptive_single_frame_native_grid_active_owner_synthetic_quality failures={failures:?} hard={} regressions={} evidence={:016x} direction={:?}",
        first.hard_failures,
        first.combined_regressions,
        first.evidence_hash,
        first.direction,
    );
    assert_eq!(first, repeated);
    assert_eq!(first.cases.len(), 48);
    assert_eq!(first.hard_failures, 3, "{failures:?}");
    assert_eq!(first.combined_regressions, 0, "{failures:?}");
    assert_eq!(failures.len(), 3);
    assert!(failures
        .iter()
        .all(|failure| failure.0
            == super::adaptive_single_frame_synthesis::quality::Control::LowTone));
    assert_eq!(first.evidence_hash, 0x2410_e339_4421_4b72);
    assert_eq!(first.direction, QualityDirection::SuccessorOwningMechanism);
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
fn complete_system_exports_concealed_development_pack() {
    let review = super::complete_system_tuning::export_development_pack();
    eprintln!("complete_system_development_pack {review:?}");
    assert_eq!(review.rows, 9);
    assert_eq!(review.candidates_per_row, 5);
    assert_eq!(review.audio_files, 54);
    assert_eq!(review.holdout_reads, 0);
    assert_eq!(review.structural_failures, [0; 5], "{review:?}");
    assert!(review.hashes.iter().all(|hash| *hash != 0));
}

#[test]
#[cfg(not(debug_assertions))]
fn complete_system_attributes_cross_resolution_smear() {
    use super::complete_system_tuning::SmearAttributionDirection;

    let first = super::complete_system_tuning::smear_attribution_review();
    let repeated = super::complete_system_tuning::smear_attribution_review();
    eprintln!("complete_system_smear_attribution {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.configurations, 3);
    assert_eq!(first.development_rows, 9);
    assert_eq!(first.renders, 108);
    assert_eq!(first.holdout_reads, 0);
    assert!(first.maximum_layer_sum_error <= 1.0e-12, "{first:?}");
    assert!(first.modes.iter().all(|mode| mode.renders == 27));
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    assert_eq!(
        first.direction,
        SmearAttributionDirection::CrossResolutionRecombination
    );
}

#[test]
#[cfg(not(debug_assertions))]
fn complete_system_proves_or_rejects_shared_full_field_phase() {
    let first = super::complete_system_tuning::shared_phase_proof_review();
    let repeated = super::complete_system_tuning::shared_phase_proof_review();
    eprintln!("complete_system_shared_phase {first:?}");
    assert_eq!(first, repeated);
    assert_eq!(first.configurations, 3);
    assert_eq!(first.development_rows, 9);
    assert_eq!(first.renders, 33);
    assert_eq!(first.holdout_reads, 0);
    assert!(first.maximum_layer_sum_error <= 1.0e-12, "{first:?}");
    assert!(first.event_resets > 0, "{first:?}");
    assert!(first.shared_phase_assignments > 0, "{first:?}");
    assert!(first.hashes.iter().all(|hash| *hash != 0));
    assert!(matches!(
        first.direction,
        super::complete_system_tuning::SharedPhaseProofDirection::DevelopmentListeningExport
            | super::complete_system_tuning::SharedPhaseProofDirection::NonDuplicatingCoefficientOwnership
    ));
}

#[test]
#[ignore = "superseded by the exact-excerpt comparator confirmation"]
#[cfg(not(debug_assertions))]
fn source_studied_complete_architecture_proof_selects_direction() {
    use super::source_studied::{review, Architecture, Direction};

    let result = review();
    eprintln!("source_studied_complete_architecture {result:#?}");
    assert_eq!(result.geometry, [1_024, 2_048, 4_096]);
    assert_eq!(result.development_rows, 9);
    assert_eq!(result.holdout_reads, 0);
    assert!(result.repeated, "{result:#?}");
    assert_eq!(
        result
            .architecture
            .iter()
            .map(|item| item.architecture)
            .collect::<Vec<_>>(),
        [
            Architecture::FrequencyPartitioned,
            Architecture::WeightedPredictor,
        ]
    );
    assert!(result.architecture.iter().all(|item| {
        item.synthetic_failures[..7]
            .iter()
            .all(|failure| *failure == 0)
            && item.development_failures == [0; 4]
            && item.output_hash != 0
            && item.mean_quality.iter().all(|value| value.is_finite())
    }));
    let partitioned = &result.architecture[0];
    assert!(partitioned
        .frequency_owner_counts
        .iter()
        .all(|count| *count > 0));
    assert!(partitioned.state_counts[0] > 0);
    assert!(partitioned.state_counts[1] > 0);
    assert!(partitioned.state_counts[2] > 0);
    assert!(partitioned.state_counts[4] > 0);
    assert!(partitioned.state_counts[5] > 0);
    assert!(result.comparators.iter().all(|item| {
        item.available_rows == 9
            && item.structural_failures == 0
            && item.output_hash != 0
            && item.mean_quality.iter().all(|value| value.is_finite())
    }));
    assert_eq!(result.architecture[0].synthetic_failures[7], 1);
    assert_eq!(result.architecture[1].synthetic_failures[7], 0);
    assert_eq!(
        result.direction,
        if result
            .architecture
            .iter()
            .all(|item| item.synthetic_failures == [0; 8])
        {
            Direction::MonoDecisionCheckpoint
        } else {
            Direction::ArchitectureResearch
        }
    );
}

#[test]
#[ignore = "superseded by the exact-excerpt comparator confirmation"]
#[cfg(not(debug_assertions))]
fn source_studied_complete_architecture_exports_concealed_pack() {
    let result = super::source_studied::export_development_pack();
    eprintln!("source_studied_complete_architecture_export {result:#?}");
    assert_eq!(result.rows, 9);
    assert_eq!(result.candidates_per_row, 5);
    assert_eq!(result.audio_files, 54);
    assert_eq!(result.holdout_reads, 0);
    assert_eq!(result.structural_failures, [0; 4], "{result:#?}");
    assert!(result.hashes.iter().all(|hash| *hash != 0));
}

#[test]
#[ignore = "requires pinned local Rubber Band and Signalsmith Stretch CLIs"]
#[cfg(not(debug_assertions))]
fn source_studied_exact_excerpt_comparator_confirmation_exports_pack() {
    let result = super::source_studied::confirmation::run();
    eprintln!("source_studied_exact_excerpt_confirmation {result:#?}");
    assert_eq!(result.rows, 9);
    assert_eq!(result.candidates_per_row, 4);
    assert_eq!(result.input_files, 9);
    assert_eq!(result.external_files, 18);
    assert_eq!(result.audio_files, 45);
    assert_eq!(result.holdout_reads, 0);
    assert_eq!(result.structural_failures, [0; 4], "{result:#?}");
    assert_eq!(
        result.hashes,
        [
            0x6988_7b15_e842_0fd7,
            0x9547_b0d5_e924_d8fa,
            0x5e79_eb98_f2fb_dc78,
            0x2f18_94d7_c22b_23de,
            0x2e09_fb7c_e672_ec30,
        ]
    );
    assert_eq!(result.rubber_band_version, "4.0.0");
    assert_eq!(result.signalsmith_version, "1.3.2");
}

#[test]
#[ignore = "requires local Rubber Band 4.0.0 CLI"]
#[cfg(not(debug_assertions))]
fn source_studied_long_form_musical_comparison_exports_pack() {
    let result = super::source_studied::long_form::run();
    eprintln!("source_studied_long_form_confirmation {result:#?}");
    assert_eq!(result.rows, 6);
    assert_eq!(result.candidates_per_row, 3);
    assert_eq!(result.input_files, 6);
    assert_eq!(result.external_files, 6);
    assert_eq!(result.audio_files, 24);
    assert_eq!(result.holdout_reads, 0);
    assert_eq!(result.structural_failures, [0; 4], "{result:#?}");
    assert_eq!(
        result.hashes,
        [
            0xf822_38ad_4e33_2c26,
            0x7848_5bfe_53e1_a1d9,
            0x43b1_b127_91ce_d723,
            0x69b3_3fe2_cc5f_77ec,
            0x605f_25c6_68ff_5db9,
        ]
    );
    assert_eq!(result.rubber_band_version, "4.0.0");
}

#[test]
#[cfg(not(debug_assertions))]
fn source_studied_faithful_predictor_synthetic_proof() {
    use super::source_studied::faithful_predictor::{review, Direction};

    let result = review();
    eprintln!("source_studied_faithful_predictor {result:#?}");
    assert!(result.repeated);
    assert_eq!(result.structural_failures, [0; 5]);
    assert!(result.maximum_bass_error_hz <= 0.5);
    assert_eq!(result.octave_failures, 0);
    assert!(result.maximum_chord_peak_error_hz <= 0.5);
    assert!(result.chord_input_out_of_band_db <= -60.0);
    assert!(result.chord_out_of_band_db > -60.0);
    assert!(result.maximum_event_error_frames <= 256);
    assert_eq!(result.replica_failures, 0);
    assert_eq!(result.silence_peak, 0.0);
    assert_eq!(result.output_hash, 0xe7cc_3f04_c24b_5d18);
    assert_eq!(result.direction, Direction::PinnedSourceParity);
}

#[test]
#[cfg(not(debug_assertions))]
fn source_studied_faithful_predictor_sideband_attribution() {
    use super::source_studied::faithful_predictor::{attribution::review, TraceStage};

    let result = review();
    eprintln!("source_studied_faithful_predictor_attribution {result:#?}");
    assert!(result.repeated);
    assert_eq!(result.stages.len(), 6);
    assert!(result.overlap_oracle_out_of_band_db <= -60.0);
    assert!(result.maximum_normalization_phase_delta <= f64::EPSILON * 2.0);
    assert_eq!(result.significant_fallback, 0);
    assert_eq!(
        result.stages.map(|stage| stage.output_hash),
        [
            0xd580_6c0a_7812_2f0d,
            0x620a_7f62_94cf_49b4,
            0xc8c6_7ce3_00ba_84cc,
            0x8017_e923_2161_37a9,
            0x9c2d_3632_f76c_62f3,
            0xabfe_4a58_2352_59ba,
        ]
    );
    assert_eq!(result.earliest_failure, TraceStage::Horizontal);
}

#[test]
#[cfg(not(debug_assertions))]
fn source_studied_faithful_predictor_horizontal_mixture_attribution() {
    use super::source_studied::faithful_predictor::attribution::{
        mixture_review, MixtureDirection,
    };

    let result = mixture_review();
    eprintln!("source_studied_faithful_predictor_mixture {result:#?}");
    assert!(result.repeated);
    assert!(result.mixed_out_of_band_db > -60.0);
    assert!(result
        .tones
        .iter()
        .all(|tone| tone.isolated_out_of_band_db > -60.0));
    assert!(result
        .tones
        .iter()
        .all(|tone| tone.mixed_ratio_variance > tone.isolated_ratio_variance));
    assert_eq!(
        result.tones.map(|tone| tone.isolated_hash),
        [
            0xdb66_2ac6_cf32_fb17,
            0xd218_9114_bff2_9738,
            0x24e6_eb18_6e42_241a,
            0x33bc_2750_eb5a_6da1,
        ]
    );
    assert_eq!(result.mixed_hash, 0xd580_6c0a_7812_2f0d);
    assert_eq!(result.direction, MixtureDirection::PredictorEquation);
}

#[test]
#[cfg(not(debug_assertions))]
fn source_studied_faithful_predictor_state_lineage_attribution() {
    use super::source_studied::faithful_predictor::attribution::{
        state_lineage_review, StateLineageDirection,
    };

    let result = state_lineage_review();
    eprintln!("source_studied_faithful_predictor_state_lineage {result:#?}");
    assert!(result.repeated);
    assert_eq!(result.tones.len(), 4);
    assert!(result
        .tones
        .iter()
        .all(|tone| tone.corrected_feedback_out_of_band_db > -60.0));
    assert!(result.tones.iter().all(|tone| {
        tone.horizontal_phase_recurrence_out_of_band_db > -60.0
            && tone.horizontal_phase_recurrence_out_of_band_db
                < tone.corrected_feedback_out_of_band_db
            && (tone.horizontal_phase_recurrence_sideband_offset_hz - 100.0 / 3.0).abs() <= 0.25
    }));
    assert_eq!(
        result
            .tones
            .map(|tone| tone.horizontal_phase_recurrence_hash),
        [
            0xc444_b985_3804_2212,
            0xaf17_c2de_ab11_133d,
            0xb606_b032_3bac_0dba,
            0x5f08_9be7_738b_f6b0,
        ]
    );
    assert_eq!(
        result.mixed_horizontal_phase_recurrence_hash,
        0x3f0d_01c0_2056_3c31
    );
    assert!(
        result.mixed_horizontal_phase_recurrence_out_of_band_db
            < result.mixed_corrected_feedback_out_of_band_db
    );
    assert_eq!(
        result.direction,
        StateLineageDirection::DirectHorizontalRecurrence
    );
}

#[test]
#[ignore = "requires pinned local Signalsmith Stretch revision 57b93f4e"]
#[cfg(not(debug_assertions))]
fn source_studied_pinned_signalsmith_synthetic_comparator() {
    use super::source_studied::faithful_predictor::pinned_source::{
        review, PinnedSourceDirection, PinnedSourceInternalDifferential,
    };

    let result = review();
    eprintln!("source_studied_pinned_signalsmith {result:#?}");
    assert!(result.repeated);
    assert_eq!(result.structural_failures, [0; 6]);
    assert_eq!(result.geometry, [240, 960, 4]);
    assert_eq!(result.revision, "57b93f4e9206a089a45387eaa39bdc9f310d3308");
    assert_eq!(result.version, "1.3.2");
    assert!(result.tones.iter().all(|tone| {
        tone.input_out_of_band_db <= -60.0
            && tone.output_out_of_band_db > -60.0
            && tone.output_peak_error_hz <= 0.5
            && (tone.strongest_sideband_offset_hz - 100.0 / 3.0).abs() <= 0.25
    }));
    assert!(result.chord_input_out_of_band_db <= -60.0);
    assert!(result.chord_output_out_of_band_db > -60.0);
    assert!(result.chord_peak_error_hz <= 0.5);
    assert_eq!(result.absolute_diagnostic_failures, [4, 1]);
    assert_eq!(result.source_relative_failures, [3, 1]);
    assert_eq!(result.zero_extended_source_relative_failures, [3, 1]);
    assert!(result.tones.iter().all(|tone| {
        tone.zero_extended_out_of_band_db.is_finite()
            && tone.zero_extended_peak_error_hz <= 0.5
            && tone.zero_extended_hash != 0
    }));
    assert!(result.chord_zero_extended_out_of_band_db.is_finite());
    assert!(result.chord_zero_extended_peak_error_hz <= 0.5);
    assert_eq!(
        result.internal_differential,
        PinnedSourceInternalDifferential::FractionalFrequencyBoundaryPolicy
    );
    assert_eq!(result.affected_frequency_observations_per_frame, 10);
    assert!(result
        .tones
        .iter()
        .all(|tone| tone.zero_extended_minus_clamped_db.abs() < 0.04));
    assert!(result.chord_zero_extended_minus_clamped_db.abs() < 0.07);
    assert_eq!(
        result.output_hashes,
        [
            0x7069_b2be_6cef_6725,
            0x570e_dabe_6cef_6725,
            0xa76d_aebe_6cef_6725,
            0xee8e_d9be_6cef_6725,
            0xc4a9_f43e_6cef_6725,
        ]
    );
    assert_eq!(
        result.signal_hashes,
        [
            0xece1_0d1f_7f11_15e8,
            0xbea6_92e6_1f3a_72c5,
            0x218c_b9d3_0316_ce82,
            0xf966_7d3e_af80_c2a9,
            0x8d12_3802_d064_25c0,
        ]
    );
    assert_eq!(
        result.zero_extended_hashes,
        [
            0xad83_e502_f285_7859,
            0xd3ad_a3ac_a57d_9041,
            0xa066_a6c2_165f_6fd1,
            0x68dd_4e32_317a_46fb,
            0x35ec_df54_6775_3361,
        ]
    );
    assert_eq!(
        result.direction,
        PinnedSourceDirection::FrequencyBoundaryPolicyRejected
    );
}

#[test]
#[ignore = "requires pinned local Signalsmith Stretch source and a C++17 compiler"]
#[cfg(not(debug_assertions))]
fn source_studied_pinned_signalsmith_stage_trace() {
    use super::source_studied::faithful_predictor::stage_trace::{review, StageTraceDirection};

    let result = review();
    eprintln!("source_studied_pinned_signalsmith_stage_trace {result:#?}");
    assert!(result.repeated);
    assert_eq!(
        result.source_revision,
        "57b93f4e9206a089a45387eaa39bdc9f310d3308"
    );
    assert_eq!(
        result.linear_revision,
        "5668673560146a9cfe38c25315071e3fd68c8317"
    );
    assert_eq!(result.controls.len(), 3);
    assert_eq!(result.source_geometry.block_frames, 960);
    assert_eq!(result.source_geometry.interval_frames, 240);
    assert_eq!(result.source_geometry.transform_frames, 1024);
    assert_eq!(result.source_geometry.bands, 512);
    assert!(result.source_geometry.modified_grid);
    assert_eq!(result.source_geometry.first_bin_hz, 3.90625);
    assert_eq!(result.source_geometry.bin_step_hz, 7.8125);
    assert_eq!(result.source_geometry.source_center, 8_400);
    assert_eq!(result.signal_geometry.transform_frames, 960);
    assert_eq!(result.signal_geometry.bands, 481);
    assert!(!result.signal_geometry.modified_grid);
    assert_eq!(result.signal_geometry.first_bin_hz, 0.0);
    assert_eq!(result.signal_geometry.bin_step_hz, 25.0 / 3.0);
    assert!(result.controls.iter().all(|control| {
        control.source_hashes.iter().all(|hash| *hash != 0)
            && control.signal_hashes.iter().all(|hash| *hash != 0)
            && control
                .normalized_magnitude_deltas
                .iter()
                .all(|delta| delta.is_finite())
            && control
                .relative_phase_deltas
                .iter()
                .all(|delta| delta.is_finite())
    }));
    assert_eq!(
        result
            .controls
            .iter()
            .map(|control| control.source_hashes)
            .collect::<Vec<_>>(),
        vec![
            [
                0x900c_6f81_4d64_d4e5,
                0x7aca_7cb5_2a25_4a16,
                0x0bf9_39ce_f8ec_3304,
            ],
            [
                0xa220_3b80_4256_6f4d,
                0x6676_ee83_6d75_4e9d,
                0xa674_81f4_52c6_f55e,
            ],
            [
                0xd242_be28_9a89_286c,
                0x2b65_e1d0_9eda_c920,
                0x98d5_b0da_848b_32af,
            ],
        ]
    );
    assert_eq!(
        result
            .controls
            .iter()
            .map(|control| control.signal_hashes)
            .collect::<Vec<_>>(),
        vec![
            [
                0xbcd8_03b5_f136_9855,
                0x5faf_1f54_0aeb_e235,
                0xa205_cce3_3610_e027,
            ],
            [
                0x0d68_2dcd_a081_bea2,
                0x29ce_3d32_ac17_f9ac,
                0x5faf_3ed8_6ad8_95a4,
            ],
            [
                0x7411_d6fe_fdd8_50ab,
                0xb4b3_1fae_c344_8c14,
                0x1b50_e5e2_1f58_d59a,
            ],
        ]
    );
    assert_eq!(result.direction, StageTraceDirection::AnalysisTransformGrid);
}

#[test]
#[ignore = "requires pinned local Signalsmith Stretch revision 57b93f4e"]
#[cfg(not(debug_assertions))]
fn source_studied_modified_half_bin_analysis_grid() {
    use super::source_studied::faithful_predictor::analysis_grid::{review, ModifiedGridDirection};

    let result = review();
    eprintln!("source_studied_modified_half_bin_analysis_grid {result:#?}");
    assert!(result.repeated);
    assert_eq!(result.geometry, [960, 240, 1_024, 512]);
    assert!(result.identity_maximum_error <= 1.0e-10);
    assert_eq!(result.structural_failures, [0; 6]);
    assert_eq!(result.baseline_source_relative_failures, [3, 1]);
    assert_eq!(result.source_relative_failures, [4, 1]);
    assert!(result
        .tones
        .iter()
        .all(|tone| tone.peak_error_hz <= 0.5 && tone.hash != 0));
    assert!(result.chord_peak_error_hz <= 0.5);
    assert_eq!(
        result
            .tones
            .iter()
            .map(|tone| tone.hash)
            .collect::<Vec<_>>(),
        vec![
            0xed99_b730_4bdf_dd6b,
            0xbced_cde4_945b_ad2f,
            0x2c3c_ee95_8e77_c777,
            0x9e90_6652_07d8_57bd,
        ]
    );
    assert_eq!(result.chord_hash, 0x4408_80e3_f642_c797);
    assert_eq!(result.direction, ModifiedGridDirection::GridRejected);
}

#[test]
#[ignore = "requires pinned local Signalsmith Stretch revision 57b93f4e"]
#[cfg(not(debug_assertions))]
fn source_studied_pinned_kaiser_analysis_window() {
    use super::source_studied::faithful_predictor::analysis_window::{
        review, KaiserWindowDirection,
    };

    let result = review();
    eprintln!("source_studied_pinned_kaiser_analysis_window {result:#?}");
    assert!(result.repeated);
    assert_eq!(result.geometry, [960, 240, 960, 481]);
    assert_eq!(
        result.coefficient_hashes,
        [0xcd81_1c4f_82d1_61be, 0xcd81_1c4f_82d1_61be]
    );
    assert_eq!(result.overlap_product_hash, 0x6dad_f0c9_86c4_bd49);
    assert_eq!(result.maximum_analysis_synthesis_delta, 0.0);
    assert!((0.0025..0.0026).contains(&result.maximum_symmetry_delta));
    assert!(result.maximum_overlap_error <= 1.0e-6);
    assert!(result.identity_maximum_error <= 1.0e-10);
    assert_eq!(result.structural_failures, [0; 6]);
    assert_eq!(result.baseline_source_relative_failures, [3, 1]);
    assert_eq!(result.source_relative_failures, [4, 1]);
    assert!(result
        .tones
        .iter()
        .all(|tone| tone.peak_error_hz <= 0.5 && tone.hash != 0));
    assert!(result.chord_peak_error_hz <= 0.5);
    assert_eq!(
        result
            .tones
            .iter()
            .map(|tone| tone.hash)
            .collect::<Vec<_>>(),
        vec![
            0x99ae_e1fb_82b4_aea1,
            0x7a22_fba5_3bb2_e333,
            0xcfd6_c08b_12f2_9e5f,
            0x2261_9018_da9f_7247,
        ]
    );
    assert_eq!(result.chord_hash, 0x943f_51d0_3c8b_b374);
    assert_eq!(result.direction, KaiserWindowDirection::WindowRejected);
}

#[test]
#[ignore = "requires pinned local Signalsmith Stretch revision 57b93f4e"]
#[cfg(not(debug_assertions))]
fn source_studied_pinned_analysis_representation_interaction() {
    use super::source_studied::faithful_predictor::analysis_interaction::{
        review, AnalysisInteractionDirection,
    };

    let result = review();
    eprintln!("source_studied_pinned_analysis_representation_interaction {result:#?}");
    assert!(result.repeated);
    assert_eq!(result.geometry, [960, 240, 1_024, 512]);
    assert!(result.identity_maximum_error <= 1.0e-10);
    assert_eq!(result.structural_failures, [0; 6]);
    assert_eq!(result.baseline_source_relative_failures, [3, 1]);
    assert_eq!(result.grid_source_relative_failures, [4, 1]);
    assert_eq!(result.window_source_relative_failures, [4, 1]);
    assert_eq!(result.source_relative_failures, [0, 0]);
    assert!(result
        .tones
        .iter()
        .all(|tone| tone.peak_error_hz <= 0.5 && tone.combined_minus_pinned_db <= 1.0));
    assert!(result.chord.peak_error_hz <= 0.5);
    assert!(result.chord.combined_minus_pinned_db <= 1.0);
    assert_eq!(
        result
            .tones
            .iter()
            .map(|tone| tone.hash)
            .collect::<Vec<_>>(),
        vec![
            0x1497_ff00_420e_bf4e,
            0x34d3_f1e1_8ab5_6752,
            0x1dda_3a2c_0163_ac8f,
            0x1146_5d18_4b11_1c89,
        ]
    );
    assert_eq!(result.chord.hash, 0xd23c_d768_f2a4_61bd);
    assert_eq!(
        result.direction,
        AnalysisInteractionDirection::SourceParityClosed
    );
}

#[test]
#[ignore = "requires pinned local Signalsmith Stretch revision 57b93f4e"]
#[cfg(not(debug_assertions))]
fn source_studied_coherent_representation_synthetic_gate() {
    use super::source_studied::faithful_predictor::coherent_representation::{
        review, source_geometry, CoherentRepresentationDirection,
    };

    let result = review();
    eprintln!("source_studied_coherent_representation_synthetic_gate {result:#?}");
    assert_eq!(source_geometry(11_025), [1_323, 330, 1_536, 768]);
    assert!(result.repeated);
    assert_eq!(result.geometry, [960, 240, 1_024, 512]);
    assert_eq!(result.structural_failures, [0; 5]);
    assert!(result.maximum_bass_error_hz <= 0.5);
    assert_eq!(result.octave_failures, 0);
    assert!(result.maximum_chord_peak_error_hz <= 0.5);
    assert!(result.chord_input_out_of_band_db <= -60.0);
    assert!(result.chord_out_of_band_db > -60.0);
    assert!(result.maximum_event_error_frames <= 256);
    assert_eq!(result.replica_failures, 0);
    assert_eq!(result.silence_peak, 0.0);
    assert_eq!(result.source_relative_failures, [0, 0]);
    assert_eq!(result.window_hash, 0x7409_3f3e_27a2_8e25);
    assert!(result.pinned_window_maximum_delta <= 1.0e-9);
    assert!(result.mechanisms.horizontal > 0);
    assert!(result.mechanisms.short_lower > 0);
    assert!(result.mechanisms.short_upper > 0);
    assert!(result.mechanisms.long_lower > 0);
    assert!(result.mechanisms.long_upper > 0);
    assert!(result.mechanisms.corrected > 0);
    assert!(result.mechanisms.fallback > 0);
    assert_eq!(result.output_hash, 0x0905_a7fd_4180_bff4);
    assert_eq!(
        result.source_parity_hashes,
        [
            0x1497_ff00_420e_bf4e,
            0x34d3_f1e1_8ab5_6752,
            0x1dda_3a2c_0163_ac8f,
            0x1146_5d18_4b11_1c89,
            0xd23c_d768_f2a4_61bd,
        ]
    );
    assert_eq!(
        result.direction,
        CoherentRepresentationDirection::ExactInputRealSourceConfirmation
    );
}

#[test]
#[ignore = "requires pinned Signalsmith Stretch and long-form source pack"]
#[cfg(not(debug_assertions))]
fn source_studied_exact_input_real_source_confirmation() {
    use super::source_studied::faithful_predictor::real_source_confirmation::{
        review, RealSourceConfirmationDirection,
    };

    let result = review();
    eprintln!("source_studied_exact_input_real_source_confirmation {result:#?}");
    assert_eq!(result.rows.len(), 6);
    assert_eq!(result.geometry, [5_292, 1_323, 6_144, 3_072]);
    assert_eq!(result.window_hash, 0x70ba_1688_509b_2915);
    assert_eq!(result.structural_failures, [0; 5]);
    assert_eq!(result.coherent_hard_failures, 0);
    assert_eq!(result.pinned_hard_failures, 0);
    assert_eq!(result.coherent_regression_rows, [2, 3, 2, 6]);
    assert_eq!(
        result.hashes,
        [
            0x8ede_75db_ae22_54b2,
            0x7ec6_54eb_4140_41ce,
            0xee39_390a_1e17_d923,
            0xd9f2_2286_61af_1e53,
            0x7a6b_1e7d_d7ba_5c13,
        ]
    );
    assert_eq!(
        result
            .rows
            .iter()
            .map(|row| [row.coherent_hash, row.pinned_hash])
            .collect::<Vec<_>>(),
        [
            [0xabbc_3c07_2f98_d138, 0xd325_5fce_23c2_439d],
            [0xe552_674a_adaf_187b, 0x42df_d521_c8f5_899d],
            [0x7116_c515_39bb_7653, 0x4d94_4e1e_d711_1845],
            [0xafd2_e093_6115_a798, 0x09b0_bac2_8ff4_da45],
            [0x80f3_fa01_c76b_8ea6, 0x7a5d_1184_d823_099d],
            [0x4eed_ee52_ad33_c269, 0xb2da_890d_54d1_6c45],
        ]
    );
    assert!(result.repeated);
    assert!(result.pinned_repeated);
    assert_eq!(result.signalsmith_version, "1.3.2");
    assert_eq!(
        result.direction,
        RealSourceConfirmationDirection::ConcealedMusicalComparison
    );
}

#[test]
#[ignore = "requires fixed-seed pinned Signalsmith Stretch and long-form source pack"]
#[cfg(not(debug_assertions))]
fn source_studied_concealed_coherent_source_comparison_exports_pack() {
    use super::source_studied::faithful_predictor::concealed_comparison::export;

    let first = export();
    let repeated = export();
    eprintln!("source_studied_concealed_coherent_source_comparison {first:#?}");
    assert_eq!(first, repeated);
    assert_eq!(first.rows, 6);
    assert_eq!(first.candidates_per_row, 2);
    assert_eq!(first.audio_files, 18);
    assert_eq!(first.holdout_reads, 0);
    assert_eq!(first.structural_failures, [0; 7], "{first:#?}");
    assert!(first.maximum_candidate_rms_delta_db <= 1.0e-5);
    assert_eq!(
        first.hashes,
        [
            0x7605_7724_1605_fb24,
            0x64c2_874d_d6e4_7521,
            0x7bba_88c9_c701_bf1c,
            0xfd12_55a2_fc00_7590,
            0xbb19_74bb_a5a2_a8b0,
            0x91d6_8633_349f_1944,
            0xde41_7d1f_00e5_5f88,
        ]
    );
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
