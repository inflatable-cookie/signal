use signal_dsp_stretch::{
    StretchAdaptiveTimelineRender, StretchFormantBoundaryMeasurement,
    StretchTonalTextureMeasurement, StretchTransientDetailMeasurement, StretchTransientEventDetail,
};

use super::quoted_report_field;

pub(super) struct TimelineReviewEvidence<'a> {
    pub case_id: &'a str,
    pub source_path: &'a str,
    pub ratio: f64,
    pub render: &'a StretchAdaptiveTimelineRender,
    pub current_tonal: StretchTonalTextureMeasurement,
    pub candidate_tonal: StretchTonalTextureMeasurement,
    pub current_formant: StretchFormantBoundaryMeasurement,
    pub candidate_formant: StretchFormantBoundaryMeasurement,
    pub current_transient: StretchTransientDetailMeasurement,
    pub candidate_transient: StretchTransientDetailMeasurement,
    pub anchor_events: Option<(StretchTransientEventDetail, StretchTransientEventDetail)>,
    pub candidate_integrity_passed: bool,
}

impl TimelineReviewEvidence<'_> {
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
        let boundary_regression_free = self.candidate_formant.max_boundary_step_dbfs
            <= self.current_formant.max_boundary_step_dbfs + 0.1;

        format!(
            "external_benchmark_adaptive_timeline_review case={} source={} ratio={:.6} protected_onsets={} reinitialized_frames={} dense_conflicts={} schedule_fallback={} min_synthesis_hop_frames={} max_synthesis_hop_frames={} max_anchor_error_frames={:.6} uncovered_output_frames={} anchor_input_frame={} current_anchor_crest_growth_db={:.6} candidate_anchor_crest_growth_db={:.6} anchor_crest_improvement_db={:.6} current_max_crest_growth_db={:.6} candidate_max_crest_growth_db={:.6} current_mean_absolute_offset_frames={:.6} candidate_mean_absolute_offset_frames={:.6} current_tonal_residual_ratio={:.6} candidate_tonal_residual_ratio={:.6} current_tonal_sideband_ratio={:.6} candidate_tonal_sideband_ratio={:.6} current_spectral_modulation_delta={:.6} candidate_spectral_modulation_delta={:.6} current_formant_residual_ratio={:.6} candidate_formant_residual_ratio={:.6} current_formant_centroid_shift_hz={:.6} candidate_formant_centroid_shift_hz={:.6} current_max_boundary_step_dbfs={:.6} candidate_max_boundary_step_dbfs={:.6} boundary_regression_free={} candidate_integrity_passed={}",
            self.case_id,
            quoted_report_field(self.source_path),
            self.ratio,
            self.render.protected_onset_count,
            self.render.reinitialized_frames.len(),
            self.render.dense_conflict_count,
            self.render.schedule_fallback,
            self.render.min_synthesis_hop_frames,
            self.render.max_synthesis_hop_frames,
            self.render.max_anchor_error_frames,
            self.render.uncovered_output_frames,
            anchor_input_frame,
            current_anchor_crest_db,
            candidate_anchor_crest_db,
            current_anchor_crest_db - candidate_anchor_crest_db,
            self.current_transient.max_transient_crest_growth_db,
            self.candidate_transient.max_transient_crest_growth_db,
            self.current_transient.mean_absolute_timing_offset_frames,
            self.candidate_transient.mean_absolute_timing_offset_frames,
            self.current_tonal.mean_spectral_residual_ratio,
            self.candidate_tonal.mean_spectral_residual_ratio,
            self.current_tonal.mean_added_sideband_ratio,
            self.candidate_tonal.mean_added_sideband_ratio,
            self.current_tonal.spectral_modulation_delta,
            self.candidate_tonal.spectral_modulation_delta,
            self.current_formant.mean_envelope_residual_ratio,
            self.candidate_formant.mean_envelope_residual_ratio,
            self.current_formant.mean_envelope_centroid_shift_hz,
            self.candidate_formant.mean_envelope_centroid_shift_hz,
            self.current_formant.max_boundary_step_dbfs,
            self.candidate_formant.max_boundary_step_dbfs,
            boundary_regression_free,
            self.candidate_integrity_passed,
        )
    }
}
