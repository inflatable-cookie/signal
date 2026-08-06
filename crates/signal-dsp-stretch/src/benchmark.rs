use crate::phase_vocoder::{
    phase_locked_phase_vocoder, phase_vocoder, transient_reset_phase_vocoder,
    transient_reset_phase_vocoder_linked_stereo,
};
use crate::transient_smear::{
    measure_transient_smear, transient_smear_nan, StretchTransientSmearMeasurement,
    StretchTransientSmearPolicies,
};
use crate::{
    dynamic_ratio_output_boundaries, dynamic_ratio_output_frames,
    stretch_dynamic_ratio_mono_with_engine, OfflineHighQualityStretcher, RealtimePreviewStretcher,
    StretchRatioPoint, TimeStretcher,
};
use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::{Sample, SampleRate};

/// Corpus family required by the Signal-native stretch benchmark program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCorpusFamily {
    /// Transient-heavy drum and percussion material.
    DrumsPercussion,
    /// Bass material with sustained notes and plucked attacks.
    Bass,
    /// Spoken or sung vocal material.
    Vocals,
    /// Sustained harmonic pads, piano tails, and reverberant material.
    PadsSustains,
    /// Full stereo mixes with dense cross-band interaction.
    FullMix,
    /// Material rendered against tempo ramps and dynamic ratio curves.
    TempoRamp,
    /// Looping material with boundary and warp-marker seam pressure.
    LoopSeam,
    /// Material that exercises wide stretch ratios and degradation policy.
    ExtremeRatio,
}

/// Source/provenance class for a stretch benchmark case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCorpusSource {
    /// Generated in the test harness.
    Synthetic,
    /// Repository-local fixture with checked-in license/provenance.
    LocalFixture,
    /// External benchmark output used only as comparison evidence.
    ExternalBenchmark,
    /// Operator-provided licensed listening material.
    LicensedListening,
}

/// One required stretch benchmark case blueprint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchCorpusCase {
    /// Stable case identifier.
    pub case_id: &'static str,
    /// Required material family.
    pub family: StretchCorpusFamily,
    /// Provenance class.
    pub source: StretchCorpusSource,
    /// Output/input duration ratios this case must exercise.
    pub ratios: &'static [f64],
    /// What artifact the case is intended to expose.
    pub intent: &'static str,
}

/// How a corpus manifest entry expects source audio to be provided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCorpusAssetRequirement {
    /// The case is generated in Signal and requires no file I/O.
    InlineSynthetic,
    /// The operator supplies licensed source audio outside the repository.
    OperatorProvidedAudio,
    /// External rendered output may be supplied for clean-room comparison.
    OptionalExternalBenchmark,
}

/// Behavior when one manifest entry's source asset is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCorpusMissingAssetBehavior {
    /// Generate the source inside Signal.
    GenerateInlineSynthetic,
    /// Record the missing asset in the report and skip the case.
    ReportMissingAndSkipCase,
    /// Skip an optional external benchmark comparison.
    SkipOptionalBenchmark,
}

/// Source-audio policy attached to the stretch corpus manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchCorpusSourcePolicy {
    /// Policy for generated source material.
    pub synthetic_audio_policy: &'static str,
    /// Policy for operator-provided licensed source audio.
    pub licensed_audio_policy: &'static str,
    /// Policy for external benchmark renders.
    pub external_benchmark_policy: &'static str,
}

/// One concrete manifest entry used by real-corpus report runners.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchCorpusManifestEntry {
    /// Stable benchmark case blueprint.
    pub case: StretchCorpusCase,
    /// How the source audio is supplied.
    pub asset_requirement: StretchCorpusAssetRequirement,
    /// What a report runner should do if the asset is unavailable.
    pub missing_asset_behavior: StretchCorpusMissingAssetBehavior,
    /// Stable location hint for operator-supplied or generated source audio.
    pub source_path_hint: &'static str,
    /// Human-readable license/provenance rule for the source.
    pub provenance_note: &'static str,
}

/// Checked-in stretch corpus manifest shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchCorpusManifest {
    /// Stable manifest id.
    pub manifest_id: &'static str,
    /// Manifest schema version.
    pub schema_version: u32,
    /// Expected source sample rate for comparable corpus runs.
    pub sample_rate_hz: u32,
    /// Expected source channel count for comparable corpus runs.
    pub channels: u16,
    /// Source-audio policy.
    pub source_policy: StretchCorpusSourcePolicy,
    /// Manifest entries.
    pub entries: &'static [StretchCorpusManifestEntry],
}

const STANDARD_RATIOS: &[f64] = &[0.75, 1.25, 1.5];
const RAMP_RATIOS: &[f64] = &[0.75, 1.0, 1.5];
const LOOP_RATIOS: &[f64] = &[0.5, 1.0, 2.0];
const EXTREME_RATIOS: &[f64] = &[0.5, 0.75, 1.5, 2.0];

/// Required benchmark corpus blueprint for Signal-owned stretch promotion.
pub const STRETCH_BENCHMARK_CORPUS: [StretchCorpusCase; 8] = [
    StretchCorpusCase {
        case_id: "stretch:drums_percussion",
        family: StretchCorpusFamily::DrumsPercussion,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "transient preservation, cymbal texture, kick timing",
    },
    StretchCorpusCase {
        case_id: "stretch:bass",
        family: StretchCorpusFamily::Bass,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "low-frequency stability, pluck attack preservation",
    },
    StretchCorpusCase {
        case_id: "stretch:vocals",
        family: StretchCorpusFamily::Vocals,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "consonants, breath noise, vibrato, formant-adjacent artifacts",
    },
    StretchCorpusCase {
        case_id: "stretch:pads_sustains",
        family: StretchCorpusFamily::PadsSustains,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "phasiness, beating, reverb-tail stability",
    },
    StretchCorpusCase {
        case_id: "stretch:full_mix",
        family: StretchCorpusFamily::FullMix,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "cross-band coherence and stereo image stability",
    },
    StretchCorpusCase {
        case_id: "stretch:tempo_ramp",
        family: StretchCorpusFamily::TempoRamp,
        source: StretchCorpusSource::Synthetic,
        ratios: RAMP_RATIOS,
        intent: "dynamic-ratio drift and automation alignment",
    },
    StretchCorpusCase {
        case_id: "stretch:loop_seam",
        family: StretchCorpusFamily::LoopSeam,
        source: StretchCorpusSource::Synthetic,
        ratios: LOOP_RATIOS,
        intent: "loop-boundary click and warp-marker seam behavior",
    },
    StretchCorpusCase {
        case_id: "stretch:extreme_ratio",
        family: StretchCorpusFamily::ExtremeRatio,
        source: StretchCorpusSource::Synthetic,
        ratios: EXTREME_RATIOS,
        intent: "wide-ratio quality and out-of-support degradation policy",
    },
];

/// Source-audio policy for the first real stretch corpus manifest.
pub const STRETCH_CORPUS_SOURCE_POLICY: StretchCorpusSourcePolicy = StretchCorpusSourcePolicy {
    synthetic_audio_policy: "Synthetic cases are generated by Signal and need no checked-in audio.",
    licensed_audio_policy: "Licensed listening material is operator-provided outside the repository; do not commit source audio.",
    external_benchmark_policy: "External benchmark renders are optional clean-room comparison outputs, never source material.",
};

/// Manifest entries for the first real stretch corpus.
pub const STRETCH_CORPUS_MANIFEST_ENTRIES: [StretchCorpusManifestEntry; 8] = [
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[0],
        asset_requirement: StretchCorpusAssetRequirement::OperatorProvidedAudio,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase,
        source_path_hint: "fixtures/stretch-corpus/licensed-listening/drums-percussion/",
        provenance_note: "operator must provide licensed transient-heavy drum/percussion material",
    },
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[1],
        asset_requirement: StretchCorpusAssetRequirement::OperatorProvidedAudio,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase,
        source_path_hint: "fixtures/stretch-corpus/licensed-listening/bass/",
        provenance_note: "operator must provide licensed bass material",
    },
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[2],
        asset_requirement: StretchCorpusAssetRequirement::OperatorProvidedAudio,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase,
        source_path_hint: "fixtures/stretch-corpus/licensed-listening/vocals/",
        provenance_note: "operator must provide licensed vocal material",
    },
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[3],
        asset_requirement: StretchCorpusAssetRequirement::OperatorProvidedAudio,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase,
        source_path_hint: "fixtures/stretch-corpus/licensed-listening/pads-sustains/",
        provenance_note: "operator must provide licensed sustained harmonic material",
    },
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[4],
        asset_requirement: StretchCorpusAssetRequirement::OperatorProvidedAudio,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase,
        source_path_hint: "fixtures/stretch-corpus/licensed-listening/full-mix/",
        provenance_note: "operator must provide licensed full-mix material",
    },
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[5],
        asset_requirement: StretchCorpusAssetRequirement::InlineSynthetic,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::GenerateInlineSynthetic,
        source_path_hint: "inline:tempo-ramp",
        provenance_note: "generated by Signal",
    },
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[6],
        asset_requirement: StretchCorpusAssetRequirement::InlineSynthetic,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::GenerateInlineSynthetic,
        source_path_hint: "inline:loop-seam",
        provenance_note: "generated by Signal",
    },
    StretchCorpusManifestEntry {
        case: STRETCH_BENCHMARK_CORPUS[7],
        asset_requirement: StretchCorpusAssetRequirement::InlineSynthetic,
        missing_asset_behavior: StretchCorpusMissingAssetBehavior::GenerateInlineSynthetic,
        source_path_hint: "inline:extreme-ratio",
        provenance_note: "generated by Signal",
    },
];

