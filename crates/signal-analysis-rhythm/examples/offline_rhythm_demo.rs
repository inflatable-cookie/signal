use signal_analysis::AnalysisStage;
use signal_analysis_rhythm::{BeatTracker, BeatTrackerConfig};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

// Practical inspection example for the public rhythm result surface.
//
// Run with:
// `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120`
//
// The printed `tempo_interpretation` and meter lines are the intended integration
// hooks for deciding whether to trust the tempo estimate and detected meter, or to
// fall back to beat-only handling.
fn main() {
    let bpm = parse_arg("--bpm").unwrap_or(120.0);
    let seconds = parse_arg("--seconds").unwrap_or(8.0);
    let sample_rate = parse_arg("--sample-rate").unwrap_or(48_000.0) as u32;

    let audio = synthetic_click_track(sample_rate, bpm, seconds);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    println!("requested_bpm={bpm:.2}");
    println!("estimated_bpm={:.2}", result.bpm);
    println!("confidence={:.3}", result.confidence.0);
    println!("tempo_ambiguity={:.3}", result.tempo_ambiguity.0);
    println!("beats={}", result.beat_positions_seconds.len());
    println!(
        "tempo_diagnostics=median:{:.2},mad:{:.3},span:{:.3},windowed_median:{:.2},windowed_mad:{:.3},windowed_span:{:.3},core_windowed_median:{:.2},core_windowed_mad:{:.3},core_windowed_span:{:.3},boundary_bias:{:.3},intervals:{},windowed:{}",
        result.tempo_diagnostics.median_bpm,
        result.tempo_diagnostics.mean_abs_deviation_bpm,
        result.tempo_diagnostics.drift_span_bpm,
        result.tempo_diagnostics.windowed_median_bpm,
        result.tempo_diagnostics.windowed_mean_abs_deviation_bpm,
        result.tempo_diagnostics.windowed_drift_span_bpm,
        result.tempo_diagnostics.core_windowed_median_bpm,
        result.tempo_diagnostics.core_windowed_mean_abs_deviation_bpm,
        result.tempo_diagnostics.core_windowed_drift_span_bpm,
        result.tempo_diagnostics.boundary_bias_bpm,
        result.tempo_diagnostics.interval_tempi.len(),
        result.tempo_diagnostics.windowed_tempi.len(),
    );
    println!(
        "tempo_trend=direction:{:?},start:{:.2},end:{:.2},drift:{:.3},slope_per_beat:{:.4},fit_mad:{:.3}",
        result.tempo_diagnostics.trend.direction,
        result.tempo_diagnostics.trend.start_bpm,
        result.tempo_diagnostics.trend.end_bpm,
        result.tempo_diagnostics.trend.total_drift_bpm,
        result.tempo_diagnostics.trend.slope_bpm_per_beat,
        result.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm,
    );
    println!(
        "beat_grid_error=mean_abs_ms:{:.3},max_abs_ms:{:.3},edge_abs_ms:{:.3},core_abs_ms:{:.3},end_anchored_ms:{:.3},anchored_mad_ms:{:.3}",
        result.tempo_diagnostics.beat_grid_error.mean_abs_residual_ms,
        result.tempo_diagnostics.beat_grid_error.max_abs_residual_ms,
        result.tempo_diagnostics.beat_grid_error.edge_mean_abs_residual_ms,
        result.tempo_diagnostics.beat_grid_error.core_mean_abs_residual_ms,
        result.tempo_diagnostics.beat_grid_error.end_anchored_drift_ms,
        result.tempo_diagnostics.beat_grid_error.mean_abs_anchored_drift_ms,
    );
    println!(
        "tempo_stability_scope={:?}/edge_trimmed:{:.3}/contiguous:{:.3}/interior:{:.3}/edge_locality:{:.3}",
        result.tempo_diagnostics.stability_scope.scope,
        result.tempo_diagnostics.stability_scope.support.edge_trimmed_coverage.0,
        result.tempo_diagnostics.stability_scope.support.contiguous_core_coverage.0,
        result.tempo_diagnostics.stability_scope.support.interior_stability.0,
        result.tempo_diagnostics.stability_scope.support.edge_locality.0,
    );
    if let Some(span) = result.tempo_diagnostics.edge_trimmed_stable_span {
        println!(
            "edge_trimmed_stable_span=beats:{}..{}/seconds:{:.2}..{:.2}/coverage:{:.3}/windows:{}/{} trim:{}:{} interior:{}",
            span.start_beat_index,
            span.end_beat_index,
            span.start_seconds,
            span.end_seconds,
            span.coverage.0,
            span.retained_windows,
            span.total_windows,
            span.trimmed_leading_windows,
            span.trimmed_trailing_windows,
            span.interior_rejected_windows,
        );
    } else {
        println!("edge_trimmed_stable_span=none");
    }
    if let Some(span) = result.tempo_diagnostics.stable_core_span {
        println!(
            "stable_core_span=beats:{}..{}/seconds:{:.2}..{:.2}/coverage:{:.3}/windows:{}/{} trim:{}:{} interior:{}",
            span.start_beat_index,
            span.end_beat_index,
            span.start_seconds,
            span.end_seconds,
            span.coverage.0,
            span.retained_windows,
            span.total_windows,
            span.trimmed_leading_windows,
            span.trimmed_trailing_windows,
            span.interior_rejected_windows,
        );
    } else {
        println!("stable_core_span=none");
    }
    println!(
        "tempo_interpretation=trust:{:?},recommendation:{:?},reason:{:?},recommended:{:.2},snapped:{:?}",
        result.tempo_interpretation.trust,
        result.tempo_interpretation.recommendation,
        result.tempo_interpretation.reason,
        result.tempo_interpretation.recommended_bpm,
        result.tempo_interpretation.snapped_bpm,
    );
    println!(
        "tempo_support=core:{:.3},drift:{:.3},grid:{:.3},integer:{:.3},boundary:{:.3}",
        result.tempo_interpretation.support.core_consensus.0,
        result.tempo_interpretation.support.drift_stability.0,
        result.tempo_interpretation.support.grid_stability.0,
        result.tempo_interpretation.support.integer_closeness.0,
        result.tempo_interpretation.support.boundary_pressure.0,
    );
    println!(
        "tempo_profile=refined:{:.2},core:{:.2},nearest_integer:{:.2},snap_error:{:.3},stability:{:.3},boundary_gap_ms:{:.3}",
        result.tempo_interpretation.profile.refined_bpm,
        result.tempo_interpretation.profile.core_window_bpm,
        result.tempo_interpretation.profile.nearest_integer_bpm,
        result.tempo_interpretation.profile.snap_error_bpm,
        result.tempo_interpretation.profile.stability_score.0,
        result.tempo_interpretation.profile.boundary_edge_gap_ms,
    );
    let candidates: Vec<String> = result
        .tempo_candidates
        .iter()
        .map(|candidate| format!("{:.2}@{:.3}", candidate.bpm, candidate.confidence.0))
        .collect();
    println!("tempo_candidates={candidates:?}");
    if let Some(meter) = &result.meter {
        println!("beats_per_bar={}", meter.beats_per_bar);
        println!("meter_confidence={:.3}", meter.confidence.0);
        println!("meter_detection={:?}", meter.detection_kind);
        println!("meter_trust={:?}", meter.trust);
        println!("meter_recommendation={:?}", meter.recommendation);
        println!(
            "meter_support_profile=whole_track:{:.3},segment_recovery:{:.3},recovery_duration:{:.3}",
            meter.support_profile.whole_track_strength.0,
            meter.support_profile.segment_recovery_strength.0,
            meter.support_profile.recovery_duration_strength.0,
        );
        println!(
            "meter_confidence_breakdown=margin:{:.3},support:{:.3},meter_support:{:.3},regularity:{:.3},recent:{:.3},salience:{:.3}",
            meter.confidence_breakdown.phase_margin,
            meter.confidence_breakdown.support,
            meter.confidence_breakdown.meter_support,
            meter.confidence_breakdown.regularity,
            meter.confidence_breakdown.recent_stability,
            meter.confidence_breakdown.salience,
        );
        if let Some(recovery) = &meter.recovery {
            println!(
                "meter_recovery=start_beat:{},end_beat:{},beats:{},bars:{},start_seconds:{:.3},end_seconds:{:.3},supporting_windows:{}",
                recovery.start_beat_index,
                recovery.end_beat_index,
                recovery.recovered_beats,
                recovery.recovered_bars,
                recovery.start_seconds,
                recovery.end_seconds,
                recovery.supporting_windows,
            );
        } else {
            println!("meter_recovery=none");
        }
        println!(
            "downbeats={:?}",
            &meter.downbeat_positions_seconds[..meter.downbeat_positions_seconds.len().min(6)]
        );
    } else {
        println!("beats_per_bar=unknown");
        println!("meter_confidence=0.000");
        println!("meter_detection=unknown");
        println!("meter_trust=unknown");
        println!("meter_recommendation=unknown");
        println!("meter_support_profile=none");
        println!("meter_confidence_breakdown=none");
        println!("meter_recovery=none");
        println!("downbeats=[]");
    }
    println!(
        "first_beats={:?}",
        &result.beat_positions_seconds[..result.beat_positions_seconds.len().min(8)]
    );
    let first_local_tempo: Vec<String> = result
        .tempo_diagnostics
        .interval_tempi
        .iter()
        .take(6)
        .map(|point| {
            format!(
                "{}-{}:{:.2}",
                point.start_beat_index, point.end_beat_index, point.bpm
            )
        })
        .collect();
    println!("first_local_tempo={first_local_tempo:?}");
    let first_windowed_tempo: Vec<String> = result
        .tempo_diagnostics
        .windowed_tempi
        .iter()
        .take(4)
        .map(|point| {
            format!(
                "{}-{}:{:.2}",
                point.start_beat_index, point.end_beat_index, point.bpm
            )
        })
        .collect();
    println!("first_windowed_tempo={first_windowed_tempo:?}");
    let first_grid_error: Vec<String> = result
        .tempo_diagnostics
        .beat_grid_error
        .residuals
        .iter()
        .take(6)
        .map(|point| {
            format!(
                "{}:fit{:.2}/anchor{:.2}",
                point.beat_index, point.fitted_residual_ms, point.anchored_drift_ms
            )
        })
        .collect();
    println!("first_grid_error={first_grid_error:?}");
}

fn parse_arg(flag: &str) -> Option<f32> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next()?.parse().ok();
        }
    }
    None
}

fn synthetic_click_track(sample_rate: u32, bpm: f32, seconds: f32) -> AudioBuffer {
    let frames = (sample_rate as f32 * seconds).round() as usize;
    let interval = (60.0 / bpm * sample_rate as f32).round() as usize;
    let mut samples = vec![0.0; frames];

    let mut index = 0usize;
    while index < frames {
        for offset in 0..128usize {
            if let Some(sample) = samples.get_mut(index + offset) {
                *sample = 1.0 - offset as f32 / 128.0;
            }
        }
        index = index.saturating_add(interval.max(1));
    }

    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}
