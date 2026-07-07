use crate::benchmark::{
    compare_synthetic_stretch_backends, generate_synthetic_stretch_audio, StretchBenchmarkPath,
    StretchCorpusAssetRequirement, StretchCorpusCase, StretchCorpusManifest,
    StretchCorpusManifestEntry, StretchCorpusMissingAssetBehavior,
    StretchSyntheticBenchmarkComparison, StretchSyntheticBenchmarkComparisonReport,
    STRETCH_CORPUS_MANIFEST,
};
use crate::cache_identity::SIGNAL_STRETCH_ENGINE_VERSION;

/// One corpus manifest case skipped because its source asset is unavailable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchCorpusSkippedAsset {
    /// Corpus case blueprint.
    pub case: StretchCorpusCase,
    /// How the source audio was expected to be provided.
    pub asset_requirement: StretchCorpusAssetRequirement,
    /// Skip behavior declared by the manifest.
    pub missing_asset_behavior: StretchCorpusMissingAssetBehavior,
    /// Stable source location hint.
    pub source_path_hint: &'static str,
    /// License/provenance rule for the skipped source.
    pub provenance_note: &'static str,
}

/// One listening-note slot attached to a corpus report row.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCorpusListeningNoteSlot {
    /// Corpus or derived case id.
    pub case_id: &'static str,
    /// Source location hint used by the row.
    pub source_path_hint: &'static str,
    /// Backend path under review, when this is a measured comparison row.
    pub path: Option<StretchBenchmarkPath>,
    /// Output/input duration ratio, when measured.
    pub ratio: Option<f64>,
    /// Requested pitch shift, when measured.
    pub pitch_shift_semitones: Option<f64>,
    /// Deterministic prompt label for operator listening notes.
    pub prompt: &'static str,
}

/// Deterministic corpus report for draft-vs-OfflineHighQuality evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCorpusComparisonReport {
    /// Human-readable report name.
    pub report_name: String,
    /// Projection epoch associated with the render plan under test.
    pub projection_epoch: String,
    /// Checked-in corpus manifest used for the run.
    pub manifest: StretchCorpusManifest,
    /// Signal stretch engine version.
    pub engine_version: &'static str,
    /// Synthetic draft-vs-OfflineHighQuality comparison rows.
    pub synthetic_report: StretchSyntheticBenchmarkComparisonReport,
    /// Optional external rendered-output comparisons supplied by the operator.
    pub external_benchmark_comparisons: Vec<StretchExternalBenchmarkComparison>,
    /// Required operator-provided source assets that were unavailable.
    pub missing_assets: Vec<StretchCorpusSkippedAsset>,
    /// Optional external benchmark rows skipped because no comparator output
    /// was supplied.
    pub optional_benchmark_skips: Vec<StretchCorpusSkippedAsset>,
    /// Listening-note slots next to the objective report rows.
    pub listening_note_slots: Vec<StretchCorpusListeningNoteSlot>,
}

/// Metadata for one operator-supplied external benchmark render.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchExternalBenchmarkRender {
    /// Corpus or derived case id the external render claims to cover.
    pub case_id: String,
    /// Output/input duration ratio rendered by the external tool.
    pub ratio: f64,
    /// Requested pitch shift, when the render covers pitch-shift behavior.
    pub pitch_shift_semitones: Option<f64>,
    /// Human-readable external tool identity.
    pub tool_name: String,
    /// Local path to the rendered output supplied by the operator.
    pub rendered_path: String,
    /// Rendered output sample-frame count.
    pub rendered_frames: usize,
    /// Rendered output sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Rendered output channel count.
    pub channels: u16,
}

/// One clean-room comparison against operator-supplied rendered output.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchExternalBenchmarkComparison {
    /// Corpus or derived case id.
    pub case_id: String,
    /// External tool identity.
    pub tool_name: String,
    /// Local rendered-output path.
    pub rendered_path: String,
    /// Output/input duration ratio.
    pub ratio: f64,
    /// Requested pitch shift, when present.
    pub pitch_shift_semitones: Option<f64>,
    /// Rendered output sample-frame count.
    pub rendered_frames: usize,
    /// Expected output sample-frame count when Signal can derive it.
    pub expected_frames: Option<usize>,
    /// Absolute rendered length drift in samples when expectation is known.
    pub timing_drift_samples: Option<f64>,
    /// Rendered output sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Rendered output channel count.
    pub channels: u16,
    /// Clean-room boundary applied to this comparison.
    pub source_boundary: &'static str,
}

/// Build the deterministic stretch corpus report for the checked-in manifest.
///
/// Required licensed listening assets are reported as skipped until the
/// operator supplies them outside the repository. Inline synthetic cases are
/// measured immediately so the report remains useful in ordinary local test
/// runs.
pub fn build_stretch_corpus_comparison_report(
    report_name: &str,
    projection_epoch: &str,
) -> StretchCorpusComparisonReport {
    build_stretch_corpus_comparison_report_with_external(report_name, projection_epoch, &[])
}

