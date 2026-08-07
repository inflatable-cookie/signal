use std::time::Instant;

use signal_dsp_stretch::{
    assess_stretch_render_integrity, measure_formant_boundary, measure_stretch_render_integrity,
    measure_tonal_texture, measure_transient_detail, output_length_drift_samples,
    OfflineHighQualityPath, OfflineHighQualityStretcher, StretchCorpusListeningSource,
    StretchRenderIntegrityLimits, TimeStretcher,
};

use crate::alloc_tracker::measure_peak_live_heap;
use crate::external::{
    decode_external_benchmark_render_audio, source_for_external_quality_render,
    ExternalBenchmarkQualityRender, ExternalBenchmarkQualitySource,
};
use crate::listening::decode_listening_source_audio;

const QUALITY_WINDOW_SIZE: usize = 1_024;
const QUALITY_HOP_SIZE: usize = 256;
const INTEGRITY_ENDPOINT_FRAMES: usize = 1_024;
const INTEGRITY_SILENCE_THRESHOLD: f32 = 1.0e-6;
const ALIGNMENT_MAX_LAG_FRAMES: isize = 2_048;
const ALIGNMENT_MAX_COMPARE_FRAMES: usize = 65_536;

pub(super) fn format_external_benchmark_quality_metrics(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    signal_path: OfflineHighQualityPath,
) -> Result<String, String> {
    let limits = StretchRenderIntegrityLimits::offline_high_quality();
    let mut lines = Vec::new();
    for render in renders {
        let source = match source_for_external_quality_render(sources, render) {
            ExternalBenchmarkQualitySource::Found(source) => source,
            ExternalBenchmarkQualitySource::Missing => {
                lines.push(format_quality_skip(
                    render,
                    signal_path,
                    "MissingListeningSource",
                ));
                continue;
            }
            ExternalBenchmarkQualitySource::Ambiguous => {
                lines.push(format_quality_skip(
                    render,
                    signal_path,
                    "AmbiguousListeningSource",
                ));
                continue;
            }
        };
        let source_audio = decode_listening_source_audio(source.as_ref(), frame_limit)?;
        let external = decode_external_benchmark_render_audio(render)?;
        if source_audio.sample_rate_hz != external.sample_rate_hz {
            lines.push(format_quality_skip(
                render,
                signal_path,
                "SampleRateMismatch",
            ));
            continue;
        }
        let source_mono = source_audio.mono_samples();
        let ((signal, render_seconds), heap) = measure_peak_live_heap(|| {
            let started = Instant::now();
            let output = OfflineHighQualityStretcher::with_path(render.ratio, signal_path)
                .stretch_mono(&source_mono)
                .expect("render fits the offline output bound");
            (output, started.elapsed().as_secs_f64())
        });
        let signal_transient = measure_transient_detail(
            &source_mono,
            &signal,
            render.ratio,
            QUALITY_WINDOW_SIZE,
            QUALITY_HOP_SIZE,
        );
        let external_transient = measure_transient_detail(
            &source_mono,
            &external.mono_samples,
            render.ratio,
            QUALITY_WINDOW_SIZE,
            QUALITY_HOP_SIZE,
        );
        let signal_tonal = measure_tonal_texture(&source_mono, &signal, render.ratio);
        let external_tonal =
            measure_tonal_texture(&source_mono, &external.mono_samples, render.ratio);
        let signal_formant = measure_formant_boundary(
            &source_mono,
            &signal,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let external_formant = measure_formant_boundary(
            &source_mono,
            &external.mono_samples,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let signal_integrity = measure_stretch_render_integrity(
            &source_mono,
            &signal,
            render.ratio,
            INTEGRITY_ENDPOINT_FRAMES,
            INTEGRITY_SILENCE_THRESHOLD,
        );
        let external_integrity = measure_stretch_render_integrity(
            &source_mono,
            &external.mono_samples,
            render.ratio,
            INTEGRITY_ENDPOINT_FRAMES,
            INTEGRITY_SILENCE_THRESHOLD,
        );
        let aligned = align_and_measure_error(&signal, &external.mono_samples);
        let rendered_seconds = signal.len() as f64 / source_audio.sample_rate_hz as f64;
        lines.push(format!(
            "external_benchmark_quality case={} source={} signal_path={:?} render={} tool={} ratio={:.6} status=Measured source_sample_rate_hz={} external_channels={} source_frames={} signal_frames={} external_frames={} signal_timing_drift_samples={:.6} external_timing_drift_samples={:.6} alignment_lag_frames={} aligned_frames={} aligned_correlation={:.6} aligned_rms_error={:.6} aligned_rms_error_ratio={:.6} signal_transient_matches={} external_transient_matches={} signal_transient_mean_absolute_offset_frames={:.6} external_transient_mean_absolute_offset_frames={:.6} signal_transient_max_crest_growth_db={:.6} external_transient_max_crest_growth_db={:.6} signal_tonal_residual_ratio={:.6} external_tonal_residual_ratio={:.6} signal_added_sideband_ratio={:.6} external_added_sideband_ratio={:.6} signal_formant_residual_ratio={:.6} external_formant_residual_ratio={:.6} signal_formant_centroid_shift_hz={:.6} external_formant_centroid_shift_hz={:.6} signal_boundary_step_growth_db={:.6} external_boundary_step_growth_db={:.6} signal_integrity_passed={} external_integrity_passed={} signal_endpoint_energy_delta_db={:.6} external_endpoint_energy_delta_db={:.6} signal_added_silence_frames={} external_added_silence_frames={} signal_peak_growth_db={:.6} external_peak_growth_db={:.6} signal_render_seconds={:.6} signal_cpu_realtime_factor={:.6} signal_peak_working_memory_bytes={}",
            render.case_id,
            quoted_report_field(&source_audio.source_path),
            signal_path,
            quoted_report_field(&render.rendered_path),
            quoted_report_field(&render.tool_name),
            render.ratio,
            source_audio.sample_rate_hz,
            external.channels,
            source_audio.analyzed_frames(),
            signal.len(),
            external.frames(),
            output_length_drift_samples(source_mono.len(), signal.len(), render.ratio),
            output_length_drift_samples(source_mono.len(), external.frames(), render.ratio),
            aligned.lag_frames,
            aligned.compared_frames,
            aligned.correlation,
            aligned.rms_error,
            finite_ratio(aligned.rms_error, aligned.external_rms),
            signal_transient.matched_transients,
            external_transient.matched_transients,
            signal_transient.mean_absolute_timing_offset_frames,
            external_transient.mean_absolute_timing_offset_frames,
            signal_transient.max_transient_crest_growth_db,
            external_transient.max_transient_crest_growth_db,
            signal_tonal.mean_spectral_residual_ratio,
            external_tonal.mean_spectral_residual_ratio,
            signal_tonal.mean_added_sideband_ratio,
            external_tonal.mean_added_sideband_ratio,
            signal_formant.mean_envelope_residual_ratio,
            external_formant.mean_envelope_residual_ratio,
            signal_formant.mean_envelope_centroid_shift_hz,
            external_formant.mean_envelope_centroid_shift_hz,
            signal_formant.max_boundary_step_crest_growth_db,
            external_formant.max_boundary_step_crest_growth_db,
            assess_stretch_render_integrity(signal_integrity, limits).passed,
            assess_stretch_render_integrity(external_integrity, limits).passed,
            signal_integrity.endpoint_energy_delta_db,
            external_integrity.endpoint_energy_delta_db,
            signal_integrity.added_silence_frames,
            external_integrity.added_silence_frames,
            signal_integrity.peak_growth_db,
            external_integrity.peak_growth_db,
            render_seconds,
            finite_ratio(render_seconds, rendered_seconds),
            heap.peak_growth_bytes,
        ));
    }
    Ok(lines.join("\n"))
}

fn format_quality_skip(
    render: &ExternalBenchmarkQualityRender,
    signal_path: OfflineHighQualityPath,
    reason: &str,
) -> String {
    format!(
        "external_benchmark_quality case={} signal_path={:?} render={} tool={} ratio={:.6} status=Skipped reason={reason}",
        render.case_id,
        signal_path,
        quoted_report_field(&render.rendered_path),
        quoted_report_field(&render.tool_name),
        render.ratio,
    )
}

#[derive(Clone, Debug, PartialEq)]
struct AlignedErrorMeasurement {
    lag_frames: isize,
    compared_frames: usize,
    correlation: f64,
    rms_error: f64,
    external_rms: f64,
}

fn align_and_measure_error(signal: &[f32], external: &[f32]) -> AlignedErrorMeasurement {
    let mut best_lag = 0isize;
    let mut best_correlation = f64::NEG_INFINITY;
    for lag in -ALIGNMENT_MAX_LAG_FRAMES..=ALIGNMENT_MAX_LAG_FRAMES {
        let Some((signal_start, external_start, frames)) = aligned_ranges(signal, external, lag)
        else {
            continue;
        };
        let correlation = normalized_correlation(
            &signal[signal_start..signal_start + frames],
            &external[external_start..external_start + frames],
        );
        if correlation > best_correlation + 1.0e-12
            || ((correlation - best_correlation).abs() <= 1.0e-12 && lag.abs() < best_lag.abs())
        {
            best_lag = lag;
            best_correlation = correlation;
        }
    }
    let Some((signal_start, external_start, frames)) = aligned_ranges(signal, external, best_lag)
    else {
        return AlignedErrorMeasurement {
            lag_frames: 0,
            compared_frames: 0,
            correlation: f64::NAN,
            rms_error: f64::NAN,
            external_rms: f64::NAN,
        };
    };
    let mut error_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    for (signal_sample, external_sample) in signal[signal_start..signal_start + frames]
        .iter()
        .zip(&external[external_start..external_start + frames])
    {
        let error = *signal_sample as f64 - *external_sample as f64;
        error_square_sum += error * error;
        external_square_sum += (*external_sample as f64) * (*external_sample as f64);
    }
    AlignedErrorMeasurement {
        lag_frames: best_lag,
        compared_frames: frames,
        correlation: best_correlation,
        rms_error: (error_square_sum / frames as f64).sqrt(),
        external_rms: (external_square_sum / frames as f64).sqrt(),
    }
}

fn aligned_ranges(signal: &[f32], external: &[f32], lag: isize) -> Option<(usize, usize, usize)> {
    let signal_start = if lag < 0 { (-lag) as usize } else { 0 };
    let external_start = if lag > 0 { lag as usize } else { 0 };
    if signal_start >= signal.len() || external_start >= external.len() {
        return None;
    }
    let frames = (signal.len() - signal_start)
        .min(external.len() - external_start)
        .min(ALIGNMENT_MAX_COMPARE_FRAMES);
    (frames > 0).then_some((signal_start, external_start, frames))
}

fn normalized_correlation(signal: &[f32], external: &[f32]) -> f64 {
    let mut dot = 0.0;
    let mut signal_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    for (signal_sample, external_sample) in signal.iter().zip(external) {
        dot += *signal_sample as f64 * *external_sample as f64;
        signal_square_sum += (*signal_sample as f64) * (*signal_sample as f64);
        external_square_sum += (*external_sample as f64) * (*external_sample as f64);
    }
    finite_ratio(dot, (signal_square_sum * external_square_sum).sqrt())
}

fn finite_ratio(numerator: f64, denominator: f64) -> f64 {
    if numerator.is_finite() && denominator.is_finite() && denominator.abs() > 1.0e-12 {
        numerator / denominator
    } else {
        f64::NAN
    }
}

fn quoted_report_field(value: &str) -> String {
    format!("{:?}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_recovers_known_lag() {
        let mut signal = vec![0.0; 128];
        let mut external = vec![0.0; 128];
        for index in 16..96 {
            signal[index] = (index as f32 * 0.17).sin();
            external[index + 5] = signal[index];
        }
        let aligned = align_and_measure_error(&signal, &external);
        assert_eq!(aligned.lag_frames, 5);
        assert!(aligned.correlation > 0.999);
    }
}
