use super::*;

#[test]
fn beat_tracker_resolves_tempo_consumption_across_real_analysis_paths() {
    let (_, neutral) = analyze_preset(RhythmPreset::NeutralClick120);
    let neutral_consumption = neutral.tempo_consumption(Some(119.5));

    assert_eq!(neutral_consumption.action, super::TempoStateAction::Lock);
    assert_eq!(
        neutral_consumption.continuity_action,
        super::TempoContinuityAction::Lock
    );
    assert_eq!(
        neutral_consumption.current.source,
        super::TempoConsumptionSource::SnappedCurrentTempo
    );
    assert_eq!(neutral_consumption.current.bpm, Some(120.0));
    assert_eq!(
        neutral_consumption.fallback.source,
        super::TempoConsumptionSource::SnappedCurrentTempo
    );
    assert_eq!(neutral_consumption.fallback.bpm, Some(120.0));
    assert_eq!(neutral_consumption.fallback_after_beats, 20);

    let slow = analyze_fixture(&click_track(48_000, 90.0, 8.0));
    let slow_with_prior = slow.tempo_consumption(Some(89.75));
    let slow_without_prior = slow.tempo_consumption(None);

    assert_eq!(slow_with_prior.action, super::TempoStateAction::Monitor);
    assert_eq!(
        slow_with_prior.continuity_action,
        super::TempoContinuityAction::Reacquire
    );
    assert_eq!(
        slow_with_prior.current.source,
        super::TempoConsumptionSource::SnappedCurrentTempo
    );
    assert!(slow_with_prior
        .current
        .bpm
        .map(|bpm| (bpm - 90.0).abs() < 0.1)
        .unwrap_or(false));
    assert_eq!(
        slow_with_prior.fallback.source,
        super::TempoConsumptionSource::PriorTempo
    );
    assert_eq!(slow_with_prior.fallback.bpm, Some(89.75));
    assert_eq!(slow_with_prior.fallback_after_beats, 8);
    assert_eq!(
        slow_without_prior.fallback.source,
        super::TempoConsumptionSource::NoTempo
    );
    assert_eq!(slow_without_prior.fallback.bpm, None);
    assert_eq!(slow_without_prior.fallback_after_beats, 8);
    assert_eq!(
        slow_with_prior.stability_scope.scope,
        super::TempoStabilityScope::CoreStableOnly
    );

    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let weak_backbeat_consumption = weak_backbeat.tempo_consumption(Some(118.2));

    assert_eq!(
        weak_backbeat_consumption.action,
        super::TempoStateAction::Lock
    );
    assert_eq!(
        weak_backbeat_consumption.continuity_action,
        super::TempoContinuityAction::Lock
    );
    assert_eq!(
        weak_backbeat_consumption.current.source,
        super::TempoConsumptionSource::RefinedCurrentTempo
    );
    assert!(weak_backbeat_consumption
        .current
        .bpm
        .map(|bpm| (bpm - weak_backbeat.tempo_interpretation.recommended_bpm).abs() < 0.001)
        .unwrap_or(false));
    assert_eq!(
        weak_backbeat_consumption.fallback.source,
        super::TempoConsumptionSource::RefinedCurrentTempo
    );
    assert!(weak_backbeat_consumption
        .fallback
        .bpm
        .map(|bpm| (bpm - weak_backbeat.tempo_interpretation.recommended_bpm).abs() < 0.001)
        .unwrap_or(false));

    let silence = AudioBuffer::new(
        SampleRate(48_000),
        ChannelLayout::Mono,
        signal_primitives::FrameCount(48_000),
    );
    let cleared = analyze_fixture(&silence).tempo_consumption(Some(120.0));

    assert_eq!(cleared.action, super::TempoStateAction::Defer);
    assert_eq!(
        cleared.continuity_action,
        super::TempoContinuityAction::Clear
    );
    assert_eq!(
        cleared.current.source,
        super::TempoConsumptionSource::NoTempo
    );
    assert_eq!(cleared.current.bpm, None);
    assert_eq!(
        cleared.fallback.source,
        super::TempoConsumptionSource::NoTempo
    );
    assert_eq!(cleared.fallback.bpm, None);
    assert_eq!(cleared.fallback_after_beats, 0);
}
