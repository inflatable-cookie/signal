#[cfg(test)]
use super::*;
#[cfg(test)]
use signal_analysis::{
    run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
    AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
};
#[cfg(test)]
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

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
#[path = "rhythm_tests/detection_tempo_drift.rs"]
mod detection_tempo_drift;
#[cfg(test)]
#[path = "rhythm_tests/detection_tempo_metrics.rs"]
mod detection_tempo_metrics;
#[cfg(test)]
#[path = "rhythm_tests/meter_confidence_dropouts.rs"]
mod meter_confidence_dropouts;
#[cfg(test)]
#[path = "rhythm_tests/stability_scope.rs"]
mod stability_scope;
#[cfg(test)]
#[path = "rhythm_tests/tempo_interpretation.rs"]
mod tempo_interpretation;
#[cfg(test)]
#[path = "rhythm_tests/tempo_refine_outliers.rs"]
mod tempo_refine_outliers;