/// First real stretch corpus manifest.
pub const STRETCH_CORPUS_MANIFEST: StretchCorpusManifest = StretchCorpusManifest {
    manifest_id: "stretch-corpus-v1",
    schema_version: 1,
    sample_rate_hz: 48_000,
    channels: 2,
    source_policy: STRETCH_CORPUS_SOURCE_POLICY,
    entries: &STRETCH_CORPUS_MANIFEST_ENTRIES,
};

/// Inline synthetic audio generated for stretch benchmark bootstrap cases.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchSyntheticAudio {
    /// Sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Interleaved channel count.
    pub channels: u16,
    /// Interleaved sample frames.
    pub samples: Vec<Sample>,
}

impl StretchSyntheticAudio {
    /// Number of sample frames.
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels.max(1) as usize
    }
}

/// Generate the synthetic benchmark audio for a corpus family.
pub fn generate_synthetic_stretch_audio(
    family: StretchCorpusFamily,
) -> Option<StretchSyntheticAudio> {
    match family {
        StretchCorpusFamily::TempoRamp => Some(synthetic_tempo_ramp()),
        StretchCorpusFamily::LoopSeam => Some(synthetic_loop_seam()),
        StretchCorpusFamily::ExtremeRatio => Some(synthetic_extreme_ratio()),
        _ => None,
    }
}

/// Generate all inline synthetic benchmark cases declared in the corpus
/// blueprint.
pub fn synthetic_stretch_corpus_cases() -> Vec<(StretchCorpusCase, StretchSyntheticAudio)> {
    STRETCH_BENCHMARK_CORPUS
        .iter()
        .filter_map(|case| {
            if case.source == StretchCorpusSource::Synthetic {
                generate_synthetic_stretch_audio(case.family).map(|audio| (*case, audio))
            } else {
                None
            }
        })
        .collect()
}

/// Objective metric family used by the stretch benchmark harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchMetric {
    /// Absolute output-length or segment-boundary drift in samples.
    TimingDriftSamples,
    /// Attack widening around detected transients, in frames.
    TransientSmearFrames,
    /// Inter-bin or peak-neighborhood phase-coherence delta.
    VerticalCoherenceDelta,
    /// Mid/side or channel-correlation image delta.
    StereoImageDelta,
    /// Highest click or discontinuity at a loop boundary, in dBFS.
    LoopBoundaryClickDbfs,
    /// Highest click or discontinuity at a dynamic-ratio segment boundary, in
    /// dBFS.
    DynamicSegmentSeamClickDbfs,
    /// CPU time relative to rendered audio duration.
    CpuRealtimeFactor,
    /// Reported algorithmic latency, in frames.
    LatencyFrames,
    /// Peak memory used by the render, in bytes.
    PeakMemoryBytes,
    /// Absolute pitch error from the requested pitch shift, in cents.
    PitchErrorCents,
}

/// Severity for one stretch metric limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchAcceptanceSeverity {
    /// Limit breach should be visible but not fail the run.
    Warn,
    /// Limit breach fails the run.
    Fail,
}

/// Aggregated result for one metric or report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchAcceptanceStatus {
    /// All limits passed.
    Pass,
    /// At least one warning limit breached and no failure limit breached.
    Warn,
    /// At least one failure limit breached.
    Fail,
}

/// One measured stretch benchmark metric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchMetricValue {
    /// Metric identity.
    pub metric: StretchMetric,
    /// Metric value. Limits interpret the value as "lower is better".
    pub value: f64,
}

impl StretchMetricValue {
    /// Construct a metric value.
    pub fn new(metric: StretchMetric, value: f64) -> Self {
        Self { metric, value }
    }
}

/// Benchmark backend identity used for draft-vs-prototype comparison reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBenchmarkBackend {
    /// Current independent-bin phase-vocoder baseline.
    Draft,
    /// RealtimePreview prototype path.
    RealtimePreviewPrototype,
    /// OfflineHighQuality prototype path: identity phase locking plus
    /// transient phase resets.
    OfflineHighQualityPrototype,
}

/// Execution path used for one synthetic benchmark comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBenchmarkPath {
    /// Fixed-ratio mono or independent-channel stereo stretch.
    FixedRatio,
    /// Identity phase-locked sustained-material path.
    PhaseLocked,
    /// Linked-stereo candidate path.
    LinkedStereo,
    /// Stepwise dynamic-ratio candidate path.
    DynamicRatio,
    /// Independent tempo plus pitch-shift candidate path.
    PitchShift,
}

/// Direction of one metric comparison. All stretch metrics in this harness are
/// lower-is-better.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBenchmarkComparisonOutcome {
    /// The prototype value is lower than the draft value.
    Improved,
    /// The prototype value is higher than the draft value.
    Regressed,
    /// The values are equal within the report tolerance.
    Unchanged,
    /// One or both values were not finite.
    Inconclusive,
}

/// Quality work area identified from a benchmark metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchQualityWorkArea {
    /// Output duration or marker alignment.
    TimingAlignment,
    /// Transient preservation and attack width.
    TransientPreservation,
    /// Vertical phase coherence for sustained material.
    VerticalCoherence,
    /// Stereo image and mid/side stability.
    StereoImageStability,
    /// Loop boundary click control.
    LoopBoundaryClicks,
    /// Dynamic-ratio segment seam smoothing.
    DynamicRatioSeams,
    /// Independent pitch-shift accuracy.
    PitchShiftAccuracy,
    /// CPU, latency, or memory budget.
    ResourceBudget,
}

/// One synthetic-corpus metric comparison between a baseline and prototype
/// output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchSyntheticBenchmarkComparison {
    /// Corpus case identifier.
    pub case_id: &'static str,
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Metric identity.
    pub metric: StretchMetric,
    /// Execution path under measurement.
    pub path: StretchBenchmarkPath,
    /// Requested pitch shift in semitones, when this row measures pitch-shift
    /// behavior.
    pub pitch_shift_semitones: Option<f64>,
    /// Baseline backend measured.
    pub baseline_backend: StretchBenchmarkBackend,
    /// Candidate backend measured.
    pub candidate_backend: StretchBenchmarkBackend,
    /// Baseline metric value.
    pub baseline_value: f64,
    /// Candidate metric value.
    pub candidate_value: f64,
    /// Candidate minus baseline. Negative means improvement.
    pub delta: f64,
    /// Direction of the comparison.
    pub outcome: StretchBenchmarkComparisonOutcome,
}

/// One prioritized quality-tuning item derived from comparison evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchQualityPriority {
    /// Work area the metric maps to.
    pub area: StretchQualityWorkArea,
    /// Corpus case identifier.
    pub case_id: &'static str,
    /// Execution path under measurement.
    pub path: StretchBenchmarkPath,
    /// Metric identity.
    pub metric: StretchMetric,
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Requested pitch shift in semitones, when present.
    pub pitch_shift_semitones: Option<f64>,
    /// Baseline metric value.
    pub baseline_value: f64,
    /// Candidate metric value.
    pub candidate_value: f64,
    /// Candidate minus baseline. Positive values are regressions.
    pub delta: f64,
    /// Comparison outcome that caused this priority.
    pub outcome: StretchBenchmarkComparisonOutcome,
    /// Normalized sorting score for this work item. Higher means more urgent.
    pub priority_score: f64,
}

/// Aggregate synthetic-corpus comparison report for one prototype against the
/// draft baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchSyntheticBenchmarkComparisonReport {
    /// Per-case, per-ratio, per-metric comparisons.
    pub comparisons: Vec<StretchSyntheticBenchmarkComparison>,
    /// Number of improved metric comparisons.
    pub improved_count: usize,
    /// Number of regressed metric comparisons.
    pub regressed_count: usize,
    /// Number of unchanged metric comparisons.
    pub unchanged_count: usize,
    /// Number of inconclusive metric comparisons.
    pub inconclusive_count: usize,
}

/// Draft-vs-prototype sustained-material coherence measurement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchCoherenceComparison {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Draft independent-bin phase-vocoder coherence score. Lower is better.
    pub draft_vertical_coherence_score: f64,
    /// Identity phase-locked prototype coherence score. Lower is better.
    pub phase_locked_vertical_coherence_score: f64,
    /// Gap metric reported as locked score minus draft score.
    pub metric: StretchMetricValue,
}

/// Loop-boundary click measurement for one rendered loop candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchLoopBoundaryMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Number of interleaved channels measured.
    pub channels: u16,
    /// Worst absolute boundary discontinuity.
    pub peak_boundary_delta: f64,
    /// Boundary discontinuity converted to dBFS.
    pub click_dbfs: f64,
    /// Metric reported to the acceptance harness.
    pub metric: StretchMetricValue,
}

/// Dynamic-ratio segment seam click measurement for one rendered output.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchDynamicSegmentSeamMeasurement {
    /// Effective output/input duration ratio measured.
    pub ratio: f64,
    /// Number of interleaved channels measured.
    pub channels: u16,
    /// Output-frame seam positions measured.
    pub seam_frames: Vec<usize>,
    /// Worst absolute discontinuity across any measured seam.
    pub peak_seam_delta: f64,
    /// Seam discontinuity converted to dBFS.
    pub click_dbfs: f64,
    /// Metric reported to the comparison harness.
    pub metric: StretchMetricValue,
}

/// Pitch-shift accuracy measurement for one rendered output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchPitchShiftMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Requested pitch shift in semitones.
    pub pitch_shift_semitones: f64,
    /// Expected dominant frequency after shifting.
    pub expected_frequency_hz: f64,
    /// Measured dominant frequency in the rendered output.
    pub measured_frequency_hz: f64,
    /// Absolute pitch error from the requested shift, in cents.
    pub pitch_error_cents: f64,
    /// Metric reported to the comparison harness.
    pub metric: StretchMetricValue,
}

