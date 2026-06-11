use std::env;
use std::error::Error;
use std::path::Path;

use hound::{SampleFormat, WavReader};
use signal_analysis::AnalysisStage;
use signal_analysis_rhythm::{
    BeatGridErrorDiagnostics, BeatTracker, BeatTrackerConfig, LocalTempoPoint,
    TempoTrendDiagnostics,
};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).ok_or(
        "usage: cargo run -p signal-analysis-rhythm --example file_rhythm_probe -- <path-to-wav>",
    )?;
    let audio = read_wav_mono(&path)?;
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    println!("file={}", path);
    println!("bpm={:.5}", result.bpm);
    println!("confidence={:.3}", result.confidence.0);
    println!(
        "tempo_interpretation={:?}/{:?}/recommended:{:.5}/snapped:{:?}",
        result.tempo_interpretation.recommendation,
        result.tempo_interpretation.reason,
        result.tempo_interpretation.recommended_bpm,
        result.tempo_interpretation.snapped_bpm
    );
    println!(
        "tempo_support=core:{:.3}/drift:{:.3}/grid:{:.3}/integer:{:.3}/boundary:{:.3}",
        result.tempo_interpretation.support.core_consensus.0,
        result.tempo_interpretation.support.drift_stability.0,
        result.tempo_interpretation.support.grid_stability.0,
        result.tempo_interpretation.support.integer_closeness.0,
        result.tempo_interpretation.support.boundary_pressure.0
    );
    println!(
        "tempo_profile=refined:{:.5}/core:{:.5}/nearest_integer:{:.2}/snap_error:{:.5}/stability:{:.3}/boundary_gap_ms:{:.3}",
        result.tempo_interpretation.profile.refined_bpm,
        result.tempo_interpretation.profile.core_window_bpm,
        result.tempo_interpretation.profile.nearest_integer_bpm,
        result.tempo_interpretation.profile.snap_error_bpm,
        result.tempo_interpretation.profile.stability_score.0,
        result.tempo_interpretation.profile.boundary_edge_gap_ms
    );
    println!("tempo_candidates={}", result.tempo_candidates.len());
    for candidate in result.tempo_candidates.iter().take(6) {
        println!(
            "tempo_candidate=bpm:{:.5}/confidence:{:.3}",
            candidate.bpm, candidate.confidence.0
        );
    }
    println!("tempo_ambiguity={:.3}", result.tempo_ambiguity.0);
    println!("beats={}", result.beat_positions_seconds.len());
    print_tempo_diagnostics(&result.tempo_diagnostics);
    print_interval_sample("interval", &result.tempo_diagnostics.interval_tempi);
    print_interval_sample("windowed", &result.tempo_diagnostics.windowed_tempi);

    Ok(())
}

