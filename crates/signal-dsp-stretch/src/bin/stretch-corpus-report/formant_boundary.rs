use signal_dsp_stretch::StretchFormantBoundaryMeasurement;

use super::quoted_report_field;

pub(super) fn format_external_benchmark_formant_boundary_line(
    case_id: &str,
    source_path: &str,
    signal: StretchFormantBoundaryMeasurement,
    external: StretchFormantBoundaryMeasurement,
    draft: StretchFormantBoundaryMeasurement,
) -> String {
    format!(
        "external_benchmark_formant_boundary case={} source={} ratio={:.6} sample_rate={} envelope_windows={} signal_mean_envelope_residual_ratio={:.6} external_mean_envelope_residual_ratio={:.6} draft_mean_envelope_residual_ratio={:.6} signal_envelope_residual_delta_vs_external={:.6} signal_max_envelope_residual_ratio={:.6} external_max_envelope_residual_ratio={:.6} signal_mean_envelope_centroid_shift_hz={:.6} external_mean_envelope_centroid_shift_hz={:.6} draft_mean_envelope_centroid_shift_hz={:.6} signal_envelope_centroid_shift_delta_vs_external_hz={:.6} signal_max_envelope_centroid_shift_hz={:.6} external_max_envelope_centroid_shift_hz={:.6} signal_measured_boundary_count={} external_measured_boundary_count={} draft_measured_boundary_count={} signal_head_boundary_step_crest_delta_db={:.6} external_head_boundary_step_crest_delta_db={:.6} draft_head_boundary_step_crest_delta_db={:.6} signal_tail_boundary_step_crest_delta_db={:.6} external_tail_boundary_step_crest_delta_db={:.6} draft_tail_boundary_step_crest_delta_db={:.6} signal_max_boundary_step_crest_growth_db={:.6} external_max_boundary_step_crest_growth_db={:.6} draft_max_boundary_step_crest_growth_db={:.6} signal_boundary_step_crest_growth_delta_vs_external_db={:.6} signal_head_boundary_step_dbfs={:.6} signal_tail_boundary_step_dbfs={:.6} external_head_boundary_step_dbfs={:.6} external_tail_boundary_step_dbfs={:.6} draft_head_boundary_step_dbfs={:.6} draft_tail_boundary_step_dbfs={:.6} signal_max_boundary_step_dbfs={:.6} external_max_boundary_step_dbfs={:.6} draft_max_boundary_step_dbfs={:.6}",
        case_id,
        quoted_report_field(source_path),
        signal.ratio,
        signal.sample_rate_hz,
        signal
            .envelope_windows
            .min(external.envelope_windows)
            .min(draft.envelope_windows),
        signal.mean_envelope_residual_ratio,
        external.mean_envelope_residual_ratio,
        draft.mean_envelope_residual_ratio,
        signal.mean_envelope_residual_ratio - external.mean_envelope_residual_ratio,
        signal.max_envelope_residual_ratio,
        external.max_envelope_residual_ratio,
        signal.mean_envelope_centroid_shift_hz,
        external.mean_envelope_centroid_shift_hz,
        draft.mean_envelope_centroid_shift_hz,
        signal.mean_envelope_centroid_shift_hz - external.mean_envelope_centroid_shift_hz,
        signal.max_envelope_centroid_shift_hz,
        external.max_envelope_centroid_shift_hz,
        signal.measured_boundary_count,
        external.measured_boundary_count,
        draft.measured_boundary_count,
        signal.head_boundary_step_crest_delta_db,
        external.head_boundary_step_crest_delta_db,
        draft.head_boundary_step_crest_delta_db,
        signal.tail_boundary_step_crest_delta_db,
        external.tail_boundary_step_crest_delta_db,
        draft.tail_boundary_step_crest_delta_db,
        signal.max_boundary_step_crest_growth_db,
        external.max_boundary_step_crest_growth_db,
        draft.max_boundary_step_crest_growth_db,
        signal.max_boundary_step_crest_growth_db - external.max_boundary_step_crest_growth_db,
        signal.head_boundary_step_dbfs,
        signal.tail_boundary_step_dbfs,
        external.head_boundary_step_dbfs,
        external.tail_boundary_step_dbfs,
        draft.head_boundary_step_dbfs,
        draft.tail_boundary_step_dbfs,
        signal.max_boundary_step_dbfs,
        external.max_boundary_step_dbfs,
        draft.max_boundary_step_dbfs,
    )
}