/// Stereo image movement measurement for one rendered stretch output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchStereoImageMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Input left-right correlation.
    pub input_correlation: f64,
    /// Output left-right correlation.
    pub output_correlation: f64,
    /// Input side/mid RMS ratio.
    pub input_side_mid_ratio: f64,
    /// Output side/mid RMS ratio.
    pub output_side_mid_ratio: f64,
    /// Combined image delta. Lower is better.
    pub image_delta: f64,
    /// Metric reported to the acceptance harness.
    pub metric: StretchMetricValue,
}

/// Compare sustained-material vertical coherence for the draft baseline and
/// the identity phase-locked prototype.
///
/// The metric value is `phase_locked_score - draft_score`; negative values
/// mean the locked prototype improved the measured phase-curvature score.
/// Positive values log the measured gap without promoting the prototype.
pub fn compare_sustained_material_coherence(ratio: f64) -> StretchCoherenceComparison {
    if !ratio.is_finite() || ratio <= 0.0 {
        return StretchCoherenceComparison {
            ratio,
            draft_vertical_coherence_score: f64::NAN,
            phase_locked_vertical_coherence_score: f64::NAN,
            metric: StretchMetricValue::new(StretchMetric::VerticalCoherenceDelta, f64::NAN),
        };
    }

    const WINDOW_SIZE: usize = 2_048;
    const ANALYSIS_HOP: usize = WINDOW_SIZE / 4;
    let input = synthetic_sustained_material();
    let target_len = (input.len() as f64 * ratio).round() as usize;
    let draft = phase_vocoder(&input, target_len, ratio, WINDOW_SIZE, ANALYSIS_HOP);
    let phase_locked =
        phase_locked_phase_vocoder(&input, target_len, ratio, WINDOW_SIZE, ANALYSIS_HOP);
    let draft_score = peak_neighborhood_phase_curvature(&draft, WINDOW_SIZE, ANALYSIS_HOP);
    let phase_locked_score =
        peak_neighborhood_phase_curvature(&phase_locked, WINDOW_SIZE, ANALYSIS_HOP);
    let gap = phase_locked_score - draft_score;

    StretchCoherenceComparison {
        ratio,
        draft_vertical_coherence_score: draft_score,
        phase_locked_vertical_coherence_score: phase_locked_score,
        metric: StretchMetricValue::new(StretchMetric::VerticalCoherenceDelta, gap),
    }
}

/// Measure draft phase-vocoder transient smear on the synthetic extreme-ratio
/// corpus case.
pub fn measure_draft_transient_smear(ratio: f64) -> StretchTransientSmearMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return transient_smear_nan(ratio);
    }

    const WINDOW_SIZE: usize = 1_024;
    const HOP_SIZE: usize = 256;
    let input = synthetic_extreme_ratio().samples;
    let target_len = (input.len() as f64 * ratio).round() as usize;
    let output = phase_vocoder(&input, target_len, ratio, 2_048, 512);
    measure_transient_smear(
        &input,
        &output,
        ratio,
        WINDOW_SIZE,
        HOP_SIZE,
        StretchTransientSmearPolicies::production(),
    )
}

/// Measure transient-reset prototype smear on the synthetic extreme-ratio
/// corpus case.
pub fn measure_transient_reset_transient_smear(ratio: f64) -> StretchTransientSmearMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return transient_smear_nan(ratio);
    }

    const WINDOW_SIZE: usize = 1_024;
    const HOP_SIZE: usize = 256;
    let input = synthetic_extreme_ratio().samples;
    let target_len = (input.len() as f64 * ratio).round() as usize;
    let output = transient_reset_phase_vocoder(&input, target_len, ratio, 2_048, 512);
    measure_transient_smear(
        &input,
        &output,
        ratio,
        WINDOW_SIZE,
        HOP_SIZE,
        StretchTransientSmearPolicies::production(),
    )
}

/// Measure a loop-boundary click as the final-to-first-frame discontinuity.
pub fn measure_loop_boundary_click(
    interleaved_samples: &[Sample],
    channels: u16,
    ratio: f64,
) -> StretchLoopBoundaryMeasurement {
    let channel_count = channels as usize;
    if channel_count == 0 || interleaved_samples.len() < channel_count * 2 {
        return loop_boundary_nan(ratio, channels);
    }
    let frames = interleaved_samples.len() / channel_count;
    if frames < 2 {
        return loop_boundary_nan(ratio, channels);
    }

    let first = &interleaved_samples[..channel_count];
    let last_start = (frames - 1) * channel_count;
    let last = &interleaved_samples[last_start..last_start + channel_count];
    let peak_delta = first
        .iter()
        .zip(last.iter())
        .map(|(left, right)| (left - right).abs() as f64)
        .fold(0.0f64, f64::max);
    let click_dbfs = amplitude_to_dbfs(peak_delta);

    StretchLoopBoundaryMeasurement {
        ratio,
        channels,
        peak_boundary_delta: peak_delta,
        click_dbfs,
        metric: StretchMetricValue::new(StretchMetric::LoopBoundaryClickDbfs, click_dbfs),
    }
}

/// Frames either side of a seam that the measurement inspects.
///
/// Must exceed `DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES` (`256`), because a seam
/// smoother does not remove a discontinuity — it spreads it over its fade, and
/// a window narrower than the fade would miss where it went.
const SEAM_MEASUREMENT_WINDOW_FRAMES: usize = 384;

/// Measure dynamic-ratio segment seam clicks: the worst first-difference near
/// each output seam, in excess of the same render's own first-difference floor.
///
/// PREVIOUSLY, AND WRONGLY, this read exactly `|x[seam - 1] - x[seam]|`, the
/// single pair straddling the boundary. `smooth_dynamic_segment_boundaries_
/// interleaved` sets both of those samples to their midpoint on its first
/// iteration, so that pair is equal *by construction* whenever the smoother
/// ran. Measured: a full-scale `+1 -> -1` seam reads `6.0 dBFS` raw and
/// `-240.0 dBFS` (the silence sentinel) after smoothing with a one-frame fade
/// that leaves the step entirely intact. The metric reported whether the
/// smoother ran, not whether there was a seam, and a perfect score on a
/// catastrophic discontinuity is what let the seam survive being measured.
///
/// The window is what fixes the displacement. The floor is what stops ordinary
/// signal motion reading as a click: adjacent samples of any non-DC material
/// differ, so an absolute threshold cannot separate a seam from a waveform. The
/// floor is the `p99.9` first-difference taken from frames outside every seam
/// window — the render's own idea of a large step — and only the excess over it
/// is reported.
pub fn measure_dynamic_segment_seam_click(
    interleaved_samples: &[Sample],
    channels: u16,
    seam_frames: &[usize],
    ratio: f64,
) -> StretchDynamicSegmentSeamMeasurement {
    let channel_count = channels as usize;
    if channel_count == 0 || interleaved_samples.len() < channel_count * 2 {
        return dynamic_segment_seam_nan(ratio, channels);
    }

    let frames = interleaved_samples.len() / channel_count;
    let measured_seams: Vec<usize> = seam_frames
        .iter()
        .copied()
        .filter(|seam| *seam > 0 && *seam < frames)
        .collect();
    if measured_seams.is_empty() {
        return dynamic_segment_seam_nan(ratio, channels);
    }

    let near_a_seam = |frame: usize| {
        measured_seams.iter().any(|seam| {
            frame + SEAM_MEASUREMENT_WINDOW_FRAMES >= *seam
                && *seam + SEAM_MEASUREMENT_WINDOW_FRAMES >= frame
        })
    };
    let step_at = |frame: usize| {
        (0..channel_count)
            .map(|channel| {
                let current = interleaved_samples[frame * channel_count + channel];
                let previous = interleaved_samples[(frame - 1) * channel_count + channel];
                (current - previous).abs() as f64
            })
            .fold(0.0f64, f64::max)
    };

    let mut background: Vec<f64> = (1..frames)
        .filter(|frame| !near_a_seam(*frame))
        .map(step_at)
        .collect();
    // A render entirely inside its own seam windows has no background, and
    // there is then no way to tell a seam from the waveform. Unmeasurable, not
    // zero: falling back to all steps would let the seam set its own floor and
    // score itself perfect, which is the failure this measurement replaced.
    if background.is_empty() {
        return dynamic_segment_seam_nan(ratio, channels);
    }
    background.sort_by(|left, right| left.partial_cmp(right).expect("finite steps"));
    let floor = background[((background.len() as f64 * 0.999) as usize).min(background.len() - 1)];

    let mut peak_delta = 0.0f64;
    for seam in &measured_seams {
        let low = seam.saturating_sub(SEAM_MEASUREMENT_WINDOW_FRAMES).max(1);
        let high = (*seam + SEAM_MEASUREMENT_WINDOW_FRAMES).min(frames);
        for frame in low..high {
            peak_delta = peak_delta.max(step_at(frame) - floor);
        }
    }
    let peak_delta = peak_delta.max(0.0);

    let click_dbfs = amplitude_to_dbfs(peak_delta);
    StretchDynamicSegmentSeamMeasurement {
        ratio,
        channels,
        seam_frames: measured_seams,
        peak_seam_delta: peak_delta,
        click_dbfs,
        metric: StretchMetricValue::new(StretchMetric::DynamicSegmentSeamClickDbfs, click_dbfs),
    }
}