fn print_tempo_diagnostics(diagnostics: &signal_analysis_rhythm::TempoDiagnostics) {
    let trend: &TempoTrendDiagnostics = &diagnostics.trend;
    let grid: &BeatGridErrorDiagnostics = &diagnostics.beat_grid_error;
    println!(
        "trend={:?}/start:{:.5}/end:{:.5}/drift:{:.5}/slope:{:.6}/fit_mad:{:.5}/boundary_bias:{:.5}/windowed_mad:{:.5}/core_windowed_mad:{:.5}",
        trend.direction,
        trend.start_bpm,
        trend.end_bpm,
        trend.total_drift_bpm,
        trend.slope_bpm_per_beat,
        trend.fit_mean_abs_deviation_bpm,
        diagnostics.boundary_bias_bpm,
        diagnostics.windowed_mean_abs_deviation_bpm,
        diagnostics.core_windowed_mean_abs_deviation_bpm
    );
    println!(
        "grid=mean_abs_residual_ms:{:.3}/max_abs_residual_ms:{:.3}/edge_mean_abs_residual_ms:{:.3}/core_mean_abs_residual_ms:{:.3}/end_anchored_drift_ms:{:.3}/mean_abs_anchored_drift_ms:{:.3}",
        grid.mean_abs_residual_ms,
        grid.max_abs_residual_ms,
        grid.edge_mean_abs_residual_ms,
        grid.core_mean_abs_residual_ms,
        grid.end_anchored_drift_ms,
        grid.mean_abs_anchored_drift_ms
    );
    println!(
        "interval_outliers=total:{}/retained:{}/rejected:{}/leading:{}/trailing:{}/median:{:.6}/mad:{:.6}/max_ratio:{:.3}",
        diagnostics.beat_interval_outliers.total_intervals,
        diagnostics.beat_interval_outliers.retained_intervals,
        diagnostics.beat_interval_outliers.rejected_intervals,
        diagnostics.beat_interval_outliers.leading_rejected_intervals,
        diagnostics.beat_interval_outliers.trailing_rejected_intervals,
        diagnostics.beat_interval_outliers.median_interval,
        diagnostics.beat_interval_outliers.median_abs_deviation,
        diagnostics.beat_interval_outliers.max_rejected_deviation_ratio
    );
    println!(
        "stability_scope={:?}/edge_trimmed:{:.3}/contiguous:{:.3}/interior:{:.3}/edge_locality:{:.3}",
        diagnostics.stability_scope.scope,
        diagnostics.stability_scope.support.edge_trimmed_coverage.0,
        diagnostics.stability_scope.support.contiguous_core_coverage.0,
        diagnostics.stability_scope.support.interior_stability.0,
        diagnostics.stability_scope.support.edge_locality.0
    );
    if let Some(span) = diagnostics.edge_trimmed_stable_span {
        println!(
            "edge_trimmed_stable_span=beats:{}..{}/seconds:{:.3}..{:.3}/coverage:{:.3}/windows:{}/{} trim:{}:{} interior:{}",
            span.start_beat_index,
            span.end_beat_index,
            span.start_seconds,
            span.end_seconds,
            span.coverage.0,
            span.retained_windows,
            span.total_windows,
            span.trimmed_leading_windows,
            span.trimmed_trailing_windows,
            span.interior_rejected_windows
        );
    } else {
        println!("edge_trimmed_stable_span=none");
    }
    if let Some(span) = diagnostics.stable_core_span {
        println!(
            "stable_core_span=beats:{}..{}/seconds:{:.3}..{:.3}/coverage:{:.3}/windows:{}/{} trim:{}:{} interior:{}",
            span.start_beat_index,
            span.end_beat_index,
            span.start_seconds,
            span.end_seconds,
            span.coverage.0,
            span.retained_windows,
            span.total_windows,
            span.trimmed_leading_windows,
            span.trimmed_trailing_windows,
            span.interior_rejected_windows
        );
    } else {
        println!("stable_core_span=none");
    }
}

fn print_interval_sample(label: &str, points: &[LocalTempoPoint]) {
    println!("{}_count={}", label, points.len());
    for point in points.iter().take(12) {
        println!(
            "{}={}..{} {:.5}s..{:.5}s bpm:{:.5}",
            label,
            point.start_beat_index,
            point.end_beat_index,
            point.start_seconds,
            point.end_seconds,
            point.bpm
        );
    }
    for point in points
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        println!(
            "{}_tail={}..{} {:.5}s..{:.5}s bpm:{:.5}",
            label,
            point.start_beat_index,
            point.end_beat_index,
            point.start_seconds,
            point.end_seconds,
            point.bpm
        );
    }
}

fn read_wav_mono(path: &str) -> Result<AudioBuffer, Box<dyn Error>> {
    let mut reader = WavReader::open(Path::new(path))?;
    let spec = reader.spec();
    let channel_count = spec.channels as usize;
    if channel_count == 0 {
        return Err("wav has zero channels".into());
    }

    let interleaved = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        (SampleFormat::Int, bits) if bits <= 16 => {
            let scale = i16::MAX as f32;
            reader
                .samples::<i16>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()?
        }
        (SampleFormat::Int, bits) if bits <= 24 => {
            let scale = ((1_i64 << (bits - 1)) - 1) as f32;
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()?
        }
        (SampleFormat::Int, bits) if bits <= 32 => {
            let scale = i32::MAX as f32;
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()?
        }
        _ => {
            return Err(format!(
                "unsupported wav format: {:?} {} bits",
                spec.sample_format, spec.bits_per_sample
            )
            .into())
        }
    };

    let mono = if channel_count == 1 {
        interleaved
    } else {
        interleaved
            .chunks_exact(channel_count)
            .map(|frame| frame.iter().copied().sum::<f32>() / channel_count as f32)
            .collect()
    };

    Ok(AudioBuffer::from_interleaved(
        SampleRate(spec.sample_rate),
        ChannelLayout::Mono,
        mono,
    ))
}
