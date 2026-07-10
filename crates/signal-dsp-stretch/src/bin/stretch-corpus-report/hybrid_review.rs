use signal_dsp_stretch::{
    StretchFormantBoundaryMeasurement, StretchHybridRender, StretchHybridTransitionRejection,
    StretchRenderIntegrityMeasurement, StretchTonalTextureMeasurement,
    StretchTransientDetailMeasurement, StretchTransientEventDetail,
};

use super::quoted_report_field;

pub(super) struct HybridReviewEvidence<'a> {
    pub case_id: &'a str,
    pub source_path: &'a str,
    pub ratio: f64,
    pub render: &'a StretchHybridRender,
    pub current_tonal: StretchTonalTextureMeasurement,
    pub candidate_tonal: StretchTonalTextureMeasurement,
    pub current_formant: StretchFormantBoundaryMeasurement,
    pub candidate_formant: StretchFormantBoundaryMeasurement,
    pub current_transient: StretchTransientDetailMeasurement,
    pub candidate_transient: StretchTransientDetailMeasurement,
    pub anchor_events: Option<(StretchTransientEventDetail, StretchTransientEventDetail)>,
    pub candidate_integrity: StretchRenderIntegrityMeasurement,
    pub candidate_integrity_passed: bool,
}

impl HybridReviewEvidence<'_> {
    pub fn format_report_line(&self) -> String {
        let applied_decisions = self
            .render
            .transition_decisions
            .iter()
            .filter(|decision| decision.applied)
            .count();
        let min_applied_correlation = self
            .render
            .transition_decisions
            .iter()
            .filter(|decision| decision.applied)
            .map(|decision| decision.correlation)
            .reduce(f64::min)
            .unwrap_or(f64::NAN);
        let max_applied_normalization_db = self
            .render
            .transition_decisions
            .iter()
            .filter(|decision| decision.applied)
            .map(|decision| decision.max_normalization_gain_db)
            .reduce(f64::max)
            .unwrap_or(f64::NAN);
        let rejection_count = |reason| {
            self.render
                .transition_decisions
                .iter()
                .filter(|decision| decision.rejection == Some(reason))
                .count()
        };
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

        format!(
            "external_benchmark_structural_hybrid_review case={} source={} ratio={:.6} applied_spans={} rejected_spans={} applied_decisions={} low_correlation_decisions={} excess_normalization_decisions={} span_too_short_decisions={} min_applied_correlation={:.6} max_applied_normalization_db={:.6} anchor_input_frame={} current_anchor_crest_growth_db={:.6} candidate_anchor_crest_growth_db={:.6} anchor_crest_improvement_db={:.6} current_max_crest_growth_db={:.6} candidate_max_crest_growth_db={:.6} current_mean_absolute_offset_frames={:.6} candidate_mean_absolute_offset_frames={:.6} current_tonal_residual_ratio={:.6} candidate_tonal_residual_ratio={:.6} current_tonal_sideband_ratio={:.6} candidate_tonal_sideband_ratio={:.6} current_spectral_modulation_delta={:.6} candidate_spectral_modulation_delta={:.6} current_formant_residual_ratio={:.6} candidate_formant_residual_ratio={:.6} current_formant_centroid_shift_hz={:.6} candidate_formant_centroid_shift_hz={:.6} candidate_integrity_passed={} candidate_endpoint_energy_delta_db={:.6} candidate_added_silence_frames={} candidate_peak_growth_db={:.6}",
            self.case_id,
            quoted_report_field(self.source_path),
            self.ratio,
            self.render.applied_span_count,
            self.render.rejected_span_count,
            applied_decisions,
            rejection_count(StretchHybridTransitionRejection::LowCorrelation),
            rejection_count(StretchHybridTransitionRejection::ExcessNormalization),
            rejection_count(StretchHybridTransitionRejection::SpanTooShort),
            min_applied_correlation,
            max_applied_normalization_db,
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
            self.candidate_integrity_passed,
            self.candidate_integrity.endpoint_energy_delta_db,
            self.candidate_integrity.added_silence_frames,
            self.candidate_integrity.peak_growth_db,
        )
    }
}
