use signal_dsp_stretch::{
    StretchFormantBoundaryMeasurement, StretchPhaseGradientRender,
    StretchRenderIntegrityMeasurement, StretchTonalTextureMeasurement,
    StretchTransientDetailMeasurement, StretchTransientEventDetail,
};

use super::quoted_report_field;

const REPLICA_WINDOW_FRAMES: usize = 512;

pub(super) struct PhaseGradientReviewEvidence<'a> {
    pub case_id: &'a str,
    pub source_path: &'a str,
    pub ratio: f64,
    pub render: &'a StretchPhaseGradientRender,
    pub current_output: &'a [f32],
    pub current_tonal: StretchTonalTextureMeasurement,
    pub candidate_tonal: StretchTonalTextureMeasurement,
    pub current_formant: StretchFormantBoundaryMeasurement,
    pub candidate_formant: StretchFormantBoundaryMeasurement,
    pub current_transient: StretchTransientDetailMeasurement,
    pub candidate_transient: StretchTransientDetailMeasurement,
    pub anchor_events: Option<(StretchTransientEventDetail, StretchTransientEventDetail)>,
    pub candidate_integrity: StretchRenderIntegrityMeasurement,
    pub candidate_integrity_passed: bool,
    pub external_tonal: StretchTonalTextureMeasurement,
    pub external_formant: StretchFormantBoundaryMeasurement,
    pub external_transient: StretchTransientDetailMeasurement,
    pub external_integrity_passed: bool,
    pub comparator_alignment_lag_frames: isize,
    pub comparator_aligned_frames: usize,
    pub comparator_aligned_correlation: f64,
    pub comparator_aligned_rms_error: f64,
}