/// Measure pitch-shift accuracy from the dominant frequency in a rendered
/// output.
pub fn measure_pitch_shift_error_cents(
    output_samples: &[Sample],
    sample_rate_hz: u32,
    source_frequency_hz: f64,
    pitch_shift_semitones: f64,
    ratio: f64,
) -> StretchPitchShiftMeasurement {
    if output_samples.is_empty()
        || sample_rate_hz == 0
        || !source_frequency_hz.is_finite()
        || source_frequency_hz <= 0.0
        || !pitch_shift_semitones.is_finite()
        || !ratio.is_finite()
        || ratio <= 0.0
    {
        return pitch_shift_nan(ratio, pitch_shift_semitones, source_frequency_hz);
    }

    let expected_frequency_hz = source_frequency_hz * 2.0f64.powf(pitch_shift_semitones / 12.0);
    let measured_frequency_hz = dominant_frequency_hz(output_samples, sample_rate_hz);
    if !expected_frequency_hz.is_finite()
        || expected_frequency_hz <= 0.0
        || !measured_frequency_hz.is_finite()
        || measured_frequency_hz <= 0.0
    {
        return pitch_shift_nan(ratio, pitch_shift_semitones, source_frequency_hz);
    }

    let pitch_error_cents = (1200.0 * (measured_frequency_hz / expected_frequency_hz).log2()).abs();
    StretchPitchShiftMeasurement {
        ratio,
        pitch_shift_semitones,
        expected_frequency_hz,
        measured_frequency_hz,
        pitch_error_cents,
        metric: StretchMetricValue::new(StretchMetric::PitchErrorCents, pitch_error_cents),
    }
}

/// Measure draft phase-vocoder loop-boundary click on the synthetic loop-seam
/// corpus case.
pub fn measure_draft_loop_boundary_click(ratio: f64) -> StretchLoopBoundaryMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return loop_boundary_nan(ratio, 0);
    }

    let input = synthetic_loop_seam();
    let channel_count = input.channels as usize;
    let frame_count = input.frame_count();
    let target_len = (frame_count as f64 * ratio).round() as usize;
    let mut output_channels = Vec::with_capacity(channel_count);
    for channel in 0..channel_count {
        let mono = deinterleave_channel(&input.samples, channel_count, channel);
        output_channels.push(phase_vocoder(&mono, target_len, ratio, 2_048, 512));
    }
    let output = interleave_channels(&output_channels);
    measure_loop_boundary_click(&output, input.channels, ratio)
}

/// Measure transient-reset prototype loop-boundary click on the synthetic
/// loop-seam corpus case.
pub fn measure_transient_reset_loop_boundary_click(ratio: f64) -> StretchLoopBoundaryMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return loop_boundary_nan(ratio, 0);
    }

    let input = synthetic_loop_seam();
    let channel_count = input.channels as usize;
    let frame_count = input.frame_count();
    let target_len = (frame_count as f64 * ratio).round() as usize;
    let mut output_channels = Vec::with_capacity(channel_count);
    for channel in 0..channel_count {
        let mono = deinterleave_channel(&input.samples, channel_count, channel);
        output_channels.push(transient_reset_phase_vocoder(
            &mono, target_len, ratio, 2_048, 512,
        ));
    }
    let mut output = interleave_channels(&output_channels);
    smooth_loop_boundary_interleaved(&mut output, input.channels, 256);
    measure_loop_boundary_click(&output, input.channels, ratio)
}

/// Measure stereo image movement from left-right correlation and mid/side
/// balance deltas. This is measurement-only and does not imply linked
/// synthesis.
pub fn measure_stereo_image_delta(
    input_interleaved: &[Sample],
    output_interleaved: &[Sample],
    ratio: f64,
) -> StretchStereoImageMeasurement {
    if !ratio.is_finite()
        || ratio <= 0.0
        || input_interleaved.len() < 4
        || output_interleaved.len() < 4
        || !input_interleaved.len().is_multiple_of(2)
        || !output_interleaved.len().is_multiple_of(2)
    {
        return stereo_image_nan(ratio);
    }

    let input_stats = stereo_image_stats(input_interleaved);
    let output_stats = stereo_image_stats(output_interleaved);
    if !input_stats.correlation.is_finite()
        || !output_stats.correlation.is_finite()
        || !input_stats.side_mid_ratio.is_finite()
        || !output_stats.side_mid_ratio.is_finite()
    {
        return stereo_image_nan(ratio);
    }

    let image_delta = (output_stats.correlation - input_stats.correlation).abs()
        + (output_stats.side_mid_ratio - input_stats.side_mid_ratio).abs();

    StretchStereoImageMeasurement {
        ratio,
        input_correlation: input_stats.correlation,
        output_correlation: output_stats.correlation,
        input_side_mid_ratio: input_stats.side_mid_ratio,
        output_side_mid_ratio: output_stats.side_mid_ratio,
        image_delta,
        metric: StretchMetricValue::new(StretchMetric::StereoImageDelta, image_delta),
    }
}

/// Measure draft phase-vocoder stereo image movement on the synthetic
/// loop-seam corpus case.
pub fn measure_draft_stereo_image_delta(ratio: f64) -> StretchStereoImageMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return stereo_image_nan(ratio);
    }

    let input = synthetic_loop_seam();
    let output = stretch_stereo_synthetic(&input, ratio, phase_vocoder);
    measure_stereo_image_delta(&input.samples, &output, ratio)
}

/// Measure transient-reset linked-stereo prototype image movement on the
/// synthetic loop-seam corpus case.
pub fn measure_transient_reset_stereo_image_delta(ratio: f64) -> StretchStereoImageMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return stereo_image_nan(ratio);
    }

    let input = synthetic_loop_seam();
    let output = stretch_stereo_synthetic_linked(&input, ratio);
    measure_stereo_image_delta(&input.samples, &output, ratio)
}

/// Compare the OfflineHighQuality prototype against the draft baseline across
/// all repository-local synthetic stretch corpus cases.
///
/// This is a measurement report, not a promotion decision. It intentionally
/// includes both improvements and regressions so later promotion work can tune
/// thresholds from evidence.
pub fn compare_synthetic_stretch_backends() -> StretchSyntheticBenchmarkComparisonReport {
    let mut comparisons = Vec::new();

    for case in STRETCH_BENCHMARK_CORPUS
        .iter()
        .filter(|case| case.source == StretchCorpusSource::Synthetic)
    {
        for ratio in case.ratios {
            comparisons.push(compare_metric(
                case.case_id,
                *ratio,
                StretchMetric::TimingDriftSamples,
                measure_synthetic_length_drift(case.family, *ratio, phase_vocoder),
                measure_synthetic_length_drift(case.family, *ratio, transient_reset_phase_vocoder),
            ));

            match case.family {
                StretchCorpusFamily::LoopSeam => {
                    comparisons.push(compare_metric(
                        case.case_id,
                        *ratio,
                        StretchMetric::LoopBoundaryClickDbfs,
                        measure_draft_loop_boundary_click(*ratio).metric.value,
                        measure_transient_reset_loop_boundary_click(*ratio)
                            .metric
                            .value,
                    ));
                    comparisons.push(
                        compare_metric(
                            case.case_id,
                            *ratio,
                            StretchMetric::StereoImageDelta,
                            measure_draft_stereo_image_delta(*ratio).metric.value,
                            measure_transient_reset_stereo_image_delta(*ratio)
                                .metric
                                .value,
                        )
                        .with_path(StretchBenchmarkPath::LinkedStereo),
                    );
                }
                StretchCorpusFamily::ExtremeRatio => {
                    comparisons.push(compare_metric(
                        case.case_id,
                        *ratio,
                        StretchMetric::TransientSmearFrames,
                        measure_draft_transient_smear(*ratio).metric.value,
                        measure_transient_reset_transient_smear(*ratio).metric.value,
                    ));
                }
                _ => {}
            }
        }

        if case.family == StretchCorpusFamily::TempoRamp {
            comparisons.extend(compare_dynamic_tempo_ramp(case.case_id));
        }
    }
    comparisons.extend(compare_pitch_shift());
    comparisons.extend(compare_sustained_coherence());

    finish_synthetic_benchmark_report(comparisons)
}

/// Compare the RealtimePreview prototype against the draft baseline across
/// the synthetic corpus subset relevant to low-latency preview.
pub fn compare_synthetic_realtime_preview_backends() -> StretchSyntheticBenchmarkComparisonReport {
    let mut comparisons = Vec::new();

    for case in STRETCH_BENCHMARK_CORPUS
        .iter()
        .filter(|case| case.source == StretchCorpusSource::Synthetic)
    {
        for ratio in case.ratios {
            comparisons.push(compare_metric_for_backend(
                case.case_id,
                *ratio,
                StretchMetric::TimingDriftSamples,
                StretchBenchmarkBackend::RealtimePreviewPrototype,
                measure_synthetic_length_drift(case.family, *ratio, phase_vocoder),
                measure_synthetic_length_drift_realtime_preview(case.family, *ratio),
            ));

            match case.family {
                StretchCorpusFamily::LoopSeam => {
                    comparisons.push(compare_metric_for_backend(
                        case.case_id,
                        *ratio,
                        StretchMetric::LoopBoundaryClickDbfs,
                        StretchBenchmarkBackend::RealtimePreviewPrototype,
                        measure_draft_loop_boundary_click(*ratio).metric.value,
                        measure_realtime_preview_loop_boundary_click(*ratio)
                            .metric
                            .value,
                    ));
                    comparisons.push(
                        compare_metric_for_backend(
                            case.case_id,
                            *ratio,
                            StretchMetric::StereoImageDelta,
                            StretchBenchmarkBackend::RealtimePreviewPrototype,
                            measure_draft_stereo_image_delta(*ratio).metric.value,
                            measure_realtime_preview_stereo_image_delta(*ratio)
                                .metric
                                .value,
                        )
                        .with_path(StretchBenchmarkPath::LinkedStereo),
                    );
                }
                StretchCorpusFamily::ExtremeRatio => {
                    comparisons.push(compare_metric_for_backend(
                        case.case_id,
                        *ratio,
                        StretchMetric::TransientSmearFrames,
                        StretchBenchmarkBackend::RealtimePreviewPrototype,
                        measure_draft_transient_smear(*ratio).metric.value,
                        measure_realtime_preview_transient_smear(*ratio)
                            .metric
                            .value,
                    ));
                }
                _ => {}
            }
        }

        if case.family == StretchCorpusFamily::TempoRamp {
            comparisons.extend(compare_dynamic_tempo_ramp_realtime_preview(case.case_id));
        }
    }
    comparisons.extend(compare_pitch_shift_realtime_preview());

    finish_synthetic_benchmark_report(comparisons)
}

