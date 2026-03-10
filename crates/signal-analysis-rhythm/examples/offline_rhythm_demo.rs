use signal_analysis::AnalysisStage;
use signal_analysis_rhythm::{
    BeatTracker, BeatTrackerConfig, MeterContinuityCauseStack, MeterContinuityPlan,
    MeterContinuityTransition,
};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

// Practical inspection example for the public rhythm result surface.
//
// Run with:
// `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120`
//
// The printed `tempo_state` and `meter_state` lines are the intended integration
// hooks for deciding whether to lock transport tempo and meter, keep monitoring, or
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
    println!(
        "tempo_state=action:{:?},reason:{:?},confidence:{:.3}",
        result.tempo_state.action, result.tempo_state.reason, result.tempo_state.confidence.0,
    );
    let tempo_consumption = result.tempo_consumption(None);
    println!(
        "tempo_consumption=current:{:?}@{:?}/fallback:{:?}@{:?}/action:{:?}/{:?}/after:{}@{:.3}/scope:{:?}",
        tempo_consumption.current.source,
        tempo_consumption.current.bpm,
        tempo_consumption.fallback.source,
        tempo_consumption.fallback.bpm,
        tempo_consumption.action,
        tempo_consumption.continuity_action,
        tempo_consumption.fallback_after_beats,
        tempo_consumption.confidence.0,
        tempo_consumption.stability_scope.scope,
    );
    println!(
        "tempo_continuity=current:{:?}/{:?}/{:?}/{:?}/arc:{:?}/{:?}/decision:{:?}/{:?}/{:?}->{:?}/{:?}/{:?}/{:?}@{:.3}/arc_meta:{:?}/{}:{}:{}:{}/downgrade:{:.3},{:.3},{:.3},{:.3},{:.3}/trend:{:.3},{:.3},{:.3}/inflection:{:?}@{}:{:.3}:{:.3}/competing:{:?}@{}:{:.3}:{:.3}/balance:{:.3},{:.3},{:.3},{:.3}/support:{:.3},{:.3},{:.3}/{:?}/{:?}/{:?}/unresolved:{}:{}@{:.3}/refresh:{:.3}/trusted:{} recheck:{}",
        result.tempo_state.continuity.action,
        result.tempo_state.continuity.source,
        result.tempo_state.continuity.severity,
        result.tempo_state.continuity.history,
        result.tempo_state.continuity.arc,
        result.tempo_state.continuity.arc_rationale,
        result.tempo_state.continuity.arc_decision.recommendation,
        result.tempo_state.continuity.arc_decision.action,
        result.tempo_state.continuity.arc_decision.severity,
        result.tempo_state.continuity.arc_decision.fallback_action,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_rationale,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_trend,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_trend_rationale,
        result.tempo_state.continuity.arc_decision.confidence.0,
        result.tempo_state.continuity.arc_decision.provenance,
        result
            .tempo_state
            .continuity
            .arc_decision
            .expiry
            .guaranteed_until_beats,
        result
            .tempo_state
            .continuity
            .arc_decision
            .expiry
            .fallback_after_beats,
        result
            .tempo_state
            .continuity
            .arc_decision
            .expiry
            .clear_after_beats,
        result
            .tempo_state
            .continuity
            .arc_decision
            .expiry
            .max_failed_revalidations,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_support
            .stability_window_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_support
            .boundary_drift_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_support
            .ambiguity_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_support
            .failed_revalidation_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_support
            .evidence_loss_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_trend_support
            .current_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_trend_support
            .next_stage_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_trend_support
            .terminal_pressure
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .stage,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .after_beats,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .next_stage_delta
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .terminal_delta
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .competing_stage,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .competing_after_beats,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .competing_delta
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .competing_support
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .balance
            .primary_weight
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .balance
            .competing_weight
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .balance
            .unattributed_weight
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .balance
            .dominance
            .0,
        result.tempo_state.continuity.arc_support.refresh_strength.0,
        result.tempo_state.continuity.arc_support.drift_pressure.0,
        result
            .tempo_state
            .continuity
            .arc_support
            .instability_pressure
            .0,
        result.tempo_state.continuity.trigger,
        result.tempo_state.continuity.reason,
        result.tempo_state.continuity.provenance,
        result.tempo_state.continuity.unresolved.beats,
        result
            .tempo_state
            .continuity
            .unresolved
            .failed_revalidations,
        result.tempo_state.continuity.confidence.0,
        result.tempo_state.continuity.refresh_strength.0,
        result.tempo_state.continuity.trusted_beats,
        result.tempo_state.continuity.revalidate_after_beats,
    );
    println!(
        "tempo_continuity.inflection_rationale.primary={:?}:{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .dominant,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .stability_window
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .boundary_drift
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .ambiguity_carry
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .prior_tempo_drift
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .revalidation_decay
            .0,
        result
            .tempo_state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .evidence_loss
            .0,
    );
    if let Some(competing_rationale) = result
        .tempo_state
        .continuity
        .arc_decision
        .downgrade_inflection
        .rationale_balance
        .competing
    {
        println!(
            "tempo_continuity.inflection_rationale.competing={:?}:{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            competing_rationale.dominant,
            competing_rationale.stability_window.0,
            competing_rationale.boundary_drift.0,
            competing_rationale.ambiguity_carry.0,
            competing_rationale.prior_tempo_drift.0,
            competing_rationale.revalidation_decay.0,
            competing_rationale.evidence_loss.0,
        );
    } else {
        println!("tempo_continuity.inflection_rationale.competing=None");
    }
    println!(
        "tempo_continuity.causes={}",
        format_tempo_cause_stack(result.tempo_state.continuity.causes),
    );
    println!(
        "tempo_continuity.expiry=guaranteed:{} downgrade:{} clear:{} failed_rechecks:{}",
        result.tempo_state.continuity.expiry.guaranteed_until_beats,
        result.tempo_state.continuity.expiry.downgrade_after_beats,
        result.tempo_state.continuity.expiry.clear_after_beats,
        result
            .tempo_state
            .continuity
            .expiry
            .max_failed_revalidations,
    );
    println!(
        "tempo_continuity.refresh=after:{}:{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/unresolved:{}:{}@{:.3}/refresh:{:.3}",
        result.tempo_state.continuity.lifecycle.refresh.after_beats,
        result.tempo_state.continuity.lifecycle.refresh.action,
        result.tempo_state.continuity.lifecycle.refresh.source,
        result.tempo_state.continuity.lifecycle.refresh.severity,
        result.tempo_state.continuity.lifecycle.refresh.history,
        result.tempo_state.continuity.lifecycle.refresh.trigger,
        result.tempo_state.continuity.lifecycle.refresh.reason,
        result.tempo_state.continuity.lifecycle.refresh.provenance,
        result.tempo_state.continuity.lifecycle.refresh.unresolved.beats,
        result
            .tempo_state
            .continuity
            .lifecycle
            .refresh
            .unresolved
            .failed_revalidations,
        result.tempo_state.continuity.lifecycle.refresh.confidence.0,
        result
            .tempo_state
            .continuity
            .lifecycle
            .refresh
            .refresh_strength
            .0,
    );
    println!(
        "tempo_continuity.decay0=after:{}:{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/unresolved:{}:{}@{:.3}/refresh:{:.3}",
        result.tempo_state.continuity.lifecycle.decay[0].after_beats,
        result.tempo_state.continuity.lifecycle.decay[0].action,
        result.tempo_state.continuity.lifecycle.decay[0].source,
        result.tempo_state.continuity.lifecycle.decay[0].severity,
        result.tempo_state.continuity.lifecycle.decay[0].history,
        result.tempo_state.continuity.lifecycle.decay[0].trigger,
        result.tempo_state.continuity.lifecycle.decay[0].reason,
        result.tempo_state.continuity.lifecycle.decay[0].provenance,
        result.tempo_state.continuity.lifecycle.decay[0].unresolved.beats,
        result
            .tempo_state
            .continuity
            .lifecycle
            .decay[0]
            .unresolved
            .failed_revalidations,
        result.tempo_state.continuity.lifecycle.decay[0]
            .confidence
            .0,
        result.tempo_state.continuity.lifecycle.decay[0]
            .refresh_strength
            .0,
    );
    println!(
        "tempo_continuity.decay1=after:{}:{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/unresolved:{}:{}@{:.3}/refresh:{:.3}",
        result.tempo_state.continuity.lifecycle.decay[1].after_beats,
        result.tempo_state.continuity.lifecycle.decay[1].action,
        result.tempo_state.continuity.lifecycle.decay[1].source,
        result.tempo_state.continuity.lifecycle.decay[1].severity,
        result.tempo_state.continuity.lifecycle.decay[1].history,
        result.tempo_state.continuity.lifecycle.decay[1].trigger,
        result.tempo_state.continuity.lifecycle.decay[1].reason,
        result.tempo_state.continuity.lifecycle.decay[1].provenance,
        result.tempo_state.continuity.lifecycle.decay[1].unresolved.beats,
        result
            .tempo_state
            .continuity
            .lifecycle
            .decay[1]
            .unresolved
            .failed_revalidations,
        result.tempo_state.continuity.lifecycle.decay[1]
            .confidence
            .0,
        result.tempo_state.continuity.lifecycle.decay[1]
            .refresh_strength
            .0,
    );
    let candidates: Vec<String> = result
        .tempo_candidates
        .iter()
        .map(|candidate| format!("{:.2}@{:.3}", candidate.bpm, candidate.confidence.0))
        .collect();
    println!("tempo_candidates={candidates:?}");
    println!(
        "meter_state=action:{:?},reason:{:?},confidence:{:.3}",
        result.meter_state.action, result.meter_state.reason, result.meter_state.confidence.0
    );
    print_continuity_plan("bar_length", &result.meter_state.continuity.bar_length);
    print_continuity_plan(
        "downbeat_phase",
        &result.meter_state.continuity.downbeat_phase,
    );
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

