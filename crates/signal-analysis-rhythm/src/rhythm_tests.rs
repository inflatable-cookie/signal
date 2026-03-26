use super::*;
use signal_analysis::{
    run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
    AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate, Seconds};

#[path = "tests/rhythm_test_bar_transition_basic.rs"]
mod bar_transition_basic;
#[path = "tests/rhythm_test_bar_transition_metrics.rs"]
mod bar_transition_metrics;
#[path = "tests/rhythm_test_bar_transition_recovery_metrics.rs"]
mod bar_transition_recovery_metrics;
#[path = "tests/rhythm_test_bar_transition_reentry.rs"]
mod bar_transition_reentry;
#[path = "tests/rhythm_test_bar_transition_reentry_extended.rs"]
mod bar_transition_reentry_extended;
#[path = "tests/rhythm_test_named_preset_metrics.rs"]
mod named_preset_metrics;
#[path = "tests/rhythm_test_named_preset_monotonicity.rs"]
mod named_preset_monotonicity;
#[path = "tests/rhythm_test_named_preset_surface.rs"]
mod named_preset_surface;
#[path = "tests/rhythm_test_named_preset_surface_cases.rs"]
mod named_preset_surface_cases;
#[path = "tests/rhythm_test_presets.rs"]
mod presets;
#[path = "tests/rhythm_test_transition_fixtures.rs"]
mod transition_fixtures;
use presets::{
    build_structured_harmony_preset, render_preset, BarTransitionVariant, HarmonicRhythmVariant,
    RhythmPreset,
};
use transition_fixtures::{DropoutVariant, FillDensityVariant};

include!("rhythm_tests/support_fixtures.rs");
include!("rhythm_tests/support_acceptance.rs");
include!("rhythm_tests/support_tempo_synthetic.rs");
include!("rhythm_tests/support_tempo_assertions.rs");
include!("rhythm_tests/support_audio.rs");

#[path = "rhythm_tests/analysis_rate_acceptance.rs"]
mod analysis_rate_acceptance;
#[path = "rhythm_tests/detection_clicks.rs"]
mod detection_clicks;
#[path = "rhythm_tests/detection_patterns.rs"]
mod detection_patterns;
#[path = "rhythm_tests/detection_sections.rs"]
mod detection_sections;
#[path = "rhythm_tests/detection_tempo_consumption.rs"]
mod detection_tempo_consumption;
#[path = "rhythm_tests/detection_tempo_drift.rs"]
mod detection_tempo_drift;
#[path = "rhythm_tests/detection_tempo_metrics.rs"]
mod detection_tempo_metrics;
#[path = "rhythm_tests/meter_confidence_dropouts.rs"]
mod meter_confidence_dropouts;
#[path = "rhythm_tests/meter_continuity_actions.rs"]
mod meter_continuity_actions;
#[path = "rhythm_tests/meter_continuity_arc_classification.rs"]
mod meter_continuity_arc_classification;
#[path = "rhythm_tests/meter_continuity_arc_support.rs"]
mod meter_continuity_arc_support;
#[path = "rhythm_tests/meter_continuity_causes.rs"]
mod meter_continuity_causes;
#[path = "rhythm_tests/meter_continuity_expiry.rs"]
mod meter_continuity_expiry;
#[path = "rhythm_tests/meter_continuity_history.rs"]
mod meter_continuity_history;
#[path = "rhythm_tests/meter_continuity_provenance.rs"]
mod meter_continuity_provenance;
#[path = "rhythm_tests/meter_continuity_reason.rs"]
mod meter_continuity_reason;
#[path = "rhythm_tests/meter_continuity_severity.rs"]
mod meter_continuity_severity;
#[path = "rhythm_tests/meter_continuity_triggers.rs"]
mod meter_continuity_triggers;
#[path = "rhythm_tests/meter_public_categories.rs"]
mod meter_public_categories;
#[path = "rhythm_tests/meter_structure_fallback.rs"]
mod meter_structure_fallback;
#[path = "rhythm_tests/stability_scope.rs"]
mod stability_scope;
#[path = "rhythm_tests/tempo_continuity_arc_surface.rs"]
mod tempo_continuity_arc_surface;
#[path = "rhythm_tests/tempo_continuity_calibration.rs"]
mod tempo_continuity_calibration;
#[path = "rhythm_tests/tempo_interpretation.rs"]
mod tempo_interpretation;
#[path = "rhythm_tests/tempo_refine_outliers.rs"]
mod tempo_refine_outliers;
#[path = "rhythm_tests/tempo_state_core_window.rs"]
mod tempo_state_core_window;
#[path = "rhythm_tests/tempo_state_deferred.rs"]
mod tempo_state_deferred;
#[path = "rhythm_tests/tempo_state_guarded_refined.rs"]
mod tempo_state_guarded_refined;
#[path = "rhythm_tests/tempo_state_scope_edges.rs"]
mod tempo_state_scope_edges;
#[path = "rhythm_tests/tempo_state_stable_integer.rs"]
mod tempo_state_stable_integer;
#[path = "rhythm_tests/tempo_structure_summary.rs"]
mod tempo_structure_summary;
#[path = "rhythm_tests/tempo_structure_windows.rs"]
mod tempo_structure_windows;