fn finish_synthetic_benchmark_report(
    comparisons: Vec<StretchSyntheticBenchmarkComparison>,
) -> StretchSyntheticBenchmarkComparisonReport {
    let mut report = StretchSyntheticBenchmarkComparisonReport {
        comparisons,
        improved_count: 0,
        regressed_count: 0,
        unchanged_count: 0,
        inconclusive_count: 0,
    };
    for comparison in &report.comparisons {
        match comparison.outcome {
            StretchBenchmarkComparisonOutcome::Improved => report.improved_count += 1,
            StretchBenchmarkComparisonOutcome::Regressed => report.regressed_count += 1,
            StretchBenchmarkComparisonOutcome::Unchanged => report.unchanged_count += 1,
            StretchBenchmarkComparisonOutcome::Inconclusive => report.inconclusive_count += 1,
        }
    }
    report
}

/// Rank quality-tuning work from comparison evidence.
///
/// Only regressions and inconclusive rows become priorities. Lower-is-better
/// metric values are normalized by metric family so the result is useful for
/// ordering, not for acceptance.
pub fn prioritize_stretch_quality_work(
    report: &StretchSyntheticBenchmarkComparisonReport,
    limit: usize,
) -> Vec<StretchQualityPriority> {
    let mut priorities = report
        .comparisons
        .iter()
        .filter_map(priority_from_comparison)
        .collect::<Vec<_>>();
    priorities.sort_by(|left, right| {
        right
            .priority_score
            .total_cmp(&left.priority_score)
            .then_with(|| left.case_id.cmp(right.case_id))
            .then_with(|| format!("{:?}", left.metric).cmp(&format!("{:?}", right.metric)))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
    });
    priorities.truncate(limit);
    priorities
}

/// Upper-bound limit for a stretch benchmark metric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchMetricLimit {
    /// Metric identity.
    pub metric: StretchMetric,
    /// Inclusive upper bound.
    pub max: f64,
    /// Severity to report when the value exceeds `max` or is not finite.
    pub severity: StretchAcceptanceSeverity,
}

impl StretchMetricLimit {
    /// Construct a metric upper-bound limit.
    pub fn max(metric: StretchMetric, max: f64, severity: StretchAcceptanceSeverity) -> Self {
        Self {
            metric,
            max,
            severity,
        }
    }
}

/// Assessment for one metric limit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchMetricAssessment {
    /// Metric identity.
    pub metric: StretchMetric,
    /// Measured value, or `NaN` when the metric was missing.
    pub value: f64,
    /// Inclusive upper bound.
    pub max: f64,
    /// Result for this metric.
    pub status: StretchAcceptanceStatus,
}

/// Assessment report for a stretch benchmark case or tier.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchAcceptanceReport {
    /// Aggregated worst status.
    pub status: StretchAcceptanceStatus,
    /// Metric assessments in limit order.
    pub metrics: Vec<StretchMetricAssessment>,
}

/// Evaluate measured stretch metrics against upper-bound limits.
pub fn assess_stretch_metrics(
    measurements: &[StretchMetricValue],
    limits: &[StretchMetricLimit],
) -> StretchAcceptanceReport {
    let mut status = StretchAcceptanceStatus::Pass;
    let mut metrics = Vec::with_capacity(limits.len());
    for limit in limits {
        let value = measurements
            .iter()
            .find(|measurement| measurement.metric == limit.metric)
            .map(|measurement| measurement.value)
            .unwrap_or(f64::NAN);
        let metric_status = if value.is_finite() && value <= limit.max {
            StretchAcceptanceStatus::Pass
        } else {
            severity_to_stretch_status(limit.severity)
        };
        status = combine_stretch_status(status, metric_status);
        metrics.push(StretchMetricAssessment {
            metric: limit.metric,
            value,
            max: limit.max,
            status: metric_status,
        });
    }

    StretchAcceptanceReport { status, metrics }
}

/// Absolute output-length drift from the fixed-ratio length contract.
pub fn output_length_drift_samples(input_frames: usize, output_frames: usize, ratio: f64) -> f64 {
    if !ratio.is_finite() || ratio <= 0.0 {
        return f64::NAN;
    }
    let expected = (input_frames as f64 * ratio).round();
    (output_frames as f64 - expected).abs()
}

/// Deterministic line-oriented report for one stretch acceptance result.
pub fn format_stretch_acceptance_report(case_id: &str, report: &StretchAcceptanceReport) -> String {
    let mut lines = Vec::with_capacity(report.metrics.len() + 1);
    lines.push(format!("case={case_id} status={:?}", report.status));
    for metric in &report.metrics {
        lines.push(format!(
            "metric={:?} value={:.6} max={:.6} status={:?}",
            metric.metric, metric.value, metric.max, metric.status
        ));
    }
    lines.join("\n")
}

/// Deterministic line-oriented report for synthetic baseline-vs-prototype
/// benchmark comparisons.
pub fn format_synthetic_stretch_comparison_report(
    report: &StretchSyntheticBenchmarkComparisonReport,
) -> String {
    let mut lines = Vec::with_capacity(report.comparisons.len() + 1);
    lines.push(format!(
        "synthetic_stretch_comparison improved={} regressed={} unchanged={} inconclusive={}",
        report.improved_count,
        report.regressed_count,
        report.unchanged_count,
        report.inconclusive_count
    ));
    for comparison in &report.comparisons {
        let pitch_shift = comparison
            .pitch_shift_semitones
            .map(|semitones| format!("{semitones:.6}"))
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "case={} ratio={:.6} path={:?} pitch_shift={} metric={:?} baseline_backend={:?} candidate_backend={:?} baseline={:.6} candidate={:.6} delta={:.6} outcome={:?}",
            comparison.case_id,
            comparison.ratio,
            comparison.path,
            pitch_shift,
            comparison.metric,
            comparison.baseline_backend,
            comparison.candidate_backend,
            comparison.baseline_value,
            comparison.candidate_value,
            comparison.delta,
            comparison.outcome
        ));
    }
    lines.join("\n")
}

/// Deterministic line-oriented report for prioritized stretch quality work.
pub fn format_stretch_quality_priority_report(priorities: &[StretchQualityPriority]) -> String {
    let mut lines = Vec::with_capacity(priorities.len() + 1);
    lines.push(format!(
        "stretch_quality_priorities count={}",
        priorities.len()
    ));
    for priority in priorities {
        let pitch_shift = priority
            .pitch_shift_semitones
            .map(|semitones| format!("{semitones:.6}"))
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "area={:?} case={} ratio={:.6} path={:?} pitch_shift={} metric={:?} baseline={:.6} candidate={:.6} delta={:.6} outcome={:?} score={:.6}",
            priority.area,
            priority.case_id,
            priority.ratio,
            priority.path,
            pitch_shift,
            priority.metric,
            priority.baseline_value,
            priority.candidate_value,
            priority.delta,
            priority.outcome,
            priority.priority_score
        ));
    }
    lines.join("\n")
}

fn compare_metric(
    case_id: &'static str,
    ratio: f64,
    metric: StretchMetric,
    baseline_value: f64,
    candidate_value: f64,
) -> StretchSyntheticBenchmarkComparison {
    compare_metric_for_backend(
        case_id,
        ratio,
        metric,
        StretchBenchmarkBackend::OfflineHighQualityPrototype,
        baseline_value,
        candidate_value,
    )
}

fn compare_metric_for_backend(
    case_id: &'static str,
    ratio: f64,
    metric: StretchMetric,
    candidate_backend: StretchBenchmarkBackend,
    baseline_value: f64,
    candidate_value: f64,
) -> StretchSyntheticBenchmarkComparison {
    let delta = candidate_value - baseline_value;
    let outcome =
        if !baseline_value.is_finite() || !candidate_value.is_finite() || !delta.is_finite() {
            StretchBenchmarkComparisonOutcome::Inconclusive
        } else if delta < -comparison_tolerance(metric) {
            StretchBenchmarkComparisonOutcome::Improved
        } else if delta > comparison_tolerance(metric) {
            StretchBenchmarkComparisonOutcome::Regressed
        } else {
            StretchBenchmarkComparisonOutcome::Unchanged
        };

    StretchSyntheticBenchmarkComparison {
        case_id,
        ratio,
        metric,
        path: StretchBenchmarkPath::FixedRatio,
        pitch_shift_semitones: None,
        baseline_backend: StretchBenchmarkBackend::Draft,
        candidate_backend,
        baseline_value,
        candidate_value,
        delta,
        outcome,
    }
}

fn comparison_tolerance(metric: StretchMetric) -> f64 {
    match metric {
        StretchMetric::TransientSmearFrames => 1.0,
        _ => 1.0e-9,
    }
}

