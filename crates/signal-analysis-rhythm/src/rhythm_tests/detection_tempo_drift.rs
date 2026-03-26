use super::*;

#[test]
fn beat_tracker_calibrates_local_tempo_drift_between_stable_and_irregular_fixtures() {
    let (_, stable) = analyze_preset(RhythmPreset::NeutralClick120);
    let slow = analyze_fixture(&click_track(48_000, 90.0, 8.0));
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, section) = analyze_preset(RhythmPreset::SectionTransition122);
    let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

    assert!(
        weak_backbeat.tempo_diagnostics.mean_abs_deviation_bpm
            > stable.tempo_diagnostics.mean_abs_deviation_bpm
    );
    assert!(section.tempo_diagnostics.drift_span_bpm >= stable.tempo_diagnostics.drift_span_bpm);
    assert!(!weak_backbeat.tempo_diagnostics.windowed_tempi.is_empty());
    assert!(!section.tempo_diagnostics.windowed_tempi.is_empty());
    assert!(stable.tempo_diagnostics.boundary_bias_bpm > 0.0);
    assert!(
        section.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
            >= stable.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
    );
    assert!(
        slow.tempo_diagnostics
            .beat_grid_error
            .edge_mean_abs_residual_ms
            > slow
                .tempo_diagnostics
                .beat_grid_error
                .core_mean_abs_residual_ms
    );
    assert!(
        slow.tempo_diagnostics
            .beat_grid_error
            .mean_abs_anchored_drift_ms
            > stable
                .tempo_diagnostics
                .beat_grid_error
                .mean_abs_anchored_drift_ms
    );
    assert_eq!(
        slow.tempo_interpretation.recommendation,
        super::TempoRecommendation::SnapInteger
    );
    assert_eq!(
        slow.tempo_interpretation.reason,
        super::TempoInterpretationReason::NearIntegerPulse
    );
    assert!(
        (slow.tempo_interpretation.recommended_bpm - 90.0).abs() < 0.1,
        "slow recommended bpm {}",
        slow.tempo_interpretation.recommended_bpm
    );
    assert!(
        slow.tempo_interpretation.profile.boundary_edge_gap_ms > 0.0,
        "slow boundary edge gap {}",
        slow.tempo_interpretation.profile.boundary_edge_gap_ms
    );
    assert_eq!(
        slow.tempo_diagnostics.stability_scope.scope,
        super::TempoStabilityScope::CoreStableOnly
    );
    assert_eq!(slow.tempo_state.action, super::TempoStateAction::Monitor);
    assert_eq!(
        slow.tempo_state.reason,
        super::TempoStateReason::CoreStableTempo
    );
    assert_eq!(
        slow.tempo_state.continuity.action,
        super::TempoContinuityAction::Reacquire
    );
    assert_eq!(
        slow.tempo_state.continuity.source,
        super::TempoContinuitySource::CurrentTempo
    );
    assert_eq!(
        slow.tempo_state.continuity.reason,
        super::TempoContinuityReason::RevalidationDecay
    );
    assert_eq!(
        slow.tempo_state.continuity.provenance,
        super::TempoContinuityProvenance::GuardedRefinedEstimate
    );
    assert_eq!(
        slow.tempo_state.continuity.severity,
        super::TempoContinuitySeverity::Fragile
    );
    assert_eq!(
        slow.tempo_state.continuity.history,
        super::TempoContinuityHistory::Preserving
    );
    assert!(matches!(
        slow.tempo_state.continuity.trigger,
        super::TempoContinuityTrigger::StableRevalidation
            | super::TempoContinuityTrigger::AmbiguityCarry
    ));
    assert!(slow.tempo_state.continuity.unresolved.beats >= 4);
    assert!(matches!(
        slow.tempo_state.continuity.causes.primary,
        super::TempoContinuityCause::StableTempoEvidence
            | super::TempoContinuityCause::TempoAmbiguity
    ));
    assert_eq!(slow.tempo_state.continuity.expiry.guaranteed_until_beats, 4);
    assert_eq!(slow.tempo_state.continuity.expiry.downgrade_after_beats, 8);
    assert_eq!(slow.tempo_state.continuity.expiry.clear_after_beats, 12);
    assert_eq!(
        weak_backbeat.tempo_interpretation.recommendation,
        super::TempoRecommendation::UseRefined
    );
    assert_eq!(
        weak_backbeat.tempo_interpretation.reason,
        super::TempoInterpretationReason::StableRefinedPulse
    );
    assert_eq!(
        weak_backbeat.tempo_state.action,
        super::TempoStateAction::Lock
    );
    assert_eq!(
        weak_backbeat.tempo_state.reason,
        super::TempoStateReason::StableRefinedTempo
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.action,
        super::TempoContinuityAction::Lock
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.source,
        super::TempoContinuitySource::CurrentTempo
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.reason,
        super::TempoContinuityReason::StableTempo
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.provenance,
        super::TempoContinuityProvenance::StableRefinedEstimate
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.severity,
        super::TempoContinuitySeverity::Confirmed
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.history,
        super::TempoContinuityHistory::Reinforcing
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.trigger,
        super::TempoContinuityTrigger::StableRevalidation
    );
    assert_eq!(
        weak_backbeat.tempo_state.continuity.causes.primary,
        super::TempoContinuityCause::StableTempoEvidence
    );
    assert_eq!(
        weak_backbeat
            .tempo_state
            .continuity
            .expiry
            .max_failed_revalidations,
        3
    );
    assert!(matches!(
        ambiguous.tempo_interpretation.recommendation,
        super::TempoRecommendation::UseCoreWindow | super::TempoRecommendation::UseRefined
    ));
    assert!(matches!(
        ambiguous.tempo_interpretation.trust,
        super::TempoTrustLevel::Guarded | super::TempoTrustLevel::Stable
    ));
    assert!(ambiguous.tempo_interpretation.profile.stability_score.0 < 0.85);
    assert!(matches!(
        ambiguous.tempo_state.action,
        super::TempoStateAction::Monitor
            | super::TempoStateAction::Lock
            | super::TempoStateAction::Defer
    ));
    assert!(matches!(
        ambiguous.tempo_state.reason,
        super::TempoStateReason::CoreWindowFallback
            | super::TempoStateReason::StableRefinedTempo
            | super::TempoStateReason::CoreStableTempo
            | super::TempoStateReason::StableTempoWithEdgeDamage
            | super::TempoStateReason::TempoDeferred
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.action,
        super::TempoContinuityAction::Retain
            | super::TempoContinuityAction::Lock
            | super::TempoContinuityAction::Clear
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.source,
        super::TempoContinuitySource::CoreWindow
            | super::TempoContinuitySource::CurrentTempo
            | super::TempoContinuitySource::Cleared
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.reason,
        super::TempoContinuityReason::CoreWindowCarry
            | super::TempoContinuityReason::StableTempo
            | super::TempoContinuityReason::InsufficientEvidence
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.provenance,
        super::TempoContinuityProvenance::CoreWindowEstimate
            | super::TempoContinuityProvenance::StableRefinedEstimate
            | super::TempoContinuityProvenance::NoTempo
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.severity,
        super::TempoContinuitySeverity::Guarded
            | super::TempoContinuitySeverity::Confirmed
            | super::TempoContinuitySeverity::Cleared
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.history,
        super::TempoContinuityHistory::Preserving
            | super::TempoContinuityHistory::Reinforcing
            | super::TempoContinuityHistory::Degrading
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.trigger,
        super::TempoContinuityTrigger::BoundaryDrift
            | super::TempoContinuityTrigger::StableRevalidation
            | super::TempoContinuityTrigger::EvidenceLoss
    ));
    assert!(matches!(
        ambiguous.tempo_state.continuity.causes.primary,
        super::TempoContinuityCause::BoundaryDrift
            | super::TempoContinuityCause::StableTempoEvidence
            | super::TempoContinuityCause::EvidenceLoss
            | super::TempoContinuityCause::TempoAmbiguity
    ));
    assert!(matches!(
        ambiguous
            .tempo_state
            .continuity
            .expiry
            .max_failed_revalidations,
        0 | 3
    ));
}
