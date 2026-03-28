#[cfg(test)]
use super::*;
#[cfg(test)]
use signal_analysis::{
    run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
    AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
};
#[cfg(test)]
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate, Seconds};

#[cfg(test)]
fn default_tempo_stability_scope() -> TempoStabilityScopeSummary {
    TempoStabilityScopeSummary {
        scope: TempoStabilityScope::WholeTrackStable,
        support: TempoStabilityScopeSupport {
            edge_trimmed_coverage: Confidence::new(1.0),
            contiguous_core_coverage: Confidence::new(1.0),
            interior_stability: Confidence::new(1.0),
            edge_locality: Confidence::new(0.0),
        },
    }
}

#[cfg(test)]
pub(crate) fn tempo_state_recommendation(
    interpretation: TempoInterpretation,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
) -> TempoStateRecommendation {
    tempo_state_recommendation_with_scope(
        interpretation,
        confidence,
        tempo_ambiguity,
        default_tempo_stability_scope(),
    )
}
#[cfg(test)]
#[path = "tests/rhythm_test_bar_transition_basic.rs"]
mod bar_transition_basic;
#[cfg(test)]
#[path = "tests/rhythm_test_bar_transition_metrics.rs"]
mod bar_transition_metrics;
#[cfg(test)]
#[path = "tests/rhythm_test_bar_transition_recovery_metrics.rs"]
mod bar_transition_recovery_metrics;
#[cfg(test)]
#[path = "tests/rhythm_test_bar_transition_reentry.rs"]
mod bar_transition_reentry;
#[cfg(test)]
#[path = "tests/rhythm_test_bar_transition_reentry_extended.rs"]
mod bar_transition_reentry_extended;
#[cfg(test)]
#[path = "tests/rhythm_test_named_preset_metrics.rs"]
mod named_preset_metrics;
#[cfg(test)]
#[path = "tests/rhythm_test_named_preset_monotonicity.rs"]
mod named_preset_monotonicity;
#[cfg(test)]
#[path = "tests/rhythm_test_named_preset_surface.rs"]
mod named_preset_surface;
#[cfg(test)]
#[path = "tests/rhythm_test_named_preset_surface_cases.rs"]
mod named_preset_surface_cases;
#[cfg(test)]
#[path = "tests/rhythm_test_presets.rs"]
mod presets;
#[cfg(test)]
#[path = "tests/rhythm_test_transition_fixtures.rs"]
mod transition_fixtures;
#[cfg(test)]
use presets::{
    build_structured_harmony_preset, render_preset, BarTransitionVariant, HarmonicRhythmVariant,
    RhythmPreset,
};
#[cfg(test)]
use transition_fixtures::{DropoutVariant, FillDensityVariant};

#[cfg(test)]
include!("rhythm_tests/support_fixtures.rs");
#[cfg(test)]
include!("rhythm_tests/support_acceptance.rs");
#[cfg(test)]
include!("rhythm_tests/support_tempo_synthetic.rs");
#[cfg(test)]
include!("rhythm_tests/support_tempo_assertions.rs");
#[cfg(test)]
include!("rhythm_tests/support_audio.rs");

#[cfg(test)]
#[path = "rhythm_tests/analysis_rate_acceptance.rs"]
mod analysis_rate_acceptance;
#[cfg(test)]
#[path = "rhythm_tests/detection_clicks.rs"]
mod detection_clicks;
#[cfg(test)]
#[path = "rhythm_tests/detection_patterns.rs"]
mod detection_patterns;
#[cfg(test)]
#[path = "rhythm_tests/detection_sections.rs"]
mod detection_sections;
#[cfg(test)]
#[path = "rhythm_tests/detection_tempo_consumption.rs"]
mod detection_tempo_consumption;
#[cfg(test)]
#[path = "rhythm_tests/detection_tempo_drift.rs"]
mod detection_tempo_drift;
#[cfg(test)]
#[path = "rhythm_tests/detection_tempo_metrics.rs"]
mod detection_tempo_metrics;
#[cfg(test)]
#[path = "rhythm_tests/meter_confidence_dropouts.rs"]
mod meter_confidence_dropouts;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_actions.rs"]
mod meter_continuity_actions;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_arc_classification.rs"]
mod meter_continuity_arc_classification;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_arc_support.rs"]
mod meter_continuity_arc_support;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_causes.rs"]
mod meter_continuity_causes;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_expiry.rs"]
mod meter_continuity_expiry;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_history.rs"]
mod meter_continuity_history;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_provenance.rs"]
mod meter_continuity_provenance;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_reason.rs"]
mod meter_continuity_reason;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_severity.rs"]
mod meter_continuity_severity;
#[cfg(test)]
#[path = "rhythm_tests/meter_continuity_triggers.rs"]
mod meter_continuity_triggers;
#[cfg(test)]
#[path = "rhythm_tests/meter_public_categories.rs"]
mod meter_public_categories;
#[cfg(test)]
#[path = "rhythm_tests/meter_structure_fallback.rs"]
mod meter_structure_fallback;
#[cfg(test)]
#[path = "rhythm_tests/stability_scope.rs"]
mod stability_scope;
#[cfg(test)]
#[path = "rhythm_tests/tempo_continuity_arc_surface.rs"]
mod tempo_continuity_arc_surface;
#[cfg(test)]
#[path = "rhythm_tests/tempo_continuity_calibration.rs"]
mod tempo_continuity_calibration;
#[cfg(test)]
#[path = "rhythm_tests/tempo_interpretation.rs"]
mod tempo_interpretation;
#[cfg(test)]
#[path = "rhythm_tests/tempo_refine_outliers.rs"]
mod tempo_refine_outliers;
#[cfg(test)]
#[path = "rhythm_tests/tempo_state_core_window.rs"]
mod tempo_state_core_window;
#[cfg(test)]
#[path = "rhythm_tests/tempo_state_deferred.rs"]
mod tempo_state_deferred;
#[cfg(test)]
#[path = "rhythm_tests/tempo_state_guarded_refined.rs"]
mod tempo_state_guarded_refined;
#[cfg(test)]
#[path = "rhythm_tests/tempo_state_scope_edges.rs"]
mod tempo_state_scope_edges;
#[cfg(test)]
#[path = "rhythm_tests/tempo_state_stable_integer.rs"]
mod tempo_state_stable_integer;
#[cfg(test)]
#[path = "rhythm_tests/tempo_structure_summary.rs"]
mod tempo_structure_summary;
#[cfg(test)]
#[path = "rhythm_tests/tempo_structure_windows.rs"]
mod tempo_structure_windows;
