use super::*;

#[test]
fn bounded_trailing_windows_preserve_stable_structure_and_tempo_summaries() {
    let (_, audio) = render_preset(
        RhythmPreset::StructuredHarmony120(HarmonicRhythmVariant::Active),
        48_000,
    );
    let full = analyze_fixture(&audio);
    let full_structure = full
        .rhythm_structure_assessment()
        .structure
        .expect("full structure summary");
    let full_tempo = full.tempo_structure_summary();

    for seconds in [6.0, 8.0, 10.0] {
        let bounded = analyze_trailing_window(&audio, super::BeatTrackerConfig::default(), seconds);
        let structure = bounded
            .rhythm_structure_assessment()
            .structure
            .expect("bounded structure summary");
        let tempo = bounded.tempo_structure_summary();

        assert_eq!(structure.beats_per_bar, full_structure.beats_per_bar);
        assert!(matches!(
            structure.continuity.action,
            super::MeterStateAction::Lock | super::MeterStateAction::Hold
        ));
        assert!(matches!(
            tempo.stability_scope.scope,
            super::TempoStabilityScope::WholeTrackStable
                | super::TempoStabilityScope::StableWithLocalizedEdgeDamage
                | super::TempoStabilityScope::CoreStableOnly
        ));
        assert!(matches!(
            tempo.continuity.action,
            super::TempoStateAction::Lock | super::TempoStateAction::Monitor
        ));
        assert!(tempo.selected_bpm.is_some());
        assert!(
            (tempo.selected_bpm.unwrap_or(0.0) - full_tempo.selected_bpm.unwrap_or(0.0)).abs()
                < 1.0
        );
        assert!(
            (tempo.core_window_bpm - full_tempo.core_window_bpm).abs() < 1.0,
            "seconds={seconds} core_window={} full={}",
            tempo.core_window_bpm,
            full_tempo.core_window_bpm,
        );
    }
}

#[test]
fn bounded_trailing_windows_preserve_weak_accent_and_actionable_tempo_summary() {
    let (_, audio) = render_preset(RhythmPreset::WeakBackbeat118, 48_000);
    let full = analyze_fixture(&audio);
    let full_tempo = full.tempo_structure_summary();

    for seconds in [10.0, 12.0, 14.0] {
        let bounded = analyze_trailing_window(&audio, super::BeatTrackerConfig::default(), seconds);
        let assessment = bounded.rhythm_structure_assessment();
        let tempo = bounded.tempo_structure_summary();

        assert_ne!(
            assessment.ambiguity.kind,
            super::RhythmStructureAmbiguityKind::InsufficientEvidence
        );
        assert!(assessment.ambiguity.confidence.0 > 0.1);
        assert!(assessment.structure.is_some() || assessment.fallback.recovery_window_available);
        assert!(matches!(
            tempo.continuity.action,
            super::TempoStateAction::Lock | super::TempoStateAction::Monitor
        ));
        assert!(tempo.selected_bpm.is_some());
        assert!(
            (tempo.selected_bpm.unwrap_or(0.0) - full_tempo.selected_bpm.unwrap_or(0.0)).abs()
                < 1.0
        );
        assert_ne!(
            tempo.continuity.current.source,
            super::TempoConsumptionSource::NoTempo
        );
    }
}
