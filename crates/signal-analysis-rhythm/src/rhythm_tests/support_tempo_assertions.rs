fn scope_summary(scope: super::TempoStabilityScope) -> super::TempoStabilityScopeSummary {
    match scope {
        super::TempoStabilityScope::WholeTrackStable => super::TempoStabilityScopeSummary {
            scope,
            support: super::TempoStabilityScopeSupport {
                edge_trimmed_coverage: super::Confidence::new(1.0),
                contiguous_core_coverage: super::Confidence::new(0.98),
                interior_stability: super::Confidence::new(1.0),
                edge_locality: super::Confidence::new(0.05),
            },
        },
        super::TempoStabilityScope::StableWithLocalizedEdgeDamage => {
            super::TempoStabilityScopeSummary {
                scope,
                support: super::TempoStabilityScopeSupport {
                    edge_trimmed_coverage: super::Confidence::new(0.99),
                    contiguous_core_coverage: super::Confidence::new(0.66),
                    interior_stability: super::Confidence::new(0.98),
                    edge_locality: super::Confidence::new(0.95),
                },
            }
        }
        super::TempoStabilityScope::CoreStableOnly => super::TempoStabilityScopeSummary {
            scope,
            support: super::TempoStabilityScopeSupport {
                edge_trimmed_coverage: super::Confidence::new(0.61),
                contiguous_core_coverage: super::Confidence::new(0.54),
                interior_stability: super::Confidence::new(0.88),
                edge_locality: super::Confidence::new(0.32),
            },
        },
        super::TempoStabilityScope::MidTrackUnstable => super::TempoStabilityScopeSummary {
            scope,
            support: super::TempoStabilityScopeSupport {
                edge_trimmed_coverage: super::Confidence::new(0.28),
                contiguous_core_coverage: super::Confidence::new(0.24),
                interior_stability: super::Confidence::new(0.42),
                edge_locality: super::Confidence::new(0.18),
            },
        },
    }
}

fn assert_detected_bpm(
    preset: RhythmPreset,
    result: &super::BeatAnalysisResult,
    expected_bpm: f32,
    tolerance: f32,
) {
    assert!(
        (result.bpm - expected_bpm).abs() < tolerance,
        "preset {:?} detected bpm {} expected {} +/- {}",
        preset,
        result.bpm,
        expected_bpm,
        tolerance
    );
}

fn assert_meter(
    preset: RhythmPreset,
    result: &super::BeatAnalysisResult,
    beats_per_bar: usize,
    min_confidence: f32,
) -> &super::MeterEstimate {
    let meter = result
        .meter
        .as_ref()
        .unwrap_or_else(|| panic!("preset {:?} expected meter estimate", preset));
    assert_eq!(
        meter.beats_per_bar, beats_per_bar,
        "preset {:?} beats_per_bar {}",
        preset, meter.beats_per_bar
    );
    assert!(
        meter.confidence.0 > min_confidence,
        "preset {:?} meter confidence {}",
        preset,
        meter.confidence.0
    );
    meter
}