fn format_tempo_cause_stack(stack: signal_analysis_rhythm::TempoContinuityCauseStack) -> String {
    let mut labels = vec![format!("{:?}", stack.primary)];
    for cause in stack.secondary.into_iter().flatten() {
        labels.push(format!("{:?}", cause));
    }
    labels.join("+")
}

fn print_continuity_plan(label: &str, plan: &MeterContinuityPlan) {
    println!(
        "{label}=current:{:?}/{:?}/{:?}/{:?}/arc:{:?}/{:?}/support:{:.3},{:.3},{:.3}/{:?}@{:.3}/trigger:{:?}/u:{}b:{}bar:{}f/causes:{} trusted:{} recheck:{}",
        plan.action,
        plan.source,
        plan.severity,
        plan.history,
        plan.arc,
        plan.arc_rationale,
        plan.arc_support.refresh_strength.0,
        plan.arc_support.drift_pressure.0,
        plan.arc_support.structural_pressure.0,
        plan.reason,
        plan.confidence.0,
        plan.trigger,
        plan.unresolved.beats,
        plan.unresolved.bars,
        plan.unresolved.failed_revalidations,
        format_cause_stack(plan.causes),
        plan.trusted_beats,
        plan.revalidate_after_beats,
    );
    print_continuity_transition(label, "refresh", &plan.lifecycle.refresh);
    print_continuity_transition(label, "decay0", &plan.lifecycle.decay[0]);
    print_continuity_transition(label, "decay1", &plan.lifecycle.decay[1]);
}

fn print_continuity_transition(label: &str, stage: &str, transition: &MeterContinuityTransition) {
    println!(
        "{label}.{stage}=after:{}:{:?}/{:?}/{:?}/{:?}/{:?}@{:.3}/trigger:{:?}/u:{}b:{}bar:{}f/causes:{}",
        transition.after_beats,
        transition.action,
        transition.source,
        transition.severity,
        transition.history,
        transition.reason,
        transition.confidence.0,
        transition.trigger,
        transition.unresolved.beats,
        transition.unresolved.bars,
        transition.unresolved.failed_revalidations,
        format_cause_stack(transition.causes),
    );
}

fn format_cause_stack(stack: MeterContinuityCauseStack) -> String {
    let mut causes = Vec::with_capacity(stack.count.max(1));
    causes.push(format!("{:?}", stack.primary));
    for cause in stack.secondary.into_iter().flatten() {
        causes.push(format!("{:?}", cause));
    }
    causes.join("+")
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
