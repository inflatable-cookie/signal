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