/// Build the stretch corpus report with optional external rendered-output
/// comparisons.
pub fn build_stretch_corpus_comparison_report_with_external(
    report_name: &str,
    projection_epoch: &str,
    external_renders: &[StretchExternalBenchmarkRender],
) -> StretchCorpusComparisonReport {
    let manifest = STRETCH_CORPUS_MANIFEST;
    let synthetic_report = compare_synthetic_stretch_backends();
    let external_benchmark_comparisons = external_renders
        .iter()
        .map(compare_external_benchmark_render)
        .collect::<Vec<_>>();
    let mut missing_assets = Vec::new();
    let mut optional_benchmark_skips = Vec::new();
    let mut listening_note_slots = Vec::new();

    for entry in manifest.entries {
        match entry.asset_requirement {
            StretchCorpusAssetRequirement::InlineSynthetic => {}
            StretchCorpusAssetRequirement::OperatorProvidedAudio => {
                missing_assets.push(skipped_asset_from_manifest_entry(entry));
                listening_note_slots.push(StretchCorpusListeningNoteSlot {
                    case_id: entry.case.case_id,
                    source_path_hint: entry.source_path_hint,
                    path: None,
                    ratio: None,
                    pitch_shift_semitones: None,
                    prompt: "operator-note: add listening observations when licensed source is supplied",
                });
            }
            StretchCorpusAssetRequirement::OptionalExternalBenchmark => {
                optional_benchmark_skips.push(skipped_asset_from_manifest_entry(entry));
            }
        }
    }

    for comparison in &synthetic_report.comparisons {
        listening_note_slots.push(StretchCorpusListeningNoteSlot {
            case_id: comparison.case_id,
            source_path_hint: source_path_hint_for_comparison(comparison.case_id),
            path: Some(comparison.path),
            ratio: Some(comparison.ratio),
            pitch_shift_semitones: comparison.pitch_shift_semitones,
            prompt: "operator-note: record audible artifacts beside objective metrics",
        });
    }

    StretchCorpusComparisonReport {
        report_name: report_name.to_string(),
        projection_epoch: projection_epoch.to_string(),
        manifest,
        engine_version: SIGNAL_STRETCH_ENGINE_VERSION,
        synthetic_report,
        external_benchmark_comparisons,
        missing_assets,
        optional_benchmark_skips,
        listening_note_slots,
    }
}

/// Deterministic line-oriented report for the real stretch corpus manifest.
pub fn format_stretch_corpus_comparison_report(report: &StretchCorpusComparisonReport) -> String {
    let mut lines = Vec::with_capacity(
        7 + report.missing_assets.len()
            + report.optional_benchmark_skips.len()
            + report.synthetic_report.comparisons.len()
            + report.external_benchmark_comparisons.len()
            + report.listening_note_slots.len(),
    );
    lines.push(format!(
        "stretch_corpus_report name={} corpus={} schema={} engine={} projection_epoch={} sample_rate={} channels={}",
        quoted_report_field(&report.report_name),
        report.manifest.manifest_id,
        report.manifest.schema_version,
        report.engine_version,
        quoted_report_field(&report.projection_epoch),
        report.manifest.sample_rate_hz,
        report.manifest.channels
    ));
    lines.push(format!(
        "source_policy synthetic={} licensed={} external={}",
        quoted_report_field(report.manifest.source_policy.synthetic_audio_policy),
        quoted_report_field(report.manifest.source_policy.licensed_audio_policy),
        quoted_report_field(report.manifest.source_policy.external_benchmark_policy)
    ));
    lines.push(format!(
        "summary comparisons={} external_benchmark_comparisons={} missing_assets={} optional_benchmark_skips={} listening_note_slots={} improved={} regressed={} unchanged={} inconclusive={}",
        report.synthetic_report.comparisons.len(),
        report.external_benchmark_comparisons.len(),
        report.missing_assets.len(),
        report.optional_benchmark_skips.len(),
        report.listening_note_slots.len(),
        report.synthetic_report.improved_count,
        report.synthetic_report.regressed_count,
        report.synthetic_report.unchanged_count,
        report.synthetic_report.inconclusive_count
    ));

    for missing in &report.missing_assets {
        lines.push(format_skipped_asset("missing_required", missing));
    }
    for skipped in &report.optional_benchmark_skips {
        lines.push(format_skipped_asset("skipped_optional", skipped));
    }

    for comparison in &report.external_benchmark_comparisons {
        lines.push(format!(
            "external_benchmark case={} tool={} source_boundary={} render={} ratio={:.6} pitch_shift={} sample_rate={} channels={} rendered_frames={} expected_frames={} timing_drift_samples={}",
            comparison.case_id,
            quoted_report_field(&comparison.tool_name),
            quoted_report_field(comparison.source_boundary),
            quoted_report_field(&comparison.rendered_path),
            comparison.ratio,
            optional_f64_report_field(comparison.pitch_shift_semitones),
            comparison.sample_rate_hz,
            comparison.channels,
            comparison.rendered_frames,
            optional_usize_report_field(comparison.expected_frames),
            optional_f64_report_field(comparison.timing_drift_samples)
        ));
    }

    for comparison in &report.synthetic_report.comparisons {
        lines.push(format!(
            "comparison case={} source={} ratio={:.6} ratio_curve={} path={:?} pitch_curve={} pitch_shift={} metric={:?} draft={:.6} offline_hq={:.6} delta={:.6} outcome={:?}",
            comparison.case_id,
            source_path_hint_for_comparison(comparison.case_id),
            comparison.ratio,
            ratio_curve_label(comparison),
            comparison.path,
            pitch_curve_label(comparison),
            optional_f64_report_field(comparison.pitch_shift_semitones),
            comparison.metric,
            comparison.draft_value,
            comparison.offline_high_quality_value,
            comparison.delta,
            comparison.outcome
        ));
    }

    for slot in &report.listening_note_slots {
        let path = slot
            .path
            .map(|path| format!("{path:?}"))
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "listening_note case={} source={} ratio={} path={} pitch_shift={} prompt={}",
            slot.case_id,
            slot.source_path_hint,
            optional_f64_report_field(slot.ratio),
            path,
            optional_f64_report_field(slot.pitch_shift_semitones),
            quoted_report_field(slot.prompt)
        ));
    }

    lines.join("\n")
}

