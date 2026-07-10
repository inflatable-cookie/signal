use signal_dsp_stretch::{
    StretchFixedMapPeakTransientRender, StretchFormantBoundaryMeasurement,
    StretchTonalTextureMeasurement, StretchTransientDetailMeasurement, StretchTransientEventDetail,
};

use super::quoted_report_field;

pub(super) struct PeakTransientReviewEvidence<'a> {
    pub case_id: &'a str,
    pub source_path: &'a str,
    pub ratio: f64,
    pub render: &'a StretchFixedMapPeakTransientRender,
    pub current_tonal: StretchTonalTextureMeasurement,
    pub candidate_tonal: StretchTonalTextureMeasurement,
    pub current_formant: StretchFormantBoundaryMeasurement,
    pub candidate_formant: StretchFormantBoundaryMeasurement,
    pub current_transient: StretchTransientDetailMeasurement,
    pub candidate_transient: StretchTransientDetailMeasurement,
    pub anchor_events: Option<(StretchTransientEventDetail, StretchTransientEventDetail)>,
    pub candidate_integrity_passed: bool,
}

impl PeakTransientReviewEvidence<'_> {
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
        let unmatched_events = self
            .render
            .events
            .iter()
            .filter(|event| event.reinitialized_analysis_frame.is_none())
            .count();
        let reinitialized_frames = self
            .render
            .events
            .iter()
            .filter_map(|event| event.reinitialized_analysis_frame)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let collected_regions = self
            .render
            .events
            .iter()
            .map(|event| event.collected_peak_regions)
            .sum::<usize>();
        let reinitialized_bins = self
            .render
            .events
            .iter()
            .map(|event| event.reinitialized_bins)
            .sum::<usize>();
        let boundary_regression_free = self.candidate_formant.max_boundary_step_dbfs
            <= self.current_formant.max_boundary_step_dbfs + 0.1;

        format!(
            "external_benchmark_fixed_map_peak_transient_review case={} source={} ratio={:.6} fixed_global_time_map=true guarded_events={} unmatched_events={} candidate_peaks={} collected_peak_regions={} threshold_crossings={} reinitialized_frames={} reinitialized_bins={} center_threshold_frames={:.6} uncovered_output_frames={} anchor_input_frame={} current_anchor_crest_growth_db={:.6} candidate_anchor_crest_growth_db={:.6} anchor_crest_improvement_db={:.6} current_max_crest_growth_db={:.6} candidate_max_crest_growth_db={:.6} current_mean_absolute_offset_frames={:.6} candidate_mean_absolute_offset_frames={:.6} current_tonal_residual_ratio={:.6} candidate_tonal_residual_ratio={:.6} current_tonal_sideband_ratio={:.6} candidate_tonal_sideband_ratio={:.6} current_spectral_modulation_delta={:.6} candidate_spectral_modulation_delta={:.6} current_formant_residual_ratio={:.6} candidate_formant_residual_ratio={:.6} current_formant_centroid_shift_hz={:.6} candidate_formant_centroid_shift_hz={:.6} current_max_boundary_step_dbfs={:.6} candidate_max_boundary_step_dbfs={:.6} boundary_regression_free={} candidate_integrity_passed={}",
            self.case_id,
            quoted_report_field(self.source_path),
            self.ratio,
            self.render.events.len(),
            unmatched_events,
            self.render.candidate_regions.len(),
            collected_regions,
            self.render.threshold_crossings,
            reinitialized_frames,
            reinitialized_bins,
            self.render.center_threshold_frames,
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