impl PhaseGradientReviewEvidence<'_> {
    pub fn format_report_line(&self) -> String {
        let (anchor_input_frame, current_anchor_crest_db, candidate_anchor_crest_db) = self
            .anchor_events
            .map(|(current, candidate)| {
                (
                    current.input_frame,
                    current.crest_growth_db,
                    candidate.crest_growth_db,
                )
            })
            .unwrap_or((0, f64::NAN, f64::NAN));
        let (current_replica_ratio, candidate_replica_ratio) = self
            .anchor_events
            .map(|(current, candidate)| {
                (
                    post_attack_secondary_peak_ratio(
                        self.current_output,
                        current.output_frame,
                        REPLICA_WINDOW_FRAMES,
                    ),
                    post_attack_secondary_peak_ratio(
                        &self.render.samples,
                        candidate.output_frame,
                        REPLICA_WINDOW_FRAMES,
                    ),
                )
            })
            .unwrap_or((f64::NAN, f64::NAN));
        let evidence = &self.render.evidence;

        format!(
            "external_benchmark_phase_gradient_review case={} source={} ratio={:.6} window_frames={} fft_frames={} analysis_hop_frames={} synthesis_hop_frames={} synthesis_frames={} significant_bins={} insignificant_bins={} horizontal_assignments={} vertical_assignments={} duplicate_assignments={} missing_assignments={} heap_high_water={} heap_capacity_bound={} max_conjugate_symmetry_error={:.9} uncovered_output_samples={} derivatives_finite={} all_samples_finite={} synthesis_positions_monotonic={} sample_hash={:016x} trace_hash={:016x} anchor_input_frame={} current_anchor_crest_growth_db={:.6} candidate_anchor_crest_growth_db={:.6} anchor_crest_improvement_db={:.6} current_max_crest_growth_db={:.6} candidate_max_crest_growth_db={:.6} external_max_crest_growth_db={:.6} current_mean_absolute_offset_frames={:.6} candidate_mean_absolute_offset_frames={:.6} external_mean_absolute_offset_frames={:.6} current_post_attack_secondary_peak_ratio={:.6} candidate_post_attack_secondary_peak_ratio={:.6} post_attack_secondary_peak_ratio_delta={:.6} current_tonal_residual_ratio={:.6} candidate_tonal_residual_ratio={:.6} external_tonal_residual_ratio={:.6} current_tonal_sideband_ratio={:.6} candidate_tonal_sideband_ratio={:.6} external_tonal_sideband_ratio={:.6} current_spectral_modulation_delta={:.6} candidate_spectral_modulation_delta={:.6} external_spectral_modulation_delta={:.6} current_formant_residual_ratio={:.6} candidate_formant_residual_ratio={:.6} external_formant_residual_ratio={:.6} current_formant_centroid_shift_hz={:.6} candidate_formant_centroid_shift_hz={:.6} external_formant_centroid_shift_hz={:.6} current_max_boundary_step_dbfs={:.6} candidate_max_boundary_step_dbfs={:.6} external_max_boundary_step_dbfs={:.6} candidate_endpoint_energy_delta_db={:.6} candidate_added_silence_frames={} candidate_peak_growth_db={:.6} candidate_integrity_passed={} external_integrity_passed={} comparator_alignment_lag_frames={} comparator_aligned_frames={} comparator_aligned_correlation={:.9} comparator_aligned_rms_error={:.9}",
            self.case_id,
            quoted_report_field(self.source_path),
            self.ratio,
            evidence.window_frames,
            evidence.fft_frames,
            evidence.analysis_hop_frames,
            evidence.synthesis_hop_frames,
            evidence.synthesis_frames,
            evidence.significant_bins,
            evidence.insignificant_bins,
            evidence.horizontal_assignments,
            evidence.vertical_assignments,
            evidence.duplicate_assignments,
            evidence.missing_assignments,
            evidence.heap_high_water,
            evidence.heap_capacity_bound,
            evidence.max_conjugate_symmetry_error,
            evidence.uncovered_output_samples,
            evidence.derivatives_finite,
            evidence.all_samples_finite,
            evidence.synthesis_positions_monotonic,
            evidence.sample_hash,
            evidence.trace_hash,
            anchor_input_frame,
            current_anchor_crest_db,
            candidate_anchor_crest_db,
            current_anchor_crest_db - candidate_anchor_crest_db,
            self.current_transient.max_transient_crest_growth_db,
            self.candidate_transient.max_transient_crest_growth_db,
            self.external_transient.max_transient_crest_growth_db,
            self.current_transient.mean_absolute_timing_offset_frames,
            self.candidate_transient.mean_absolute_timing_offset_frames,
            self.external_transient.mean_absolute_timing_offset_frames,
            current_replica_ratio,
            candidate_replica_ratio,
            candidate_replica_ratio - current_replica_ratio,
            self.current_tonal.mean_spectral_residual_ratio,
            self.candidate_tonal.mean_spectral_residual_ratio,
            self.external_tonal.mean_spectral_residual_ratio,
            self.current_tonal.mean_added_sideband_ratio,
            self.candidate_tonal.mean_added_sideband_ratio,
            self.external_tonal.mean_added_sideband_ratio,
            self.current_tonal.spectral_modulation_delta,
            self.candidate_tonal.spectral_modulation_delta,
            self.external_tonal.spectral_modulation_delta,
            self.current_formant.mean_envelope_residual_ratio,
            self.candidate_formant.mean_envelope_residual_ratio,
            self.external_formant.mean_envelope_residual_ratio,
            self.current_formant.mean_envelope_centroid_shift_hz,
            self.candidate_formant.mean_envelope_centroid_shift_hz,
            self.external_formant.mean_envelope_centroid_shift_hz,
            self.current_formant.max_boundary_step_dbfs,
            self.candidate_formant.max_boundary_step_dbfs,
            self.external_formant.max_boundary_step_dbfs,
            self.candidate_integrity.endpoint_energy_delta_db,
            self.candidate_integrity.added_silence_frames,
            self.candidate_integrity.peak_growth_db,
            self.candidate_integrity_passed,
            self.external_integrity_passed,
            self.comparator_alignment_lag_frames,
            self.comparator_aligned_frames,
            self.comparator_aligned_correlation,
            self.comparator_aligned_rms_error,
        )
    }
}

fn post_attack_secondary_peak_ratio(samples: &[f32], center: usize, frame_size: usize) -> f64 {
    if samples.is_empty() || center >= samples.len() || frame_size < 4 {
        return f64::NAN;
    }
    let guard = frame_size / 8;
    let primary_start = center.saturating_sub(guard);
    let primary_end = center.saturating_add(guard).min(samples.len());
    let secondary_start = primary_end;
    let secondary_end = center.saturating_add(frame_size).min(samples.len());
    let peak = |range: std::ops::Range<usize>| {
        samples[range]
            .iter()
            .map(|sample| f64::from(sample.abs()))
            .fold(0.0_f64, f64::max)
    };
    peak(secondary_start..secondary_end) / peak(primary_start..primary_end).max(1.0e-12)
}