fn priority_from_comparison(
    comparison: &StretchSyntheticBenchmarkComparison,
) -> Option<StretchQualityPriority> {
    let priority_score = match comparison.outcome {
        StretchBenchmarkComparisonOutcome::Regressed => {
            priority_score(comparison.metric, comparison.delta)
        }
        StretchBenchmarkComparisonOutcome::Inconclusive => 1.0e9,
        StretchBenchmarkComparisonOutcome::Improved
        | StretchBenchmarkComparisonOutcome::Unchanged => {
            return None;
        }
    };
    if !priority_score.is_finite() || priority_score <= 0.0 {
        return None;
    }

    Some(StretchQualityPriority {
        area: quality_work_area(comparison.metric),
        case_id: comparison.case_id,
        path: comparison.path,
        metric: comparison.metric,
        ratio: comparison.ratio,
        pitch_shift_semitones: comparison.pitch_shift_semitones,
        baseline_value: comparison.baseline_value,
        candidate_value: comparison.candidate_value,
        delta: comparison.delta,
        outcome: comparison.outcome,
        priority_score,
    })
}

fn quality_work_area(metric: StretchMetric) -> StretchQualityWorkArea {
    match metric {
        StretchMetric::TimingDriftSamples => StretchQualityWorkArea::TimingAlignment,
        StretchMetric::TransientSmearFrames => StretchQualityWorkArea::TransientPreservation,
        StretchMetric::VerticalCoherenceDelta => StretchQualityWorkArea::VerticalCoherence,
        StretchMetric::StereoImageDelta => StretchQualityWorkArea::StereoImageStability,
        StretchMetric::LoopBoundaryClickDbfs => StretchQualityWorkArea::LoopBoundaryClicks,
        StretchMetric::DynamicSegmentSeamClickDbfs => StretchQualityWorkArea::DynamicRatioSeams,
        StretchMetric::PitchErrorCents => StretchQualityWorkArea::PitchShiftAccuracy,
        StretchMetric::CpuRealtimeFactor
        | StretchMetric::LatencyFrames
        | StretchMetric::PeakMemoryBytes => StretchQualityWorkArea::ResourceBudget,
    }
}

fn priority_score(metric: StretchMetric, delta: f64) -> f64 {
    if !delta.is_finite() || delta <= 0.0 {
        return f64::NAN;
    }

    match metric {
        StretchMetric::LoopBoundaryClickDbfs | StretchMetric::DynamicSegmentSeamClickDbfs => {
            delta / 6.0
        }
        StretchMetric::StereoImageDelta | StretchMetric::VerticalCoherenceDelta => delta * 10.0,
        StretchMetric::PitchErrorCents => delta / 10.0,
        _ => delta,
    }
}

impl StretchSyntheticBenchmarkComparison {
    fn with_path(mut self, path: StretchBenchmarkPath) -> Self {
        self.path = path;
        self
    }

    fn with_pitch_shift(mut self, pitch_shift_semitones: f64) -> Self {
        self.path = StretchBenchmarkPath::PitchShift;
        self.pitch_shift_semitones = Some(pitch_shift_semitones);
        self
    }
}

fn measure_synthetic_length_drift(
    family: StretchCorpusFamily,
    ratio: f64,
    stretcher: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> f64 {
    let Some(input) = generate_synthetic_stretch_audio(family) else {
        return f64::NAN;
    };
    if !ratio.is_finite() || ratio <= 0.0 {
        return f64::NAN;
    }

    let input_frames = input.frame_count();
    let output_frames = match input.channels {
        1 => {
            let target_len = (input_frames as f64 * ratio).round() as usize;
            stretcher(&input.samples, target_len, ratio, 2_048, 512).len()
        }
        2 => stretch_stereo_synthetic(&input, ratio, stretcher).len() / 2,
        _ => return f64::NAN,
    };

    output_length_drift_samples(input_frames, output_frames, ratio)
}

fn measure_synthetic_length_drift_realtime_preview(family: StretchCorpusFamily, ratio: f64) -> f64 {
    let Some(input) = generate_synthetic_stretch_audio(family) else {
        return f64::NAN;
    };
    if !ratio.is_finite() || ratio <= 0.0 {
        return f64::NAN;
    }

    let input_frames = input.frame_count();
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let output_frames = match input.channels {
        1 => preview
            .stretch_mono(&input.samples)
            .expect("corpus render fits the offline output bound")
            .len(),
        2 => {
            preview
                .stretch_interleaved_stereo(&input.samples)
                .expect("corpus render fits the offline output bound")
                .len()
                / 2
        }
        _ => return f64::NAN,
    };

    output_length_drift_samples(input_frames, output_frames, ratio)
}

fn measure_realtime_preview_loop_boundary_click(ratio: f64) -> StretchLoopBoundaryMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return loop_boundary_nan(ratio, 2);
    }

    let input = synthetic_loop_seam();
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let mut output = preview
        .stretch_interleaved_stereo(&input.samples)
        .expect("corpus render fits the offline output bound");
    smooth_loop_boundary_interleaved(&mut output, input.channels, 128);
    measure_loop_boundary_click(&output, input.channels, ratio)
}

fn measure_realtime_preview_stereo_image_delta(ratio: f64) -> StretchStereoImageMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return stereo_image_nan(ratio);
    }

    let input = synthetic_loop_seam();
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let output = preview
        .stretch_interleaved_stereo(&input.samples)
        .expect("corpus render fits the offline output bound");
    measure_stereo_image_delta(&input.samples, &output, ratio)
}

fn measure_realtime_preview_transient_smear(ratio: f64) -> StretchTransientSmearMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return transient_smear_nan(ratio);
    }

    const WINDOW_SIZE: usize = 1_024;
    const HOP_SIZE: usize = 256;
    let input = synthetic_extreme_ratio().samples;
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let output = preview
        .stretch_mono(&input)
        .expect("corpus render fits the offline output bound");
    measure_transient_smear(
        &input,
        &output,
        ratio,
        WINDOW_SIZE,
        HOP_SIZE,
        StretchTransientSmearPolicies::production(),
    )
}