fn compare_external_benchmark_render(
    render: &StretchExternalBenchmarkRender,
) -> StretchExternalBenchmarkComparison {
    let expected_frames = expected_external_render_frames(&render.case_id, render.ratio);
    let timing_drift_samples =
        expected_frames.map(|expected| render.rendered_frames.abs_diff(expected) as f64);

    StretchExternalBenchmarkComparison {
        case_id: render.case_id.clone(),
        tool_name: render.tool_name.clone(),
        rendered_path: render.rendered_path.clone(),
        ratio: render.ratio,
        pitch_shift_semitones: render.pitch_shift_semitones,
        rendered_frames: render.rendered_frames,
        expected_frames,
        timing_drift_samples,
        sample_rate_hz: render.sample_rate_hz,
        channels: render.channels,
        source_boundary: "rendered-output-only; no external source or library dependency",
    }
}

fn expected_external_render_frames(case_id: &str, ratio: f64) -> Option<usize> {
    if !ratio.is_finite() || ratio <= 0.0 {
        return None;
    }
    let entry = STRETCH_CORPUS_MANIFEST
        .entries
        .iter()
        .find(|entry| entry.case.case_id == case_id)?;
    let audio = generate_synthetic_stretch_audio(entry.case.family)?;
    Some((audio.frame_count() as f64 * ratio).round() as usize)
}

fn skipped_asset_from_manifest_entry(
    entry: &StretchCorpusManifestEntry,
) -> StretchCorpusSkippedAsset {
    StretchCorpusSkippedAsset {
        case: entry.case,
        asset_requirement: entry.asset_requirement,
        missing_asset_behavior: entry.missing_asset_behavior,
        source_path_hint: entry.source_path_hint,
        provenance_note: entry.provenance_note,
    }
}

fn format_skipped_asset(status: &str, skipped: &StretchCorpusSkippedAsset) -> String {
    format!(
        "asset case={} status={} requirement={:?} behavior={:?} source={} provenance={}",
        skipped.case.case_id,
        status,
        skipped.asset_requirement,
        skipped.missing_asset_behavior,
        skipped.source_path_hint,
        quoted_report_field(skipped.provenance_note)
    )
}

fn source_path_hint_for_comparison(case_id: &str) -> &'static str {
    STRETCH_CORPUS_MANIFEST
        .entries
        .iter()
        .find(|entry| entry.case.case_id == case_id)
        .map(|entry| entry.source_path_hint)
        .unwrap_or(match case_id {
            "stretch:pitch_shift" => "inline:pitch-shift-tone",
            "stretch:sustained_coherence" => "inline:sustained-coherence",
            _ => "inline:derived",
        })
}

fn ratio_curve_label(comparison: &StretchSyntheticBenchmarkComparison) -> String {
    if comparison.path == StretchBenchmarkPath::DynamicRatio {
        "synthetic_tempo_ramp:0.750000@0,1.000000@1/3,1.500000@2/3".to_string()
    } else {
        format!("fixed:{:.6}", comparison.ratio)
    }
}

fn pitch_curve_label(comparison: &StretchSyntheticBenchmarkComparison) -> String {
    comparison
        .pitch_shift_semitones
        .map(|semitones| format!("constant:{semitones:.6}"))
        .unwrap_or_else(|| "none".to_string())
}

fn optional_f64_report_field(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "none".to_string())
}

fn optional_usize_report_field(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn quoted_report_field(value: &str) -> String {
    format!("{value:?}")
}
