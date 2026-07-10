use signal_dsp_stretch::{
    StretchFormantBoundaryMeasurement, StretchRenderIntegrityMeasurement,
    StretchTonalTextureMeasurement, StretchTransientDetailMeasurement,
};

use super::quoted_report_field;

const LOUD_BOUNDARY_DBFS: f64 = -20.0;
const MATERIAL_BOUNDARY_IMPROVEMENT_DB: f64 = 3.0;
const TRANSIENT_OFFSET_TOLERANCE_FRAMES: f64 = 0.25;
const TRANSIENT_CREST_TOLERANCE_DB: f64 = 0.1;
const TONAL_RATIO_TOLERANCE: f64 = 0.001;
const FORMANT_RESIDUAL_TOLERANCE: f64 = 0.001;
const FORMANT_CENTROID_TOLERANCE_HZ: f64 = 2.0;

pub(super) struct TailAnchorReviewEvidence<'a> {
    pub control_id: &'static str,
    pub case_id: &'a str,
    pub source_path: &'a str,
    pub ratio: f64,
    pub current_output: &'a [f32],
    pub candidate_output: &'a [f32],
    pub current_boundary: StretchFormantBoundaryMeasurement,
    pub candidate_boundary: StretchFormantBoundaryMeasurement,
    pub current_tonal: StretchTonalTextureMeasurement,
    pub candidate_tonal: StretchTonalTextureMeasurement,
    pub current_formant: StretchFormantBoundaryMeasurement,
    pub candidate_formant: StretchFormantBoundaryMeasurement,
    pub current_transient: StretchTransientDetailMeasurement,
    pub candidate_transient: StretchTransientDetailMeasurement,
    pub candidate_integrity: StretchRenderIntegrityMeasurement,
    pub candidate_integrity_passed: bool,
}

impl TailAnchorReviewEvidence<'_> {
    pub fn format_report_line(&self) -> String {
        let changed_frames = self
            .current_output
            .iter()
            .zip(self.candidate_output)
            .filter(|(current, candidate)| current != candidate)
            .count();
        let peak_correction = self
            .current_output
            .iter()
            .zip(self.candidate_output)
            .map(|(current, candidate)| (candidate - current).abs() as f64)
            .fold(0.0, f64::max);
        let boundary_improvement_db = self.current_boundary.max_boundary_step_dbfs
            - self.candidate_boundary.max_boundary_step_dbfs;
        let loud_boundary_target =
            self.current_boundary.max_boundary_step_dbfs > LOUD_BOUNDARY_DBFS;
        let material_boundary_improvement =
            !loud_boundary_target || boundary_improvement_db >= MATERIAL_BOUNDARY_IMPROVEMENT_DB;
        let transient_regression_free = no_regression(
            self.candidate_transient.mean_absolute_timing_offset_frames,
            self.current_transient.mean_absolute_timing_offset_frames,
            TRANSIENT_OFFSET_TOLERANCE_FRAMES,
        ) && no_regression(
            self.candidate_transient.max_transient_crest_growth_db,
            self.current_transient.max_transient_crest_growth_db,
            TRANSIENT_CREST_TOLERANCE_DB,
        );
        let tonal_regression_free = no_regression(
            self.candidate_tonal.mean_spectral_residual_ratio,
            self.current_tonal.mean_spectral_residual_ratio,
            TONAL_RATIO_TOLERANCE,
        ) && no_regression(
            self.candidate_tonal.mean_added_sideband_ratio,
            self.current_tonal.mean_added_sideband_ratio,
            TONAL_RATIO_TOLERANCE,
        ) && no_regression(
            self.candidate_tonal.spectral_modulation_delta,
            self.current_tonal.spectral_modulation_delta,
            TONAL_RATIO_TOLERANCE,
        );
        let formant_regression_free = no_regression(
            self.candidate_formant.mean_envelope_residual_ratio,
            self.current_formant.mean_envelope_residual_ratio,
            FORMANT_RESIDUAL_TOLERANCE,
        ) && no_regression(
            self.candidate_formant.mean_envelope_centroid_shift_hz,
            self.current_formant.mean_envelope_centroid_shift_hz,
            FORMANT_CENTROID_TOLERANCE_HZ,
        );
        let combined_regression_gate_passed = self.candidate_integrity_passed
            && transient_regression_free
            && tonal_regression_free
            && formant_regression_free;

        format!(
            "external_benchmark_tail_anchor_review control={} case={} source={} ratio={:.6} changed_frames={} peak_correction={:.9} current_max_boundary_step_dbfs={:.6} candidate_max_boundary_step_dbfs={:.6} boundary_improvement_db={:.6} loud_boundary_target={} material_boundary_improvement={} candidate_integrity_passed={} candidate_endpoint_energy_delta_db={:.6} candidate_added_silence_frames={} candidate_peak_growth_db={:.6} current_transient_mean_absolute_offset_frames={:.6} candidate_transient_mean_absolute_offset_frames={:.6} current_transient_max_crest_growth_db={:.6} candidate_transient_max_crest_growth_db={:.6} transient_regression_free={} current_tonal_residual_ratio={:.6} candidate_tonal_residual_ratio={:.6} current_tonal_sideband_ratio={:.6} candidate_tonal_sideband_ratio={:.6} current_spectral_modulation_delta={:.6} candidate_spectral_modulation_delta={:.6} tonal_regression_free={} current_formant_envelope_residual_ratio={:.6} candidate_formant_envelope_residual_ratio={:.6} current_formant_centroid_shift_hz={:.6} candidate_formant_centroid_shift_hz={:.6} formant_regression_free={} combined_regression_gate_passed={}",
            self.control_id,
            self.case_id,
            quoted_report_field(self.source_path),
            self.ratio,
            changed_frames,
            peak_correction,
            self.current_boundary.max_boundary_step_dbfs,
            self.candidate_boundary.max_boundary_step_dbfs,
            boundary_improvement_db,
            loud_boundary_target,
            material_boundary_improvement,
            self.candidate_integrity_passed,
            self.candidate_integrity.endpoint_energy_delta_db,
            self.candidate_integrity.added_silence_frames,
            self.candidate_integrity.peak_growth_db,
            self.current_transient.mean_absolute_timing_offset_frames,
            self.candidate_transient
                .mean_absolute_timing_offset_frames,
            self.current_transient.max_transient_crest_growth_db,
            self.candidate_transient.max_transient_crest_growth_db,
            transient_regression_free,
            self.current_tonal.mean_spectral_residual_ratio,
            self.candidate_tonal.mean_spectral_residual_ratio,
            self.current_tonal.mean_added_sideband_ratio,
            self.candidate_tonal.mean_added_sideband_ratio,
            self.current_tonal.spectral_modulation_delta,
            self.candidate_tonal.spectral_modulation_delta,
            tonal_regression_free,
            self.current_formant.mean_envelope_residual_ratio,
            self.candidate_formant.mean_envelope_residual_ratio,
            self.current_formant.mean_envelope_centroid_shift_hz,
            self.candidate_formant.mean_envelope_centroid_shift_hz,
            formant_regression_free,
            combined_regression_gate_passed,
        )
    }
}

fn no_regression(candidate: f64, current: f64, tolerance: f64) -> bool {
    (candidate.is_finite() && current.is_finite() && candidate <= current + tolerance)
        || (candidate.is_nan() && current.is_nan())
}
