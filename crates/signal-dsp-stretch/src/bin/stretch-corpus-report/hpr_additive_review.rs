use signal_dsp_stretch::{
    StretchFormantBoundaryMeasurement, StretchHprAdditiveRender, StretchRenderIntegrityMeasurement,
    StretchTonalTextureMeasurement, StretchTransientDetailMeasurement, StretchTransientEventDetail,
};

use super::quoted_report_field;

pub(super) struct HprAdditiveReviewEvidence<'a> {
    pub case_id: &'a str,
    pub source_path: &'a str,
    pub ratio: f64,
    pub render: &'a StretchHprAdditiveRender,
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
}

impl HprAdditiveReviewEvidence<'_> {
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
                        self.render.separation.short_window_frames,
                    ),
                    post_attack_secondary_peak_ratio(
                        &self.render.samples,
                        candidate.output_frame,
                        self.render.separation.short_window_frames,
                    ),
                )
            })
            .unwrap_or((f64::NAN, f64::NAN));

        format!(
            "external_benchmark_hpr_additive_review case={} source={} ratio={:.6} component_time_map=global-fixed harmonic_processor=long-identity-locked-pv residual_processor=current-2048-512-pv percussive_processor=short-normalized-ola target_frames={} harmonic_frames={} residual_frames={} percussive_frames={} component_lengths_match={} percussive_positions={} percussive_positions_monotonic={} percussive_uncovered_output_frames={} hidden_component_gain_applied={} harmonic_source_energy_share={:.9} residual_source_energy_share={:.9} percussive_source_energy_share={:.9} harmonic_peak_growth_db={:.6} residual_peak_growth_db={:.6} percussive_peak_growth_db={:.6} recombination_peak_growth_db={:.6} anchor_input_frame={} current_anchor_crest_growth_db={:.6} candidate_anchor_crest_growth_db={:.6} anchor_crest_improvement_db={:.6} current_max_crest_growth_db={:.6} candidate_max_crest_growth_db={:.6} current_mean_absolute_offset_frames={:.6} candidate_mean_absolute_offset_frames={:.6} current_post_attack_secondary_peak_ratio={:.6} candidate_post_attack_secondary_peak_ratio={:.6} post_attack_secondary_peak_ratio_delta={:.6} current_tonal_residual_ratio={:.6} candidate_tonal_residual_ratio={:.6} current_tonal_sideband_ratio={:.6} candidate_tonal_sideband_ratio={:.6} current_spectral_modulation_delta={:.6} candidate_spectral_modulation_delta={:.6} current_formant_residual_ratio={:.6} candidate_formant_residual_ratio={:.6} current_formant_centroid_shift_hz={:.6} candidate_formant_centroid_shift_hz={:.6} current_max_boundary_step_dbfs={:.6} candidate_max_boundary_step_dbfs={:.6} candidate_endpoint_energy_delta_db={:.6} candidate_added_silence_frames={} candidate_integrity_passed={} candidate_all_samples_finite={}",
            self.case_id,
            quoted_report_field(self.source_path),
            self.ratio,
            self.render.samples.len(),
            self.render.harmonic.len(),
            self.render.residual.len(),
            self.render.percussive.len(),
            self.render.component_lengths_match,
            self.render.percussive_synthesis_positions.len(),
            self.render.percussive_positions_monotonic,
            self.render.percussive_uncovered_output_frames,
            self.render.hidden_component_gain_applied,
            self.render.separation.harmonic.energy_share,
            self.render.separation.residual.energy_share,
            self.render.separation.percussive.energy_share,
            self.render.harmonic_peak_growth_db,
            self.render.residual_peak_growth_db,
            self.render.percussive_peak_growth_db,
            self.render.recombination_peak_growth_db,
            anchor_input_frame,
            current_anchor_crest_db,
            candidate_anchor_crest_db,
            current_anchor_crest_db - candidate_anchor_crest_db,
            self.current_transient.max_transient_crest_growth_db,
            self.candidate_transient.max_transient_crest_growth_db,
            self.current_transient.mean_absolute_timing_offset_frames,
            self.candidate_transient.mean_absolute_timing_offset_frames,
            current_replica_ratio,
            candidate_replica_ratio,
            candidate_replica_ratio - current_replica_ratio,
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
            self.candidate_integrity.endpoint_energy_delta_db,
            self.candidate_integrity.added_silence_frames,
            self.candidate_integrity_passed,
            self.render.samples.iter().all(|sample| sample.is_finite()),
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
    let primary = peak(primary_start..primary_end);
    let secondary = peak(secondary_start..secondary_end);
    secondary / primary.max(1.0e-12)
}

#[cfg(test)]
mod tests {
    use super::post_attack_secondary_peak_ratio;

    #[test]
    fn post_attack_ratio_detects_a_secondary_replica() {
        let mut samples = vec![0.0; 1024];
        samples[256] = 1.0;
        samples[512] = 0.4;

        assert!((post_attack_secondary_peak_ratio(&samples, 256, 512) - 0.4).abs() < 1.0e-6);
    }
}