fn compare_dynamic_tempo_ramp(case_id: &'static str) -> Vec<StretchSyntheticBenchmarkComparison> {
    let input = synthetic_tempo_ramp();
    let ratio_curve = synthetic_tempo_ramp_ratio_curve(input.frame_count());
    let expected_frames = dynamic_ratio_output_frames(input.frame_count(), &ratio_curve, 1.0);
    let seam_frames = dynamic_ratio_output_boundaries(input.frame_count(), &ratio_curve, 1.0);
    let effective_ratio = expected_frames as f64 / input.frame_count() as f64;
    let draft_output =
        stretch_dynamic_ratio_stereo_independent(&input, &ratio_curve, phase_vocoder);
    let mut offline_high_quality = OfflineHighQualityStretcher::new(1.0);
    let offline_high_quality_output = offline_high_quality
        .stretch_dynamic_ratio_interleaved_stereo(&input.samples, &ratio_curve)
        .expect("corpus render fits the offline output bound");

    vec![
        compare_metric(
            case_id,
            effective_ratio,
            StretchMetric::TimingDriftSamples,
            (draft_output.len() / 2).abs_diff(expected_frames) as f64,
            (offline_high_quality_output.len() / 2).abs_diff(expected_frames) as f64,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
        compare_metric(
            case_id,
            effective_ratio,
            StretchMetric::DynamicSegmentSeamClickDbfs,
            measure_dynamic_segment_seam_click(
                &draft_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
            measure_dynamic_segment_seam_click(
                &offline_high_quality_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
    ]
}

fn compare_dynamic_tempo_ramp_realtime_preview(
    case_id: &'static str,
) -> Vec<StretchSyntheticBenchmarkComparison> {
    let input = synthetic_tempo_ramp();
    let ratio_curve = synthetic_tempo_ramp_ratio_curve(input.frame_count());
    let expected_frames = dynamic_ratio_output_frames(input.frame_count(), &ratio_curve, 1.0);
    let seam_frames = dynamic_ratio_output_boundaries(input.frame_count(), &ratio_curve, 1.0);
    let effective_ratio = expected_frames as f64 / input.frame_count() as f64;
    let draft_output =
        stretch_dynamic_ratio_stereo_independent(&input, &ratio_curve, phase_vocoder);
    let mut preview = RealtimePreviewStretcher::new(1.0);
    let preview_output = preview
        .stretch_dynamic_ratio_interleaved_stereo(&input.samples, &ratio_curve)
        .expect("corpus render fits the offline output bound");

    vec![
        compare_metric_for_backend(
            case_id,
            effective_ratio,
            StretchMetric::TimingDriftSamples,
            StretchBenchmarkBackend::RealtimePreviewPrototype,
            (draft_output.len() / 2).abs_diff(expected_frames) as f64,
            (preview_output.len() / 2).abs_diff(expected_frames) as f64,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
        compare_metric_for_backend(
            case_id,
            effective_ratio,
            StretchMetric::DynamicSegmentSeamClickDbfs,
            StretchBenchmarkBackend::RealtimePreviewPrototype,
            measure_dynamic_segment_seam_click(
                &draft_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
            measure_dynamic_segment_seam_click(
                &preview_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
    ]
}

fn compare_pitch_shift() -> Vec<StretchSyntheticBenchmarkComparison> {
    const CASE_ID: &str = "stretch:pitch_shift";
    const SAMPLE_RATE_HZ: u32 = 48_000;
    const SOURCE_FREQUENCY_HZ: f64 = 440.0;

    let input = synthetic_pitch_shift_tone(SOURCE_FREQUENCY_HZ, SAMPLE_RATE_HZ, 48_000);
    [(1.0, 12.0), (1.25, -5.0)]
        .into_iter()
        .map(|(ratio, semitones)| {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft_output = phase_vocoder(&input, target_len, ratio, 2_048, 512);
            let mut offline_high_quality = OfflineHighQualityStretcher::new(ratio);
            let offline_high_quality_output = offline_high_quality
                .stretch_pitch_mono(&input, SampleRate(SAMPLE_RATE_HZ), semitones)
                .expect("corpus render fits the offline output bound");

            compare_metric(
                CASE_ID,
                ratio,
                StretchMetric::PitchErrorCents,
                measure_pitch_shift_error_cents(
                    &draft_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
                measure_pitch_shift_error_cents(
                    &offline_high_quality_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
            )
            .with_pitch_shift(semitones)
        })
        .collect()
}

fn compare_pitch_shift_realtime_preview() -> Vec<StretchSyntheticBenchmarkComparison> {
    const CASE_ID: &str = "stretch:pitch_shift";
    const SAMPLE_RATE_HZ: u32 = 48_000;
    const SOURCE_FREQUENCY_HZ: f64 = 440.0;

    let input = synthetic_pitch_shift_tone(SOURCE_FREQUENCY_HZ, SAMPLE_RATE_HZ, 48_000);
    [(1.0, 12.0), (1.25, -5.0)]
        .into_iter()
        .map(|(ratio, semitones)| {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft_output = phase_vocoder(&input, target_len, ratio, 2_048, 512);
            let mut preview = RealtimePreviewStretcher::new(ratio);
            let preview_output = preview
                .stretch_pitch_mono(&input, SampleRate(SAMPLE_RATE_HZ), semitones)
                .expect("corpus render fits the offline output bound");

            compare_metric_for_backend(
                CASE_ID,
                ratio,
                StretchMetric::PitchErrorCents,
                StretchBenchmarkBackend::RealtimePreviewPrototype,
                measure_pitch_shift_error_cents(
                    &draft_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
                measure_pitch_shift_error_cents(
                    &preview_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
            )
            .with_pitch_shift(semitones)
        })
        .collect()
}

fn compare_sustained_coherence() -> Vec<StretchSyntheticBenchmarkComparison> {
    const CASE_ID: &str = "stretch:sustained_coherence";

    [0.75, 1.25, 1.5]
        .into_iter()
        .map(|ratio| {
            let coherence = compare_sustained_material_coherence(ratio);
            compare_metric(
                CASE_ID,
                ratio,
                StretchMetric::VerticalCoherenceDelta,
                coherence.draft_vertical_coherence_score,
                coherence.phase_locked_vertical_coherence_score,
            )
            .with_path(StretchBenchmarkPath::PhaseLocked)
        })
        .collect()
}

fn synthetic_tempo_ramp_ratio_curve(input_frames: usize) -> Vec<StretchRatioPoint> {
    vec![
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new((input_frames / 3) as i64, 1.0),
        StretchRatioPoint::new((input_frames * 2 / 3) as i64, 1.5),
    ]
}

fn synthetic_tempo_ramp() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize * 2;
    let mut samples = Vec::with_capacity(FRAMES * 2);
    for frame in 0..FRAMES {
        let progress = frame as f32 / FRAMES as f32;
        let frequency = 220.0 + 220.0 * progress;
        let carrier = (std::f32::consts::TAU * frequency * frame as f32 / SAMPLE_RATE as f32).sin();
        let pulse = if frame % 12_000 < 96 { 0.7 } else { 0.0 };
        let sample = (carrier * 0.25 + pulse) * (1.0 - 0.25 * progress);
        samples.push(sample);
        samples.push(sample);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 2,
        samples,
    }
}

fn synthetic_loop_seam() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize;
    let mut samples = Vec::with_capacity(FRAMES * 2);
    for frame in 0..FRAMES {
        let phase = frame as f32 / FRAMES as f32;
        let body = (std::f32::consts::TAU * 110.0 * frame as f32 / SAMPLE_RATE as f32).sin() * 0.2;
        let boundary_probe = if !(128..FRAMES - 128).contains(&frame) {
            0.8 * (1.0 - frame.min(FRAMES - 1 - frame) as f32 / 128.0)
        } else {
            0.0
        };
        let left = body + boundary_probe;
        let right = body * (0.95 + 0.05 * phase) + boundary_probe;
        samples.push(left);
        samples.push(right);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 2,
        samples,
    }
}

fn synthetic_extreme_ratio() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize * 2;
    let mut samples = Vec::with_capacity(FRAMES);
    for frame in 0..FRAMES {
        let tonal =
            (std::f32::consts::TAU * 330.0 * frame as f32 / SAMPLE_RATE as f32).sin() * 0.25;
        let transient = if frame % 8_000 < 64 {
            0.9 * (1.0 - (frame % 8_000) as f32 / 64.0)
        } else {
            0.0
        };
        samples.push(tonal + transient);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 1,
        samples,
    }
}

fn synthetic_pitch_shift_tone(
    source_frequency_hz: f64,
    sample_rate_hz: u32,
    frames: usize,
) -> Vec<Sample> {
    (0..frames)
        .map(|frame| {
            let time = frame as f64 / sample_rate_hz as f64;
            let fade_in = (frame as f32 / 1_024.0).min(1.0);
            let fade_out = ((frames - 1 - frame) as f32 / 1_024.0).min(1.0);
            let fade = fade_in.min(fade_out);
            (std::f64::consts::TAU * source_frequency_hz * time).sin() as f32 * 0.7 * fade
        })
        .collect()
}

fn synthetic_sustained_material() -> Vec<Sample> {
    const SAMPLE_RATE: usize = 48_000;
    const FRAMES: usize = SAMPLE_RATE * 2;
    const FADE_FRAMES: usize = 1_024;
    let bin_frequency = SAMPLE_RATE as f32 / 2048.0;
    let partials = [
        (9.0 * bin_frequency, 0.38),
        (17.0 * bin_frequency, 0.24),
        (29.0 * bin_frequency, 0.16),
        (43.0 * bin_frequency, 0.10),
    ];

    (0..FRAMES)
        .map(|frame| {
            let time = frame as f32 / SAMPLE_RATE as f32;
            let fade_in = (frame as f32 / FADE_FRAMES as f32).min(1.0);
            let fade_out = ((FRAMES - 1 - frame) as f32 / FADE_FRAMES as f32).min(1.0);
            let fade = fade_in.min(fade_out);
            let motion = 0.78 + 0.12 * (std::f32::consts::TAU * 0.35 * time).sin();
            partials
                .iter()
                .map(|(frequency, gain)| gain * (std::f32::consts::TAU * frequency * time).sin())
                .sum::<f32>()
                * motion
                * fade
        })
        .collect()
}

fn peak_neighborhood_phase_curvature(samples: &[Sample], window_size: usize, hop: usize) -> f64 {
    if samples.len() < window_size || hop == 0 {
        return f64::NAN;
    }

    let bins = window_size / 2 + 1;
    if bins < 5 {
        return f64::NAN;
    }
    let window: Vec<f32> = (0..window_size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos())
        .collect();
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(window_size);
    let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];
    let mut magnitudes = vec![0.0f32; bins];
    let mut phases = vec![0.0f32; bins];
    let mut weighted_curvature = 0.0f64;
    let mut weight_sum = 0.0f64;

    for start in (0..=samples.len() - window_size).step_by(hop) {
        for (slot, (sample, weight)) in buffer.iter_mut().zip(
            samples[start..start + window_size]
                .iter()
                .zip(window.iter()),
        ) {
            *slot = Complex32::new(sample * weight, 0.0);
        }
        forward.process(&mut buffer);

        let mut peak_magnitude = 0.0f32;
        for bin in 0..bins {
            let spectrum = buffer[bin];
            magnitudes[bin] = spectrum.norm();
            phases[bin] = spectrum.arg();
            peak_magnitude = peak_magnitude.max(magnitudes[bin]);
        }
        let threshold = peak_magnitude * 0.05;

        for bin in 2..bins - 2 {
            let magnitude = magnitudes[bin];
            if magnitude < threshold {
                continue;
            }
            if magnitude > magnitudes[bin - 1] && magnitude >= magnitudes[bin + 1] {
                let left_offset = wrap_phase(phases[bin - 1] - phases[bin]);
                let right_offset = wrap_phase(phases[bin + 1] - phases[bin]);
                let curvature = wrap_phase(right_offset - left_offset).abs() as f64;
                let weight = magnitude as f64;
                weighted_curvature += curvature * weight;
                weight_sum += weight;
            }
        }
    }

    if weight_sum > 0.0 {
        weighted_curvature / weight_sum
    } else {
        f64::NAN
    }
}

fn loop_boundary_nan(ratio: f64, channels: u16) -> StretchLoopBoundaryMeasurement {
    StretchLoopBoundaryMeasurement {
        ratio,
        channels,
        peak_boundary_delta: f64::NAN,
        click_dbfs: f64::NAN,
        metric: StretchMetricValue::new(StretchMetric::LoopBoundaryClickDbfs, f64::NAN),
    }
}

fn dynamic_segment_seam_nan(ratio: f64, channels: u16) -> StretchDynamicSegmentSeamMeasurement {
    StretchDynamicSegmentSeamMeasurement {
        ratio,
        channels,
        seam_frames: Vec::new(),
        peak_seam_delta: f64::NAN,
        click_dbfs: f64::NAN,
        metric: StretchMetricValue::new(StretchMetric::DynamicSegmentSeamClickDbfs, f64::NAN),
    }
}

fn pitch_shift_nan(
    ratio: f64,
    pitch_shift_semitones: f64,
    source_frequency_hz: f64,
) -> StretchPitchShiftMeasurement {
    StretchPitchShiftMeasurement {
        ratio,
        pitch_shift_semitones,
        expected_frequency_hz: source_frequency_hz * 2.0f64.powf(pitch_shift_semitones / 12.0),
        measured_frequency_hz: f64::NAN,
        pitch_error_cents: f64::NAN,
        metric: StretchMetricValue::new(StretchMetric::PitchErrorCents, f64::NAN),
    }
}

fn stereo_image_nan(ratio: f64) -> StretchStereoImageMeasurement {
    StretchStereoImageMeasurement {
        ratio,
        input_correlation: f64::NAN,
        output_correlation: f64::NAN,
        input_side_mid_ratio: f64::NAN,
        output_side_mid_ratio: f64::NAN,
        image_delta: f64::NAN,
        metric: StretchMetricValue::new(StretchMetric::StereoImageDelta, f64::NAN),
    }
}

fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: u32) -> f64 {
    if samples.len() < 2 || sample_rate_hz == 0 {
        return f64::NAN;
    }

    let fft_size = samples.len().next_power_of_two();
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    let mut buffer = vec![Complex32::new(0.0, 0.0); fft_size];
    for (index, sample) in samples.iter().enumerate() {
        let window =
            0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / samples.len() as f32).cos();
        buffer[index] = Complex32::new(sample * window, 0.0);
    }
    fft.process(&mut buffer);

    let mut best_bin = 0usize;
    let mut best_magnitude = 0.0f32;
    for (bin, spectrum) in buffer.iter().take(fft_size / 2).enumerate().skip(1) {
        let magnitude = spectrum.norm_sqr();
        if magnitude > best_magnitude {
            best_magnitude = magnitude;
            best_bin = bin;
        }
    }
    if best_bin == 0 {
        f64::NAN
    } else {
        best_bin as f64 * sample_rate_hz as f64 / fft_size as f64
    }
}

#[derive(Clone, Copy, Debug)]
struct StereoImageStats {
    correlation: f64,
    side_mid_ratio: f64,
}

fn stereo_image_stats(interleaved_samples: &[Sample]) -> StereoImageStats {
    let frames = interleaved_samples.len() / 2;
    if frames == 0 {
        return StereoImageStats {
            correlation: f64::NAN,
            side_mid_ratio: f64::NAN,
        };
    }

    let mut left_square_sum = 0.0f64;
    let mut right_square_sum = 0.0f64;
    let mut cross_sum = 0.0f64;
    let mut mid_square_sum = 0.0f64;
    let mut side_square_sum = 0.0f64;
    for frame in interleaved_samples.chunks_exact(2) {
        let left = frame[0] as f64;
        let right = frame[1] as f64;
        left_square_sum += left * left;
        right_square_sum += right * right;
        cross_sum += left * right;
        let mid = 0.5 * (left + right);
        let side = 0.5 * (left - right);
        mid_square_sum += mid * mid;
        side_square_sum += side * side;
    }

    let correlation = cross_sum / ((left_square_sum * right_square_sum).sqrt() + 1.0e-12);
    let side_mid_ratio = (side_square_sum / frames as f64).sqrt()
        / ((mid_square_sum / frames as f64).sqrt() + 1.0e-12);
    StereoImageStats {
        correlation,
        side_mid_ratio,
    }
}

fn stretch_stereo_synthetic(
    input: &StretchSyntheticAudio,
    ratio: f64,
    stretcher: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    let channel_count = input.channels as usize;
    if channel_count != 2 {
        return Vec::new();
    }
    let target_len = (input.frame_count() as f64 * ratio).round() as usize;
    let mut output_channels = Vec::with_capacity(channel_count);
    for channel in 0..channel_count {
        let mono = deinterleave_channel(&input.samples, channel_count, channel);
        output_channels.push(stretcher(&mono, target_len, ratio, 2_048, 512));
    }
    interleave_channels(&output_channels)
}

fn stretch_dynamic_ratio_stereo_independent(
    input: &StretchSyntheticAudio,
    ratio_curve: &[StretchRatioPoint],
    stretcher: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    let channel_count = input.channels as usize;
    if channel_count != 2 {
        return Vec::new();
    }
    let mut output_channels = Vec::with_capacity(channel_count);
    for channel in 0..channel_count {
        let mono = deinterleave_channel(&input.samples, channel_count, channel);
        output_channels.push(
            stretch_dynamic_ratio_mono_with_engine(&mono, ratio_curve, 1.0, 2_048, 512, stretcher)
                .expect("corpus render fits the offline output bound"),
        );
    }
    interleave_channels(&output_channels)
}

fn stretch_stereo_synthetic_linked(input: &StretchSyntheticAudio, ratio: f64) -> Vec<Sample> {
    if input.channels != 2 {
        return Vec::new();
    }
    let target_len = (input.frame_count() as f64 * ratio).round() as usize;
    transient_reset_phase_vocoder_linked_stereo(&input.samples, target_len, ratio, 2_048, 512)
}

fn amplitude_to_dbfs(amplitude: f64) -> f64 {
    if amplitude <= 1.0e-12 {
        -240.0
    } else {
        20.0 * amplitude.log10()
    }
}

pub(super) fn smooth_loop_boundary_interleaved(
    interleaved_samples: &mut [Sample],
    channels: u16,
    fade_frames: usize,
) {
    let channel_count = channels as usize;
    if channel_count == 0 || fade_frames == 0 {
        return;
    }
    let frames = interleaved_samples.len() / channel_count;
    if frames < 2 {
        return;
    }

    let fade_frames = fade_frames.min(frames / 2).max(1);
    for channel in 0..channel_count {
        let first = interleaved_samples[channel];
        let last = interleaved_samples[(frames - 1) * channel_count + channel];
        let correction = (first - last) * 0.5;
        for frame in 0..fade_frames {
            let weight = (fade_frames - frame) as f32 / fade_frames as f32;
            interleaved_samples[frame * channel_count + channel] -= correction * weight;
            let tail_frame = frames - 1 - frame;
            interleaved_samples[tail_frame * channel_count + channel] += correction * weight;
        }
    }
}

fn deinterleave_channel(samples: &[Sample], channels: usize, channel: usize) -> Vec<Sample> {
    samples
        .chunks_exact(channels)
        .map(|frame| frame[channel])
        .collect()
}

fn interleave_channels(channels: &[Vec<Sample>]) -> Vec<Sample> {
    let Some(first) = channels.first() else {
        return Vec::new();
    };
    let frames = channels.iter().map(Vec::len).min().unwrap_or(first.len());
    let mut output = Vec::with_capacity(frames * channels.len());
    for frame in 0..frames {
        for channel in channels {
            output.push(channel[frame]);
        }
    }
    output
}

fn severity_to_stretch_status(severity: StretchAcceptanceSeverity) -> StretchAcceptanceStatus {
    match severity {
        StretchAcceptanceSeverity::Warn => StretchAcceptanceStatus::Warn,
        StretchAcceptanceSeverity::Fail => StretchAcceptanceStatus::Fail,
    }
}

fn wrap_phase(phase: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    phase - tau * (phase / tau).round()
}

fn combine_stretch_status(
    left: StretchAcceptanceStatus,
    right: StretchAcceptanceStatus,
) -> StretchAcceptanceStatus {
    match (left, right) {
        (StretchAcceptanceStatus::Fail, _) | (_, StretchAcceptanceStatus::Fail) => {
            StretchAcceptanceStatus::Fail
        }
        (StretchAcceptanceStatus::Warn, _) | (_, StretchAcceptanceStatus::Warn) => {
            StretchAcceptanceStatus::Warn
        }
        _ => StretchAcceptanceStatus::Pass,
    }
}

/// `g10.041` Batch 41.3: the `A18` fix must not trade the pop for smearing.
///
/// The candidate resets transient phase only above a crossover. This asserts it
/// against the corpus's own transient-smear measurement and production
/// policies, rather than an ad-hoc proxy — the first proxy tried here disagreed
/// with itself across ratios, which is exactly how Batch 41.1 went wrong.
#[cfg(test)]
mod a18_crossover_smear {
    use super::*;
    use crate::phase_vocoder::{
        high_band_transient_reset_phase_vocoder, phase_locked_phase_vocoder,
        transient_reset_phase_vocoder,
    };

    /// Frozen crossover as a fraction of Nyquist: `240 Hz` at `48 kHz`.
    const CROSSOVER: f64 = 0.010;

    fn smear(input: &[Sample], output: &[Sample], ratio: f64) -> f64 {
        measure_transient_smear(
            input,
            output,
            ratio,
            1_024,
            256,
            StretchTransientSmearPolicies::production(),
        )
        .metric
        .value
    }

    #[test]
    fn candidate_crossover_does_not_regress_transient_smear() {
        let input = synthetic_extreme_ratio().samples;
        for ratio in [1.5f64, 2.0, 3.0] {
            let target = (input.len() as f64 * ratio).round() as usize;
            let shipped = transient_reset_phase_vocoder(&input, target, ratio, 2_048, 512);
            let candidate = high_band_transient_reset_phase_vocoder(
                &input, target, ratio, 2_048, 512, CROSSOVER,
            );
            assert!(
                smear(&input, &candidate, ratio) <= smear(&input, &shipped, ratio),
                "ratio {ratio}: candidate smeared {} against shipped {}",
                smear(&input, &candidate, ratio),
                smear(&input, &shipped, ratio),
            );
        }
    }

    /// Removing the reset outright is not the fix. It regresses smear at ratio
    /// `3.0`, which is why the reset is kept above the crossover rather than
    /// deleted.
    #[test]
    fn dropping_the_reset_entirely_regresses_smear() {
        let input = synthetic_extreme_ratio().samples;
        let ratio = 3.0;
        let target = (input.len() as f64 * ratio).round() as usize;
        let shipped = transient_reset_phase_vocoder(&input, target, ratio, 2_048, 512);
        let none = phase_locked_phase_vocoder(&input, target, ratio, 2_048, 512);
        assert!(
            smear(&input, &none, ratio) > smear(&input, &shipped, ratio),
            "expected removing the reset to smear more at ratio {ratio}"
        );
    }
}
