//! Emit deterministic Signal stretch corpus evidence reports.

use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

#[path = "stretch-corpus-report/alloc_tracker.rs"]
mod alloc_tracker;
#[path = "stretch-corpus-report/formant_boundary.rs"]
mod formant_boundary;
#[path = "stretch-corpus-report/hybrid_review.rs"]
mod hybrid_review;
#[path = "stretch-corpus-report/listening_pack.rs"]
mod listening_pack;
#[path = "stretch-corpus-report/peak_transient_review.rs"]
mod peak_transient_review;
#[path = "stretch-corpus-report/tail_anchor.rs"]
mod tail_anchor;
#[path = "stretch-corpus-report/tail_features.rs"]
mod tail_features;
#[path = "stretch-corpus-report/timeline_review.rs"]
mod timeline_review;

use alloc_tracker::measure_peak_live_heap;
use peak_transient_review::PeakTransientReviewEvidence;

use rustfft::{num_complex::Complex32, FftPlanner};
use signal_analysis_character::{CharacterAnalyzer, CharacterAnalyzerConfig};
use signal_dsp_stretch::{
    build_stretch_corpus_comparison_report_with_sources, detect_stretch_transients,
    format_stretch_corpus_comparison_report, measure_formant_boundary,
    measure_stretch_render_integrity, measure_tonal_texture, measure_transient_detail,
    measure_transient_event_detail, measure_transient_smear,
    measure_transient_smear_with_output_recovery_policy, measure_transient_smear_with_policies,
    measure_transient_smear_with_policy, output_length_drift_samples, OfflineHighQualityPath,
    OfflineHighQualityStretcher, PhaseVocoderStretcher, StretchCorpusAssetRequirement,
    StretchCorpusListeningSource, StretchExternalBenchmarkRender, StretchRenderIntegrityLimits,
    StretchTransientDetectorPolicy, TimeStretcher, COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE, STRETCH_CORPUS_MANIFEST,
    SUSTAINED_COHERENCE_BLEND_REVIEW_WEIGHT,
};

use formant_boundary::format_external_benchmark_formant_boundary_line;
use hybrid_review::HybridReviewEvidence;
use symphonia::core::{
    audio::SampleBuffer as SymphoniaSampleBuffer,
    codecs::{DecoderOptions as SymphoniaDecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymphoniaError,
    formats::FormatOptions as SymphoniaFormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions as SymphoniaMetadataOptions,
    probe::Hint as SymphoniaHint,
};
use tail_anchor::TailAnchorReviewEvidence;
use tail_features::format_tail_local_feature_line;
use timeline_review::TimelineReviewEvidence;

const DEFAULT_REPORT_NAME: &str = "stretch-corpus-v1-offline-evidence";
const DEFAULT_PROJECTION_EPOCH: &str = "projection:deterministic-report-v1";
const DEFAULT_EXTERNAL_BENCHMARK_TOOL: &str = "external-render";
const DEFAULT_DECODE_SOURCE_FRAME_LIMIT: usize = 48_000 * 60;
const DEFAULT_DECODED_STRETCH_FRAME_LIMIT: usize = 48_000 * 10;
const QUALITY_METRIC_WINDOW_SIZE: usize = 1_024;
const QUALITY_METRIC_HOP_SIZE: usize = 256;
const RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES: usize = 1_024;
const RENDER_INTEGRITY_SILENCE_THRESHOLD: f32 = 1.0e-6;
const OFFLINE_HIGH_QUALITY_INTEGRITY_LIMIT_ID: &str = "offline-high-quality-v1";
const MAX_TRANSIENT_ALIGNMENT_EVENTS_PER_BACKEND: usize = 3;
const TRANSIENT_ALIGNMENT_WINDOW_RADIUS: usize = QUALITY_METRIC_WINDOW_SIZE;
const EXPECTED_TRANSIENT_ENERGY_PRESENT_RATIO: f64 = 0.50;
const EXPECTED_TRANSIENT_ENERGY_WEAK_RATIO: f64 = 0.10;
const RECOVERY_GATE_MIN_RECOVERED_MISSES: usize = 1;
const RECOVERY_GATE_MAX_MISSED_WORSENED_ROWS: usize = 0;
const RECOVERY_GATE_MAX_SMEAR_WORSENED_ROWS: usize = 0;
const RECOVERY_GATE_MAX_GLOBAL_CANDIDATE_INPUT_RATIO: f64 = 2.0;
const WIDTH_CONTROL_EDIT_GATE_MAX_SAMPLE_DELTA: f64 = 0.25;
const WIDTH_CONTROL_EDIT_GATE_MAX_ADDED_ADJACENT_STEP_DELTA: f64 = 0.05;
const EXTERNAL_BENCHMARK_ALIGNMENT_MAX_LAG_FRAMES: isize = 2_048;
const EXTERNAL_BENCHMARK_ALIGNMENT_MAX_COMPARE_FRAMES: usize = 65_536;
const EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE: usize = 2_048;
const EXTERNAL_BENCHMARK_FEATURE_MAX_WINDOWS: usize = 16;
const EXTERNAL_BENCHMARK_GAIN_ENVELOPE_REVIEW_ROWS: usize = 8;
const EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE: usize = 4_096;
const EXTERNAL_BENCHMARK_GAIN_ENVELOPE_HOP_SIZE: usize = 2_048;
const EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB: f64 = 0.5;
const EXTERNAL_BENCHMARK_LEVEL_NORMALIZED_REVIEW_ROWS: usize =
    EXTERNAL_BENCHMARK_GAIN_ENVELOPE_REVIEW_ROWS;
const EXTERNAL_BENCHMARK_RESIDUAL_COHERENCE_REVIEW_ROWS: usize =
    EXTERNAL_BENCHMARK_LEVEL_NORMALIZED_REVIEW_ROWS;
const EXTERNAL_BENCHMARK_COHERENCE_TARGET_REVIEW_ROWS: usize = 6;
const EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS: usize = 64;
const EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_GATE: &str = "spectral-magnitude-material-guard";
const EXTERNAL_BENCHMARK_COHERENCE_PRODUCT_PROBE: &str = "source-character-v1";
const EXTERNAL_BENCHMARK_COHERENCE_BLEND_CANDIDATE_PATH: &str = "current-long-window-half-blend";
const EXTERNAL_BENCHMARK_COHERENCE_ENVELOPE_CANDIDATE_PATH: &str =
    "long-window-current-envelope-match";
const EXTERNAL_BENCHMARK_COHERENCE_EXPANSION_RESET_CANDIDATE_PATH: &str =
    "expansion-long-window-transient-reset";
const EXTERNAL_BENCHMARK_COHERENCE_STABILITY_ADAPTIVE_CANDIDATE_PATH: &str =
    "expansion-long-window-stability-adaptive";
const EXTERNAL_BENCHMARK_COHERENCE_TRACKED_PEAK_CANDIDATE_PATH: &str =
    "expansion-long-window-tracked-peak-regions";
const EXTERNAL_BENCHMARK_COHERENCE_MAGNITUDE_SLEW_CANDIDATE_PATH: &str =
    "expansion-long-window-magnitude-slew";
const MAX_EXTERNAL_BENCHMARK_MISSING_RENDER_ROWS: usize = 20;
const DETECTOR_POLICY: StretchTransientDetectorPolicy =
    StretchTransientDetectorPolicy::production();
const CANDIDATE_DETECTOR_POLICY: StretchTransientDetectorPolicy =
    StretchTransientDetectorPolicy::candidate_review();

#[derive(Debug, PartialEq, Eq)]
struct ReportArgs {
    report_name: String,
    projection_epoch: String,
    output: Option<PathBuf>,
    external_benchmark_tool: String,
    external_benchmark_renders: Vec<ExternalBenchmarkRenderArg>,
    external_benchmark_render_manifests: Vec<PathBuf>,
    export_external_benchmark_pack: Option<PathBuf>,
    external_benchmark_render_plan_status_manifests: Vec<PathBuf>,
    listening_source_manifests: Vec<PathBuf>,
    decode_listening_sources: bool,
    decode_source_frame_limit: usize,
    measure_decoded_stretch: bool,
    decoded_stretch_report_mode: DecodedStretchReportMode,
    decoded_stretch_frame_limit: usize,
    measure_external_benchmark_quality: bool,
    external_benchmark_quality_mode: ExternalBenchmarkQualityMode,
    external_benchmark_signal_path: OfflineHighQualityPath,
    export_blind_listening_pack: Option<PathBuf>,
    export_tail_listening_pack: Option<PathBuf>,
    export_tail_classifier_validation_pack: Option<PathBuf>,
    blind_listening_note_manifests: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExternalBenchmarkRenderArg {
    case_id: String,
    ratio: String,
    path: PathBuf,
    tool_name: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DecodedStretchReportMode {
    Full,
    ExpansionSelector,
}

impl DecodedStretchReportMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "full" => Ok(Self::Full),
            "expansion-selector" => Ok(Self::ExpansionSelector),
            _ => Err(format!(
                "invalid --decoded-stretch-report-mode value: {value}; expected full or expansion-selector"
            )),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ExternalBenchmarkQualityMode {
    Core,
    Full,
}

impl ExternalBenchmarkQualityMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "core" => Ok(Self::Core),
            "full" => Ok(Self::Full),
            _ => Err(format!(
                "invalid --external-benchmark-quality-mode value: {value}; expected core or full"
            )),
        }
    }
}

fn parse_offline_high_quality_path(value: &str) -> Result<OfflineHighQualityPath, String> {
    match value {
        "default" => Ok(OfflineHighQualityPath::Default),
        "compression-short-window-selector" => {
            Ok(OfflineHighQualityPath::CompressionShortWindowSelector)
        }
        "expansion-short-window-selector" => {
            Ok(OfflineHighQualityPath::ExpansionShortWindowSelector)
        }
        _ => Err(format!(
            "invalid offline high-quality path: {value}; expected default, compression-short-window-selector, or expansion-short-window-selector"
        )),
    }
}

impl Default for ReportArgs {
    fn default() -> Self {
        Self {
            report_name: DEFAULT_REPORT_NAME.to_string(),
            projection_epoch: DEFAULT_PROJECTION_EPOCH.to_string(),
            output: None,
            external_benchmark_tool: DEFAULT_EXTERNAL_BENCHMARK_TOOL.to_string(),
            external_benchmark_renders: Vec::new(),
            external_benchmark_render_manifests: Vec::new(),
            export_external_benchmark_pack: None,
            external_benchmark_render_plan_status_manifests: Vec::new(),
            listening_source_manifests: Vec::new(),
            decode_listening_sources: false,
            decode_source_frame_limit: DEFAULT_DECODE_SOURCE_FRAME_LIMIT,
            measure_decoded_stretch: false,
            decoded_stretch_report_mode: DecodedStretchReportMode::Full,
            decoded_stretch_frame_limit: DEFAULT_DECODED_STRETCH_FRAME_LIMIT,
            measure_external_benchmark_quality: false,
            external_benchmark_quality_mode: ExternalBenchmarkQualityMode::Full,
            external_benchmark_signal_path: OfflineHighQualityPath::Default,
            export_blind_listening_pack: None,
            export_tail_listening_pack: None,
            export_tail_classifier_validation_pack: None,
            blind_listening_note_manifests: Vec::new(),
        }
    }
}

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(ParseOutcome::Run(args)) => *args,
        Ok(ParseOutcome::Help) => {
            println!("{}", usage());
            return;
        }
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            process::exit(2);
        }
    };

    let external_renders = match load_external_benchmark_renders(&args) {
        Ok(renders) => renders,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };
    let listening_sources = match load_listening_sources(&args) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };
    let report = build_stretch_corpus_comparison_report_with_sources(
        &args.report_name,
        &args.projection_epoch,
        &external_renders,
        &listening_sources,
    );
    let mut formatted = format_stretch_corpus_comparison_report(&report);
    if args.decode_listening_sources {
        match format_decoded_listening_source_profiles(
            &listening_sources,
            args.decode_source_frame_limit,
        ) {
            Ok(decoded_profiles) => {
                if !decoded_profiles.is_empty() {
                    formatted.push('\n');
                    formatted.push_str(&decoded_profiles);
                }
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    if args.measure_decoded_stretch {
        match format_decoded_stretch_metrics(
            &listening_sources,
            args.decoded_stretch_frame_limit,
            args.decoded_stretch_report_mode,
        ) {
            Ok(decoded_metrics) => {
                if !decoded_metrics.is_empty() {
                    formatted.push('\n');
                    formatted.push_str(&decoded_metrics);
                }
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    if args.measure_external_benchmark_quality {
        let quality_renders = match load_external_benchmark_quality_renders(&args) {
            Ok(renders) => renders,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        match format_external_benchmark_quality_metrics(
            &listening_sources,
            &quality_renders,
            args.decoded_stretch_frame_limit,
            args.external_benchmark_quality_mode,
            args.external_benchmark_signal_path,
        ) {
            Ok(external_quality) => {
                if !external_quality.is_empty() {
                    formatted.push('\n');
                    formatted.push_str(&external_quality);
                }
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    if let Some(export_dir) = &args.export_blind_listening_pack {
        let quality_renders = match load_external_benchmark_quality_renders(&args) {
            Ok(renders) => renders,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        match listening_pack::export_blind_listening_pack(
            &listening_sources,
            &quality_renders,
            args.decoded_stretch_frame_limit,
            args.external_benchmark_signal_path,
            export_dir,
        ) {
            Ok(pack_report) => {
                formatted.push('\n');
                formatted.push_str(&pack_report);
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    if let Some(export_dir) = &args.export_tail_listening_pack {
        let quality_renders = match load_external_benchmark_quality_renders(&args) {
            Ok(renders) => renders,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        match listening_pack::export_tail_listening_pack(
            &listening_sources,
            &quality_renders,
            args.decoded_stretch_frame_limit,
            args.external_benchmark_signal_path,
            export_dir,
        ) {
            Ok(pack_report) => {
                formatted.push('\n');
                formatted.push_str(&pack_report);
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    if let Some(export_dir) = &args.export_tail_classifier_validation_pack {
        let quality_renders = match load_external_benchmark_quality_renders(&args) {
            Ok(renders) => renders,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        match listening_pack::export_tail_classifier_validation_pack(
            &listening_sources,
            &quality_renders,
            args.decoded_stretch_frame_limit,
            args.external_benchmark_signal_path,
            export_dir,
        ) {
            Ok(pack_report) => {
                formatted.push('\n');
                formatted.push_str(&pack_report);
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    for notes_manifest in &args.blind_listening_note_manifests {
        match listening_pack::format_blind_listening_note_status(notes_manifest) {
            Ok(status_report) => {
                formatted.push('\n');
                formatted.push_str(&status_report);
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    if let Some(export_dir) = &args.export_external_benchmark_pack {
        match export_external_benchmark_pack(
            &listening_sources,
            export_dir,
            &args.external_benchmark_tool,
            args.decoded_stretch_frame_limit,
        ) {
            Ok(export_report) => {
                if !export_report.is_empty() {
                    formatted.push('\n');
                    formatted.push_str(&export_report);
                }
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }
    for manifest in &args.external_benchmark_render_plan_status_manifests {
        match format_external_benchmark_render_plan_status(manifest) {
            Ok(status_report) => {
                if !status_report.is_empty() {
                    formatted.push('\n');
                    formatted.push_str(&status_report);
                }
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        }
    }

    if let Some(output) = args.output {
        if let Err(error) = fs::write(&output, formatted) {
            eprintln!("failed to write {}: {error}", output.display());
            process::exit(1);
        }
    } else {
        println!("{formatted}");
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseOutcome {
    Run(Box<ReportArgs>),
    Help,
}

fn parse_args<I>(args: I) -> Result<ParseOutcome, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = ReportArgs::default();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--report-name" => {
                parsed.report_name = next_value(&mut iter, "--report-name")?;
            }
            "--projection-epoch" => {
                parsed.projection_epoch = next_value(&mut iter, "--projection-epoch")?;
            }
            "--output" => {
                parsed.output = Some(PathBuf::from(next_value(&mut iter, "--output")?));
            }
            "--external-benchmark-tool" => {
                parsed.external_benchmark_tool =
                    next_value(&mut iter, "--external-benchmark-tool")?;
            }
            "--external-benchmark-render" => {
                parsed
                    .external_benchmark_renders
                    .push(ExternalBenchmarkRenderArg {
                        case_id: next_value(&mut iter, "--external-benchmark-render CASE")?,
                        ratio: next_value(&mut iter, "--external-benchmark-render RATIO")?,
                        path: PathBuf::from(next_value(
                            &mut iter,
                            "--external-benchmark-render WAV",
                        )?),
                        tool_name: None,
                    });
            }
            "--external-benchmark-render-manifest" => {
                parsed
                    .external_benchmark_render_manifests
                    .push(PathBuf::from(next_value(
                        &mut iter,
                        "--external-benchmark-render-manifest",
                    )?));
            }
            "--export-external-benchmark-pack" => {
                parsed.export_external_benchmark_pack = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--export-external-benchmark-pack",
                )?));
            }
            "--check-external-benchmark-render-plan" => {
                parsed
                    .external_benchmark_render_plan_status_manifests
                    .push(PathBuf::from(next_value(
                        &mut iter,
                        "--check-external-benchmark-render-plan",
                    )?));
            }
            "--listening-source-manifest" => {
                parsed
                    .listening_source_manifests
                    .push(PathBuf::from(next_value(
                        &mut iter,
                        "--listening-source-manifest",
                    )?));
            }
            "--decode-listening-sources" => {
                parsed.decode_listening_sources = true;
            }
            "--decode-source-frame-limit" => {
                parsed.decode_source_frame_limit =
                    next_value(&mut iter, "--decode-source-frame-limit")?
                        .parse::<usize>()
                        .map_err(|error| {
                            format!("invalid --decode-source-frame-limit value: {error}")
                        })?;
            }
            "--measure-decoded-stretch" => {
                parsed.measure_decoded_stretch = true;
            }
            "--decoded-stretch-report-mode" => {
                parsed.decoded_stretch_report_mode = DecodedStretchReportMode::parse(&next_value(
                    &mut iter,
                    "--decoded-stretch-report-mode",
                )?)?;
            }
            "--measure-external-benchmark-quality" => {
                parsed.measure_external_benchmark_quality = true;
            }
            "--external-benchmark-quality-mode" => {
                parsed.external_benchmark_quality_mode = ExternalBenchmarkQualityMode::parse(
                    &next_value(&mut iter, "--external-benchmark-quality-mode")?,
                )?;
            }
            "--external-benchmark-signal-path" => {
                parsed.external_benchmark_signal_path = parse_offline_high_quality_path(
                    &next_value(&mut iter, "--external-benchmark-signal-path")?,
                )?;
            }
            "--export-blind-listening-pack" => {
                parsed.export_blind_listening_pack = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--export-blind-listening-pack",
                )?));
            }
            "--export-tail-listening-pack" => {
                parsed.export_tail_listening_pack = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--export-tail-listening-pack",
                )?));
            }
            "--export-tail-classifier-validation-pack" => {
                parsed.export_tail_classifier_validation_pack = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--export-tail-classifier-validation-pack",
                )?));
            }
            "--check-blind-listening-notes" => {
                parsed
                    .blind_listening_note_manifests
                    .push(PathBuf::from(next_value(
                        &mut iter,
                        "--check-blind-listening-notes",
                    )?));
            }
            "--decoded-stretch-frame-limit" => {
                parsed.decoded_stretch_frame_limit =
                    next_value(&mut iter, "--decoded-stretch-frame-limit")?
                        .parse::<usize>()
                        .map_err(|error| {
                            format!("invalid --decoded-stretch-frame-limit value: {error}")
                        })?;
            }
            "--help" | "-h" => {
                return Ok(ParseOutcome::Help);
            }
            unknown => {
                return Err(format!("unknown argument: {unknown}"));
            }
        }
    }
    Ok(ParseOutcome::Run(Box::new(parsed)))
}

fn next_value<I>(iter: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing value for {name}"))
}

fn usage() -> &'static str {
    "usage: stretch-corpus-report [--report-name NAME] [--projection-epoch EPOCH] [--listening-source-manifest TSV] [--decode-listening-sources] [--decode-source-frame-limit N] [--measure-decoded-stretch] [--decoded-stretch-report-mode full|expansion-selector] [--measure-external-benchmark-quality] [--external-benchmark-quality-mode core|full] [--external-benchmark-signal-path default|compression-short-window-selector|expansion-short-window-selector] [--export-blind-listening-pack DIR] [--export-tail-listening-pack DIR] [--export-tail-classifier-validation-pack DIR] [--check-blind-listening-notes TSV] [--decoded-stretch-frame-limit N] [--external-benchmark-tool NAME] [--external-benchmark-render CASE RATIO WAV] [--external-benchmark-render-manifest TSV] [--export-external-benchmark-pack DIR] [--check-external-benchmark-render-plan TSV] [--output PATH]"
}

fn load_external_benchmark_renders(
    args: &ReportArgs,
) -> Result<Vec<StretchExternalBenchmarkRender>, String> {
    let mut renders = args
        .external_benchmark_renders
        .iter()
        .map(|render| load_external_benchmark_render(args, render))
        .collect::<Result<Vec<_>, _>>()?;
    for manifest in &args.external_benchmark_render_manifests {
        renders.extend(load_external_benchmark_render_manifest(args, manifest)?);
    }
    Ok(renders)
}

fn load_external_benchmark_render_manifest(
    args: &ReportArgs,
    manifest: &PathBuf,
) -> Result<Vec<StretchExternalBenchmarkRender>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(manifest)
        .map_err(|error| format!("failed to open {}: {error}", manifest.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to read {} headers: {error}", manifest.display()))?
        .clone();
    let mut renders = Vec::new();
    for row in reader.records() {
        let record =
            row.map_err(|error| format!("failed to read {} row: {error}", manifest.display()))?;
        let case_id = required_field(manifest, &headers, &record, "case_id")?;
        let ratio = required_field(manifest, &headers, &record, "ratio")?;
        let path = required_any_field(manifest, &headers, &record, &["rendered_path", "path"])?;
        let tool_name = field(&headers, &record, "tool_name")
            .or_else(|| field(&headers, &record, "tool"))
            .map(str::to_string);
        renders.push(load_external_benchmark_render(
            args,
            &ExternalBenchmarkRenderArg {
                case_id: case_id.to_string(),
                ratio: ratio.to_string(),
                path: PathBuf::from(path),
                tool_name,
            },
        )?);
    }
    Ok(renders)
}

fn load_external_benchmark_render(
    args: &ReportArgs,
    render: &ExternalBenchmarkRenderArg,
) -> Result<StretchExternalBenchmarkRender, String> {
    let ratio = parse_external_benchmark_ratio(&render.ratio)?;

    let reader = hound::WavReader::open(&render.path)
        .map_err(|error| format!("failed to open {}: {error}", render.path.display()))?;
    let spec = reader.spec();
    let channels = spec.channels;
    if channels == 0 {
        return Err(format!(
            "invalid external benchmark WAV {}: channel count is zero",
            render.path.display()
        ));
    }
    let rendered_frames = reader.duration() as usize;

    Ok(StretchExternalBenchmarkRender {
        case_id: render.case_id.clone(),
        ratio,
        pitch_shift_semitones: None,
        tool_name: render
            .tool_name
            .clone()
            .unwrap_or_else(|| args.external_benchmark_tool.clone()),
        rendered_path: render.path.display().to_string(),
        rendered_frames,
        sample_rate_hz: spec.sample_rate,
        channels,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkQualityRender {
    case_id: String,
    ratio: f64,
    tool_name: String,
    rendered_path: String,
    source_wav: Option<String>,
}

fn load_external_benchmark_quality_renders(
    args: &ReportArgs,
) -> Result<Vec<ExternalBenchmarkQualityRender>, String> {
    let mut renders = args
        .external_benchmark_renders
        .iter()
        .map(|render| {
            let ratio = parse_external_benchmark_ratio(&render.ratio)?;
            Ok(ExternalBenchmarkQualityRender {
                case_id: render.case_id.clone(),
                ratio,
                tool_name: render
                    .tool_name
                    .clone()
                    .unwrap_or_else(|| args.external_benchmark_tool.clone()),
                rendered_path: render.path.display().to_string(),
                source_wav: None,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    for manifest in &args.external_benchmark_render_manifests {
        renders.extend(load_external_benchmark_quality_render_manifest(
            args, manifest,
        )?);
    }
    Ok(renders)
}

fn load_external_benchmark_quality_render_manifest(
    args: &ReportArgs,
    manifest: &PathBuf,
) -> Result<Vec<ExternalBenchmarkQualityRender>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(manifest)
        .map_err(|error| format!("failed to open {}: {error}", manifest.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to read {} headers: {error}", manifest.display()))?
        .clone();
    let mut renders = Vec::new();
    for row in reader.records() {
        let record =
            row.map_err(|error| format!("failed to read {} row: {error}", manifest.display()))?;
        let case_id = required_field(manifest, &headers, &record, "case_id")?;
        let ratio =
            parse_external_benchmark_ratio(required_field(manifest, &headers, &record, "ratio")?)?;
        let rendered_path =
            required_any_field(manifest, &headers, &record, &["rendered_path", "path"])?;
        let tool_name = field(&headers, &record, "tool_name")
            .or_else(|| field(&headers, &record, "tool"))
            .unwrap_or(&args.external_benchmark_tool);
        let source_wav = field(&headers, &record, "source_wav").map(str::to_string);
        renders.push(ExternalBenchmarkQualityRender {
            case_id: case_id.to_string(),
            ratio,
            tool_name: tool_name.to_string(),
            rendered_path: rendered_path.to_string(),
            source_wav,
        });
    }
    Ok(renders)
}

fn parse_external_benchmark_ratio(ratio: &str) -> Result<f64, String> {
    let parsed = ratio
        .parse::<f64>()
        .map_err(|error| format!("invalid external benchmark ratio {ratio}: {error}"))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!(
            "invalid external benchmark ratio {ratio}: expected positive finite value",
        ));
    }
    Ok(parsed)
}

fn export_external_benchmark_pack(
    sources: &[StretchCorpusListeningSource],
    export_dir: &Path,
    tool_name: &str,
    frame_limit: usize,
) -> Result<String, String> {
    let source_dir = export_dir.join("sources");
    let render_dir = export_dir.join("renders");
    fs::create_dir_all(&source_dir)
        .map_err(|error| format!("failed to create {}: {error}", source_dir.display()))?;
    fs::create_dir_all(&render_dir)
        .map_err(|error| format!("failed to create {}: {error}", render_dir.display()))?;

    let manifest_path = export_dir.join("external-benchmark-render-plan.tsv");
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&manifest_path)
        .map_err(|error| format!("failed to create {}: {error}", manifest_path.display()))?;
    writer
        .write_record([
            "case_id",
            "ratio",
            "source_wav",
            "rendered_path",
            "tool_name",
        ])
        .map_err(|error| {
            format!(
                "failed to write {} header: {error}",
                manifest_path.display()
            )
        })?;

    let mut lines = Vec::new();
    let mut exported_sources = 0usize;
    let mut render_slots = 0usize;
    for (index, source) in sources.iter().enumerate() {
        let audio = decode_listening_source_audio(source, frame_limit)?;
        let source_stem = source_export_stem(index, source);
        let source_wav = source_dir.join(format!("{source_stem}.wav"));
        write_decoded_audio_wav(&source_wav, &audio)?;
        exported_sources += 1;
        for &ratio in listening_source_ratios(&source.case_id)? {
            let ratio_label = ratio_export_label(ratio);
            let rendered_path = render_dir.join(format!("{source_stem}-ratio-{ratio_label}.wav"));
            let ratio_field = format!("{ratio:.6}");
            let source_wav_field = source_wav.display().to_string();
            let rendered_path_field = rendered_path.display().to_string();
            writer
                .write_record([
                    source.case_id.as_str(),
                    &ratio_field,
                    &source_wav_field,
                    &rendered_path_field,
                    tool_name,
                ])
                .map_err(|error| {
                    format!("failed to write {} row: {error}", manifest_path.display())
                })?;
            render_slots += 1;
            lines.push(format!(
                "external_benchmark_render_plan case={} source={} ratio={:.6} source_wav={} rendered_path={} tool={} source_boundary={}",
                source.case_id,
                quoted_report_field(&source.source_path),
                ratio,
                quoted_report_field(&source_wav.display().to_string()),
                quoted_report_field(&rendered_path.display().to_string()),
                quoted_report_field(tool_name),
                quoted_report_field("operator-provided rendered output; no external library dependency"),
            ));
        }
    }
    writer
        .flush()
        .map_err(|error| format!("failed to flush {}: {error}", manifest_path.display()))?;

    let mut report = vec![format!(
        "external_benchmark_render_pack export_dir={} manifest={} exported_sources={} render_slots={} tool={} frame_limit={} source_boundary={}",
        quoted_report_field(&export_dir.display().to_string()),
        quoted_report_field(&manifest_path.display().to_string()),
        exported_sources,
        render_slots,
        quoted_report_field(tool_name),
        frame_limit,
        quoted_report_field("source excerpts are operator-provided licensed local audio; rendered outputs remain external black-box evidence"),
    )];
    report.extend(lines);
    Ok(report.join("\n"))
}

fn format_external_benchmark_render_plan_status(manifest: &PathBuf) -> Result<String, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(manifest)
        .map_err(|error| format!("failed to open {}: {error}", manifest.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to read {} headers: {error}", manifest.display()))?
        .clone();

    let mut planned_rows = 0usize;
    let mut present_rows = 0usize;
    let mut missing_rows = 0usize;
    let mut invalid_rows = 0usize;
    let mut missing_detail_rows = Vec::new();
    for row in reader.records() {
        let record =
            row.map_err(|error| format!("failed to read {} row: {error}", manifest.display()))?;
        planned_rows += 1;
        let case_id = required_field(manifest, &headers, &record, "case_id")?;
        let ratio = required_field(manifest, &headers, &record, "ratio")?;
        let source_wav = field(&headers, &record, "source_wav").unwrap_or("");
        let tool_name = field(&headers, &record, "tool_name")
            .or_else(|| field(&headers, &record, "tool"))
            .unwrap_or("");
        let rendered_path =
            required_any_field(manifest, &headers, &record, &["rendered_path", "path"])?;
        let rendered_path = PathBuf::from(rendered_path);
        if !rendered_path.exists() {
            missing_rows += 1;
            if missing_detail_rows.len() < MAX_EXTERNAL_BENCHMARK_MISSING_RENDER_ROWS {
                missing_detail_rows.push(format!(
                    "external_benchmark_render_plan_missing case={} ratio={} source_wav={} rendered_path={} tool={} manifest={}",
                    case_id,
                    ratio,
                    quoted_report_field(source_wav),
                    quoted_report_field(&rendered_path.display().to_string()),
                    quoted_report_field(tool_name),
                    quoted_report_field(&manifest.display().to_string()),
                ));
            }
            continue;
        }

        match hound::WavReader::open(&rendered_path) {
            Ok(reader) if reader.spec().channels > 0 && reader.spec().sample_rate > 0 => {
                present_rows += 1;
            }
            Ok(_) | Err(_) => {
                invalid_rows += 1;
            }
        }
    }
    let status = if planned_rows == 0 {
        "Empty"
    } else if missing_rows == 0 && invalid_rows == 0 {
        "Complete"
    } else {
        "Incomplete"
    };

    let mut lines = vec![format!(
        "external_benchmark_render_plan_status manifest={} status={} planned_rows={} present_rows={} missing_rows={} invalid_rows={} capped_missing_rows={} source_boundary={}",
        quoted_report_field(&manifest.display().to_string()),
        status,
        planned_rows,
        present_rows,
        missing_rows,
        invalid_rows,
        missing_detail_rows.len(),
        quoted_report_field("rendered-output-only readiness check; does not load external code"),
    )];
    lines.extend(missing_detail_rows);
    Ok(lines.join("\n"))
}

fn write_decoded_audio_wav(path: &Path, audio: &DecodedListeningSourceAudio) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: audio.channels,
        sample_rate: audio.sample_rate_hz,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
    for sample in &audio.samples {
        writer
            .write_sample(*sample)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }
    writer
        .finalize()
        .map_err(|error| format!("failed to finalize {}: {error}", path.display()))
}

fn source_export_stem(index: usize, source: &StretchCorpusListeningSource) -> String {
    let source_name = Path::new(&source.source_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("source");
    format!(
        "{index:04}-{}-{}",
        sanitize_path_component(&source.case_id.replace("stretch:", "")),
        sanitize_path_component(source_name)
    )
}

fn ratio_export_label(ratio: f64) -> String {
    format!("{ratio:.6}").replace('.', "p")
}

fn sanitize_path_component(value: &str) -> String {
    let mut sanitized = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
            sanitized.push(character);
        } else {
            sanitized.push('-');
        }
    }
    sanitized.trim_matches('-').to_string()
}

fn load_listening_sources(args: &ReportArgs) -> Result<Vec<StretchCorpusListeningSource>, String> {
    let mut sources = Vec::new();
    for manifest in &args.listening_source_manifests {
        sources.extend(load_listening_source_manifest(manifest)?);
    }
    Ok(sources)
}

fn load_listening_source_manifest(
    manifest: &PathBuf,
) -> Result<Vec<StretchCorpusListeningSource>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(manifest)
        .map_err(|error| format!("failed to open {}: {error}", manifest.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to read {} headers: {error}", manifest.display()))?
        .clone();
    let mut sources = Vec::new();

    for row in reader.records() {
        let record =
            row.map_err(|error| format!("failed to read {} row: {error}", manifest.display()))?;
        let case_id = required_field(manifest, &headers, &record, "case_id")?;
        if !is_operator_listening_case(case_id) {
            return Err(format!(
                "{} uses unsupported listening case id {case_id}",
                manifest.display()
            ));
        }
        let source_path =
            required_any_field(manifest, &headers, &record, &["source_path", "local_path"])?;
        let source_path_buf = PathBuf::from(source_path);
        if !source_path_buf.exists() {
            return Err(format!(
                "{} references missing source {}",
                manifest.display(),
                source_path_buf.display()
            ));
        }

        sources.push(StretchCorpusListeningSource {
            case_id: case_id.to_string(),
            source_path: source_path.to_string(),
            source_label: listening_source_label(&headers, &record),
            license_title: field(&headers, &record, "license_title")
                .unwrap_or("unknown")
                .to_string(),
            license_url: field(&headers, &record, "license_url")
                .unwrap_or("")
                .to_string(),
            provenance_url: field(&headers, &record, "provenance_url")
                .or_else(|| field(&headers, &record, "track_url"))
                .unwrap_or("")
                .to_string(),
        });
    }

    Ok(sources)
}

fn is_operator_listening_case(case_id: &str) -> bool {
    STRETCH_CORPUS_MANIFEST.entries.iter().any(|entry| {
        entry.case.case_id == case_id
            && entry.asset_requirement == StretchCorpusAssetRequirement::OperatorProvidedAudio
    })
}

fn required_any_field<'a>(
    manifest: &PathBuf,
    headers: &csv::StringRecord,
    record: &'a csv::StringRecord,
    names: &[&str],
) -> Result<&'a str, String> {
    names
        .iter()
        .find_map(|name| field(headers, record, name))
        .ok_or_else(|| {
            format!(
                "{} is missing required field {}",
                manifest.display(),
                names.join(" or ")
            )
        })
}

fn required_field<'a>(
    manifest: &PathBuf,
    headers: &csv::StringRecord,
    record: &'a csv::StringRecord,
    name: &str,
) -> Result<&'a str, String> {
    field(headers, record, name)
        .ok_or_else(|| format!("{} is missing required field {name}", manifest.display()))
}

fn field<'a>(
    headers: &csv::StringRecord,
    record: &'a csv::StringRecord,
    name: &str,
) -> Option<&'a str> {
    headers
        .iter()
        .position(|header| header == name)
        .and_then(|index| record.get(index))
        .filter(|value| !value.is_empty())
}

fn listening_source_label(headers: &csv::StringRecord, record: &csv::StringRecord) -> String {
    if let Some(label) = field(headers, record, "source_label") {
        return label.to_string();
    }
    match (
        field(headers, record, "artist"),
        field(headers, record, "title"),
    ) {
        (Some(artist), Some(title)) => format!("{artist} - {title}"),
        (Some(artist), None) => artist.to_string(),
        (None, Some(title)) => title.to_string(),
        (None, None) => "operator listening source".to_string(),
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DecodedListeningSourceProfile {
    case_id: String,
    source_path: String,
    sample_rate_hz: u32,
    channels: u16,
    analyzed_frames: usize,
    analysis_limited: bool,
    peak: f64,
    rms: f64,
    zero_crossings_per_second: f64,
    transient_count: usize,
    transient_density_per_second: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct DecodedListeningSourceAudio {
    case_id: String,
    source_path: String,
    sample_rate_hz: u32,
    channels: u16,
    samples: Vec<f32>,
    analysis_limited: bool,
}

impl DecodedListeningSourceAudio {
    fn analyzed_frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    fn mono_samples(&self) -> Vec<f32> {
        let channel_count = self.channels as usize;
        self.samples
            .chunks_exact(channel_count)
            .map(|frame| frame.iter().sum::<f32>() / channel_count as f32)
            .collect()
    }
}

fn format_decoded_listening_source_profiles(
    sources: &[StretchCorpusListeningSource],
    frame_limit: usize,
) -> Result<String, String> {
    let mut lines = Vec::new();
    for source in sources {
        let profile = decode_listening_source_profile(source, frame_limit)?;
        lines.push(format!(
            "decoded_listening_source case={} source={} sample_rate={} channels={} analyzed_frames={} analysis_limited={} peak={:.6} rms={:.6} zero_crossings_per_second={:.6} transient_count={} transient_density_per_second={:.6}",
            profile.case_id,
            quoted_report_field(&profile.source_path),
            profile.sample_rate_hz,
            profile.channels,
            profile.analyzed_frames,
            profile.analysis_limited,
            profile.peak,
            profile.rms,
            profile.zero_crossings_per_second,
            profile.transient_count,
            profile.transient_density_per_second
        ));
    }
    Ok(lines.join("\n"))
}

fn decode_listening_source_profile(
    source: &StretchCorpusListeningSource,
    frame_limit: usize,
) -> Result<DecodedListeningSourceProfile, String> {
    let audio = decode_listening_source_audio(source, frame_limit)?;
    profile_from_decoded_audio(&audio)
}

fn decode_listening_source_audio(
    source: &StretchCorpusListeningSource,
    frame_limit: usize,
) -> Result<DecodedListeningSourceAudio, String> {
    let path = PathBuf::from(&source.source_path);
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
    {
        decode_wav_source_audio(source, &path, frame_limit)
    } else {
        decode_symphonia_source_audio(source, &path, frame_limit)
    }
}

fn decode_wav_source_audio(
    source: &StretchCorpusListeningSource,
    path: &Path,
    frame_limit: usize,
) -> Result<DecodedListeningSourceAudio, String> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|error| format!("failed to open source {}: {error}", path.display()))?;
    let spec = reader.spec();
    if spec.channels == 0 || spec.sample_rate == 0 {
        return Err(format!(
            "invalid source WAV {}: sample rate and channels must be non-zero",
            path.display()
        ));
    }
    let total_frames = reader.duration() as usize;
    let max_samples = sample_limit(frame_limit, spec.channels as usize);
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .take(max_samples.unwrap_or(usize::MAX))
            .map(|sample| {
                sample.map_err(|error| format!("failed to read {}: {error}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = integer_sample_scale(spec.bits_per_sample);
            reader
                .samples::<i32>()
                .take(max_samples.unwrap_or(usize::MAX))
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / scale)
                        .map_err(|error| format!("failed to read {}: {error}", path.display()))
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    decoded_audio_from_interleaved_samples(
        source,
        spec.sample_rate,
        spec.channels,
        samples,
        frame_analysis_limited(frame_limit, total_frames),
    )
}

fn decode_symphonia_source_audio(
    source: &StretchCorpusListeningSource,
    path: &Path,
    frame_limit: usize,
) -> Result<DecodedListeningSourceAudio, String> {
    let file = File::open(path)
        .map_err(|error| format!("failed to open source {}: {error}", path.display()))?;
    let media_stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = SymphoniaHint::new();
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            media_stream,
            &SymphoniaFormatOptions::default(),
            &SymphoniaMetadataOptions::default(),
        )
        .map_err(|error| format!("failed to probe source {}: {error}", path.display()))?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|track| track.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| format!("source {} has no supported audio track", path.display()))?;
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &SymphoniaDecoderOptions::default())
        .map_err(|error| format!("failed to create decoder for {}: {error}", path.display()))?;

    let mut sample_rate_hz = None;
    let mut output_channels = None;
    let mut samples = Vec::new();
    let mut analyzed_frames = 0usize;
    let mut limited = false;

    loop {
        if frame_limit > 0 && analyzed_frames >= frame_limit {
            limited = true;
            break;
        }
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "failed to read source packet for {}: {error}",
                    path.display()
                ));
            }
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => {
                return Err(format!(
                    "failed to decode source {}: {error}",
                    path.display()
                ));
            }
        };
        let spec = *decoded.spec();
        let actual_channels = spec.channels.count();
        if actual_channels == 0 || spec.rate == 0 {
            continue;
        }
        let channels = actual_channels.clamp(1, 2);
        sample_rate_hz.get_or_insert(spec.rate);
        output_channels.get_or_insert(channels as u16);

        let mut sample_buffer = SymphoniaSampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        sample_buffer.copy_interleaved_ref(decoded);
        let interleaved = sample_buffer.samples();
        for frame in interleaved.chunks(actual_channels) {
            if frame_limit > 0 && analyzed_frames >= frame_limit {
                limited = true;
                break;
            }
            for sample in frame.iter().take(channels) {
                samples.push(*sample);
            }
            analyzed_frames += 1;
        }
    }

    decoded_audio_from_interleaved_samples(
        source,
        sample_rate_hz
            .ok_or_else(|| format!("source {} produced no sample rate", path.display()))?,
        output_channels.ok_or_else(|| format!("source {} produced no channels", path.display()))?,
        samples,
        limited,
    )
}

fn decoded_audio_from_interleaved_samples(
    source: &StretchCorpusListeningSource,
    sample_rate_hz: u32,
    channels: u16,
    samples: Vec<f32>,
    analysis_limited: bool,
) -> Result<DecodedListeningSourceAudio, String> {
    let channel_count = channels as usize;
    if sample_rate_hz == 0 || channel_count == 0 || samples.len() < channel_count {
        return Err(format!(
            "source {} produced no decodable audio frames",
            source.source_path
        ));
    }
    Ok(DecodedListeningSourceAudio {
        case_id: source.case_id.clone(),
        source_path: source.source_path.clone(),
        sample_rate_hz,
        channels,
        samples,
        analysis_limited,
    })
}

fn profile_from_decoded_audio(
    audio: &DecodedListeningSourceAudio,
) -> Result<DecodedListeningSourceProfile, String> {
    let channel_count = audio.channels as usize;
    if audio.sample_rate_hz == 0 || channel_count == 0 || audio.samples.len() < channel_count {
        return Err(format!(
            "source {} produced no decodable audio frames",
            audio.source_path
        ));
    }
    let analyzed_frames = audio.analyzed_frames();
    let mono = audio.mono_samples();
    if mono.is_empty() {
        return Err(format!(
            "source {} produced no decodable audio frames",
            audio.source_path
        ));
    }

    let peak = mono
        .iter()
        .map(|sample| sample.abs() as f64)
        .fold(0.0, f64::max);
    let rms = (mono
        .iter()
        .map(|sample| {
            let value = *sample as f64;
            value * value
        })
        .sum::<f64>()
        / mono.len() as f64)
        .sqrt();
    let zero_crossings = mono
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count();
    let duration_seconds = analyzed_frames as f64 / audio.sample_rate_hz as f64;
    let transients = detect_stretch_transients(&mono, 1024, 256);

    Ok(DecodedListeningSourceProfile {
        case_id: audio.case_id.clone(),
        source_path: audio.source_path.clone(),
        sample_rate_hz: audio.sample_rate_hz,
        channels: audio.channels,
        analyzed_frames,
        analysis_limited: audio.analysis_limited,
        peak,
        rms,
        zero_crossings_per_second: zero_crossings as f64 / duration_seconds.max(1.0e-12),
        transient_count: transients.len(),
        transient_density_per_second: transients.len() as f64 / duration_seconds.max(1.0e-12),
    })
}

fn format_decoded_stretch_metrics(
    sources: &[StretchCorpusListeningSource],
    frame_limit: usize,
    mode: DecodedStretchReportMode,
) -> Result<String, String> {
    let mut lines = Vec::new();
    let mut draft_recovery_gate = TransientRecoveryGateAccumulator::new("draft");
    let mut offline_recovery_gate = TransientRecoveryGateAccumulator::new("offline_hq");
    let mut compression_ablation = CompressionPhaseLockAblationAccumulator::default();
    let mut compression_anchor_candidate = CompressionReviewCandidateAccumulator::new(
        "decoded_compression_transient_anchor_candidate",
        "offline_hq_compression_anchor",
    );
    let mut compression_short_window_candidate = CompressionReviewCandidateAccumulator::new(
        "decoded_compression_short_window_candidate",
        "offline_hq_short_window",
    )
    .with_feature_report("decoded_compression_short_window_feature");
    let mut expansion_short_window_candidate =
        CompressionReviewCandidateAccumulator::new_expansion(
            "decoded_expansion_short_window_candidate",
            "offline_hq_short_window",
        )
        .with_feature_report("decoded_expansion_short_window_feature");
    let mut expansion_short_window_selector_candidate =
        ExpansionShortWindowSelectorCandidateAccumulator::default();
    let mut compression_short_window_selector_candidate =
        ShortWindowSelectorCandidateAccumulator::default();
    let mut compression_short_window_selector_path = ShortWindowSelectorPathAccumulator::default();
    let mut matched_width_review = MatchedTransientWidthReviewAccumulator::default();
    let mut width_control_candidate = TransientWidthControlCandidateAccumulator::default();
    let mut width_control_edit_gate = TransientWidthControlEditGateAccumulator::default();
    for source in sources {
        let audio = decode_listening_source_audio(source, frame_limit)?;
        let mono = audio.mono_samples();
        for &ratio in listening_source_ratios(&source.case_id)? {
            if mode == DecodedStretchReportMode::ExpansionSelector && ratio <= 1.0 {
                continue;
            }
            let mut draft = PhaseVocoderStretcher::new(ratio);
            let draft_output = draft.stretch_mono(&mono);
            let mut offline = OfflineHighQualityStretcher::new(ratio);
            let offline_output = offline.stretch_mono(&mono);

            if mode == DecodedStretchReportMode::ExpansionSelector {
                let draft_smear = measure_transient_smear(
                    &mono,
                    &draft_output,
                    ratio,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                );
                let offline_smear = measure_transient_smear(
                    &mono,
                    &offline_output,
                    ratio,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                );
                let mut offline_short_window = OfflineHighQualityStretcher::with_window(
                    ratio,
                    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
                    COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
                );
                let offline_short_window_output = offline_short_window.stretch_mono(&mono);
                let offline_short_window_smear = measure_transient_smear(
                    &mono,
                    &offline_short_window_output,
                    ratio,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                );
                expansion_short_window_candidate.record(
                    &audio,
                    ratio,
                    &draft_smear,
                    &offline_smear,
                    &offline_short_window_smear,
                );
                expansion_short_window_selector_candidate.record(
                    &audio,
                    ratio,
                    &draft_smear,
                    &offline_smear,
                    &offline_short_window_smear,
                );
                continue;
            }

            lines.push(format_decoded_stretch_metric_line(
                &audio,
                ratio,
                "TimingDriftSamples",
                output_length_drift_samples(mono.len(), draft_output.len(), ratio),
                output_length_drift_samples(mono.len(), offline_output.len(), ratio),
                None,
            ));

            let draft_strict_smear = measure_transient_smear_with_policy(
                &mono,
                &draft_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
                DETECTOR_POLICY,
            );
            let offline_strict_smear = measure_transient_smear_with_policy(
                &mono,
                &offline_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
                DETECTOR_POLICY,
            );
            let draft_candidate_smear = measure_transient_smear_with_policy(
                &mono,
                &draft_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
                CANDIDATE_DETECTOR_POLICY,
            );
            let offline_candidate_smear = measure_transient_smear_with_policy(
                &mono,
                &offline_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
                CANDIDATE_DETECTOR_POLICY,
            );
            let draft_candidate_output_smear = measure_transient_smear_with_policies(
                &mono,
                &draft_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
                DETECTOR_POLICY,
                CANDIDATE_DETECTOR_POLICY,
            );
            let offline_candidate_output_smear = measure_transient_smear_with_policies(
                &mono,
                &offline_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
                DETECTOR_POLICY,
                CANDIDATE_DETECTOR_POLICY,
            );
            let draft_candidate_recovery_smear =
                measure_transient_smear_with_output_recovery_policy(
                    &mono,
                    &draft_output,
                    ratio,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                    DETECTOR_POLICY,
                    DETECTOR_POLICY,
                    CANDIDATE_DETECTOR_POLICY,
                );
            let offline_candidate_recovery_smear =
                measure_transient_smear_with_output_recovery_policy(
                    &mono,
                    &offline_output,
                    ratio,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                    DETECTOR_POLICY,
                    DETECTOR_POLICY,
                    CANDIDATE_DETECTOR_POLICY,
                );
            let draft_smear = measure_transient_smear(
                &mono,
                &draft_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            );
            let offline_smear = measure_transient_smear(
                &mono,
                &offline_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            );
            let offline_compression_anchor_output =
                offline.stretch_compression_transient_anchor_review_mono(&mono);
            let offline_compression_anchor_smear = measure_transient_smear(
                &mono,
                &offline_compression_anchor_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            );
            let mut offline_short_window = OfflineHighQualityStretcher::with_window(
                ratio,
                COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
                COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
            );
            let offline_short_window_output = offline_short_window.stretch_mono(&mono);
            let offline_short_window_smear = measure_transient_smear(
                &mono,
                &offline_short_window_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            );
            let mut offline_short_window_selector = OfflineHighQualityStretcher::with_path(
                ratio,
                OfflineHighQualityPath::CompressionShortWindowSelector,
            );
            let offline_short_window_selector_output =
                offline_short_window_selector.stretch_mono(&mono);
            let offline_short_window_selector_smear = measure_transient_smear(
                &mono,
                &offline_short_window_selector_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            );
            let offline_width_control_output =
                apply_transient_width_control_candidate(&mono, &offline_output, ratio);
            let offline_width_control_smear = measure_transient_smear(
                &mono,
                &offline_width_control_output,
                ratio,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            );
            let offline_width_control_edit =
                width_control_edit_stats(&offline_output, &offline_width_control_output, ratio);
            compression_ablation.record(&audio, ratio, &draft_smear, &offline_smear);
            compression_anchor_candidate.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_compression_anchor_smear,
            );
            compression_short_window_candidate.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_short_window_smear,
            );
            expansion_short_window_candidate.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_short_window_smear,
            );
            expansion_short_window_selector_candidate.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_short_window_smear,
            );
            compression_short_window_selector_candidate.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_short_window_smear,
            );
            compression_short_window_selector_path.record(
                ratio,
                &offline_output,
                &offline_short_window_output,
                &offline_short_window_selector_output,
                &offline_smear,
                &offline_short_window_smear,
                &offline_short_window_selector_smear,
            );
            matched_width_review.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_short_window_smear,
                &offline_short_window_selector_smear,
            );
            width_control_candidate.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_width_control_smear,
                offline_width_control_edit.clone(),
            );
            width_control_edit_gate.record(
                &audio,
                ratio,
                &offline_smear,
                &offline_width_control_smear,
                &offline_width_control_edit,
            );
            draft_recovery_gate.record(
                &draft_strict_smear,
                &draft_candidate_smear,
                &draft_candidate_recovery_smear,
            );
            offline_recovery_gate.record(
                &offline_strict_smear,
                &offline_candidate_smear,
                &offline_candidate_recovery_smear,
            );
            let draft_alignment = transient_alignment_diagnostic(&mono, &draft_output, ratio);
            let offline_alignment = transient_alignment_diagnostic(&mono, &offline_output, ratio);
            lines.push(format_decoded_stretch_metric_line(
                &audio,
                ratio,
                "TransientSmearFrames",
                draft_smear.max_smear_frames,
                offline_smear.max_smear_frames,
                Some(format_transient_metric_detail(
                    &draft_smear,
                    &offline_smear,
                    &draft_strict_smear,
                    &offline_strict_smear,
                    &draft_candidate_smear,
                    &offline_candidate_smear,
                    &draft_candidate_output_smear,
                    &offline_candidate_output_smear,
                    &draft_candidate_recovery_smear,
                    &offline_candidate_recovery_smear,
                    &draft_alignment,
                    &offline_alignment,
                )),
            ));
            lines.extend(format_transient_alignment_event_lines(
                &audio,
                ratio,
                "draft",
                &draft_alignment,
            ));
            lines.extend(format_transient_alignment_event_lines(
                &audio,
                ratio,
                "offline_hq",
                &offline_alignment,
            ));
        }
    }
    if draft_recovery_gate.rows > 0 {
        lines.push(draft_recovery_gate.format_report_line());
    }
    if offline_recovery_gate.rows > 0 {
        lines.push(offline_recovery_gate.format_report_line());
    }
    if compression_ablation.rows > 0 {
        lines.push(compression_ablation.format_report_line());
    }
    if compression_anchor_candidate.rows > 0 {
        lines.push(compression_anchor_candidate.format_report_line());
    }
    if compression_short_window_candidate.rows > 0 {
        lines.push(compression_short_window_candidate.format_report_line());
        lines.extend(compression_short_window_candidate.format_feature_lines());
    }
    if expansion_short_window_candidate.rows > 0 {
        lines.push(expansion_short_window_candidate.format_report_line());
        lines.extend(expansion_short_window_candidate.format_feature_lines());
    }
    if expansion_short_window_selector_candidate.rows > 0 {
        lines.push(expansion_short_window_selector_candidate.format_report_line());
    }
    if compression_short_window_selector_candidate.rows > 0 {
        lines.push(compression_short_window_selector_candidate.format_report_line());
    }
    if compression_short_window_selector_path.rows > 0 {
        lines.push(compression_short_window_selector_path.format_report_line());
    }
    if matched_width_review.rows > 0 {
        lines.push(matched_width_review.format_report_line());
    }
    if width_control_candidate.rows > 0 {
        lines.push(width_control_candidate.format_report_line());
        lines.extend(width_control_candidate.format_edit_event_lines());
    }
    if width_control_edit_gate.rows > 0 {
        lines.push(width_control_edit_gate.format_report_line());
    }
    Ok(lines.join("\n"))
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkQualityMeasurement {
    case_id: String,
    source_path: String,
    signal_path: OfflineHighQualityPath,
    render_path: String,
    tool_name: String,
    ratio: f64,
    status: &'static str,
    reason: &'static str,
    source_boundary: &'static str,
    sample_rate_match: bool,
    source_sample_rate_hz: u32,
    external_sample_rate_hz: u32,
    external_channels: u16,
    source_frames: usize,
    signal_frames: usize,
    external_frames: usize,
    signal_timing_drift_samples: f64,
    external_timing_drift_samples: f64,
    timing_drift_delta_samples: f64,
    signal_transient_smear_frames: f64,
    external_transient_smear_frames: f64,
    transient_smear_delta_frames: f64,
    signal_transient_matches: usize,
    external_transient_matches: usize,
    signal_transient_mean_signed_offset_frames: f64,
    external_transient_mean_signed_offset_frames: f64,
    signal_transient_mean_absolute_offset_frames: f64,
    external_transient_mean_absolute_offset_frames: f64,
    signal_transient_max_absolute_offset_frames: f64,
    external_transient_max_absolute_offset_frames: f64,
    signal_transient_max_crest_growth_db: f64,
    external_transient_max_crest_growth_db: f64,
    signal_transient_max_crest_input_frame: usize,
    external_transient_max_crest_input_frame: usize,
    signal_transient_max_crest_output_frame: usize,
    external_transient_max_crest_output_frame: usize,
    draft_transient_mean_absolute_offset_frames: f64,
    draft_transient_max_crest_growth_db: f64,
    draft_transient_max_crest_input_frame: usize,
    draft_transient_max_crest_output_frame: usize,
    alignment_lag_frames: isize,
    aligned_compared_frames: usize,
    aligned_correlation: f64,
    aligned_rms_error: f64,
    aligned_peak_error: f64,
    signal_rms: f64,
    external_rms: f64,
    aligned_rms_error_ratio: f64,
    integrity_limit_id: &'static str,
    signal_integrity_passed: bool,
    external_integrity_passed: bool,
    signal_measured_endpoint_count: u8,
    external_measured_endpoint_count: u8,
    signal_endpoint_energy_delta_db: f64,
    external_endpoint_energy_delta_db: f64,
    signal_added_silence_frames: usize,
    external_added_silence_frames: usize,
    signal_peak_growth_db: f64,
    external_peak_growth_db: f64,
    signal_render_seconds: f64,
    signal_cpu_realtime_factor: f64,
    signal_heap_baseline_bytes: usize,
    signal_heap_peak_bytes: usize,
    signal_peak_working_memory_bytes: usize,
}

impl ExternalBenchmarkQualityMeasurement {
    fn format_report_line(&self) -> String {
        format!(
            "external_benchmark_quality case={} source={} signal_path={:?} ratio={:.6} tool={} render={} status={} reason={} source_boundary={} sample_rate_match={} source_sample_rate={} external_sample_rate={} external_channels={} source_frames={} signal_frames={} external_frames={} signal_timing_drift_samples={:.6} external_timing_drift_samples={:.6} timing_drift_delta_samples={:.6} signal_transient_smear_frames={:.6} external_transient_smear_frames={:.6} transient_smear_delta_frames={:.6} signal_transient_matches={} external_transient_matches={} signal_transient_mean_signed_offset_frames={:.6} external_transient_mean_signed_offset_frames={:.6} signal_transient_mean_absolute_offset_frames={:.6} external_transient_mean_absolute_offset_frames={:.6} signal_transient_max_absolute_offset_frames={:.6} external_transient_max_absolute_offset_frames={:.6} signal_transient_max_crest_growth_db={:.6} external_transient_max_crest_growth_db={:.6} signal_transient_max_crest_input_frame={} external_transient_max_crest_input_frame={} signal_transient_max_crest_output_frame={} external_transient_max_crest_output_frame={} draft_transient_mean_absolute_offset_frames={:.6} draft_transient_max_crest_growth_db={:.6} draft_transient_max_crest_input_frame={} draft_transient_max_crest_output_frame={} alignment_lag_frames={} aligned_compared_frames={} aligned_correlation={:.6} aligned_rms_error={:.9} aligned_peak_error={:.9} signal_rms={:.9} external_rms={:.9} aligned_rms_error_ratio={:.6} integrity_limit_id={} signal_integrity_passed={} external_integrity_passed={} signal_measured_endpoint_count={} external_measured_endpoint_count={} signal_endpoint_energy_delta_db={:.6} external_endpoint_energy_delta_db={:.6} signal_added_silence_frames={} external_added_silence_frames={} signal_peak_growth_db={:.6} external_peak_growth_db={:.6} signal_render_seconds={:.9} signal_cpu_realtime_factor={:.6} signal_cpu_realtime_factor_basis=rendered-audio-duration signal_heap_baseline_bytes={} signal_heap_peak_bytes={} signal_peak_working_memory_bytes={} signal_peak_working_memory_scope=peak-live-heap-growth-above-pre-render-baseline",
            self.case_id,
            quoted_report_field(&self.source_path),
            self.signal_path,
            self.ratio,
            quoted_report_field(&self.tool_name),
            quoted_report_field(&self.render_path),
            self.status,
            self.reason,
            quoted_report_field(self.source_boundary),
            self.sample_rate_match,
            self.source_sample_rate_hz,
            self.external_sample_rate_hz,
            self.external_channels,
            self.source_frames,
            self.signal_frames,
            self.external_frames,
            self.signal_timing_drift_samples,
            self.external_timing_drift_samples,
            self.timing_drift_delta_samples,
            self.signal_transient_smear_frames,
            self.external_transient_smear_frames,
            self.transient_smear_delta_frames,
            self.signal_transient_matches,
            self.external_transient_matches,
            self.signal_transient_mean_signed_offset_frames,
            self.external_transient_mean_signed_offset_frames,
            self.signal_transient_mean_absolute_offset_frames,
            self.external_transient_mean_absolute_offset_frames,
            self.signal_transient_max_absolute_offset_frames,
            self.external_transient_max_absolute_offset_frames,
            self.signal_transient_max_crest_growth_db,
            self.external_transient_max_crest_growth_db,
            self.signal_transient_max_crest_input_frame,
            self.external_transient_max_crest_input_frame,
            self.signal_transient_max_crest_output_frame,
            self.external_transient_max_crest_output_frame,
            self.draft_transient_mean_absolute_offset_frames,
            self.draft_transient_max_crest_growth_db,
            self.draft_transient_max_crest_input_frame,
            self.draft_transient_max_crest_output_frame,
            self.alignment_lag_frames,
            self.aligned_compared_frames,
            self.aligned_correlation,
            self.aligned_rms_error,
            self.aligned_peak_error,
            self.signal_rms,
            self.external_rms,
            self.aligned_rms_error_ratio,
            self.integrity_limit_id,
            self.signal_integrity_passed,
            self.external_integrity_passed,
            self.signal_measured_endpoint_count,
            self.external_measured_endpoint_count,
            self.signal_endpoint_energy_delta_db,
            self.external_endpoint_energy_delta_db,
            self.signal_added_silence_frames,
            self.external_added_silence_frames,
            self.signal_peak_growth_db,
            self.external_peak_growth_db,
            self.signal_render_seconds,
            self.signal_cpu_realtime_factor,
            self.signal_heap_baseline_bytes,
            self.signal_heap_peak_bytes,
            self.signal_peak_working_memory_bytes,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkTransientControlMeasurement {
    case_id: String,
    source_path: String,
    ratio: f64,
    anchor_input_frame: usize,
    signal_event_output_frame: usize,
    external_event_output_frame: usize,
    draft_event_output_frame: usize,
    stability_event_output_frame: usize,
    tracked_peak_event_output_frame: usize,
    magnitude_slew_event_output_frame: usize,
    signal_event_crest_growth_db: f64,
    external_event_crest_growth_db: f64,
    draft_event_crest_growth_db: f64,
    stability_event_crest_growth_db: f64,
    tracked_peak_event_crest_growth_db: f64,
    magnitude_slew_event_crest_growth_db: f64,
    signal_max_crest_growth_db: f64,
    external_max_crest_growth_db: f64,
    draft_max_crest_growth_db: f64,
    stability_max_crest_growth_db: f64,
    tracked_peak_max_crest_growth_db: f64,
    magnitude_slew_max_crest_growth_db: f64,
    stability_mean_absolute_offset_frames: f64,
    tracked_peak_mean_absolute_offset_frames: f64,
    magnitude_slew_mean_absolute_offset_frames: f64,
}

impl ExternalBenchmarkTransientControlMeasurement {
    fn format_report_line(&self) -> String {
        format!(
            "external_benchmark_transient_control case={} source={} ratio={:.6} anchor_input_frame={} signal_event_output_frame={} external_event_output_frame={} draft_event_output_frame={} stability_event_output_frame={} tracked_peak_event_output_frame={} magnitude_slew_event_output_frame={} signal_event_crest_growth_db={:.6} external_event_crest_growth_db={:.6} draft_event_crest_growth_db={:.6} stability_event_crest_growth_db={:.6} tracked_peak_event_crest_growth_db={:.6} magnitude_slew_event_crest_growth_db={:.6} signal_max_crest_growth_db={:.6} external_max_crest_growth_db={:.6} draft_max_crest_growth_db={:.6} stability_max_crest_growth_db={:.6} tracked_peak_max_crest_growth_db={:.6} magnitude_slew_max_crest_growth_db={:.6} stability_mean_absolute_offset_frames={:.6} tracked_peak_mean_absolute_offset_frames={:.6} magnitude_slew_mean_absolute_offset_frames={:.6}",
            self.case_id,
            quoted_report_field(&self.source_path),
            self.ratio,
            self.anchor_input_frame,
            self.signal_event_output_frame,
            self.external_event_output_frame,
            self.draft_event_output_frame,
            self.stability_event_output_frame,
            self.tracked_peak_event_output_frame,
            self.magnitude_slew_event_output_frame,
            self.signal_event_crest_growth_db,
            self.external_event_crest_growth_db,
            self.draft_event_crest_growth_db,
            self.stability_event_crest_growth_db,
            self.tracked_peak_event_crest_growth_db,
            self.magnitude_slew_event_crest_growth_db,
            self.signal_max_crest_growth_db,
            self.external_max_crest_growth_db,
            self.draft_max_crest_growth_db,
            self.stability_max_crest_growth_db,
            self.tracked_peak_max_crest_growth_db,
            self.magnitude_slew_max_crest_growth_db,
            self.stability_mean_absolute_offset_frames,
            self.tracked_peak_mean_absolute_offset_frames,
            self.magnitude_slew_mean_absolute_offset_frames,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkTonalTextureMeasurement {
    case_id: String,
    source_path: String,
    ratio: f64,
    spectral_windows: usize,
    signal_mean_spectral_residual_ratio: f64,
    external_mean_spectral_residual_ratio: f64,
    draft_mean_spectral_residual_ratio: f64,
    signal_max_spectral_residual_ratio: f64,
    external_max_spectral_residual_ratio: f64,
    signal_mean_added_sideband_ratio: f64,
    external_mean_added_sideband_ratio: f64,
    draft_mean_added_sideband_ratio: f64,
    signal_max_added_sideband_ratio: f64,
    external_max_added_sideband_ratio: f64,
    signal_spectral_modulation_delta: f64,
    external_spectral_modulation_delta: f64,
    draft_spectral_modulation_delta: f64,
    signal_envelope_modulation_delta_db: f64,
    external_envelope_modulation_delta_db: f64,
    draft_envelope_modulation_delta_db: f64,
}

impl ExternalBenchmarkTonalTextureMeasurement {
    fn format_report_line(&self) -> String {
        format!(
            "external_benchmark_tonal_texture case={} source={} ratio={:.6} spectral_windows={} signal_mean_spectral_residual_ratio={:.6} external_mean_spectral_residual_ratio={:.6} draft_mean_spectral_residual_ratio={:.6} signal_residual_delta_vs_external={:.6} signal_max_spectral_residual_ratio={:.6} external_max_spectral_residual_ratio={:.6} signal_mean_added_sideband_ratio={:.6} external_mean_added_sideband_ratio={:.6} draft_mean_added_sideband_ratio={:.6} signal_sideband_delta_vs_external={:.6} signal_max_added_sideband_ratio={:.6} external_max_added_sideband_ratio={:.6} signal_spectral_modulation_delta={:.6} external_spectral_modulation_delta={:.6} draft_spectral_modulation_delta={:.6} signal_spectral_modulation_delta_vs_external={:.6} signal_envelope_modulation_delta_db={:.6} external_envelope_modulation_delta_db={:.6} draft_envelope_modulation_delta_db={:.6} signal_envelope_modulation_delta_vs_external_db={:.6}",
            self.case_id,
            quoted_report_field(&self.source_path),
            self.ratio,
            self.spectral_windows,
            self.signal_mean_spectral_residual_ratio,
            self.external_mean_spectral_residual_ratio,
            self.draft_mean_spectral_residual_ratio,
            self.signal_mean_spectral_residual_ratio
                - self.external_mean_spectral_residual_ratio,
            self.signal_max_spectral_residual_ratio,
            self.external_max_spectral_residual_ratio,
            self.signal_mean_added_sideband_ratio,
            self.external_mean_added_sideband_ratio,
            self.draft_mean_added_sideband_ratio,
            self.signal_mean_added_sideband_ratio - self.external_mean_added_sideband_ratio,
            self.signal_max_added_sideband_ratio,
            self.external_max_added_sideband_ratio,
            self.signal_spectral_modulation_delta,
            self.external_spectral_modulation_delta,
            self.draft_spectral_modulation_delta,
            self.signal_spectral_modulation_delta - self.external_spectral_modulation_delta,
            self.signal_envelope_modulation_delta_db,
            self.external_envelope_modulation_delta_db,
            self.draft_envelope_modulation_delta_db,
            self.signal_envelope_modulation_delta_db
                - self.external_envelope_modulation_delta_db,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkFeatureDeltaMeasurement {
    case_id: String,
    source_path: String,
    signal_path: OfflineHighQualityPath,
    render_path: String,
    tool_name: String,
    ratio: f64,
    status: &'static str,
    reason: &'static str,
    source_boundary: &'static str,
    aligned_compared_frames: usize,
    envelope_correlation: f64,
    signal_rms: f64,
    external_rms: f64,
    rms_delta_db: f64,
    signal_peak: f64,
    external_peak: f64,
    peak_delta_db: f64,
    signal_zero_crossings_per_second: f64,
    external_zero_crossings_per_second: f64,
    zero_crossings_delta_per_second: f64,
    signal_spectral_centroid_hz: f64,
    external_spectral_centroid_hz: f64,
    spectral_centroid_delta_hz: f64,
    signal_high_frequency_energy_ratio: f64,
    external_high_frequency_energy_ratio: f64,
    high_frequency_energy_ratio_delta: f64,
    feature_divergence_score: f64,
}

impl ExternalBenchmarkFeatureDeltaMeasurement {
    fn format_report_line(&self) -> String {
        format!(
            "external_benchmark_feature_delta case={} source={} signal_path={:?} ratio={:.6} tool={} render={} status={} reason={} source_boundary={} aligned_compared_frames={} envelope_correlation={:.6} signal_rms={:.9} external_rms={:.9} rms_delta_db={:.6} signal_peak={:.9} external_peak={:.9} peak_delta_db={:.6} signal_zero_crossings_per_second={:.6} external_zero_crossings_per_second={:.6} zero_crossings_delta_per_second={:.6} signal_spectral_centroid_hz={:.6} external_spectral_centroid_hz={:.6} spectral_centroid_delta_hz={:.6} signal_high_frequency_energy_ratio={:.6} external_high_frequency_energy_ratio={:.6} high_frequency_energy_ratio_delta={:.6} feature_divergence_score={:.6}",
            self.case_id,
            quoted_report_field(&self.source_path),
            self.signal_path,
            self.ratio,
            quoted_report_field(&self.tool_name),
            quoted_report_field(&self.render_path),
            self.status,
            self.reason,
            quoted_report_field(self.source_boundary),
            self.aligned_compared_frames,
            self.envelope_correlation,
            self.signal_rms,
            self.external_rms,
            self.rms_delta_db,
            self.signal_peak,
            self.external_peak,
            self.peak_delta_db,
            self.signal_zero_crossings_per_second,
            self.external_zero_crossings_per_second,
            self.zero_crossings_delta_per_second,
            self.signal_spectral_centroid_hz,
            self.external_spectral_centroid_hz,
            self.spectral_centroid_delta_hz,
            self.signal_high_frequency_energy_ratio,
            self.external_high_frequency_energy_ratio,
            self.high_frequency_energy_ratio_delta,
            self.feature_divergence_score,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkGainEnvelopeReviewMeasurement {
    case_id: String,
    source_path: String,
    signal_path: OfflineHighQualityPath,
    render_path: String,
    tool_name: String,
    ratio: f64,
    source_boundary: &'static str,
    aligned_compared_frames: usize,
    feature_divergence_score: f64,
    envelope_correlation: f64,
    rms_delta_db: f64,
    peak_delta_db: f64,
    window_count: usize,
    mean_window_rms_delta_db: f64,
    median_window_rms_delta_db: f64,
    max_abs_window_rms_delta_db: f64,
    louder_windows: usize,
    quieter_windows: usize,
    near_windows: usize,
    gain_pattern: &'static str,
}

impl ExternalBenchmarkGainEnvelopeReviewMeasurement {
    fn format_report_line(&self, rank: usize) -> String {
        format!(
            "external_benchmark_gain_envelope_review rank={} case={} source={} signal_path={:?} ratio={:.6} tool={} render={} status=Measured reason=TopFeatureDivergence source_boundary={} aligned_compared_frames={} feature_divergence_score={:.6} envelope_correlation={:.6} rms_delta_db={:.6} peak_delta_db={:.6} window_size_frames={} hop_size_frames={} window_count={} mean_window_rms_delta_db={:.6} median_window_rms_delta_db={:.6} max_abs_window_rms_delta_db={:.6} louder_windows={} quieter_windows={} near_windows={} gain_pattern={}",
            rank,
            self.case_id,
            quoted_report_field(&self.source_path),
            self.signal_path,
            self.ratio,
            quoted_report_field(&self.tool_name),
            quoted_report_field(&self.render_path),
            quoted_report_field(self.source_boundary),
            self.aligned_compared_frames,
            self.feature_divergence_score,
            self.envelope_correlation,
            self.rms_delta_db,
            self.peak_delta_db,
            EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE,
            EXTERNAL_BENCHMARK_GAIN_ENVELOPE_HOP_SIZE,
            self.window_count,
            self.mean_window_rms_delta_db,
            self.median_window_rms_delta_db,
            self.max_abs_window_rms_delta_db,
            self.louder_windows,
            self.quieter_windows,
            self.near_windows,
            self.gain_pattern,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkLevelNormalizedReviewMeasurement {
    case_id: String,
    source_path: String,
    signal_path: OfflineHighQualityPath,
    render_path: String,
    tool_name: String,
    ratio: f64,
    source_boundary: &'static str,
    aligned_compared_frames: usize,
    signal_gain_db_applied: f64,
    raw_feature_divergence_score: f64,
    normalized_feature_divergence_score: f64,
    feature_divergence_score_delta: f64,
    raw_envelope_correlation: f64,
    normalized_envelope_correlation: f64,
    raw_rms_delta_db: f64,
    normalized_rms_delta_db: f64,
    raw_peak_delta_db: f64,
    normalized_peak_delta_db: f64,
    raw_spectral_centroid_delta_hz: f64,
    normalized_spectral_centroid_delta_hz: f64,
    raw_high_frequency_energy_ratio_delta: f64,
    normalized_high_frequency_energy_ratio_delta: f64,
    normalization_pattern: &'static str,
}

impl ExternalBenchmarkLevelNormalizedReviewMeasurement {
    fn format_report_line(&self, rank: usize) -> String {
        format!(
            "external_benchmark_level_normalized_review rank={} case={} source={} signal_path={:?} ratio={:.6} tool={} render={} status=Measured reason=TopFeatureDivergence source_boundary={} aligned_compared_frames={} signal_gain_db_applied={:.6} raw_feature_divergence_score={:.6} normalized_feature_divergence_score={:.6} feature_divergence_score_delta={:.6} raw_envelope_correlation={:.6} normalized_envelope_correlation={:.6} raw_rms_delta_db={:.6} normalized_rms_delta_db={:.6} raw_peak_delta_db={:.6} normalized_peak_delta_db={:.6} raw_spectral_centroid_delta_hz={:.6} normalized_spectral_centroid_delta_hz={:.6} raw_high_frequency_energy_ratio_delta={:.6} normalized_high_frequency_energy_ratio_delta={:.6} normalization_pattern={}",
            rank,
            self.case_id,
            quoted_report_field(&self.source_path),
            self.signal_path,
            self.ratio,
            quoted_report_field(&self.tool_name),
            quoted_report_field(&self.render_path),
            quoted_report_field(self.source_boundary),
            self.aligned_compared_frames,
            self.signal_gain_db_applied,
            self.raw_feature_divergence_score,
            self.normalized_feature_divergence_score,
            self.feature_divergence_score_delta,
            self.raw_envelope_correlation,
            self.normalized_envelope_correlation,
            self.raw_rms_delta_db,
            self.normalized_rms_delta_db,
            self.raw_peak_delta_db,
            self.normalized_peak_delta_db,
            self.raw_spectral_centroid_delta_hz,
            self.normalized_spectral_centroid_delta_hz,
            self.raw_high_frequency_energy_ratio_delta,
            self.normalized_high_frequency_energy_ratio_delta,
            self.normalization_pattern,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkResidualCoherenceReviewMeasurement {
    case_id: String,
    source_path: String,
    signal_path: OfflineHighQualityPath,
    render_path: String,
    tool_name: String,
    ratio: f64,
    source_boundary: &'static str,
    aligned_compared_frames: usize,
    signal_gain_db_applied: f64,
    raw_feature_divergence_score: f64,
    normalized_feature_divergence_score: f64,
    normalized_sample_envelope_correlation: f64,
    block_rms_envelope_correlation: f64,
    mean_abs_block_rms_delta_db: f64,
    max_abs_block_rms_delta_db: f64,
    spectral_magnitude_coherence: f64,
    normalized_spectral_centroid_delta_hz: f64,
    normalized_high_frequency_energy_ratio_delta: f64,
    residual_pattern: &'static str,
}

impl ExternalBenchmarkResidualCoherenceReviewMeasurement {
    fn format_report_line(&self, rank: usize) -> String {
        format!(
            "external_benchmark_residual_coherence_review rank={} case={} source={} signal_path={:?} ratio={:.6} tool={} render={} status=Measured reason=TopFeatureDivergence source_boundary={} aligned_compared_frames={} signal_gain_db_applied={:.6} raw_feature_divergence_score={:.6} normalized_feature_divergence_score={:.6} normalized_sample_envelope_correlation={:.6} block_rms_envelope_correlation={:.6} mean_abs_block_rms_delta_db={:.6} max_abs_block_rms_delta_db={:.6} spectral_magnitude_coherence={:.6} normalized_spectral_centroid_delta_hz={:.6} normalized_high_frequency_energy_ratio_delta={:.6} residual_pattern={}",
            rank,
            self.case_id,
            quoted_report_field(&self.source_path),
            self.signal_path,
            self.ratio,
            quoted_report_field(&self.tool_name),
            quoted_report_field(&self.render_path),
            quoted_report_field(self.source_boundary),
            self.aligned_compared_frames,
            self.signal_gain_db_applied,
            self.raw_feature_divergence_score,
            self.normalized_feature_divergence_score,
            self.normalized_sample_envelope_correlation,
            self.block_rms_envelope_correlation,
            self.mean_abs_block_rms_delta_db,
            self.max_abs_block_rms_delta_db,
            self.spectral_magnitude_coherence,
            self.normalized_spectral_centroid_delta_hz,
            self.normalized_high_frequency_energy_ratio_delta,
            self.residual_pattern,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkCoherenceTargetReviewMeasurement {
    case_id: String,
    source_path: String,
    signal_path: OfflineHighQualityPath,
    render_path: String,
    tool_name: String,
    ratio: f64,
    source_boundary: &'static str,
    material_scope: &'static str,
    target_reason: &'static str,
    target_score: f64,
    aligned_compared_frames: usize,
    signal_gain_db_applied: f64,
    raw_feature_divergence_score: f64,
    normalized_feature_divergence_score: f64,
    normalized_sample_envelope_correlation: f64,
    block_rms_envelope_correlation: f64,
    mean_abs_block_rms_delta_db: f64,
    max_abs_block_rms_delta_db: f64,
    spectral_magnitude_coherence: f64,
    normalized_spectral_centroid_delta_hz: f64,
    normalized_high_frequency_energy_ratio_delta: f64,
    residual_pattern: &'static str,
}

impl ExternalBenchmarkCoherenceTargetReviewMeasurement {
    fn format_report_line(&self, rank: usize) -> String {
        format!(
            "external_benchmark_coherence_target_review rank={} case={} source={} signal_path={:?} ratio={:.6} tool={} render={} status=Measured reason=TopSustainedPolyphonicResidual source_boundary={} material_scope={} target_reason={} target_score={:.6} aligned_compared_frames={} signal_gain_db_applied={:.6} raw_feature_divergence_score={:.6} normalized_feature_divergence_score={:.6} normalized_sample_envelope_correlation={:.6} block_rms_envelope_correlation={:.6} mean_abs_block_rms_delta_db={:.6} max_abs_block_rms_delta_db={:.6} spectral_magnitude_coherence={:.6} normalized_spectral_centroid_delta_hz={:.6} normalized_high_frequency_energy_ratio_delta={:.6} residual_pattern={}",
            rank,
            self.case_id,
            quoted_report_field(&self.source_path),
            self.signal_path,
            self.ratio,
            quoted_report_field(&self.tool_name),
            quoted_report_field(&self.render_path),
            quoted_report_field(self.source_boundary),
            self.material_scope,
            self.target_reason,
            self.target_score,
            self.aligned_compared_frames,
            self.signal_gain_db_applied,
            self.raw_feature_divergence_score,
            self.normalized_feature_divergence_score,
            self.normalized_sample_envelope_correlation,
            self.block_rms_envelope_correlation,
            self.mean_abs_block_rms_delta_db,
            self.max_abs_block_rms_delta_db,
            self.spectral_magnitude_coherence,
            self.normalized_spectral_centroid_delta_hz,
            self.normalized_high_frequency_energy_ratio_delta,
            self.residual_pattern,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkCoherenceCandidateReviewMeasurement {
    case_id: String,
    source_path: String,
    signal_path: OfflineHighQualityPath,
    candidate_path: &'static str,
    render_path: String,
    tool_name: String,
    ratio: f64,
    source_boundary: &'static str,
    material_scope: &'static str,
    target_reason: &'static str,
    outcome: &'static str,
    gate_decision: &'static str,
    gate_reason: &'static str,
    product_probe_decision: &'static str,
    product_probe_reason: &'static str,
    product_probe_low_band_weight: f64,
    product_probe_sustain_body: f64,
    product_probe_rhythmic_activity: f64,
    product_probe_spectral_complexity: f64,
    product_probe_confidence: f64,
    current_target_score: f64,
    candidate_target_score: f64,
    target_score_delta: f64,
    candidate_aligned_compared_frames: usize,
    candidate_signal_gain_db_applied: f64,
    candidate_raw_feature_divergence_score: f64,
    candidate_normalized_feature_divergence_score: f64,
    candidate_normalized_sample_envelope_correlation: f64,
    candidate_block_rms_envelope_correlation: f64,
    candidate_mean_abs_block_rms_delta_db: f64,
    candidate_max_abs_block_rms_delta_db: f64,
    candidate_spectral_magnitude_coherence: f64,
    candidate_normalized_spectral_centroid_delta_hz: f64,
    candidate_normalized_high_frequency_energy_ratio_delta: f64,
    candidate_residual_pattern: &'static str,
}

impl ExternalBenchmarkCoherenceCandidateReviewMeasurement {
    fn format_report_line(&self, rank: usize) -> String {
        self.format_report_line_with_prefix("external_benchmark_coherence_candidate_review", rank)
    }

    fn format_blend_report_line(&self, rank: usize) -> String {
        self.format_report_line_with_prefix(
            "external_benchmark_coherence_blend_candidate_review",
            rank,
        )
    }

    fn format_envelope_report_line(&self, rank: usize) -> String {
        self.format_report_line_with_prefix(
            "external_benchmark_coherence_envelope_candidate_review",
            rank,
        )
    }

    fn format_expansion_reset_report_line(&self, rank: usize) -> String {
        self.format_report_line_with_prefix(
            "external_benchmark_coherence_expansion_reset_candidate_review",
            rank,
        )
    }

    fn format_stability_adaptive_report_line(&self, rank: usize) -> String {
        self.format_report_line_with_prefix(
            "external_benchmark_coherence_stability_adaptive_candidate_review",
            rank,
        )
    }

    fn format_tracked_peak_report_line(&self, rank: usize) -> String {
        self.format_report_line_with_prefix(
            "external_benchmark_coherence_tracked_peak_candidate_review",
            rank,
        )
    }

    fn format_magnitude_slew_report_line(&self, rank: usize) -> String {
        self.format_report_line_with_prefix(
            "external_benchmark_coherence_magnitude_slew_candidate_review",
            rank,
        )
    }

    fn format_report_line_with_prefix(&self, prefix: &str, rank: usize) -> String {
        format!(
            "{} rank={} case={} source={} signal_path={:?} candidate_path={} ratio={:.6} tool={} render={} status=Measured reason=ReportOnlySustainedCoherenceCandidate source_boundary={} material_scope={} target_reason={} outcome={} gate={} gate_decision={} gate_reason={} product_probe={} product_probe_decision={} product_probe_reason={} product_probe_low_band_weight={:.6} product_probe_sustain_body={:.6} product_probe_rhythmic_activity={:.6} product_probe_spectral_complexity={:.6} product_probe_confidence={:.6} current_target_score={:.6} candidate_target_score={:.6} target_score_delta={:.6} candidate_aligned_compared_frames={} candidate_signal_gain_db_applied={:.6} candidate_raw_feature_divergence_score={:.6} candidate_normalized_feature_divergence_score={:.6} candidate_normalized_sample_envelope_correlation={:.6} candidate_block_rms_envelope_correlation={:.6} candidate_mean_abs_block_rms_delta_db={:.6} candidate_max_abs_block_rms_delta_db={:.6} candidate_spectral_magnitude_coherence={:.6} candidate_normalized_spectral_centroid_delta_hz={:.6} candidate_normalized_high_frequency_energy_ratio_delta={:.6} candidate_residual_pattern={}",
            prefix,
            rank,
            self.case_id,
            quoted_report_field(&self.source_path),
            self.signal_path,
            self.candidate_path,
            self.ratio,
            quoted_report_field(&self.tool_name),
            quoted_report_field(&self.render_path),
            quoted_report_field(self.source_boundary),
            self.material_scope,
            self.target_reason,
            self.outcome,
            EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_GATE,
            self.gate_decision,
            self.gate_reason,
            EXTERNAL_BENCHMARK_COHERENCE_PRODUCT_PROBE,
            self.product_probe_decision,
            self.product_probe_reason,
            self.product_probe_low_band_weight,
            self.product_probe_sustain_body,
            self.product_probe_rhythmic_activity,
            self.product_probe_spectral_complexity,
            self.product_probe_confidence,
            self.current_target_score,
            self.candidate_target_score,
            self.target_score_delta,
            self.candidate_aligned_compared_frames,
            self.candidate_signal_gain_db_applied,
            self.candidate_raw_feature_divergence_score,
            self.candidate_normalized_feature_divergence_score,
            self.candidate_normalized_sample_envelope_correlation,
            self.candidate_block_rms_envelope_correlation,
            self.candidate_mean_abs_block_rms_delta_db,
            self.candidate_max_abs_block_rms_delta_db,
            self.candidate_spectral_magnitude_coherence,
            self.candidate_normalized_spectral_centroid_delta_hz,
            self.candidate_normalized_high_frequency_energy_ratio_delta,
            self.candidate_residual_pattern,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkDecodedAudio {
    sample_rate_hz: u32,
    channels: u16,
    samples: Vec<f32>,
    mono_samples: Vec<f32>,
}

impl ExternalBenchmarkDecodedAudio {
    fn frames(&self) -> usize {
        self.mono_samples.len()
    }
}

fn format_external_benchmark_quality_metrics(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    mode: ExternalBenchmarkQualityMode,
    signal_path: OfflineHighQualityPath,
) -> Result<String, String> {
    let integrity_limits = StretchRenderIntegrityLimits::offline_high_quality();
    let mut lines = Vec::new();
    let mut gain_envelope_reviews = Vec::new();
    let mut level_normalized_reviews = Vec::new();
    let mut residual_coherence_reviews = Vec::new();
    let mut coherence_target_reviews = Vec::new();
    let mut coherence_candidate_reviews = Vec::new();
    let mut coherence_blend_candidate_reviews = Vec::new();
    let mut coherence_envelope_candidate_reviews = Vec::new();
    let mut coherence_expansion_reset_candidate_reviews = Vec::new();
    let mut coherence_stability_adaptive_candidate_reviews = Vec::new();
    let mut coherence_tracked_peak_candidate_reviews = Vec::new();
    let mut coherence_magnitude_slew_candidate_reviews = Vec::new();
    let mut source_audio_cache = HashMap::new();
    let mut source_mono_cache = HashMap::new();
    let mut source_character_probe_cache = HashMap::new();
    for render in renders {
        let source = match source_for_external_quality_render(sources, render) {
            ExternalBenchmarkQualitySource::Found(source) => source,
            ExternalBenchmarkQualitySource::Missing => {
                lines.push(format_external_benchmark_quality_skip_line(
                    render,
                    "",
                    "MissingListeningSource",
                    signal_path,
                    0,
                    0,
                    0,
                    0,
                ));
                continue;
            }
            ExternalBenchmarkQualitySource::Ambiguous => {
                lines.push(format_external_benchmark_quality_skip_line(
                    render,
                    "",
                    "AmbiguousListeningSource",
                    signal_path,
                    0,
                    0,
                    0,
                    0,
                ));
                continue;
            }
        };
        let source_cache_key = source.source_path.clone();
        let source_audio = match source_audio_cache.get(&source_cache_key) {
            Some(audio) => audio,
            None => {
                let audio = decode_listening_source_audio(source.as_ref(), frame_limit)?;
                source_audio_cache.insert(source_cache_key.clone(), audio);
                source_audio_cache
                    .get(&source_cache_key)
                    .expect("cached decoded source audio")
            }
        };
        let external_audio = decode_external_benchmark_render_audio(render)?;
        if external_audio.frames() == 0 {
            lines.push(format_external_benchmark_quality_skip_line(
                render,
                &source.source_path,
                "NoComparatorAudio",
                signal_path,
                source_audio.sample_rate_hz,
                external_audio.sample_rate_hz,
                external_audio.channels,
                source_audio.analyzed_frames(),
            ));
            continue;
        }
        if source_audio.sample_rate_hz != external_audio.sample_rate_hz {
            lines.push(format_external_benchmark_quality_skip_line(
                render,
                &source.source_path,
                "SampleRateMismatch",
                signal_path,
                source_audio.sample_rate_hz,
                external_audio.sample_rate_hz,
                external_audio.channels,
                source_audio.analyzed_frames(),
            ));
            continue;
        }

        let source_mono = match source_mono_cache.get(&source_cache_key) {
            Some(mono) => mono,
            None => {
                source_mono_cache.insert(source_cache_key.clone(), source_audio.mono_samples());
                source_mono_cache
                    .get(&source_cache_key)
                    .expect("cached decoded mono source audio")
            }
        };
        let ((signal_output, signal_render_seconds), signal_heap) = measure_peak_live_heap(|| {
            let started = Instant::now();
            let mut signal = OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
            let output = signal.stretch_mono(source_mono);
            (output, started.elapsed().as_secs_f64())
        });
        let mut tail_anchor_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let tail_anchor_output = tail_anchor_stretcher.stretch_tail_anchor_review_mono(source_mono);
        let mut zero_tail_anchor_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let zero_tail_anchor_output =
            zero_tail_anchor_stretcher.stretch_zero_tail_anchor_review_mono(source_mono);
        let mut multiplicative_tail_fade_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let multiplicative_tail_fade_output = multiplicative_tail_fade_stretcher
            .stretch_multiplicative_tail_fade_review_mono(source_mono);
        let mut hybrid_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let hybrid_render = hybrid_stretcher.stretch_hybrid_review_mono(source_mono);
        let mut timeline_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let timeline_render = timeline_stretcher.stretch_adaptive_timeline_review_mono(source_mono);
        let mut peak_transient_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let peak_transient_render =
            peak_transient_stretcher.stretch_fixed_map_peak_transient_review_mono(source_mono);
        let tail_local_feature_line = format_tail_local_feature_line(
            &render.case_id,
            &source.source_path,
            render.ratio,
            source_audio.sample_rate_hz,
            &signal_output,
            &zero_tail_anchor_output,
            &multiplicative_tail_fade_output,
        );
        let mut draft_stretcher = PhaseVocoderStretcher::new(render.ratio);
        let draft_output = draft_stretcher.stretch_mono(source_mono);
        let signal_tonal_texture = measure_tonal_texture(source_mono, &signal_output, render.ratio);
        let tail_anchor_tonal_texture =
            measure_tonal_texture(source_mono, &tail_anchor_output, render.ratio);
        let zero_tail_anchor_tonal_texture =
            measure_tonal_texture(source_mono, &zero_tail_anchor_output, render.ratio);
        let multiplicative_tail_fade_tonal_texture =
            measure_tonal_texture(source_mono, &multiplicative_tail_fade_output, render.ratio);
        let hybrid_tonal_texture =
            measure_tonal_texture(source_mono, &hybrid_render.samples, render.ratio);
        let timeline_tonal_texture =
            measure_tonal_texture(source_mono, &timeline_render.samples, render.ratio);
        let peak_transient_tonal_texture =
            measure_tonal_texture(source_mono, &peak_transient_render.samples, render.ratio);
        let external_tonal_texture =
            measure_tonal_texture(source_mono, &external_audio.mono_samples, render.ratio);
        let draft_tonal_texture = measure_tonal_texture(source_mono, &draft_output, render.ratio);
        let tonal_texture_line = ExternalBenchmarkTonalTextureMeasurement {
            case_id: render.case_id.clone(),
            source_path: source.source_path.clone(),
            ratio: render.ratio,
            spectral_windows: signal_tonal_texture
                .spectral_windows
                .min(external_tonal_texture.spectral_windows),
            signal_mean_spectral_residual_ratio: signal_tonal_texture.mean_spectral_residual_ratio,
            external_mean_spectral_residual_ratio: external_tonal_texture
                .mean_spectral_residual_ratio,
            draft_mean_spectral_residual_ratio: draft_tonal_texture.mean_spectral_residual_ratio,
            signal_max_spectral_residual_ratio: signal_tonal_texture.max_spectral_residual_ratio,
            external_max_spectral_residual_ratio: external_tonal_texture
                .max_spectral_residual_ratio,
            signal_mean_added_sideband_ratio: signal_tonal_texture.mean_added_sideband_ratio,
            external_mean_added_sideband_ratio: external_tonal_texture.mean_added_sideband_ratio,
            draft_mean_added_sideband_ratio: draft_tonal_texture.mean_added_sideband_ratio,
            signal_max_added_sideband_ratio: signal_tonal_texture.max_added_sideband_ratio,
            external_max_added_sideband_ratio: external_tonal_texture.max_added_sideband_ratio,
            signal_spectral_modulation_delta: signal_tonal_texture.spectral_modulation_delta,
            external_spectral_modulation_delta: external_tonal_texture.spectral_modulation_delta,
            draft_spectral_modulation_delta: draft_tonal_texture.spectral_modulation_delta,
            signal_envelope_modulation_delta_db: signal_tonal_texture.envelope_modulation_delta_db,
            external_envelope_modulation_delta_db: external_tonal_texture
                .envelope_modulation_delta_db,
            draft_envelope_modulation_delta_db: draft_tonal_texture.envelope_modulation_delta_db,
        }
        .format_report_line();
        let signal_formant_boundary = measure_formant_boundary(
            source_mono,
            &signal_output,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let tail_anchor_formant_boundary = measure_formant_boundary(
            source_mono,
            &tail_anchor_output,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let zero_tail_anchor_formant_boundary = measure_formant_boundary(
            source_mono,
            &zero_tail_anchor_output,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let multiplicative_tail_fade_formant_boundary = measure_formant_boundary(
            source_mono,
            &multiplicative_tail_fade_output,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let hybrid_formant_boundary = measure_formant_boundary(
            source_mono,
            &hybrid_render.samples,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let timeline_formant_boundary = measure_formant_boundary(
            source_mono,
            &timeline_render.samples,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let peak_transient_formant_boundary = measure_formant_boundary(
            source_mono,
            &peak_transient_render.samples,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let external_formant_boundary = measure_formant_boundary(
            source_mono,
            &external_audio.mono_samples,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let draft_formant_boundary = measure_formant_boundary(
            source_mono,
            &draft_output,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let formant_boundary_line = format_external_benchmark_formant_boundary_line(
            &render.case_id,
            &source.source_path,
            signal_formant_boundary,
            external_formant_boundary,
            draft_formant_boundary,
        );
        let mut stability_stretcher = OfflineHighQualityStretcher::new(render.ratio);
        let stability_output =
            stability_stretcher.stretch_phase_lock_stability_review_mono(source_mono);
        let mut tracked_peak_stretcher = OfflineHighQualityStretcher::new(render.ratio);
        let tracked_peak_output =
            tracked_peak_stretcher.stretch_phase_lock_tracked_peak_review_mono(source_mono);
        let mut magnitude_slew_stretcher = OfflineHighQualityStretcher::new(render.ratio);
        let magnitude_slew_output =
            magnitude_slew_stretcher.stretch_phase_lock_magnitude_slew_review_mono(source_mono);
        let rendered_audio_seconds =
            signal_output.len() as f64 / source_audio.sample_rate_hz as f64;
        let signal_cpu_realtime_factor = if rendered_audio_seconds > 0.0 {
            signal_render_seconds / rendered_audio_seconds
        } else {
            f64::NAN
        };
        let signal_smear = measure_transient_smear(
            &source_mono,
            &signal_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let external_smear = measure_transient_smear(
            &source_mono,
            &external_audio.mono_samples,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let signal_transient_detail = measure_transient_detail(
            &source_mono,
            &signal_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let tail_anchor_transient_detail = measure_transient_detail(
            &source_mono,
            &tail_anchor_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let zero_tail_anchor_transient_detail = measure_transient_detail(
            &source_mono,
            &zero_tail_anchor_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let multiplicative_tail_fade_transient_detail = measure_transient_detail(
            &source_mono,
            &multiplicative_tail_fade_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let hybrid_transient_detail = measure_transient_detail(
            source_mono,
            &hybrid_render.samples,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let timeline_transient_detail = measure_transient_detail(
            source_mono,
            &timeline_render.samples,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let peak_transient_detail = measure_transient_detail(
            source_mono,
            &peak_transient_render.samples,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let external_transient_detail = measure_transient_detail(
            &source_mono,
            &external_audio.mono_samples,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let draft_transient_detail = measure_transient_detail(
            &source_mono,
            &draft_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let stability_transient_detail = measure_transient_detail(
            &source_mono,
            &stability_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let tracked_peak_transient_detail = measure_transient_detail(
            &source_mono,
            &tracked_peak_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let magnitude_slew_transient_detail = measure_transient_detail(
            &source_mono,
            &magnitude_slew_output,
            render.ratio,
            QUALITY_METRIC_WINDOW_SIZE,
            QUALITY_METRIC_HOP_SIZE,
        );
        let transient_control_line = (signal_transient_detail.matched_transients > 0).then(|| {
            let anchor = signal_transient_detail.max_crest_input_frame;
            let event = |output: &[f32]| {
                measure_transient_event_detail(
                    source_mono,
                    output,
                    render.ratio,
                    anchor,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                )
                .expect("matched transient anchor must remain measurable")
            };
            let signal_event = event(&signal_output);
            let external_event = event(&external_audio.mono_samples);
            let draft_event = event(&draft_output);
            let stability_event = event(&stability_output);
            let tracked_peak_event = event(&tracked_peak_output);
            let magnitude_slew_event = event(&magnitude_slew_output);
            ExternalBenchmarkTransientControlMeasurement {
                case_id: render.case_id.clone(),
                source_path: source.source_path.clone(),
                ratio: render.ratio,
                anchor_input_frame: anchor,
                signal_event_output_frame: signal_event.output_frame,
                external_event_output_frame: external_event.output_frame,
                draft_event_output_frame: draft_event.output_frame,
                stability_event_output_frame: stability_event.output_frame,
                tracked_peak_event_output_frame: tracked_peak_event.output_frame,
                magnitude_slew_event_output_frame: magnitude_slew_event.output_frame,
                signal_event_crest_growth_db: signal_event.crest_growth_db,
                external_event_crest_growth_db: external_event.crest_growth_db,
                draft_event_crest_growth_db: draft_event.crest_growth_db,
                stability_event_crest_growth_db: stability_event.crest_growth_db,
                tracked_peak_event_crest_growth_db: tracked_peak_event.crest_growth_db,
                magnitude_slew_event_crest_growth_db: magnitude_slew_event.crest_growth_db,
                signal_max_crest_growth_db: signal_transient_detail.max_transient_crest_growth_db,
                external_max_crest_growth_db: external_transient_detail
                    .max_transient_crest_growth_db,
                draft_max_crest_growth_db: draft_transient_detail.max_transient_crest_growth_db,
                stability_max_crest_growth_db: stability_transient_detail
                    .max_transient_crest_growth_db,
                tracked_peak_max_crest_growth_db: tracked_peak_transient_detail
                    .max_transient_crest_growth_db,
                magnitude_slew_max_crest_growth_db: magnitude_slew_transient_detail
                    .max_transient_crest_growth_db,
                stability_mean_absolute_offset_frames: stability_transient_detail
                    .mean_absolute_timing_offset_frames,
                tracked_peak_mean_absolute_offset_frames: tracked_peak_transient_detail
                    .mean_absolute_timing_offset_frames,
                magnitude_slew_mean_absolute_offset_frames: magnitude_slew_transient_detail
                    .mean_absolute_timing_offset_frames,
            }
            .format_report_line()
        });
        let signal_timing_drift =
            output_length_drift_samples(source_mono.len(), signal_output.len(), render.ratio);
        let external_timing_drift = output_length_drift_samples(
            source_mono.len(),
            external_audio.mono_samples.len(),
            render.ratio,
        );
        let aligned = align_and_measure_error(&signal_output, &external_audio.mono_samples);
        let signal_integrity = measure_stretch_render_integrity(
            source_mono,
            &signal_output,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let external_integrity = measure_stretch_render_integrity(
            source_mono,
            &external_audio.mono_samples,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let tail_anchor_integrity = measure_stretch_render_integrity(
            source_mono,
            &tail_anchor_output,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let zero_tail_anchor_integrity = measure_stretch_render_integrity(
            source_mono,
            &zero_tail_anchor_output,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let multiplicative_tail_fade_integrity = measure_stretch_render_integrity(
            source_mono,
            &multiplicative_tail_fade_output,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let hybrid_integrity = measure_stretch_render_integrity(
            source_mono,
            &hybrid_render.samples,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let timeline_integrity = measure_stretch_render_integrity(
            source_mono,
            &timeline_render.samples,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let peak_transient_integrity = measure_stretch_render_integrity(
            source_mono,
            &peak_transient_render.samples,
            render.ratio,
            RENDER_INTEGRITY_ENDPOINT_SOURCE_FRAMES,
            RENDER_INTEGRITY_SILENCE_THRESHOLD,
        );
        let signal_integrity_assessment =
            signal_dsp_stretch::assess_stretch_render_integrity(signal_integrity, integrity_limits);
        let external_integrity_assessment = signal_dsp_stretch::assess_stretch_render_integrity(
            external_integrity,
            integrity_limits,
        );
        let tail_anchor_integrity_assessment = signal_dsp_stretch::assess_stretch_render_integrity(
            tail_anchor_integrity,
            integrity_limits,
        );
        let zero_tail_anchor_integrity_assessment =
            signal_dsp_stretch::assess_stretch_render_integrity(
                zero_tail_anchor_integrity,
                integrity_limits,
            );
        let multiplicative_tail_fade_integrity_assessment =
            signal_dsp_stretch::assess_stretch_render_integrity(
                multiplicative_tail_fade_integrity,
                integrity_limits,
            );
        let hybrid_integrity_assessment =
            signal_dsp_stretch::assess_stretch_render_integrity(hybrid_integrity, integrity_limits);
        let timeline_integrity_assessment = signal_dsp_stretch::assess_stretch_render_integrity(
            timeline_integrity,
            integrity_limits,
        );
        let peak_transient_integrity_assessment =
            signal_dsp_stretch::assess_stretch_render_integrity(
                peak_transient_integrity,
                integrity_limits,
            );
        let tail_anchor_line = TailAnchorReviewEvidence {
            control_id: "source",
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            current_output: &signal_output,
            candidate_output: &tail_anchor_output,
            current_boundary: signal_formant_boundary,
            candidate_boundary: tail_anchor_formant_boundary,
            current_tonal: signal_tonal_texture,
            candidate_tonal: tail_anchor_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: tail_anchor_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: tail_anchor_transient_detail,
            candidate_integrity: tail_anchor_integrity,
            candidate_integrity_passed: tail_anchor_integrity_assessment.passed,
        }
        .format_report_line();
        let zero_tail_anchor_line = TailAnchorReviewEvidence {
            control_id: "zero",
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            current_output: &signal_output,
            candidate_output: &zero_tail_anchor_output,
            current_boundary: signal_formant_boundary,
            candidate_boundary: zero_tail_anchor_formant_boundary,
            current_tonal: signal_tonal_texture,
            candidate_tonal: zero_tail_anchor_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: zero_tail_anchor_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: zero_tail_anchor_transient_detail,
            candidate_integrity: zero_tail_anchor_integrity,
            candidate_integrity_passed: zero_tail_anchor_integrity_assessment.passed,
        }
        .format_report_line();
        let multiplicative_tail_fade_line = TailAnchorReviewEvidence {
            control_id: "multiplicative_zero",
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            current_output: &signal_output,
            candidate_output: &multiplicative_tail_fade_output,
            current_boundary: signal_formant_boundary,
            candidate_boundary: multiplicative_tail_fade_formant_boundary,
            current_tonal: signal_tonal_texture,
            candidate_tonal: multiplicative_tail_fade_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: multiplicative_tail_fade_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: multiplicative_tail_fade_transient_detail,
            candidate_integrity: multiplicative_tail_fade_integrity,
            candidate_integrity_passed: multiplicative_tail_fade_integrity_assessment.passed,
        }
        .format_report_line();
        let hybrid_anchor_events = (signal_transient_detail.matched_transients > 0).then(|| {
            let anchor = signal_transient_detail.max_crest_input_frame;
            let current = measure_transient_event_detail(
                source_mono,
                &signal_output,
                render.ratio,
                anchor,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            )
            .expect("current matched transient anchor must remain measurable");
            let candidate = measure_transient_event_detail(
                source_mono,
                &hybrid_render.samples,
                render.ratio,
                anchor,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            )
            .expect("hybrid matched transient anchor must remain measurable");
            (current, candidate)
        });
        let hybrid_line = HybridReviewEvidence {
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            render: &hybrid_render,
            current_tonal: signal_tonal_texture,
            candidate_tonal: hybrid_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: hybrid_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: hybrid_transient_detail,
            anchor_events: hybrid_anchor_events,
            candidate_integrity: hybrid_integrity,
            candidate_integrity_passed: hybrid_integrity_assessment.passed,
        }
        .format_report_line();
        let hybrid_combined_gate_line = TailAnchorReviewEvidence {
            control_id: "structural_hybrid",
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            current_output: &signal_output,
            candidate_output: &hybrid_render.samples,
            current_boundary: signal_formant_boundary,
            candidate_boundary: hybrid_formant_boundary,
            current_tonal: signal_tonal_texture,
            candidate_tonal: hybrid_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: hybrid_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: hybrid_transient_detail,
            candidate_integrity: hybrid_integrity,
            candidate_integrity_passed: hybrid_integrity_assessment.passed,
        }
        .format_report_line();
        let timeline_anchor_events = (signal_transient_detail.matched_transients > 0).then(|| {
            let anchor = signal_transient_detail.max_crest_input_frame;
            let current = measure_transient_event_detail(
                source_mono,
                &signal_output,
                render.ratio,
                anchor,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            )
            .expect("current matched transient anchor must remain measurable");
            let candidate = measure_transient_event_detail(
                source_mono,
                &timeline_render.samples,
                render.ratio,
                anchor,
                QUALITY_METRIC_WINDOW_SIZE,
                QUALITY_METRIC_HOP_SIZE,
            )
            .expect("timeline matched transient anchor must remain measurable");
            (current, candidate)
        });
        let timeline_line = TimelineReviewEvidence {
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            render: &timeline_render,
            current_tonal: signal_tonal_texture,
            candidate_tonal: timeline_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: timeline_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: timeline_transient_detail,
            anchor_events: timeline_anchor_events,
            candidate_integrity_passed: timeline_integrity_assessment.passed,
        }
        .format_report_line();
        let timeline_combined_gate_line = TailAnchorReviewEvidence {
            control_id: "adaptive_timeline",
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            current_output: &signal_output,
            candidate_output: &timeline_render.samples,
            current_boundary: signal_formant_boundary,
            candidate_boundary: timeline_formant_boundary,
            current_tonal: signal_tonal_texture,
            candidate_tonal: timeline_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: timeline_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: timeline_transient_detail,
            candidate_integrity: timeline_integrity,
            candidate_integrity_passed: timeline_integrity_assessment.passed,
        }
        .format_report_line();
        let peak_transient_anchor_events =
            (signal_transient_detail.matched_transients > 0).then(|| {
                let anchor = signal_transient_detail.max_crest_input_frame;
                let current = measure_transient_event_detail(
                    source_mono,
                    &signal_output,
                    render.ratio,
                    anchor,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                )
                .expect("current matched transient anchor must remain measurable");
                let candidate = measure_transient_event_detail(
                    source_mono,
                    &peak_transient_render.samples,
                    render.ratio,
                    anchor,
                    QUALITY_METRIC_WINDOW_SIZE,
                    QUALITY_METRIC_HOP_SIZE,
                )
                .expect("fixed-map peak matched transient anchor must remain measurable");
                (current, candidate)
            });
        let peak_transient_line = PeakTransientReviewEvidence {
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            render: &peak_transient_render,
            current_tonal: signal_tonal_texture,
            candidate_tonal: peak_transient_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: peak_transient_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: peak_transient_detail,
            anchor_events: peak_transient_anchor_events,
            candidate_integrity_passed: peak_transient_integrity_assessment.passed,
        }
        .format_report_line();
        let peak_transient_combined_gate_line = TailAnchorReviewEvidence {
            control_id: "fixed_map_peak_transient",
            case_id: &render.case_id,
            source_path: &source.source_path,
            ratio: render.ratio,
            current_output: &signal_output,
            candidate_output: &peak_transient_render.samples,
            current_boundary: signal_formant_boundary,
            candidate_boundary: peak_transient_formant_boundary,
            current_tonal: signal_tonal_texture,
            candidate_tonal: peak_transient_tonal_texture,
            current_formant: signal_formant_boundary,
            candidate_formant: peak_transient_formant_boundary,
            current_transient: signal_transient_detail,
            candidate_transient: peak_transient_detail,
            candidate_integrity: peak_transient_integrity,
            candidate_integrity_passed: peak_transient_integrity_assessment.passed,
        }
        .format_report_line();
        let mut feature_delta_line = None;
        if mode == ExternalBenchmarkQualityMode::Full {
            let feature_delta = measure_external_benchmark_feature_delta(
                &signal_output,
                &external_audio.mono_samples,
                &aligned,
                source_audio.sample_rate_hz,
            );
            let gain_envelope_review = measure_external_benchmark_gain_envelope_review(
                &signal_output,
                &external_audio.mono_samples,
                &aligned,
                &feature_delta,
            );
            let level_normalized_review = measure_external_benchmark_level_normalized_review(
                &signal_output,
                &external_audio.mono_samples,
                &aligned,
                &feature_delta,
                source_audio.sample_rate_hz,
            );
            let residual_coherence_review = measure_external_benchmark_residual_coherence_review(
                &signal_output,
                &external_audio.mono_samples,
                &aligned,
                &feature_delta,
                &level_normalized_review,
                source_audio.sample_rate_hz,
            );
            gain_envelope_reviews.push(ExternalBenchmarkGainEnvelopeReviewMeasurement {
                case_id: render.case_id.clone(),
                source_path: source.source_path.clone(),
                signal_path,
                render_path: render.rendered_path.clone(),
                tool_name: render.tool_name.clone(),
                ratio: render.ratio,
                source_boundary: "rendered-output-only; no external source or library dependency",
                aligned_compared_frames: feature_delta.compared_frames,
                feature_divergence_score: feature_delta.divergence_score(),
                envelope_correlation: feature_delta.envelope_correlation,
                rms_delta_db: feature_delta.signal.rms_db - feature_delta.external.rms_db,
                peak_delta_db: feature_delta.signal.peak_db - feature_delta.external.peak_db,
                window_count: gain_envelope_review.window_count,
                mean_window_rms_delta_db: gain_envelope_review.mean_window_rms_delta_db,
                median_window_rms_delta_db: gain_envelope_review.median_window_rms_delta_db,
                max_abs_window_rms_delta_db: gain_envelope_review.max_abs_window_rms_delta_db,
                louder_windows: gain_envelope_review.louder_windows,
                quieter_windows: gain_envelope_review.quieter_windows,
                near_windows: gain_envelope_review.near_windows,
                gain_pattern: gain_envelope_review.gain_pattern,
            });
            level_normalized_reviews.push(ExternalBenchmarkLevelNormalizedReviewMeasurement {
                case_id: render.case_id.clone(),
                source_path: source.source_path.clone(),
                signal_path,
                render_path: render.rendered_path.clone(),
                tool_name: render.tool_name.clone(),
                ratio: render.ratio,
                source_boundary: "rendered-output-only; no external source or library dependency",
                aligned_compared_frames: feature_delta.compared_frames,
                signal_gain_db_applied: level_normalized_review.signal_gain_db_applied,
                raw_feature_divergence_score: feature_delta.divergence_score(),
                normalized_feature_divergence_score: level_normalized_review
                    .normalized_feature_delta
                    .divergence_score(),
                feature_divergence_score_delta: level_normalized_review
                    .normalized_feature_delta
                    .divergence_score()
                    - feature_delta.divergence_score(),
                raw_envelope_correlation: feature_delta.envelope_correlation,
                normalized_envelope_correlation: level_normalized_review
                    .normalized_feature_delta
                    .envelope_correlation,
                raw_rms_delta_db: feature_delta.signal.rms_db - feature_delta.external.rms_db,
                normalized_rms_delta_db: level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .rms_db
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .rms_db,
                raw_peak_delta_db: feature_delta.signal.peak_db - feature_delta.external.peak_db,
                normalized_peak_delta_db: level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .peak_db
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .peak_db,
                raw_spectral_centroid_delta_hz: feature_delta.signal.spectral_centroid_hz
                    - feature_delta.external.spectral_centroid_hz,
                normalized_spectral_centroid_delta_hz: level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .spectral_centroid_hz
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .spectral_centroid_hz,
                raw_high_frequency_energy_ratio_delta: feature_delta
                    .signal
                    .high_frequency_energy_ratio
                    - feature_delta.external.high_frequency_energy_ratio,
                normalized_high_frequency_energy_ratio_delta: level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .high_frequency_energy_ratio
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .high_frequency_energy_ratio,
                normalization_pattern: level_normalized_review.normalization_pattern,
            });
            residual_coherence_reviews.push(ExternalBenchmarkResidualCoherenceReviewMeasurement {
                case_id: render.case_id.clone(),
                source_path: source.source_path.clone(),
                signal_path,
                render_path: render.rendered_path.clone(),
                tool_name: render.tool_name.clone(),
                ratio: render.ratio,
                source_boundary: "rendered-output-only; no external source or library dependency",
                aligned_compared_frames: feature_delta.compared_frames,
                signal_gain_db_applied: level_normalized_review.signal_gain_db_applied,
                raw_feature_divergence_score: feature_delta.divergence_score(),
                normalized_feature_divergence_score: level_normalized_review
                    .normalized_feature_delta
                    .divergence_score(),
                normalized_sample_envelope_correlation: level_normalized_review
                    .normalized_feature_delta
                    .envelope_correlation,
                block_rms_envelope_correlation: residual_coherence_review
                    .block_rms_envelope_correlation,
                mean_abs_block_rms_delta_db: residual_coherence_review.mean_abs_block_rms_delta_db,
                max_abs_block_rms_delta_db: residual_coherence_review.max_abs_block_rms_delta_db,
                spectral_magnitude_coherence: residual_coherence_review
                    .spectral_magnitude_coherence,
                normalized_spectral_centroid_delta_hz: level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .spectral_centroid_hz
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .spectral_centroid_hz,
                normalized_high_frequency_energy_ratio_delta: level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .high_frequency_energy_ratio
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .high_frequency_energy_ratio,
                residual_pattern: residual_coherence_review.residual_pattern,
            });
            if let Some(material_scope) =
                external_benchmark_coherence_target_material_scope(&render.case_id)
            {
                let normalized_spectral_centroid_delta_hz = level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .spectral_centroid_hz
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .spectral_centroid_hz;
                let normalized_high_frequency_energy_ratio_delta = level_normalized_review
                    .normalized_feature_delta
                    .signal
                    .high_frequency_energy_ratio
                    - level_normalized_review
                        .normalized_feature_delta
                        .external
                        .high_frequency_energy_ratio;
                let target_score = external_benchmark_coherence_target_score(
                    level_normalized_review
                        .normalized_feature_delta
                        .divergence_score(),
                    level_normalized_review
                        .normalized_feature_delta
                        .envelope_correlation,
                    residual_coherence_review.block_rms_envelope_correlation,
                    residual_coherence_review.mean_abs_block_rms_delta_db,
                    residual_coherence_review.spectral_magnitude_coherence,
                );
                let target_reason = classify_external_benchmark_coherence_target_reason(
                    target_score,
                    level_normalized_review
                        .normalized_feature_delta
                        .envelope_correlation,
                    residual_coherence_review.block_rms_envelope_correlation,
                    residual_coherence_review.mean_abs_block_rms_delta_db,
                    residual_coherence_review.spectral_magnitude_coherence,
                );
                coherence_target_reviews.push(ExternalBenchmarkCoherenceTargetReviewMeasurement {
                    case_id: render.case_id.clone(),
                    source_path: source.source_path.clone(),
                    signal_path,
                    render_path: render.rendered_path.clone(),
                    tool_name: render.tool_name.clone(),
                    ratio: render.ratio,
                    source_boundary:
                        "rendered-output-only; no external source or library dependency",
                    material_scope,
                    target_reason,
                    target_score,
                    aligned_compared_frames: feature_delta.compared_frames,
                    signal_gain_db_applied: level_normalized_review.signal_gain_db_applied,
                    raw_feature_divergence_score: feature_delta.divergence_score(),
                    normalized_feature_divergence_score: level_normalized_review
                        .normalized_feature_delta
                        .divergence_score(),
                    normalized_sample_envelope_correlation: level_normalized_review
                        .normalized_feature_delta
                        .envelope_correlation,
                    block_rms_envelope_correlation: residual_coherence_review
                        .block_rms_envelope_correlation,
                    mean_abs_block_rms_delta_db: residual_coherence_review
                        .mean_abs_block_rms_delta_db,
                    max_abs_block_rms_delta_db: residual_coherence_review
                        .max_abs_block_rms_delta_db,
                    spectral_magnitude_coherence: residual_coherence_review
                        .spectral_magnitude_coherence,
                    normalized_spectral_centroid_delta_hz,
                    normalized_high_frequency_energy_ratio_delta,
                    residual_pattern: residual_coherence_review.residual_pattern,
                });

                let mut candidate = OfflineHighQualityStretcher::new(render.ratio);
                let candidate_output =
                    candidate.stretch_sustained_coherence_review_mono(source_mono);
                let (candidate_gate_decision, candidate_gate_reason) =
                    external_benchmark_coherence_candidate_gate_decision(
                        target_reason,
                        material_scope,
                        render.ratio,
                    );
                let product_probe = source_character_probe_cache
                    .entry(source.source_path.clone())
                    .or_insert_with(|| {
                        measure_coherence_product_observable_probe(
                            source_audio.sample_rate_hz,
                            source_mono,
                        )
                    });
                let (product_probe_decision, product_probe_reason) =
                    coherence_product_observable_probe_decision(product_probe, render.ratio);
                coherence_candidate_reviews.push(
                    measure_external_benchmark_coherence_candidate_review(
                        render,
                        &source.source_path,
                        signal_path,
                        "sustained-coherence-long-window-identity-locked",
                        &candidate_output,
                        &external_audio,
                        source_audio.sample_rate_hz,
                        material_scope,
                        target_reason,
                        target_score,
                        product_probe,
                        candidate_gate_decision,
                        candidate_gate_reason,
                        product_probe_decision,
                        product_probe_reason,
                    ),
                );

                let blended_candidate_output =
                    blend_external_benchmark_candidate_output(&signal_output, &candidate_output);
                coherence_blend_candidate_reviews.push(
                    measure_external_benchmark_coherence_candidate_review(
                        render,
                        &source.source_path,
                        signal_path,
                        EXTERNAL_BENCHMARK_COHERENCE_BLEND_CANDIDATE_PATH,
                        &blended_candidate_output,
                        &external_audio,
                        source_audio.sample_rate_hz,
                        material_scope,
                        target_reason,
                        target_score,
                        product_probe,
                        "Selected",
                        "UniformBlendCandidate",
                        "Selected",
                        "UniformBlendCandidate",
                    ),
                );

                let mut envelope_candidate = OfflineHighQualityStretcher::new(render.ratio);
                let envelope_candidate_output = envelope_candidate
                    .stretch_sustained_coherence_envelope_review_mono(source_mono);
                coherence_envelope_candidate_reviews.push(
                    measure_external_benchmark_coherence_candidate_review(
                        render,
                        &source.source_path,
                        signal_path,
                        EXTERNAL_BENCHMARK_COHERENCE_ENVELOPE_CANDIDATE_PATH,
                        &envelope_candidate_output,
                        &external_audio,
                        source_audio.sample_rate_hz,
                        material_scope,
                        target_reason,
                        target_score,
                        product_probe,
                        "Selected",
                        "UniformEnvelopeCandidate",
                        "Selected",
                        "UniformEnvelopeCandidate",
                    ),
                );

                let mut expansion_reset_candidate = OfflineHighQualityStretcher::new(render.ratio);
                let expansion_reset_candidate_output = expansion_reset_candidate
                    .stretch_sustained_coherence_expansion_reset_review_mono(source_mono);
                coherence_expansion_reset_candidate_reviews.push(
                    measure_external_benchmark_coherence_candidate_review(
                        render,
                        &source.source_path,
                        signal_path,
                        EXTERNAL_BENCHMARK_COHERENCE_EXPANSION_RESET_CANDIDATE_PATH,
                        &expansion_reset_candidate_output,
                        &external_audio,
                        source_audio.sample_rate_hz,
                        material_scope,
                        target_reason,
                        target_score,
                        product_probe,
                        "Selected",
                        "ExpansionTransientResetCandidate",
                        "Selected",
                        "ExpansionTransientResetCandidate",
                    ),
                );

                let mut stability_adaptive_candidate =
                    OfflineHighQualityStretcher::new(render.ratio);
                let stability_adaptive_candidate_output = stability_adaptive_candidate
                    .stretch_sustained_coherence_stability_adaptive_review_mono(source_mono);
                coherence_stability_adaptive_candidate_reviews.push(
                    measure_external_benchmark_coherence_candidate_review(
                        render,
                        &source.source_path,
                        signal_path,
                        EXTERNAL_BENCHMARK_COHERENCE_STABILITY_ADAPTIVE_CANDIDATE_PATH,
                        &stability_adaptive_candidate_output,
                        &external_audio,
                        source_audio.sample_rate_hz,
                        material_scope,
                        target_reason,
                        target_score,
                        product_probe,
                        "Selected",
                        "StabilityAdaptiveCandidate",
                        "Selected",
                        "StabilityAdaptiveCandidate",
                    ),
                );

                let mut tracked_peak_candidate = OfflineHighQualityStretcher::new(render.ratio);
                let tracked_peak_candidate_output = tracked_peak_candidate
                    .stretch_sustained_coherence_tracked_peak_review_mono(source_mono);
                coherence_tracked_peak_candidate_reviews.push(
                    measure_external_benchmark_coherence_candidate_review(
                        render,
                        &source.source_path,
                        signal_path,
                        EXTERNAL_BENCHMARK_COHERENCE_TRACKED_PEAK_CANDIDATE_PATH,
                        &tracked_peak_candidate_output,
                        &external_audio,
                        source_audio.sample_rate_hz,
                        material_scope,
                        target_reason,
                        target_score,
                        product_probe,
                        "Selected",
                        "TrackedPeakRegionCandidate",
                        "Selected",
                        "TrackedPeakRegionCandidate",
                    ),
                );

                let mut magnitude_slew_candidate = OfflineHighQualityStretcher::new(render.ratio);
                let magnitude_slew_candidate_output = magnitude_slew_candidate
                    .stretch_sustained_coherence_magnitude_slew_review_mono(source_mono);
                coherence_magnitude_slew_candidate_reviews.push(
                    measure_external_benchmark_coherence_candidate_review(
                        render,
                        &source.source_path,
                        signal_path,
                        EXTERNAL_BENCHMARK_COHERENCE_MAGNITUDE_SLEW_CANDIDATE_PATH,
                        &magnitude_slew_candidate_output,
                        &external_audio,
                        source_audio.sample_rate_hz,
                        material_scope,
                        target_reason,
                        target_score,
                        product_probe,
                        "Selected",
                        "MagnitudeSlewCandidate",
                        "Selected",
                        "MagnitudeSlewCandidate",
                    ),
                );
            }
            feature_delta_line = Some(
                ExternalBenchmarkFeatureDeltaMeasurement {
                    case_id: render.case_id.clone(),
                    source_path: source.source_path.clone(),
                    signal_path,
                    render_path: render.rendered_path.clone(),
                    tool_name: render.tool_name.clone(),
                    ratio: render.ratio,
                    status: "Measured",
                    reason: "Ok",
                    source_boundary:
                        "rendered-output-only; no external source or library dependency",
                    aligned_compared_frames: feature_delta.compared_frames,
                    envelope_correlation: feature_delta.envelope_correlation,
                    signal_rms: feature_delta.signal.rms,
                    external_rms: feature_delta.external.rms,
                    rms_delta_db: feature_delta.signal.rms_db - feature_delta.external.rms_db,
                    signal_peak: feature_delta.signal.peak,
                    external_peak: feature_delta.external.peak,
                    peak_delta_db: feature_delta.signal.peak_db - feature_delta.external.peak_db,
                    signal_zero_crossings_per_second: feature_delta
                        .signal
                        .zero_crossings_per_second,
                    external_zero_crossings_per_second: feature_delta
                        .external
                        .zero_crossings_per_second,
                    zero_crossings_delta_per_second: feature_delta.signal.zero_crossings_per_second
                        - feature_delta.external.zero_crossings_per_second,
                    signal_spectral_centroid_hz: feature_delta.signal.spectral_centroid_hz,
                    external_spectral_centroid_hz: feature_delta.external.spectral_centroid_hz,
                    spectral_centroid_delta_hz: feature_delta.signal.spectral_centroid_hz
                        - feature_delta.external.spectral_centroid_hz,
                    signal_high_frequency_energy_ratio: feature_delta
                        .signal
                        .high_frequency_energy_ratio,
                    external_high_frequency_energy_ratio: feature_delta
                        .external
                        .high_frequency_energy_ratio,
                    high_frequency_energy_ratio_delta: feature_delta
                        .signal
                        .high_frequency_energy_ratio
                        - feature_delta.external.high_frequency_energy_ratio,
                    feature_divergence_score: feature_delta.divergence_score(),
                }
                .format_report_line(),
            );
        }

        lines.push(
            ExternalBenchmarkQualityMeasurement {
                case_id: render.case_id.clone(),
                source_path: source.source_path.clone(),
                signal_path,
                render_path: render.rendered_path.clone(),
                tool_name: render.tool_name.clone(),
                ratio: render.ratio,
                status: "Measured",
                reason: "Ok",
                source_boundary: "rendered-output-only; no external source or library dependency",
                sample_rate_match: true,
                source_sample_rate_hz: source_audio.sample_rate_hz,
                external_sample_rate_hz: external_audio.sample_rate_hz,
                external_channels: external_audio.channels,
                source_frames: source_mono.len(),
                signal_frames: signal_output.len(),
                external_frames: external_audio.frames(),
                signal_timing_drift_samples: signal_timing_drift,
                external_timing_drift_samples: external_timing_drift,
                timing_drift_delta_samples: signal_timing_drift - external_timing_drift,
                signal_transient_smear_frames: signal_smear.max_smear_frames,
                external_transient_smear_frames: external_smear.max_smear_frames,
                transient_smear_delta_frames: signal_smear.max_smear_frames
                    - external_smear.max_smear_frames,
                signal_transient_matches: signal_transient_detail.matched_transients,
                external_transient_matches: external_transient_detail.matched_transients,
                signal_transient_mean_signed_offset_frames: signal_transient_detail
                    .mean_signed_timing_offset_frames,
                external_transient_mean_signed_offset_frames: external_transient_detail
                    .mean_signed_timing_offset_frames,
                signal_transient_mean_absolute_offset_frames: signal_transient_detail
                    .mean_absolute_timing_offset_frames,
                external_transient_mean_absolute_offset_frames: external_transient_detail
                    .mean_absolute_timing_offset_frames,
                signal_transient_max_absolute_offset_frames: signal_transient_detail
                    .max_absolute_timing_offset_frames,
                external_transient_max_absolute_offset_frames: external_transient_detail
                    .max_absolute_timing_offset_frames,
                signal_transient_max_crest_growth_db: signal_transient_detail
                    .max_transient_crest_growth_db,
                external_transient_max_crest_growth_db: external_transient_detail
                    .max_transient_crest_growth_db,
                signal_transient_max_crest_input_frame: signal_transient_detail
                    .max_crest_input_frame,
                external_transient_max_crest_input_frame: external_transient_detail
                    .max_crest_input_frame,
                signal_transient_max_crest_output_frame: signal_transient_detail
                    .max_crest_output_frame,
                external_transient_max_crest_output_frame: external_transient_detail
                    .max_crest_output_frame,
                draft_transient_mean_absolute_offset_frames: draft_transient_detail
                    .mean_absolute_timing_offset_frames,
                draft_transient_max_crest_growth_db: draft_transient_detail
                    .max_transient_crest_growth_db,
                draft_transient_max_crest_input_frame: draft_transient_detail.max_crest_input_frame,
                draft_transient_max_crest_output_frame: draft_transient_detail
                    .max_crest_output_frame,
                alignment_lag_frames: aligned.lag_frames,
                aligned_compared_frames: aligned.compared_frames,
                aligned_correlation: aligned.correlation,
                aligned_rms_error: aligned.rms_error,
                aligned_peak_error: aligned.peak_error,
                signal_rms: aligned.signal_rms,
                external_rms: aligned.external_rms,
                aligned_rms_error_ratio: finite_ratio(aligned.rms_error, aligned.external_rms),
                integrity_limit_id: OFFLINE_HIGH_QUALITY_INTEGRITY_LIMIT_ID,
                signal_integrity_passed: signal_integrity_assessment.passed,
                external_integrity_passed: external_integrity_assessment.passed,
                signal_measured_endpoint_count: signal_integrity.measured_endpoint_count,
                external_measured_endpoint_count: external_integrity.measured_endpoint_count,
                signal_endpoint_energy_delta_db: signal_integrity.endpoint_energy_delta_db,
                external_endpoint_energy_delta_db: external_integrity.endpoint_energy_delta_db,
                signal_added_silence_frames: signal_integrity.added_silence_frames,
                external_added_silence_frames: external_integrity.added_silence_frames,
                signal_peak_growth_db: signal_integrity.peak_growth_db,
                external_peak_growth_db: external_integrity.peak_growth_db,
                signal_render_seconds,
                signal_cpu_realtime_factor,
                signal_heap_baseline_bytes: signal_heap.baseline_live_bytes,
                signal_heap_peak_bytes: signal_heap.peak_live_bytes,
                signal_peak_working_memory_bytes: signal_heap.peak_growth_bytes,
            }
            .format_report_line(),
        );
        if let Some(line) = transient_control_line {
            lines.push(line);
        }
        lines.push(tonal_texture_line);
        lines.push(formant_boundary_line);
        lines.push(tail_anchor_line);
        lines.push(zero_tail_anchor_line);
        lines.push(multiplicative_tail_fade_line);
        lines.push(hybrid_line);
        lines.push(hybrid_combined_gate_line);
        lines.push(timeline_line);
        lines.push(timeline_combined_gate_line);
        lines.push(peak_transient_line);
        lines.push(peak_transient_combined_gate_line);
        lines.push(tail_local_feature_line);
        if let Some(line) = feature_delta_line {
            lines.push(line);
        }
    }
    lines.push(format!(
        "stretch_render_integrity_limits id={} max_output_length_drift_frames={:.6} max_endpoint_energy_delta_db={:.6} max_added_silence_frames={} max_peak_growth_db={:.6} endpoint_policy=active-source-endpoints-only evidence={}",
        OFFLINE_HIGH_QUALITY_INTEGRITY_LIMIT_ID,
        integrity_limits.max_output_length_drift_frames,
        integrity_limits.max_endpoint_energy_delta_db,
        integrity_limits.max_added_silence_frames,
        integrity_limits.max_peak_growth_db,
        quoted_report_field("g10.029 18-row Signal/Rubber Band v2 pack"),
    ));
    if mode == ExternalBenchmarkQualityMode::Core {
        return Ok(lines.join("\n"));
    }

    gain_envelope_reviews.sort_by(|left, right| {
        right
            .feature_divergence_score
            .total_cmp(&left.feature_divergence_score)
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    for (index, review) in gain_envelope_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_GAIN_ENVELOPE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_report_line(index + 1));
    }
    level_normalized_reviews.sort_by(|left, right| {
        right
            .raw_feature_divergence_score
            .total_cmp(&left.raw_feature_divergence_score)
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    for (index, review) in level_normalized_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_LEVEL_NORMALIZED_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_report_line(index + 1));
    }
    residual_coherence_reviews.sort_by(|left, right| {
        right
            .raw_feature_divergence_score
            .total_cmp(&left.raw_feature_divergence_score)
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    for (index, review) in residual_coherence_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_RESIDUAL_COHERENCE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_report_line(index + 1));
    }
    coherence_target_reviews.sort_by(|left, right| {
        right
            .target_score
            .total_cmp(&left.target_score)
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    for (index, review) in coherence_target_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_TARGET_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_report_line(index + 1));
    }
    coherence_candidate_reviews.sort_by(|left, right| {
        left.target_score_delta
            .total_cmp(&right.target_score_delta)
            .then_with(|| {
                right
                    .current_target_score
                    .total_cmp(&left.current_target_score)
            })
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    if !coherence_candidate_reviews.is_empty() {
        lines.push(format_external_benchmark_coherence_candidate_summary(
            &coherence_candidate_reviews,
        ));
        lines.push(format_external_benchmark_coherence_candidate_gate_summary(
            &coherence_candidate_reviews,
        ));
        lines.push(format_external_benchmark_coherence_product_probe_summary(
            &coherence_candidate_reviews,
        ));
    }
    for (index, review) in coherence_candidate_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_report_line(index + 1));
    }
    coherence_blend_candidate_reviews.sort_by(|left, right| {
        left.target_score_delta
            .total_cmp(&right.target_score_delta)
            .then_with(|| {
                right
                    .current_target_score
                    .total_cmp(&left.current_target_score)
            })
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    if !coherence_blend_candidate_reviews.is_empty() {
        lines.push(format_external_benchmark_coherence_blend_candidate_summary(
            &coherence_blend_candidate_reviews,
        ));
    }
    for (index, review) in coherence_blend_candidate_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_blend_report_line(index + 1));
    }
    coherence_envelope_candidate_reviews.sort_by(|left, right| {
        left.target_score_delta
            .total_cmp(&right.target_score_delta)
            .then_with(|| {
                right
                    .current_target_score
                    .total_cmp(&left.current_target_score)
            })
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    if !coherence_envelope_candidate_reviews.is_empty() {
        lines.push(
            format_external_benchmark_coherence_envelope_candidate_summary(
                &coherence_envelope_candidate_reviews,
            ),
        );
    }
    for (index, review) in coherence_envelope_candidate_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_envelope_report_line(index + 1));
    }
    coherence_expansion_reset_candidate_reviews.sort_by(|left, right| {
        left.target_score_delta
            .total_cmp(&right.target_score_delta)
            .then_with(|| {
                right
                    .current_target_score
                    .total_cmp(&left.current_target_score)
            })
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    if !coherence_expansion_reset_candidate_reviews.is_empty() {
        lines.push(
            format_external_benchmark_coherence_expansion_reset_candidate_summary(
                &coherence_expansion_reset_candidate_reviews,
            ),
        );
    }
    for (index, review) in coherence_expansion_reset_candidate_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_expansion_reset_report_line(index + 1));
    }
    coherence_stability_adaptive_candidate_reviews.sort_by(|left, right| {
        left.target_score_delta
            .total_cmp(&right.target_score_delta)
            .then_with(|| {
                right
                    .current_target_score
                    .total_cmp(&left.current_target_score)
            })
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    if !coherence_stability_adaptive_candidate_reviews.is_empty() {
        lines.push(
            format_external_benchmark_coherence_stability_adaptive_candidate_summary(
                &coherence_stability_adaptive_candidate_reviews,
            ),
        );
    }
    for (index, review) in coherence_stability_adaptive_candidate_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_stability_adaptive_report_line(index + 1));
    }
    coherence_tracked_peak_candidate_reviews.sort_by(|left, right| {
        left.target_score_delta
            .total_cmp(&right.target_score_delta)
            .then_with(|| {
                right
                    .current_target_score
                    .total_cmp(&left.current_target_score)
            })
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    if !coherence_tracked_peak_candidate_reviews.is_empty() {
        lines.push(
            format_external_benchmark_coherence_tracked_peak_candidate_summary(
                &coherence_tracked_peak_candidate_reviews,
            ),
        );
    }
    for (index, review) in coherence_tracked_peak_candidate_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_tracked_peak_report_line(index + 1));
    }
    coherence_magnitude_slew_candidate_reviews.sort_by(|left, right| {
        left.target_score_delta
            .total_cmp(&right.target_score_delta)
            .then_with(|| {
                right
                    .current_target_score
                    .total_cmp(&left.current_target_score)
            })
            .then_with(|| left.case_id.cmp(&right.case_id))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.render_path.cmp(&right.render_path))
    });
    if !coherence_magnitude_slew_candidate_reviews.is_empty() {
        lines.push(
            format_external_benchmark_coherence_magnitude_slew_candidate_summary(
                &coherence_magnitude_slew_candidate_reviews,
            ),
        );
    }
    for (index, review) in coherence_magnitude_slew_candidate_reviews
        .iter()
        .take(EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_REVIEW_ROWS)
        .enumerate()
    {
        lines.push(review.format_magnitude_slew_report_line(index + 1));
    }
    Ok(lines.join("\n"))
}

fn external_benchmark_coherence_target_material_scope(case_id: &str) -> Option<&'static str> {
    if case_id.contains("full_mix") {
        Some("DensePolyphonic")
    } else if case_id.contains("pads") || case_id.contains("sustain") {
        Some("SustainedPolyphonic")
    } else if case_id.contains("vocals") {
        Some("VocalTonal")
    } else if case_id.contains("bass") {
        Some("BassSustain")
    } else {
        None
    }
}

fn external_benchmark_coherence_target_score(
    normalized_feature_divergence_score: f64,
    normalized_sample_envelope_correlation: f64,
    block_rms_envelope_correlation: f64,
    mean_abs_block_rms_delta_db: f64,
    spectral_magnitude_coherence: f64,
) -> f64 {
    if !normalized_feature_divergence_score.is_finite()
        || !normalized_sample_envelope_correlation.is_finite()
        || !block_rms_envelope_correlation.is_finite()
        || !mean_abs_block_rms_delta_db.is_finite()
        || !spectral_magnitude_coherence.is_finite()
    {
        return f64::NAN;
    }

    let sample_envelope_residual = (1.0 - normalized_sample_envelope_correlation).max(0.0);
    let block_envelope_residual = (1.0 - block_rms_envelope_correlation).max(0.0);
    let spectral_residual = (1.0 - spectral_magnitude_coherence).max(0.0);
    let block_gain_residual = mean_abs_block_rms_delta_db / 6.0;
    normalized_feature_divergence_score
        + sample_envelope_residual
        + block_envelope_residual
        + spectral_residual
        + block_gain_residual
}

fn classify_external_benchmark_coherence_target_reason(
    target_score: f64,
    normalized_sample_envelope_correlation: f64,
    block_rms_envelope_correlation: f64,
    mean_abs_block_rms_delta_db: f64,
    spectral_magnitude_coherence: f64,
) -> &'static str {
    if !target_score.is_finite() {
        return "Inconclusive";
    }
    if target_score <= 1.0e-9 {
        return "NoResidual";
    }
    if spectral_magnitude_coherence.is_finite() && spectral_magnitude_coherence < 0.90 {
        return "SpectralMagnitudeCoherence";
    }
    if normalized_sample_envelope_correlation.is_finite()
        && normalized_sample_envelope_correlation < 0.70
    {
        return "SampleEnvelopeCoherence";
    }
    if block_rms_envelope_correlation.is_finite() && block_rms_envelope_correlation < 0.95 {
        return "BlockEnvelopeCoherence";
    }
    if mean_abs_block_rms_delta_db.is_finite() && mean_abs_block_rms_delta_db > 0.75 {
        return "BlockGainResidual";
    }
    "ResidualFeatureDivergence"
}

fn external_benchmark_candidate_outcome(
    current_target_score: f64,
    candidate_target_score: f64,
) -> &'static str {
    if !current_target_score.is_finite() || !candidate_target_score.is_finite() {
        "Inconclusive"
    } else if candidate_target_score < current_target_score - 1.0e-9 {
        "Improved"
    } else if (candidate_target_score - current_target_score).abs() <= 1.0e-9 {
        "Unchanged"
    } else {
        "Regressed"
    }
}

fn external_benchmark_coherence_candidate_gate_decision(
    target_reason: &str,
    material_scope: &str,
    ratio: f64,
) -> (&'static str, &'static str) {
    if target_reason != "SpectralMagnitudeCoherence" {
        return ("Rejected", "NonSpectralTargetReason");
    }
    if ratio >= 1.5 && matches!(material_scope, "BassSustain" | "SustainedPolyphonic") {
        return ("Rejected", "ExtremeExpansionMaterialGuard");
    }
    ("Selected", "TargetSpectralMagnitudeCoherence")
}

#[derive(Clone, Debug, PartialEq)]
struct CoherenceProductObservableProbe {
    low_band_weight: f64,
    sustain_body: f64,
    rhythmic_activity: f64,
    spectral_complexity: f64,
    confidence: f64,
}

fn measure_coherence_product_observable_probe(
    sample_rate_hz: u32,
    source_mono: &[f32],
) -> CoherenceProductObservableProbe {
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::low());
    let analysis =
        analyzer.analyze_mono(signal_primitives::SampleRate(sample_rate_hz), source_mono);
    let spectral_profile = &analysis.spectral_profile.normalized_mel_band_profile;
    let low_band_weight = spectral_profile.first().copied().unwrap_or_default()
        + spectral_profile.get(1).copied().unwrap_or_default() * 0.5;
    let sustain_body = analysis.temporal.sustain_ratio * 0.55
        + analysis.temporal_shape.sustain_plateau_ratio * 0.45;
    let rhythmic_activity = (analysis.temporal.onset_density / 4.0).clamp(0.0, 1.0) * 0.65
        + analysis.temporal_shape.peak_transient_strength * 0.35;
    let spectral_complexity = (analysis.spectral_shape.spread_hz / 4_000.0).clamp(0.0, 1.0);

    CoherenceProductObservableProbe {
        low_band_weight: low_band_weight.clamp(0.0, 1.0) as f64,
        sustain_body: sustain_body.clamp(0.0, 1.0) as f64,
        rhythmic_activity: rhythmic_activity.clamp(0.0, 1.0) as f64,
        spectral_complexity: spectral_complexity as f64,
        confidence: analysis.confidence.0 as f64,
    }
}

fn coherence_product_observable_probe_decision(
    probe: &CoherenceProductObservableProbe,
    ratio: f64,
) -> (&'static str, &'static str) {
    if probe.confidence < 0.75 {
        return ("Rejected", "LowSourceDescriptorConfidence");
    }
    if probe.rhythmic_activity >= 0.55 {
        return ("Rejected", "PulseDrivenSource");
    }
    if ratio >= 1.5 && (probe.low_band_weight >= 0.45 || probe.sustain_body >= 0.55) {
        return ("Rejected", "ExtremeExpansionSourceGuard");
    }
    if probe.spectral_complexity >= 0.35 && probe.sustain_body >= 0.35 {
        return ("Selected", "ComplexSustainedSource");
    }
    ("Rejected", "InsufficientSourceCoherenceSignal")
}

#[allow(clippy::too_many_arguments)]
fn measure_external_benchmark_coherence_candidate_review(
    render: &ExternalBenchmarkQualityRender,
    source_path: &str,
    signal_path: OfflineHighQualityPath,
    candidate_path: &'static str,
    candidate_output: &[f32],
    external_audio: &ExternalBenchmarkDecodedAudio,
    sample_rate_hz: u32,
    material_scope: &'static str,
    target_reason: &'static str,
    current_target_score: f64,
    product_probe: &CoherenceProductObservableProbe,
    gate_decision: &'static str,
    gate_reason: &'static str,
    product_probe_decision: &'static str,
    product_probe_reason: &'static str,
) -> ExternalBenchmarkCoherenceCandidateReviewMeasurement {
    let candidate_aligned = align_and_measure_error(candidate_output, &external_audio.mono_samples);
    let candidate_feature_delta = measure_external_benchmark_feature_delta(
        candidate_output,
        &external_audio.mono_samples,
        &candidate_aligned,
        sample_rate_hz,
    );
    let candidate_level_normalized_review = measure_external_benchmark_level_normalized_review(
        candidate_output,
        &external_audio.mono_samples,
        &candidate_aligned,
        &candidate_feature_delta,
        sample_rate_hz,
    );
    let candidate_residual_coherence_review = measure_external_benchmark_residual_coherence_review(
        candidate_output,
        &external_audio.mono_samples,
        &candidate_aligned,
        &candidate_feature_delta,
        &candidate_level_normalized_review,
        sample_rate_hz,
    );
    let candidate_target_score = external_benchmark_coherence_target_score(
        candidate_level_normalized_review
            .normalized_feature_delta
            .divergence_score(),
        candidate_level_normalized_review
            .normalized_feature_delta
            .envelope_correlation,
        candidate_residual_coherence_review.block_rms_envelope_correlation,
        candidate_residual_coherence_review.mean_abs_block_rms_delta_db,
        candidate_residual_coherence_review.spectral_magnitude_coherence,
    );
    let candidate_normalized_spectral_centroid_delta_hz = candidate_level_normalized_review
        .normalized_feature_delta
        .signal
        .spectral_centroid_hz
        - candidate_level_normalized_review
            .normalized_feature_delta
            .external
            .spectral_centroid_hz;
    let candidate_normalized_high_frequency_energy_ratio_delta = candidate_level_normalized_review
        .normalized_feature_delta
        .signal
        .high_frequency_energy_ratio
        - candidate_level_normalized_review
            .normalized_feature_delta
            .external
            .high_frequency_energy_ratio;

    ExternalBenchmarkCoherenceCandidateReviewMeasurement {
        case_id: render.case_id.clone(),
        source_path: source_path.to_string(),
        signal_path,
        candidate_path,
        render_path: render.rendered_path.clone(),
        tool_name: render.tool_name.clone(),
        ratio: render.ratio,
        source_boundary: "rendered-output-only; no external source or library dependency",
        material_scope,
        target_reason,
        outcome: external_benchmark_candidate_outcome(current_target_score, candidate_target_score),
        gate_decision,
        gate_reason,
        product_probe_decision,
        product_probe_reason,
        product_probe_low_band_weight: product_probe.low_band_weight,
        product_probe_sustain_body: product_probe.sustain_body,
        product_probe_rhythmic_activity: product_probe.rhythmic_activity,
        product_probe_spectral_complexity: product_probe.spectral_complexity,
        product_probe_confidence: product_probe.confidence,
        current_target_score,
        candidate_target_score,
        target_score_delta: candidate_target_score - current_target_score,
        candidate_aligned_compared_frames: candidate_feature_delta.compared_frames,
        candidate_signal_gain_db_applied: candidate_level_normalized_review.signal_gain_db_applied,
        candidate_raw_feature_divergence_score: candidate_feature_delta.divergence_score(),
        candidate_normalized_feature_divergence_score: candidate_level_normalized_review
            .normalized_feature_delta
            .divergence_score(),
        candidate_normalized_sample_envelope_correlation: candidate_level_normalized_review
            .normalized_feature_delta
            .envelope_correlation,
        candidate_block_rms_envelope_correlation: candidate_residual_coherence_review
            .block_rms_envelope_correlation,
        candidate_mean_abs_block_rms_delta_db: candidate_residual_coherence_review
            .mean_abs_block_rms_delta_db,
        candidate_max_abs_block_rms_delta_db: candidate_residual_coherence_review
            .max_abs_block_rms_delta_db,
        candidate_spectral_magnitude_coherence: candidate_residual_coherence_review
            .spectral_magnitude_coherence,
        candidate_normalized_spectral_centroid_delta_hz,
        candidate_normalized_high_frequency_energy_ratio_delta,
        candidate_residual_pattern: candidate_residual_coherence_review.residual_pattern,
    }
}

fn blend_external_benchmark_candidate_output(
    current_output: &[f32],
    candidate_output: &[f32],
) -> Vec<f32> {
    let candidate_weight = SUSTAINED_COHERENCE_BLEND_REVIEW_WEIGHT.clamp(0.0, 1.0);
    let current_weight = 1.0 - candidate_weight;
    current_output
        .iter()
        .enumerate()
        .map(|(index, current_sample)| {
            let candidate_sample = candidate_output.get(index).copied().unwrap_or(0.0);
            current_sample * current_weight + candidate_sample * candidate_weight
        })
        .collect()
}

fn format_external_benchmark_coherence_candidate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    let mut improved_rows = 0;
    let mut unchanged_rows = 0;
    let mut regressed_rows = 0;
    let mut inconclusive_rows = 0;
    let mut best = None::<&ExternalBenchmarkCoherenceCandidateReviewMeasurement>;
    let mut worst = None::<&ExternalBenchmarkCoherenceCandidateReviewMeasurement>;

    for review in reviews {
        match review.outcome {
            "Improved" => improved_rows += 1,
            "Unchanged" => unchanged_rows += 1,
            "Regressed" => regressed_rows += 1,
            _ => inconclusive_rows += 1,
        }
        if review.target_score_delta.is_finite() {
            best = match best {
                Some(current) if current.target_score_delta <= review.target_score_delta => {
                    Some(current)
                }
                _ => Some(review),
            };
            worst = match worst {
                Some(current) if current.target_score_delta >= review.target_score_delta => {
                    Some(current)
                }
                _ => Some(review),
            };
        }
    }

    format!(
        "external_benchmark_coherence_candidate_summary rows={} improved_rows={} unchanged_rows={} regressed_rows={} inconclusive_rows={} best_improvement_delta={:.6} best_improvement_case={} best_improvement_source={} best_improvement_ratio={:.6} worst_regression_delta={:.6} worst_regression_case={} worst_regression_source={} worst_regression_ratio={:.6}",
        reviews.len(),
        improved_rows,
        unchanged_rows,
        regressed_rows,
        inconclusive_rows,
        best.map(|review| review.target_score_delta).unwrap_or(f64::NAN),
        best.map(|review| review.case_id.as_str()).unwrap_or(""),
        quoted_report_field(best.map(|review| review.source_path.as_str()).unwrap_or("")),
        best.map(|review| review.ratio).unwrap_or(f64::NAN),
        worst.map(|review| review.target_score_delta).unwrap_or(f64::NAN),
        worst.map(|review| review.case_id.as_str()).unwrap_or(""),
        quoted_report_field(worst.map(|review| review.source_path.as_str()).unwrap_or("")),
        worst.map(|review| review.ratio).unwrap_or(f64::NAN),
    )
}

fn format_external_benchmark_coherence_candidate_gate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    let mut selected_rows = 0;
    let mut rejected_rows = 0;
    let mut selected_improved_rows = 0;
    let mut selected_unchanged_rows = 0;
    let mut selected_regressed_rows = 0;
    let mut rejected_candidate_better_rows = 0;
    let mut rejected_current_better_rows = 0;
    let mut rejected_inconclusive_rows = 0;
    let mut worst_selected_regression_delta = 0.0_f64;
    let mut worst_selected_regression =
        None::<&ExternalBenchmarkCoherenceCandidateReviewMeasurement>;
    let mut best_rejected_improvement_delta = f64::NAN;

    for review in reviews {
        if review.gate_decision == "Selected" {
            selected_rows += 1;
            match review.outcome {
                "Improved" => selected_improved_rows += 1,
                "Unchanged" => selected_unchanged_rows += 1,
                "Regressed" => {
                    selected_regressed_rows += 1;
                    if review.target_score_delta > worst_selected_regression_delta {
                        worst_selected_regression_delta = review.target_score_delta;
                        worst_selected_regression = Some(review);
                    }
                }
                _ => {}
            }
        } else {
            rejected_rows += 1;
            match review.outcome {
                "Improved" => {
                    rejected_candidate_better_rows += 1;
                    if !best_rejected_improvement_delta.is_finite()
                        || review.target_score_delta < best_rejected_improvement_delta
                    {
                        best_rejected_improvement_delta = review.target_score_delta;
                    }
                }
                "Regressed" => rejected_current_better_rows += 1,
                "Inconclusive" => rejected_inconclusive_rows += 1,
                _ => {}
            }
        }
    }

    format!(
        "external_benchmark_coherence_candidate_gate_summary gate={} rows={} selected_rows={} rejected_rows={} selected_improved_rows={} selected_unchanged_rows={} selected_regressed_rows={} rejected_candidate_better_rows={} rejected_current_better_rows={} rejected_inconclusive_rows={} worst_selected_regression_delta={:.6} worst_selected_regression_case={} worst_selected_regression_source={} worst_selected_regression_ratio={:.6} best_rejected_improvement_delta={:.6}",
        EXTERNAL_BENCHMARK_COHERENCE_CANDIDATE_GATE,
        reviews.len(),
        selected_rows,
        rejected_rows,
        selected_improved_rows,
        selected_unchanged_rows,
        selected_regressed_rows,
        rejected_candidate_better_rows,
        rejected_current_better_rows,
        rejected_inconclusive_rows,
        worst_selected_regression_delta,
        worst_selected_regression
            .map(|review| review.case_id.as_str())
            .unwrap_or(""),
        quoted_report_field(
            worst_selected_regression
                .map(|review| review.source_path.as_str())
                .unwrap_or("")
        ),
        worst_selected_regression
            .map(|review| review.ratio)
            .unwrap_or(f64::NAN),
        best_rejected_improvement_delta,
    )
}

fn format_external_benchmark_coherence_product_probe_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    let mut selected_rows = 0;
    let mut rejected_rows = 0;
    let mut selected_improved_rows = 0;
    let mut selected_unchanged_rows = 0;
    let mut selected_regressed_rows = 0;
    let mut rejected_candidate_better_rows = 0;
    let mut rejected_current_better_rows = 0;
    let mut benchmark_gate_agree_rows = 0;
    let mut benchmark_gate_disagree_rows = 0;
    let mut worst_selected_regression_delta = 0.0_f64;

    for review in reviews {
        if review.product_probe_decision == review.gate_decision {
            benchmark_gate_agree_rows += 1;
        } else {
            benchmark_gate_disagree_rows += 1;
        }

        if review.product_probe_decision == "Selected" {
            selected_rows += 1;
            match review.outcome {
                "Improved" => selected_improved_rows += 1,
                "Unchanged" => selected_unchanged_rows += 1,
                "Regressed" => {
                    selected_regressed_rows += 1;
                    if review.target_score_delta > worst_selected_regression_delta {
                        worst_selected_regression_delta = review.target_score_delta;
                    }
                }
                _ => {}
            }
        } else {
            rejected_rows += 1;
            match review.outcome {
                "Improved" => rejected_candidate_better_rows += 1,
                "Regressed" => rejected_current_better_rows += 1,
                _ => {}
            }
        }
    }

    let (promotion_status, promotion_reason) =
        if selected_rows == 0 && rejected_candidate_better_rows > 0 {
            ("Rejected", "NoSelectedCandidateWins")
        } else if selected_regressed_rows > 0 {
            ("Rejected", "SelectedRegressions")
        } else if benchmark_gate_disagree_rows > 0 {
            ("NeedsReview", "BenchmarkGateDisagreement")
        } else {
            ("Candidate", "MatchesBenchmarkGate")
        };

    format!(
        "external_benchmark_coherence_product_probe_summary probe={} promotion_status={} promotion_reason={} rows={} selected_rows={} rejected_rows={} selected_improved_rows={} selected_unchanged_rows={} selected_regressed_rows={} rejected_candidate_better_rows={} rejected_current_better_rows={} benchmark_gate_agree_rows={} benchmark_gate_disagree_rows={} worst_selected_regression_delta={:.6}",
        EXTERNAL_BENCHMARK_COHERENCE_PRODUCT_PROBE,
        promotion_status,
        promotion_reason,
        reviews.len(),
        selected_rows,
        rejected_rows,
        selected_improved_rows,
        selected_unchanged_rows,
        selected_regressed_rows,
        rejected_candidate_better_rows,
        rejected_current_better_rows,
        benchmark_gate_agree_rows,
        benchmark_gate_disagree_rows,
        worst_selected_regression_delta,
    )
}

fn format_external_benchmark_coherence_blend_candidate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    format_external_benchmark_coherence_named_candidate_summary(
        "external_benchmark_coherence_blend_candidate_summary",
        EXTERNAL_BENCHMARK_COHERENCE_BLEND_CANDIDATE_PATH,
        reviews,
    )
}

fn format_external_benchmark_coherence_envelope_candidate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    format_external_benchmark_coherence_named_candidate_summary(
        "external_benchmark_coherence_envelope_candidate_summary",
        EXTERNAL_BENCHMARK_COHERENCE_ENVELOPE_CANDIDATE_PATH,
        reviews,
    )
}

fn format_external_benchmark_coherence_expansion_reset_candidate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    format_external_benchmark_coherence_named_candidate_summary(
        "external_benchmark_coherence_expansion_reset_candidate_summary",
        EXTERNAL_BENCHMARK_COHERENCE_EXPANSION_RESET_CANDIDATE_PATH,
        reviews,
    )
}

fn format_external_benchmark_coherence_stability_adaptive_candidate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    format_external_benchmark_coherence_named_candidate_summary(
        "external_benchmark_coherence_stability_adaptive_candidate_summary",
        EXTERNAL_BENCHMARK_COHERENCE_STABILITY_ADAPTIVE_CANDIDATE_PATH,
        reviews,
    )
}

fn format_external_benchmark_coherence_tracked_peak_candidate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    format_external_benchmark_coherence_named_candidate_summary(
        "external_benchmark_coherence_tracked_peak_candidate_summary",
        EXTERNAL_BENCHMARK_COHERENCE_TRACKED_PEAK_CANDIDATE_PATH,
        reviews,
    )
}

fn format_external_benchmark_coherence_magnitude_slew_candidate_summary(
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    format_external_benchmark_coherence_named_candidate_summary(
        "external_benchmark_coherence_magnitude_slew_candidate_summary",
        EXTERNAL_BENCHMARK_COHERENCE_MAGNITUDE_SLEW_CANDIDATE_PATH,
        reviews,
    )
}

fn format_external_benchmark_coherence_named_candidate_summary(
    line_name: &str,
    candidate_path: &str,
    reviews: &[ExternalBenchmarkCoherenceCandidateReviewMeasurement],
) -> String {
    let mut improved_rows = 0;
    let mut unchanged_rows = 0;
    let mut regressed_rows = 0;
    let mut inconclusive_rows = 0;
    let mut best = None::<&ExternalBenchmarkCoherenceCandidateReviewMeasurement>;
    let mut worst = None::<&ExternalBenchmarkCoherenceCandidateReviewMeasurement>;

    for review in reviews {
        match review.outcome {
            "Improved" => improved_rows += 1,
            "Unchanged" => unchanged_rows += 1,
            "Regressed" => regressed_rows += 1,
            _ => inconclusive_rows += 1,
        }
        if review.target_score_delta.is_finite() {
            best = match best {
                Some(current) if current.target_score_delta <= review.target_score_delta => {
                    Some(current)
                }
                _ => Some(review),
            };
            worst = match worst {
                Some(current) if current.target_score_delta >= review.target_score_delta => {
                    Some(current)
                }
                _ => Some(review),
            };
        }
    }

    let promotion_status = if regressed_rows == 0 && improved_rows > 0 {
        "Candidate"
    } else if regressed_rows > 0 {
        "Rejected"
    } else {
        "NeedsReview"
    };
    let promotion_reason = if regressed_rows > 0 {
        "RegressedRows"
    } else if improved_rows > 0 {
        "ImprovedWithoutRegressions"
    } else {
        "NoImprovedRows"
    };

    format!(
        "{} candidate_path={} promotion_status={} promotion_reason={} rows={} improved_rows={} unchanged_rows={} regressed_rows={} inconclusive_rows={} best_improvement_delta={:.6} best_improvement_case={} best_improvement_source={} best_improvement_ratio={:.6} worst_regression_delta={:.6} worst_regression_case={} worst_regression_source={} worst_regression_ratio={:.6}",
        line_name,
        candidate_path,
        promotion_status,
        promotion_reason,
        reviews.len(),
        improved_rows,
        unchanged_rows,
        regressed_rows,
        inconclusive_rows,
        best.map(|review| review.target_score_delta).unwrap_or(f64::NAN),
        best.map(|review| review.case_id.as_str()).unwrap_or(""),
        quoted_report_field(best.map(|review| review.source_path.as_str()).unwrap_or("")),
        best.map(|review| review.ratio).unwrap_or(f64::NAN),
        worst.map(|review| review.target_score_delta).unwrap_or(f64::NAN),
        worst.map(|review| review.case_id.as_str()).unwrap_or(""),
        quoted_report_field(worst.map(|review| review.source_path.as_str()).unwrap_or("")),
        worst.map(|review| review.ratio).unwrap_or(f64::NAN),
    )
}

enum ExternalBenchmarkQualitySource<'a> {
    Found(Cow<'a, StretchCorpusListeningSource>),
    Missing,
    Ambiguous,
}

fn source_for_external_quality_render<'a>(
    sources: &'a [StretchCorpusListeningSource],
    render: &ExternalBenchmarkQualityRender,
) -> ExternalBenchmarkQualitySource<'a> {
    if let Some(source_wav) = &render.source_wav {
        return ExternalBenchmarkQualitySource::Found(Cow::Owned(StretchCorpusListeningSource {
            case_id: render.case_id.clone(),
            source_path: source_wav.clone(),
            source_label: "external benchmark source excerpt".to_string(),
            license_title: "operator-provided local excerpt".to_string(),
            license_url: String::new(),
            provenance_url: String::new(),
        }));
    }

    let mut matches = sources
        .iter()
        .filter(|source| source.case_id == render.case_id);
    let Some(source) = matches.next() else {
        return ExternalBenchmarkQualitySource::Missing;
    };
    if matches.next().is_some() {
        return ExternalBenchmarkQualitySource::Ambiguous;
    }
    ExternalBenchmarkQualitySource::Found(Cow::Borrowed(source))
}

fn format_external_benchmark_quality_skip_line(
    render: &ExternalBenchmarkQualityRender,
    source_path: &str,
    reason: &'static str,
    signal_path: OfflineHighQualityPath,
    source_sample_rate_hz: u32,
    external_sample_rate_hz: u32,
    external_channels: u16,
    source_frames: usize,
) -> String {
    ExternalBenchmarkQualityMeasurement {
        case_id: render.case_id.clone(),
        source_path: source_path.to_string(),
        signal_path,
        render_path: render.rendered_path.clone(),
        tool_name: render.tool_name.clone(),
        ratio: render.ratio,
        status: "Skipped",
        reason,
        source_boundary: "rendered-output-only; no external source or library dependency",
        sample_rate_match: source_sample_rate_hz != 0
            && source_sample_rate_hz == external_sample_rate_hz,
        source_sample_rate_hz,
        external_sample_rate_hz,
        external_channels,
        source_frames,
        signal_frames: 0,
        external_frames: 0,
        signal_timing_drift_samples: f64::NAN,
        external_timing_drift_samples: f64::NAN,
        timing_drift_delta_samples: f64::NAN,
        signal_transient_smear_frames: f64::NAN,
        external_transient_smear_frames: f64::NAN,
        transient_smear_delta_frames: f64::NAN,
        signal_transient_matches: 0,
        external_transient_matches: 0,
        signal_transient_mean_signed_offset_frames: f64::NAN,
        external_transient_mean_signed_offset_frames: f64::NAN,
        signal_transient_mean_absolute_offset_frames: f64::NAN,
        external_transient_mean_absolute_offset_frames: f64::NAN,
        signal_transient_max_absolute_offset_frames: f64::NAN,
        external_transient_max_absolute_offset_frames: f64::NAN,
        signal_transient_max_crest_growth_db: f64::NAN,
        external_transient_max_crest_growth_db: f64::NAN,
        signal_transient_max_crest_input_frame: 0,
        external_transient_max_crest_input_frame: 0,
        signal_transient_max_crest_output_frame: 0,
        external_transient_max_crest_output_frame: 0,
        draft_transient_mean_absolute_offset_frames: f64::NAN,
        draft_transient_max_crest_growth_db: f64::NAN,
        draft_transient_max_crest_input_frame: 0,
        draft_transient_max_crest_output_frame: 0,
        alignment_lag_frames: 0,
        aligned_compared_frames: 0,
        aligned_correlation: f64::NAN,
        aligned_rms_error: f64::NAN,
        aligned_peak_error: f64::NAN,
        signal_rms: f64::NAN,
        external_rms: f64::NAN,
        aligned_rms_error_ratio: f64::NAN,
        integrity_limit_id: OFFLINE_HIGH_QUALITY_INTEGRITY_LIMIT_ID,
        signal_integrity_passed: false,
        external_integrity_passed: false,
        signal_measured_endpoint_count: 0,
        external_measured_endpoint_count: 0,
        signal_endpoint_energy_delta_db: f64::NAN,
        external_endpoint_energy_delta_db: f64::NAN,
        signal_added_silence_frames: usize::MAX,
        external_added_silence_frames: usize::MAX,
        signal_peak_growth_db: f64::NAN,
        external_peak_growth_db: f64::NAN,
        signal_render_seconds: f64::NAN,
        signal_cpu_realtime_factor: f64::NAN,
        signal_heap_baseline_bytes: 0,
        signal_heap_peak_bytes: 0,
        signal_peak_working_memory_bytes: 0,
    }
    .format_report_line()
}

fn decode_external_benchmark_render_audio(
    render: &ExternalBenchmarkQualityRender,
) -> Result<ExternalBenchmarkDecodedAudio, String> {
    let path = PathBuf::from(&render.rendered_path);
    let mut reader = hound::WavReader::open(&path)
        .map_err(|error| format!("failed to open external render {}: {error}", path.display()))?;
    let spec = reader.spec();
    if spec.channels == 0 || spec.sample_rate == 0 {
        return Err(format!(
            "invalid external render WAV {}: sample rate and channels must be non-zero",
            path.display()
        ));
    }
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| {
                sample.map_err(|error| format!("failed to read {}: {error}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = integer_sample_scale(spec.bits_per_sample);
            reader
                .samples::<i32>()
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / scale)
                        .map_err(|error| format!("failed to read {}: {error}", path.display()))
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let channel_count = spec.channels as usize;
    let mono_samples = samples
        .chunks_exact(channel_count)
        .map(|frame| frame.iter().sum::<f32>() / channel_count as f32)
        .collect();

    Ok(ExternalBenchmarkDecodedAudio {
        sample_rate_hz: spec.sample_rate,
        channels: spec.channels,
        samples,
        mono_samples,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct AlignedErrorMeasurement {
    lag_frames: isize,
    signal_start: usize,
    external_start: usize,
    compared_frames: usize,
    correlation: f64,
    rms_error: f64,
    peak_error: f64,
    signal_rms: f64,
    external_rms: f64,
}

fn align_and_measure_error(signal: &[f32], external: &[f32]) -> AlignedErrorMeasurement {
    let mut best_lag: isize = 0;
    let mut best_correlation = f64::NEG_INFINITY;
    for lag in
        -EXTERNAL_BENCHMARK_ALIGNMENT_MAX_LAG_FRAMES..=EXTERNAL_BENCHMARK_ALIGNMENT_MAX_LAG_FRAMES
    {
        let Some((signal_start, external_start, frames)) = aligned_ranges(signal, external, lag)
        else {
            continue;
        };
        let correlation = normalized_correlation(
            &signal[signal_start..signal_start + frames],
            &external[external_start..external_start + frames],
        );
        if correlation > best_correlation + 1.0e-12
            || ((correlation - best_correlation).abs() <= 1.0e-12 && lag.abs() < best_lag.abs())
        {
            best_correlation = correlation;
            best_lag = lag;
        }
    }

    let Some((signal_start, external_start, frames)) = aligned_ranges(signal, external, best_lag)
    else {
        return AlignedErrorMeasurement {
            lag_frames: 0,
            signal_start: 0,
            external_start: 0,
            compared_frames: 0,
            correlation: f64::NAN,
            rms_error: f64::NAN,
            peak_error: f64::NAN,
            signal_rms: f64::NAN,
            external_rms: f64::NAN,
        };
    };
    let signal_slice = &signal[signal_start..signal_start + frames];
    let external_slice = &external[external_start..external_start + frames];
    let mut square_error_sum = 0.0;
    let mut signal_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    let mut peak_error = 0.0f64;
    for (signal_sample, external_sample) in signal_slice.iter().zip(external_slice) {
        let signal_value = *signal_sample as f64;
        let external_value = *external_sample as f64;
        let error = signal_value - external_value;
        square_error_sum += error * error;
        signal_square_sum += signal_value * signal_value;
        external_square_sum += external_value * external_value;
        peak_error = peak_error.max(error.abs());
    }

    AlignedErrorMeasurement {
        lag_frames: best_lag,
        signal_start,
        external_start,
        compared_frames: frames,
        correlation: best_correlation,
        rms_error: (square_error_sum / frames as f64).sqrt(),
        peak_error,
        signal_rms: (signal_square_sum / frames as f64).sqrt(),
        external_rms: (external_square_sum / frames as f64).sqrt(),
    }
}

fn aligned_ranges(signal: &[f32], external: &[f32], lag: isize) -> Option<(usize, usize, usize)> {
    let signal_start = if lag < 0 { (-lag) as usize } else { 0 };
    let external_start = if lag > 0 { lag as usize } else { 0 };
    if signal_start >= signal.len() || external_start >= external.len() {
        return None;
    }
    let frames = (signal.len() - signal_start)
        .min(external.len() - external_start)
        .min(EXTERNAL_BENCHMARK_ALIGNMENT_MAX_COMPARE_FRAMES);
    (frames > 0).then_some((signal_start, external_start, frames))
}

fn normalized_correlation(signal: &[f32], external: &[f32]) -> f64 {
    let mut dot = 0.0;
    let mut signal_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    for (signal_sample, external_sample) in signal.iter().zip(external) {
        let signal_value = *signal_sample as f64;
        let external_value = *external_sample as f64;
        dot += signal_value * external_value;
        signal_square_sum += signal_value * signal_value;
        external_square_sum += external_value * external_value;
    }
    finite_ratio(dot, (signal_square_sum * external_square_sum).sqrt())
}

fn normalized_correlation_f64(signal: &[f64], external: &[f64]) -> f64 {
    let mut dot = 0.0;
    let mut signal_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    for (signal_value, external_value) in signal.iter().zip(external) {
        dot += signal_value * external_value;
        signal_square_sum += signal_value * signal_value;
        external_square_sum += external_value * external_value;
    }
    finite_ratio(dot, (signal_square_sum * external_square_sum).sqrt())
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkFeatureDelta {
    compared_frames: usize,
    envelope_correlation: f64,
    signal: ExternalBenchmarkFeatureSummary,
    external: ExternalBenchmarkFeatureSummary,
}

impl ExternalBenchmarkFeatureDelta {
    fn divergence_score(&self) -> f64 {
        if self.compared_frames == 0 {
            return f64::NAN;
        }
        let envelope_term = if self.envelope_correlation.is_finite() {
            (1.0 - self.envelope_correlation).max(0.0)
        } else {
            0.0
        };
        let rms_term = finite_abs(self.signal.rms_db - self.external.rms_db) / 12.0;
        let peak_term = finite_abs(self.signal.peak_db - self.external.peak_db) / 12.0;
        let centroid_term =
            finite_abs(self.signal.spectral_centroid_hz - self.external.spectral_centroid_hz)
                / 2_000.0;
        let high_frequency_term = finite_abs(
            self.signal.high_frequency_energy_ratio - self.external.high_frequency_energy_ratio,
        ) * 4.0;
        envelope_term + rms_term + peak_term + centroid_term + high_frequency_term
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkFeatureSummary {
    rms: f64,
    rms_db: f64,
    peak: f64,
    peak_db: f64,
    zero_crossings_per_second: f64,
    spectral_centroid_hz: f64,
    high_frequency_energy_ratio: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkGainEnvelopeReview {
    window_count: usize,
    mean_window_rms_delta_db: f64,
    median_window_rms_delta_db: f64,
    max_abs_window_rms_delta_db: f64,
    louder_windows: usize,
    quieter_windows: usize,
    near_windows: usize,
    gain_pattern: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkLevelNormalizedReview {
    signal_gain_db_applied: f64,
    normalized_feature_delta: ExternalBenchmarkFeatureDelta,
    normalization_pattern: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkResidualCoherenceReview {
    block_rms_envelope_correlation: f64,
    mean_abs_block_rms_delta_db: f64,
    max_abs_block_rms_delta_db: f64,
    spectral_magnitude_coherence: f64,
    residual_pattern: &'static str,
}

fn measure_external_benchmark_feature_delta(
    signal: &[f32],
    external: &[f32],
    aligned: &AlignedErrorMeasurement,
    sample_rate_hz: u32,
) -> ExternalBenchmarkFeatureDelta {
    if aligned.compared_frames == 0 {
        return ExternalBenchmarkFeatureDelta {
            compared_frames: 0,
            envelope_correlation: f64::NAN,
            signal: empty_external_benchmark_feature_summary(),
            external: empty_external_benchmark_feature_summary(),
        };
    }

    let signal_slice =
        &signal[aligned.signal_start..aligned.signal_start + aligned.compared_frames];
    let external_slice =
        &external[aligned.external_start..aligned.external_start + aligned.compared_frames];
    ExternalBenchmarkFeatureDelta {
        compared_frames: aligned.compared_frames,
        envelope_correlation: normalized_envelope_correlation(signal_slice, external_slice),
        signal: summarize_external_benchmark_features(signal_slice, sample_rate_hz),
        external: summarize_external_benchmark_features(external_slice, sample_rate_hz),
    }
}

fn measure_external_benchmark_level_normalized_review(
    signal: &[f32],
    external: &[f32],
    aligned: &AlignedErrorMeasurement,
    feature_delta: &ExternalBenchmarkFeatureDelta,
    sample_rate_hz: u32,
) -> ExternalBenchmarkLevelNormalizedReview {
    let signal_gain = finite_ratio(feature_delta.external.rms, feature_delta.signal.rms);
    if !signal_gain.is_finite() {
        return ExternalBenchmarkLevelNormalizedReview {
            signal_gain_db_applied: f64::NAN,
            normalized_feature_delta: empty_external_benchmark_feature_delta(),
            normalization_pattern: "Inconclusive",
        };
    }

    let normalized_signal = signal
        .iter()
        .map(|sample| (*sample as f64 * signal_gain) as f32)
        .collect::<Vec<_>>();
    let normalized_feature_delta = measure_external_benchmark_feature_delta(
        &normalized_signal,
        external,
        aligned,
        sample_rate_hz,
    );
    let normalization_pattern =
        classify_external_benchmark_level_normalization(feature_delta, &normalized_feature_delta);

    ExternalBenchmarkLevelNormalizedReview {
        signal_gain_db_applied: amplitude_db(signal_gain),
        normalized_feature_delta,
        normalization_pattern,
    }
}

fn measure_external_benchmark_residual_coherence_review(
    signal: &[f32],
    external: &[f32],
    aligned: &AlignedErrorMeasurement,
    raw_feature_delta: &ExternalBenchmarkFeatureDelta,
    normalized_review: &ExternalBenchmarkLevelNormalizedReview,
    sample_rate_hz: u32,
) -> ExternalBenchmarkResidualCoherenceReview {
    let signal_gain = amplitude_from_db(normalized_review.signal_gain_db_applied);
    if !signal_gain.is_finite()
        || aligned.compared_frames < EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE
    {
        return empty_external_benchmark_residual_coherence_review();
    }

    let normalized_signal = signal
        .iter()
        .map(|sample| (*sample as f64 * signal_gain) as f32)
        .collect::<Vec<_>>();
    let normalized_signal_slice =
        &normalized_signal[aligned.signal_start..aligned.signal_start + aligned.compared_frames];
    let external_slice =
        &external[aligned.external_start..aligned.external_start + aligned.compared_frames];
    let envelope = measure_block_rms_envelope_delta(normalized_signal_slice, external_slice);
    let spectral_magnitude_coherence = measure_spectral_magnitude_coherence(
        normalized_signal_slice,
        external_slice,
        sample_rate_hz,
    );

    ExternalBenchmarkResidualCoherenceReview {
        block_rms_envelope_correlation: envelope.block_rms_envelope_correlation,
        mean_abs_block_rms_delta_db: envelope.mean_abs_block_rms_delta_db,
        max_abs_block_rms_delta_db: envelope.max_abs_block_rms_delta_db,
        spectral_magnitude_coherence,
        residual_pattern: classify_external_benchmark_residual_coherence(
            raw_feature_delta,
            &normalized_review.normalized_feature_delta,
            envelope.block_rms_envelope_correlation,
            spectral_magnitude_coherence,
        ),
    }
}

fn empty_external_benchmark_residual_coherence_review() -> ExternalBenchmarkResidualCoherenceReview
{
    ExternalBenchmarkResidualCoherenceReview {
        block_rms_envelope_correlation: f64::NAN,
        mean_abs_block_rms_delta_db: f64::NAN,
        max_abs_block_rms_delta_db: f64::NAN,
        spectral_magnitude_coherence: f64::NAN,
        residual_pattern: "Inconclusive",
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkBlockEnvelopeDelta {
    block_rms_envelope_correlation: f64,
    mean_abs_block_rms_delta_db: f64,
    max_abs_block_rms_delta_db: f64,
}

fn measure_block_rms_envelope_delta(
    signal: &[f32],
    external: &[f32],
) -> ExternalBenchmarkBlockEnvelopeDelta {
    if signal.len() < EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE
        || external.len() < EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE
    {
        return ExternalBenchmarkBlockEnvelopeDelta {
            block_rms_envelope_correlation: f64::NAN,
            mean_abs_block_rms_delta_db: f64::NAN,
            max_abs_block_rms_delta_db: f64::NAN,
        };
    }

    let frame_count = signal.len().min(external.len());
    let max_start = frame_count - EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE;
    let mut signal_envelope = Vec::new();
    let mut external_envelope = Vec::new();
    let mut abs_delta_sum = 0.0;
    let mut max_abs_delta = 0.0f64;
    let mut start = 0;
    while start <= max_start {
        let end = start + EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE;
        let signal_rms = slice_rms(&signal[start..end]);
        let external_rms = slice_rms(&external[start..end]);
        signal_envelope.push(signal_rms);
        external_envelope.push(external_rms);
        let abs_delta = (amplitude_db(signal_rms) - amplitude_db(external_rms)).abs();
        abs_delta_sum += abs_delta;
        max_abs_delta = max_abs_delta.max(abs_delta);
        start += EXTERNAL_BENCHMARK_GAIN_ENVELOPE_HOP_SIZE;
    }

    ExternalBenchmarkBlockEnvelopeDelta {
        block_rms_envelope_correlation: normalized_correlation_f64(
            &signal_envelope,
            &external_envelope,
        ),
        mean_abs_block_rms_delta_db: finite_ratio(abs_delta_sum, signal_envelope.len() as f64),
        max_abs_block_rms_delta_db: max_abs_delta,
    }
}

fn measure_spectral_magnitude_coherence(
    signal: &[f32],
    external: &[f32],
    sample_rate_hz: u32,
) -> f64 {
    let frame_count = signal.len().min(external.len());
    if frame_count < EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE || sample_rate_hz == 0 {
        return f64::NAN;
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE);
    let window = hann_window(EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE);
    let hop = EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE;
    let max_start = frame_count - EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE;
    let window_count = (max_start / hop + 1).min(EXTERNAL_BENCHMARK_FEATURE_MAX_WINDOWS);
    let mut cosine_sum = 0.0;
    let mut measured_windows = 0;

    for window_index in 0..window_count {
        let start = window_index * hop;
        let signal_magnitudes = windowed_magnitudes(signal, start, &window, fft.clone());
        let external_magnitudes = windowed_magnitudes(external, start, &window, fft.clone());
        let cosine = normalized_correlation_f64(&signal_magnitudes, &external_magnitudes);
        if cosine.is_finite() {
            cosine_sum += cosine;
            measured_windows += 1;
        }
    }

    finite_ratio(cosine_sum, measured_windows as f64)
}

fn windowed_magnitudes(
    samples: &[f32],
    start: usize,
    window: &[f32],
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
) -> Vec<f64> {
    let mut buffer = vec![Complex32::new(0.0, 0.0); EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE];
    for index in 0..EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE {
        buffer[index].re = samples[start + index] * window[index];
    }
    fft.process(&mut buffer);
    buffer
        .iter()
        .take(EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE / 2 + 1)
        .skip(1)
        .map(|value| value.norm() as f64)
        .collect()
}

fn classify_external_benchmark_residual_coherence(
    raw: &ExternalBenchmarkFeatureDelta,
    normalized: &ExternalBenchmarkFeatureDelta,
    block_rms_envelope_correlation: f64,
    spectral_magnitude_coherence: f64,
) -> &'static str {
    let raw_score = raw.divergence_score();
    let normalized_score = normalized.divergence_score();
    if !raw_score.is_finite()
        || !normalized_score.is_finite()
        || !block_rms_envelope_correlation.is_finite()
        || !spectral_magnitude_coherence.is_finite()
    {
        return "Inconclusive";
    }
    if normalized_score <= raw_score * 0.5
        && block_rms_envelope_correlation >= 0.85
        && spectral_magnitude_coherence >= 0.85
    {
        return "MostlyPhaseOrFineTextureResidual";
    }
    if block_rms_envelope_correlation < 0.70 {
        return "ResidualEnvelopeDivergence";
    }
    if spectral_magnitude_coherence < 0.75 {
        return "ResidualSpectralMagnitudeDivergence";
    }
    "MixedResidualCoherence"
}

fn empty_external_benchmark_feature_delta() -> ExternalBenchmarkFeatureDelta {
    ExternalBenchmarkFeatureDelta {
        compared_frames: 0,
        envelope_correlation: f64::NAN,
        signal: empty_external_benchmark_feature_summary(),
        external: empty_external_benchmark_feature_summary(),
    }
}

fn classify_external_benchmark_level_normalization(
    raw: &ExternalBenchmarkFeatureDelta,
    normalized: &ExternalBenchmarkFeatureDelta,
) -> &'static str {
    let raw_score = raw.divergence_score();
    let normalized_score = normalized.divergence_score();
    if !raw_score.is_finite() || !normalized_score.is_finite() {
        return "Inconclusive";
    }
    let normalized_rms_delta_db = normalized.signal.rms_db - normalized.external.rms_db;
    if normalized_rms_delta_db.abs() <= EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB
        && normalized_score <= raw_score * 0.5
    {
        return "MostlyLevelExplained";
    }
    if normalized_rms_delta_db.abs() <= EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB
        && normalized_score < raw_score
    {
        return "LevelReducesDivergence";
    }
    if normalized_score >= raw_score - 0.05 {
        return "ResidualEnvelopeOrSpectralDivergence";
    }
    "PartlyLevelExplained"
}

fn measure_external_benchmark_gain_envelope_review(
    signal: &[f32],
    external: &[f32],
    aligned: &AlignedErrorMeasurement,
    feature_delta: &ExternalBenchmarkFeatureDelta,
) -> ExternalBenchmarkGainEnvelopeReview {
    if aligned.compared_frames < EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE {
        return empty_external_benchmark_gain_envelope_review();
    }

    let signal_slice =
        &signal[aligned.signal_start..aligned.signal_start + aligned.compared_frames];
    let external_slice =
        &external[aligned.external_start..aligned.external_start + aligned.compared_frames];
    let mut deltas = Vec::new();
    let max_start = aligned.compared_frames - EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE;
    let mut start = 0;
    while start <= max_start {
        let end = start + EXTERNAL_BENCHMARK_GAIN_ENVELOPE_WINDOW_SIZE;
        let signal_rms = slice_rms(&signal_slice[start..end]);
        let external_rms = slice_rms(&external_slice[start..end]);
        deltas.push(amplitude_db(signal_rms) - amplitude_db(external_rms));
        start += EXTERNAL_BENCHMARK_GAIN_ENVELOPE_HOP_SIZE;
    }

    let window_count = deltas.len();
    if window_count == 0 {
        return empty_external_benchmark_gain_envelope_review();
    }

    let mean = deltas.iter().sum::<f64>() / window_count as f64;
    let mut sorted = deltas.clone();
    sorted.sort_by(f64::total_cmp);
    let median = if sorted.len() % 2 == 0 {
        let upper = sorted.len() / 2;
        (sorted[upper - 1] + sorted[upper]) * 0.5
    } else {
        sorted[sorted.len() / 2]
    };
    let max_abs = deltas
        .iter()
        .map(|delta| delta.abs())
        .fold(0.0f64, f64::max);
    let louder_windows = deltas
        .iter()
        .filter(|delta| **delta > EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB)
        .count();
    let quieter_windows = deltas
        .iter()
        .filter(|delta| **delta < -EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB)
        .count();
    let near_windows = window_count - louder_windows - quieter_windows;

    ExternalBenchmarkGainEnvelopeReview {
        window_count,
        mean_window_rms_delta_db: mean,
        median_window_rms_delta_db: median,
        max_abs_window_rms_delta_db: max_abs,
        louder_windows,
        quieter_windows,
        near_windows,
        gain_pattern: classify_external_benchmark_gain_pattern(
            median,
            max_abs,
            louder_windows,
            quieter_windows,
            near_windows,
            feature_delta.envelope_correlation,
        ),
    }
}

fn empty_external_benchmark_gain_envelope_review() -> ExternalBenchmarkGainEnvelopeReview {
    ExternalBenchmarkGainEnvelopeReview {
        window_count: 0,
        mean_window_rms_delta_db: f64::NAN,
        median_window_rms_delta_db: f64::NAN,
        max_abs_window_rms_delta_db: f64::NAN,
        louder_windows: 0,
        quieter_windows: 0,
        near_windows: 0,
        gain_pattern: "Inconclusive",
    }
}

fn classify_external_benchmark_gain_pattern(
    median_delta_db: f64,
    max_abs_delta_db: f64,
    louder_windows: usize,
    quieter_windows: usize,
    near_windows: usize,
    envelope_correlation: f64,
) -> &'static str {
    if !median_delta_db.is_finite() || !max_abs_delta_db.is_finite() {
        return "Inconclusive";
    }
    if max_abs_delta_db <= EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB
        && near_windows >= louder_windows + quieter_windows
    {
        return "CloseGain";
    }
    if median_delta_db > EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB
        && louder_windows >= quieter_windows.saturating_mul(2).max(1)
    {
        return "SignalConsistentlyLouder";
    }
    if median_delta_db < -EXTERNAL_BENCHMARK_GAIN_ENVELOPE_NEAR_DB
        && quieter_windows >= louder_windows.saturating_mul(2).max(1)
    {
        return "SignalConsistentlyQuieter";
    }
    if envelope_correlation.is_finite() && envelope_correlation < 0.70 {
        return "EnvelopeShapeDivergence";
    }
    "MixedGainEnvelope"
}

fn empty_external_benchmark_feature_summary() -> ExternalBenchmarkFeatureSummary {
    ExternalBenchmarkFeatureSummary {
        rms: f64::NAN,
        rms_db: f64::NAN,
        peak: f64::NAN,
        peak_db: f64::NAN,
        zero_crossings_per_second: f64::NAN,
        spectral_centroid_hz: f64::NAN,
        high_frequency_energy_ratio: f64::NAN,
    }
}

fn summarize_external_benchmark_features(
    samples: &[f32],
    sample_rate_hz: u32,
) -> ExternalBenchmarkFeatureSummary {
    if samples.is_empty() || sample_rate_hz == 0 {
        return empty_external_benchmark_feature_summary();
    }

    let mut peak = 0.0f64;
    let mut square_sum = 0.0;
    for sample in samples {
        let value = *sample as f64;
        peak = peak.max(value.abs());
        square_sum += value * value;
    }
    let rms = (square_sum / samples.len() as f64).sqrt();
    let duration_seconds = samples.len() as f64 / sample_rate_hz as f64;
    let zero_crossings = samples
        .windows(2)
        .filter(|pair| (pair[0] < 0.0 && pair[1] >= 0.0) || (pair[0] >= 0.0 && pair[1] < 0.0))
        .count();
    let spectral = summarize_external_benchmark_spectrum(samples, sample_rate_hz);

    ExternalBenchmarkFeatureSummary {
        rms,
        rms_db: amplitude_db(rms),
        peak,
        peak_db: amplitude_db(peak),
        zero_crossings_per_second: zero_crossings as f64 / duration_seconds.max(1.0e-12),
        spectral_centroid_hz: spectral.centroid_hz,
        high_frequency_energy_ratio: spectral.high_frequency_energy_ratio,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ExternalBenchmarkSpectralSummary {
    centroid_hz: f64,
    high_frequency_energy_ratio: f64,
}

fn summarize_external_benchmark_spectrum(
    samples: &[f32],
    sample_rate_hz: u32,
) -> ExternalBenchmarkSpectralSummary {
    if samples.len() < EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE || sample_rate_hz == 0 {
        return ExternalBenchmarkSpectralSummary {
            centroid_hz: f64::NAN,
            high_frequency_energy_ratio: f64::NAN,
        };
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE);
    let window = hann_window(EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE);
    let hop = EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE;
    let max_start = samples.len() - EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE;
    let window_count = (max_start / hop + 1).min(EXTERNAL_BENCHMARK_FEATURE_MAX_WINDOWS);
    let mut centroid_weighted_hz_sum = 0.0;
    let mut magnitude_sum = 0.0;
    let mut total_energy = 0.0;
    let mut high_frequency_energy = 0.0;
    let high_frequency_bin = EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE / 6;
    let nyquist_bin = EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE / 2;
    let bin_hz = sample_rate_hz as f64 / EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE as f64;

    for window_index in 0..window_count {
        let start = window_index * hop;
        let mut buffer = vec![Complex32::new(0.0, 0.0); EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE];
        for index in 0..EXTERNAL_BENCHMARK_FEATURE_FFT_SIZE {
            buffer[index].re = samples[start + index] * window[index];
        }
        fft.process(&mut buffer);
        for (bin, value) in buffer.iter().enumerate().take(nyquist_bin + 1).skip(1) {
            let magnitude = value.norm() as f64;
            let energy = magnitude * magnitude;
            centroid_weighted_hz_sum += bin as f64 * bin_hz * magnitude;
            magnitude_sum += magnitude;
            total_energy += energy;
            if bin >= high_frequency_bin {
                high_frequency_energy += energy;
            }
        }
    }

    ExternalBenchmarkSpectralSummary {
        centroid_hz: finite_ratio(centroid_weighted_hz_sum, magnitude_sum),
        high_frequency_energy_ratio: finite_ratio(high_frequency_energy, total_energy),
    }
}

fn hann_window(size: usize) -> Vec<f32> {
    if size <= 1 {
        return vec![1.0; size];
    }
    (0..size)
        .map(|index| {
            let phase = std::f32::consts::TAU * index as f32 / (size - 1) as f32;
            0.5 - 0.5 * phase.cos()
        })
        .collect()
}

fn normalized_envelope_correlation(signal: &[f32], external: &[f32]) -> f64 {
    let mut dot = 0.0;
    let mut signal_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    for (signal_sample, external_sample) in signal.iter().zip(external) {
        let signal_value = signal_sample.abs() as f64;
        let external_value = external_sample.abs() as f64;
        dot += signal_value * external_value;
        signal_square_sum += signal_value * signal_value;
        external_square_sum += external_value * external_value;
    }
    finite_ratio(dot, (signal_square_sum * external_square_sum).sqrt())
}

fn slice_rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return f64::NAN;
    }
    let square_sum = samples
        .iter()
        .map(|sample| {
            let value = *sample as f64;
            value * value
        })
        .sum::<f64>();
    (square_sum / samples.len() as f64).sqrt()
}

fn amplitude_db(value: f64) -> f64 {
    if !value.is_finite() || value <= 1.0e-12 {
        -240.0
    } else {
        20.0 * value.log10()
    }
}

fn amplitude_from_db(value_db: f64) -> f64 {
    if !value_db.is_finite() {
        f64::NAN
    } else {
        10.0f64.powf(value_db / 20.0)
    }
}

fn finite_abs(value: f64) -> f64 {
    if value.is_finite() {
        value.abs()
    } else {
        0.0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TransientWidthControlCandidateAccumulator {
    rows: usize,
    candidate_better_rows: usize,
    current_better_rows: usize,
    unchanged_rows: usize,
    inconclusive_rows: usize,
    finite_rows: usize,
    offline_smear_sum: f64,
    candidate_smear_sum: f64,
    worst_candidate_regression_delta_frames: f64,
    worst_candidate_regression_case_id: String,
    worst_candidate_regression_source: String,
    worst_candidate_regression_ratio: f64,
    best_candidate_improvement_delta_frames: f64,
    best_candidate_improvement_case_id: String,
    best_candidate_improvement_source: String,
    best_candidate_improvement_ratio: f64,
    worst_draft_regression_delta_frames: f64,
    worst_draft_regression_case_id: String,
    worst_draft_regression_source: String,
    worst_draft_regression_ratio: f64,
    edited_rows: usize,
    edited_samples: usize,
    max_abs_sample_delta: f64,
    max_abs_sample_delta_case_id: String,
    max_abs_sample_delta_source: String,
    max_abs_sample_delta_ratio: f64,
    max_abs_sample_delta_event: Option<WidthControlEditEvent>,
    max_added_adjacent_step_delta: f64,
    max_added_adjacent_step_case_id: String,
    max_added_adjacent_step_source: String,
    max_added_adjacent_step_ratio: f64,
    max_added_adjacent_step_event: Option<WidthControlEditEvent>,
}

impl TransientWidthControlCandidateAccumulator {
    fn record(
        &mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        draft: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        offline: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        candidate: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        edit: WidthControlEditStats,
    ) {
        if ratio >= 1.0 {
            return;
        }

        self.rows += 1;
        if edit.changed_samples > 0 {
            self.edited_rows += 1;
            self.edited_samples += edit.changed_samples;
            if edit.max_abs_sample_delta > self.max_abs_sample_delta {
                self.max_abs_sample_delta = edit.max_abs_sample_delta;
                self.max_abs_sample_delta_case_id = audio.case_id.clone();
                self.max_abs_sample_delta_source = audio.source_path.clone();
                self.max_abs_sample_delta_ratio = ratio;
                self.max_abs_sample_delta_event = edit
                    .max_abs_sample_delta_event
                    .map(|event| event.with_source(audio, ratio, "MaxSampleDelta"));
            }
            if edit.max_added_adjacent_step_delta > self.max_added_adjacent_step_delta {
                self.max_added_adjacent_step_delta = edit.max_added_adjacent_step_delta;
                self.max_added_adjacent_step_case_id = audio.case_id.clone();
                self.max_added_adjacent_step_source = audio.source_path.clone();
                self.max_added_adjacent_step_ratio = ratio;
                self.max_added_adjacent_step_event = edit
                    .max_added_adjacent_step_event
                    .map(|event| event.with_source(audio, ratio, "MaxAddedAdjacentStep"));
            }
        }
        match compare_metric_values(candidate.max_smear_frames, offline.max_smear_frames) {
            MetricComparison::Improved => self.candidate_better_rows += 1,
            MetricComparison::Same => {
                if candidate.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
                    self.unchanged_rows += 1;
                } else {
                    self.inconclusive_rows += 1;
                }
            }
            MetricComparison::Worsened => self.current_better_rows += 1,
        }

        if offline.max_smear_frames.is_finite() && candidate.max_smear_frames.is_finite() {
            self.finite_rows += 1;
            self.offline_smear_sum += offline.max_smear_frames;
            self.candidate_smear_sum += candidate.max_smear_frames;
            let candidate_delta = candidate.max_smear_frames - offline.max_smear_frames;
            let improvement_delta = offline.max_smear_frames - candidate.max_smear_frames;
            if improvement_delta > self.best_candidate_improvement_delta_frames {
                self.best_candidate_improvement_delta_frames = improvement_delta;
                self.best_candidate_improvement_case_id = audio.case_id.clone();
                self.best_candidate_improvement_source = audio.source_path.clone();
                self.best_candidate_improvement_ratio = ratio;
            }
            if candidate_delta > self.worst_candidate_regression_delta_frames {
                self.worst_candidate_regression_delta_frames = candidate_delta;
                self.worst_candidate_regression_case_id = audio.case_id.clone();
                self.worst_candidate_regression_source = audio.source_path.clone();
                self.worst_candidate_regression_ratio = ratio;
            }
        }

        if draft.max_smear_frames.is_finite() && candidate.max_smear_frames.is_finite() {
            let draft_delta = candidate.max_smear_frames - draft.max_smear_frames;
            if draft_delta > self.worst_draft_regression_delta_frames {
                self.worst_draft_regression_delta_frames = draft_delta;
                self.worst_draft_regression_case_id = audio.case_id.clone();
                self.worst_draft_regression_source = audio.source_path.clone();
                self.worst_draft_regression_ratio = ratio;
            }
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_transient_width_control_candidate rows={} candidate_path=offline_hq_width_control baseline_path=offline_hq candidate_better_rows={} current_better_rows={} unchanged_rows={} inconclusive_rows={} finite_rows={} mean_candidate_smear_frames={:.6} mean_current_smear_frames={:.6} best_candidate_improvement_delta_frames={:.6} best_candidate_improvement_case={} best_candidate_improvement_source={} best_candidate_improvement_ratio={:.6} worst_candidate_regression_delta_frames={:.6} worst_candidate_regression_case={} worst_candidate_regression_source={} worst_candidate_regression_ratio={:.6} worst_draft_regression_delta_frames={:.6} worst_draft_regression_case={} worst_draft_regression_source={} worst_draft_regression_ratio={:.6} edited_rows={} edited_samples={} max_abs_sample_delta={:.9} max_abs_sample_delta_case={} max_abs_sample_delta_source={} max_abs_sample_delta_ratio={:.6} max_added_adjacent_step_delta={:.9} max_added_adjacent_step_case={} max_added_adjacent_step_source={} max_added_adjacent_step_ratio={:.6}",
            self.rows,
            self.candidate_better_rows,
            self.current_better_rows,
            self.unchanged_rows,
            self.inconclusive_rows,
            self.finite_rows,
            finite_ratio(self.candidate_smear_sum, self.finite_rows as f64),
            finite_ratio(self.offline_smear_sum, self.finite_rows as f64),
            self.best_candidate_improvement_delta_frames,
            self.best_candidate_improvement_case_id,
            quoted_report_field(&self.best_candidate_improvement_source),
            self.best_candidate_improvement_ratio,
            self.worst_candidate_regression_delta_frames,
            self.worst_candidate_regression_case_id,
            quoted_report_field(&self.worst_candidate_regression_source),
            self.worst_candidate_regression_ratio,
            self.worst_draft_regression_delta_frames,
            self.worst_draft_regression_case_id,
            quoted_report_field(&self.worst_draft_regression_source),
            self.worst_draft_regression_ratio,
            self.edited_rows,
            self.edited_samples,
            self.max_abs_sample_delta,
            self.max_abs_sample_delta_case_id,
            quoted_report_field(&self.max_abs_sample_delta_source),
            self.max_abs_sample_delta_ratio,
            self.max_added_adjacent_step_delta,
            self.max_added_adjacent_step_case_id,
            quoted_report_field(&self.max_added_adjacent_step_source),
            self.max_added_adjacent_step_ratio,
        )
    }

    fn format_edit_event_lines(&self) -> Vec<String> {
        [
            self.max_abs_sample_delta_event.as_ref(),
            self.max_added_adjacent_step_event.as_ref(),
        ]
        .into_iter()
        .flatten()
        .map(WidthControlEditEvent::format_report_line)
        .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TransientWidthControlEditGateAccumulator {
    rows: usize,
    accepted_rows: usize,
    rejected_rows: usize,
    accepted_edited_rows: usize,
    rejected_edited_rows: usize,
    gated_better_rows: usize,
    current_better_rows: usize,
    unchanged_rows: usize,
    inconclusive_rows: usize,
    finite_rows: usize,
    offline_smear_sum: f64,
    gated_smear_sum: f64,
    accepted_candidate_better_rows: usize,
    rejected_candidate_better_rows: usize,
    rejected_candidate_improvement_delta_frames: f64,
    max_rejected_abs_sample_delta: f64,
    max_rejected_abs_sample_delta_case_id: String,
    max_rejected_abs_sample_delta_source: String,
    max_rejected_abs_sample_delta_ratio: f64,
    max_rejected_added_adjacent_step_delta: f64,
    max_rejected_added_adjacent_step_case_id: String,
    max_rejected_added_adjacent_step_source: String,
    max_rejected_added_adjacent_step_ratio: f64,
}

impl TransientWidthControlEditGateAccumulator {
    fn record(
        &mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        offline: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        candidate: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        edit: &WidthControlEditStats,
    ) {
        if ratio >= 1.0 {
            return;
        }

        self.rows += 1;
        let accepted = width_control_edit_gate_accepts(edit);
        if accepted {
            self.accepted_rows += 1;
            if edit.changed_samples > 0 {
                self.accepted_edited_rows += 1;
            }
        } else {
            self.rejected_rows += 1;
            if edit.changed_samples > 0 {
                self.rejected_edited_rows += 1;
            }
            if edit.max_abs_sample_delta > self.max_rejected_abs_sample_delta {
                self.max_rejected_abs_sample_delta = edit.max_abs_sample_delta;
                self.max_rejected_abs_sample_delta_case_id = audio.case_id.clone();
                self.max_rejected_abs_sample_delta_source = audio.source_path.clone();
                self.max_rejected_abs_sample_delta_ratio = ratio;
            }
            if edit.max_added_adjacent_step_delta > self.max_rejected_added_adjacent_step_delta {
                self.max_rejected_added_adjacent_step_delta = edit.max_added_adjacent_step_delta;
                self.max_rejected_added_adjacent_step_case_id = audio.case_id.clone();
                self.max_rejected_added_adjacent_step_source = audio.source_path.clone();
                self.max_rejected_added_adjacent_step_ratio = ratio;
            }
        }

        if compare_metric_values(candidate.max_smear_frames, offline.max_smear_frames)
            == MetricComparison::Improved
        {
            if accepted {
                self.accepted_candidate_better_rows += 1;
            } else {
                self.rejected_candidate_better_rows += 1;
                if candidate.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
                    self.rejected_candidate_improvement_delta_frames +=
                        offline.max_smear_frames - candidate.max_smear_frames;
                }
            }
        }

        let gated = if accepted { candidate } else { offline };
        match compare_metric_values(gated.max_smear_frames, offline.max_smear_frames) {
            MetricComparison::Improved => self.gated_better_rows += 1,
            MetricComparison::Same => {
                if gated.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
                    self.unchanged_rows += 1;
                } else {
                    self.inconclusive_rows += 1;
                }
            }
            MetricComparison::Worsened => self.current_better_rows += 1,
        }

        if offline.max_smear_frames.is_finite() && gated.max_smear_frames.is_finite() {
            self.finite_rows += 1;
            self.offline_smear_sum += offline.max_smear_frames;
            self.gated_smear_sum += gated.max_smear_frames;
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_transient_width_control_edit_gate rows={} gate=ConservativeEditPressure max_abs_sample_delta_limit={:.9} max_added_adjacent_step_delta_limit={:.9} accepted_rows={} rejected_rows={} accepted_edited_rows={} rejected_edited_rows={} gated_better_rows={} current_better_rows={} unchanged_rows={} inconclusive_rows={} finite_rows={} mean_gated_smear_frames={:.6} mean_current_smear_frames={:.6} accepted_candidate_better_rows={} rejected_candidate_better_rows={} rejected_candidate_improvement_delta_frames={:.6} max_rejected_abs_sample_delta={:.9} max_rejected_abs_sample_delta_case={} max_rejected_abs_sample_delta_source={} max_rejected_abs_sample_delta_ratio={:.6} max_rejected_added_adjacent_step_delta={:.9} max_rejected_added_adjacent_step_case={} max_rejected_added_adjacent_step_source={} max_rejected_added_adjacent_step_ratio={:.6}",
            self.rows,
            WIDTH_CONTROL_EDIT_GATE_MAX_SAMPLE_DELTA,
            WIDTH_CONTROL_EDIT_GATE_MAX_ADDED_ADJACENT_STEP_DELTA,
            self.accepted_rows,
            self.rejected_rows,
            self.accepted_edited_rows,
            self.rejected_edited_rows,
            self.gated_better_rows,
            self.current_better_rows,
            self.unchanged_rows,
            self.inconclusive_rows,
            self.finite_rows,
            finite_ratio(self.gated_smear_sum, self.finite_rows as f64),
            finite_ratio(self.offline_smear_sum, self.finite_rows as f64),
            self.accepted_candidate_better_rows,
            self.rejected_candidate_better_rows,
            self.rejected_candidate_improvement_delta_frames,
            self.max_rejected_abs_sample_delta,
            self.max_rejected_abs_sample_delta_case_id,
            quoted_report_field(&self.max_rejected_abs_sample_delta_source),
            self.max_rejected_abs_sample_delta_ratio,
            self.max_rejected_added_adjacent_step_delta,
            self.max_rejected_added_adjacent_step_case_id,
            quoted_report_field(&self.max_rejected_added_adjacent_step_source),
            self.max_rejected_added_adjacent_step_ratio,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct WidthControlEditStats {
    changed_samples: usize,
    max_abs_sample_delta: f64,
    max_abs_sample_delta_event: Option<WidthControlEditEvent>,
    max_added_adjacent_step_delta: f64,
    max_added_adjacent_step_event: Option<WidthControlEditEvent>,
}

fn width_control_edit_gate_accepts(edit: &WidthControlEditStats) -> bool {
    edit.max_abs_sample_delta <= WIDTH_CONTROL_EDIT_GATE_MAX_SAMPLE_DELTA
        && edit.max_added_adjacent_step_delta
            <= WIDTH_CONTROL_EDIT_GATE_MAX_ADDED_ADJACENT_STEP_DELTA
}

#[derive(Clone, Debug, Default, PartialEq)]
struct WidthControlEditEvent {
    kind: &'static str,
    output_frame: usize,
    source_frame: f64,
    sample_delta: f64,
    added_adjacent_step_delta: f64,
    baseline_peak: f64,
    baseline_rms: f64,
    candidate_peak: f64,
    candidate_rms: f64,
    baseline_adjacent_step: f64,
    candidate_adjacent_step: f64,
    case_id: String,
    source_path: String,
    ratio: f64,
}

impl WidthControlEditEvent {
    fn from_output_frame(
        output_frame: usize,
        baseline: &[f32],
        candidate: &[f32],
        ratio: f64,
    ) -> Self {
        let baseline_window = window_energy_stats(baseline, output_frame as f64);
        let candidate_window = window_energy_stats(candidate, output_frame as f64);
        let baseline_adjacent_step = adjacent_step_at(baseline, output_frame);
        let candidate_adjacent_step = adjacent_step_at(candidate, output_frame);
        Self {
            kind: "",
            output_frame,
            source_frame: output_frame as f64 / ratio,
            sample_delta: sample_delta_at(baseline, candidate, output_frame),
            added_adjacent_step_delta: (candidate_adjacent_step - baseline_adjacent_step).max(0.0),
            baseline_peak: baseline_window.peak,
            baseline_rms: baseline_window.rms,
            candidate_peak: candidate_window.peak,
            candidate_rms: candidate_window.rms,
            baseline_adjacent_step,
            candidate_adjacent_step,
            case_id: String::new(),
            source_path: String::new(),
            ratio,
        }
    }

    fn with_source(
        mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        kind: &'static str,
    ) -> Self {
        self.kind = kind;
        self.case_id = audio.case_id.clone();
        self.source_path = audio.source_path.clone();
        self.ratio = ratio;
        self
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_transient_width_control_edit_event kind={} case={} source={} ratio={:.6} source_frame={:.6} output_frame={} sample_delta={:.9} added_adjacent_step_delta={:.9} baseline_peak={:.9} baseline_rms={:.9} candidate_peak={:.9} candidate_rms={:.9} baseline_adjacent_step={:.9} candidate_adjacent_step={:.9}",
            self.kind,
            self.case_id,
            quoted_report_field(&self.source_path),
            self.ratio,
            self.source_frame,
            self.output_frame,
            self.sample_delta,
            self.added_adjacent_step_delta,
            self.baseline_peak,
            self.baseline_rms,
            self.candidate_peak,
            self.candidate_rms,
            self.baseline_adjacent_step,
            self.candidate_adjacent_step,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct CompressionPhaseLockAblationAccumulator {
    rows: usize,
    phase_locked_better_rows: usize,
    independent_bins_better_rows: usize,
    unchanged_rows: usize,
    inconclusive_rows: usize,
    draft_smear_sum: f64,
    offline_smear_sum: f64,
    finite_rows: usize,
    worst_regression_delta_frames: f64,
    worst_regression_case_id: String,
    worst_regression_source: String,
    worst_regression_ratio: f64,
}

impl CompressionPhaseLockAblationAccumulator {
    fn record(
        &mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        draft: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        offline: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) {
        if ratio >= 1.0 {
            return;
        }

        self.rows += 1;
        match compare_metric_values(offline.max_smear_frames, draft.max_smear_frames) {
            MetricComparison::Improved => self.phase_locked_better_rows += 1,
            MetricComparison::Same => {
                if draft.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
                    self.unchanged_rows += 1;
                } else {
                    self.inconclusive_rows += 1;
                }
            }
            MetricComparison::Worsened => self.independent_bins_better_rows += 1,
        }

        if draft.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
            self.finite_rows += 1;
            self.draft_smear_sum += draft.max_smear_frames;
            self.offline_smear_sum += offline.max_smear_frames;
            let delta = offline.max_smear_frames - draft.max_smear_frames;
            if delta > self.worst_regression_delta_frames {
                self.worst_regression_delta_frames = delta;
                self.worst_regression_case_id = audio.case_id.clone();
                self.worst_regression_source = audio.source_path.clone();
                self.worst_regression_ratio = ratio;
            }
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_compression_phase_lock_ablation rows={} phase_locked_path=offline_hq independent_bins_path=draft phase_locked_better_rows={} independent_bins_better_rows={} unchanged_rows={} inconclusive_rows={} finite_rows={} mean_phase_locked_smear_frames={:.6} mean_independent_bins_smear_frames={:.6} worst_regression_delta_frames={:.6} worst_regression_case={} worst_regression_source={} worst_regression_ratio={:.6}",
            self.rows,
            self.phase_locked_better_rows,
            self.independent_bins_better_rows,
            self.unchanged_rows,
            self.inconclusive_rows,
            self.finite_rows,
            finite_ratio(self.offline_smear_sum, self.finite_rows as f64),
            finite_ratio(self.draft_smear_sum, self.finite_rows as f64),
            self.worst_regression_delta_frames,
            self.worst_regression_case_id,
            quoted_report_field(&self.worst_regression_source),
            self.worst_regression_ratio,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CompressionReviewCandidateAccumulator {
    report_name: &'static str,
    candidate_path: &'static str,
    ratio_scope: CandidateRatioScope,
    feature_report_name: Option<&'static str>,
    rows: usize,
    candidate_better_rows: usize,
    current_better_rows: usize,
    unchanged_rows: usize,
    inconclusive_rows: usize,
    finite_rows: usize,
    offline_smear_sum: f64,
    candidate_smear_sum: f64,
    best_candidate_improvement_delta_frames: f64,
    best_candidate_improvement_case_id: String,
    best_candidate_improvement_source: String,
    best_candidate_improvement_ratio: f64,
    worst_candidate_regression_delta_frames: f64,
    worst_candidate_regression_case_id: String,
    worst_candidate_regression_source: String,
    worst_candidate_regression_ratio: f64,
    worst_draft_regression_delta_frames: f64,
    worst_draft_regression_case_id: String,
    worst_draft_regression_source: String,
    worst_draft_regression_ratio: f64,
    baseline_worst_draft_regression_delta_frames: f64,
    baseline_worst_draft_regression_case_id: String,
    baseline_worst_draft_regression_source: String,
    baseline_worst_draft_regression_ratio: f64,
    baseline_worst_draft_smear_frames: f64,
    baseline_worst_current_smear_frames: f64,
    baseline_worst_candidate_smear_frames: f64,
    feature_rows: Vec<CompressionReviewFeatureRow>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateRatioScope {
    Compression,
    Expansion,
}

impl CandidateRatioScope {
    fn accepts(self, ratio: f64) -> bool {
        match self {
            Self::Compression => ratio < 1.0,
            Self::Expansion => ratio > 1.0,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Compression => "compression",
            Self::Expansion => "expansion",
        }
    }
}

impl CompressionReviewCandidateAccumulator {
    fn new(report_name: &'static str, candidate_path: &'static str) -> Self {
        Self {
            report_name,
            candidate_path,
            ratio_scope: CandidateRatioScope::Compression,
            feature_report_name: None,
            rows: 0,
            candidate_better_rows: 0,
            current_better_rows: 0,
            unchanged_rows: 0,
            inconclusive_rows: 0,
            finite_rows: 0,
            offline_smear_sum: 0.0,
            candidate_smear_sum: 0.0,
            best_candidate_improvement_delta_frames: 0.0,
            best_candidate_improvement_case_id: String::new(),
            best_candidate_improvement_source: String::new(),
            best_candidate_improvement_ratio: 0.0,
            worst_candidate_regression_delta_frames: 0.0,
            worst_candidate_regression_case_id: String::new(),
            worst_candidate_regression_source: String::new(),
            worst_candidate_regression_ratio: 0.0,
            worst_draft_regression_delta_frames: 0.0,
            worst_draft_regression_case_id: String::new(),
            worst_draft_regression_source: String::new(),
            worst_draft_regression_ratio: 0.0,
            baseline_worst_draft_regression_delta_frames: 0.0,
            baseline_worst_draft_regression_case_id: String::new(),
            baseline_worst_draft_regression_source: String::new(),
            baseline_worst_draft_regression_ratio: 0.0,
            baseline_worst_draft_smear_frames: f64::NAN,
            baseline_worst_current_smear_frames: f64::NAN,
            baseline_worst_candidate_smear_frames: f64::NAN,
            feature_rows: Vec::new(),
        }
    }

    fn new_expansion(report_name: &'static str, candidate_path: &'static str) -> Self {
        Self {
            ratio_scope: CandidateRatioScope::Expansion,
            ..Self::new(report_name, candidate_path)
        }
    }

    fn with_feature_report(mut self, feature_report_name: &'static str) -> Self {
        self.feature_report_name = Some(feature_report_name);
        self
    }

    fn record(
        &mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        draft: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        offline: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        candidate: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) {
        if !self.ratio_scope.accepts(ratio) {
            return;
        }

        self.rows += 1;
        let comparison =
            compare_metric_values(candidate.max_smear_frames, offline.max_smear_frames);
        match comparison {
            MetricComparison::Improved => self.candidate_better_rows += 1,
            MetricComparison::Same => {
                if candidate.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
                    self.unchanged_rows += 1;
                } else {
                    self.inconclusive_rows += 1;
                }
            }
            MetricComparison::Worsened => self.current_better_rows += 1,
        }
        if let Some(feature_report_name) = self.feature_report_name {
            if matches!(
                comparison,
                MetricComparison::Improved | MetricComparison::Worsened
            ) {
                self.feature_rows
                    .push(CompressionReviewFeatureRow::from_measurements(
                        feature_report_name,
                        self.candidate_path,
                        audio,
                        ratio,
                        comparison,
                        draft,
                        offline,
                        candidate,
                    ));
            }
        }

        if offline.max_smear_frames.is_finite() && candidate.max_smear_frames.is_finite() {
            self.finite_rows += 1;
            self.offline_smear_sum += offline.max_smear_frames;
            self.candidate_smear_sum += candidate.max_smear_frames;
            let candidate_delta = candidate.max_smear_frames - offline.max_smear_frames;
            let improvement_delta = offline.max_smear_frames - candidate.max_smear_frames;
            if improvement_delta > self.best_candidate_improvement_delta_frames {
                self.best_candidate_improvement_delta_frames = improvement_delta;
                self.best_candidate_improvement_case_id = audio.case_id.clone();
                self.best_candidate_improvement_source = audio.source_path.clone();
                self.best_candidate_improvement_ratio = ratio;
            }
            if candidate_delta > self.worst_candidate_regression_delta_frames {
                self.worst_candidate_regression_delta_frames = candidate_delta;
                self.worst_candidate_regression_case_id = audio.case_id.clone();
                self.worst_candidate_regression_source = audio.source_path.clone();
                self.worst_candidate_regression_ratio = ratio;
            }
        }

        if draft.max_smear_frames.is_finite() && candidate.max_smear_frames.is_finite() {
            let draft_delta = candidate.max_smear_frames - draft.max_smear_frames;
            if draft_delta > self.worst_draft_regression_delta_frames {
                self.worst_draft_regression_delta_frames = draft_delta;
                self.worst_draft_regression_case_id = audio.case_id.clone();
                self.worst_draft_regression_source = audio.source_path.clone();
                self.worst_draft_regression_ratio = ratio;
            }
        }

        if draft.max_smear_frames.is_finite()
            && offline.max_smear_frames.is_finite()
            && candidate.max_smear_frames.is_finite()
        {
            let baseline_draft_delta = offline.max_smear_frames - draft.max_smear_frames;
            if baseline_draft_delta > self.baseline_worst_draft_regression_delta_frames {
                self.baseline_worst_draft_regression_delta_frames = baseline_draft_delta;
                self.baseline_worst_draft_regression_case_id = audio.case_id.clone();
                self.baseline_worst_draft_regression_source = audio.source_path.clone();
                self.baseline_worst_draft_regression_ratio = ratio;
                self.baseline_worst_draft_smear_frames = draft.max_smear_frames;
                self.baseline_worst_current_smear_frames = offline.max_smear_frames;
                self.baseline_worst_candidate_smear_frames = candidate.max_smear_frames;
            }
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "{} rows={} candidate_path={} ratio_scope={} baseline_path=offline_hq candidate_better_rows={} current_better_rows={} unchanged_rows={} inconclusive_rows={} finite_rows={} mean_candidate_smear_frames={:.6} mean_current_smear_frames={:.6} best_candidate_improvement_delta_frames={:.6} best_candidate_improvement_case={} best_candidate_improvement_source={} best_candidate_improvement_ratio={:.6} worst_candidate_regression_delta_frames={:.6} worst_candidate_regression_case={} worst_candidate_regression_source={} worst_candidate_regression_ratio={:.6} worst_draft_regression_delta_frames={:.6} worst_draft_regression_case={} worst_draft_regression_source={} worst_draft_regression_ratio={:.6} baseline_worst_draft_regression_delta_frames={:.6} baseline_worst_draft_regression_case={} baseline_worst_draft_regression_source={} baseline_worst_draft_regression_ratio={:.6} baseline_worst_draft_smear_frames={:.6} baseline_worst_current_smear_frames={:.6} baseline_worst_candidate_smear_frames={:.6}",
            self.report_name,
            self.rows,
            self.candidate_path,
            self.ratio_scope.label(),
            self.candidate_better_rows,
            self.current_better_rows,
            self.unchanged_rows,
            self.inconclusive_rows,
            self.finite_rows,
            finite_ratio(self.candidate_smear_sum, self.finite_rows as f64),
            finite_ratio(self.offline_smear_sum, self.finite_rows as f64),
            self.best_candidate_improvement_delta_frames,
            self.best_candidate_improvement_case_id,
            quoted_report_field(&self.best_candidate_improvement_source),
            self.best_candidate_improvement_ratio,
            self.worst_candidate_regression_delta_frames,
            self.worst_candidate_regression_case_id,
            quoted_report_field(&self.worst_candidate_regression_source),
            self.worst_candidate_regression_ratio,
            self.worst_draft_regression_delta_frames,
            self.worst_draft_regression_case_id,
            quoted_report_field(&self.worst_draft_regression_source),
            self.worst_draft_regression_ratio,
            self.baseline_worst_draft_regression_delta_frames,
            self.baseline_worst_draft_regression_case_id,
            quoted_report_field(&self.baseline_worst_draft_regression_source),
            self.baseline_worst_draft_regression_ratio,
            self.baseline_worst_draft_smear_frames,
            self.baseline_worst_current_smear_frames,
            self.baseline_worst_candidate_smear_frames,
        )
    }

    fn format_feature_lines(&self) -> Vec<String> {
        self.feature_rows
            .iter()
            .map(CompressionReviewFeatureRow::format_report_line)
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ShortWindowSelectorCandidateAccumulator {
    rows: usize,
    accepted_rows: usize,
    rejected_rows: usize,
    accepted_by_missed_transients_rows: usize,
    accepted_by_current_smear_rows: usize,
    accepted_by_both_rows: usize,
    accepted_candidate_better_rows: usize,
    accepted_current_better_rows: usize,
    rejected_candidate_better_rows: usize,
    rejected_current_better_rows: usize,
    gated_better_rows: usize,
    current_better_rows: usize,
    unchanged_rows: usize,
    inconclusive_rows: usize,
    finite_rows: usize,
    offline_smear_sum: f64,
    gated_smear_sum: f64,
    best_gated_improvement_delta_frames: f64,
    best_gated_improvement_case_id: String,
    best_gated_improvement_source: String,
    best_gated_improvement_ratio: f64,
    worst_gated_regression_delta_frames: f64,
    worst_gated_regression_case_id: String,
    worst_gated_regression_source: String,
    worst_gated_regression_ratio: f64,
    worst_draft_regression_delta_frames: f64,
    worst_draft_regression_case_id: String,
    worst_draft_regression_source: String,
    worst_draft_regression_ratio: f64,
    rejected_candidate_improvement_delta_frames: f64,
    accepted_candidate_regression_delta_frames: f64,
}

impl ShortWindowSelectorCandidateAccumulator {
    fn record(
        &mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        draft: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        offline: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        short_window: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) {
        if ratio >= 1.0 {
            return;
        }

        self.rows += 1;
        let accepts_missed_transients =
            offline.missed_transients >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES;
        let accepts_current_smear =
            offline.max_smear_frames >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES;
        let accepted = accepts_missed_transients || accepts_current_smear;
        if accepted {
            self.accepted_rows += 1;
            if accepts_missed_transients {
                self.accepted_by_missed_transients_rows += 1;
            }
            if accepts_current_smear {
                self.accepted_by_current_smear_rows += 1;
            }
            if accepts_missed_transients && accepts_current_smear {
                self.accepted_by_both_rows += 1;
            }
        } else {
            self.rejected_rows += 1;
        }

        match compare_metric_values(short_window.max_smear_frames, offline.max_smear_frames) {
            MetricComparison::Improved => {
                if accepted {
                    self.accepted_candidate_better_rows += 1;
                } else {
                    self.rejected_candidate_better_rows += 1;
                    if offline.max_smear_frames.is_finite()
                        && short_window.max_smear_frames.is_finite()
                    {
                        self.rejected_candidate_improvement_delta_frames +=
                            offline.max_smear_frames - short_window.max_smear_frames;
                    }
                }
            }
            MetricComparison::Same => {}
            MetricComparison::Worsened => {
                if accepted {
                    self.accepted_current_better_rows += 1;
                    if offline.max_smear_frames.is_finite()
                        && short_window.max_smear_frames.is_finite()
                    {
                        self.accepted_candidate_regression_delta_frames +=
                            short_window.max_smear_frames - offline.max_smear_frames;
                    }
                } else {
                    self.rejected_current_better_rows += 1;
                }
            }
        }

        let gated = if accepted { short_window } else { offline };
        match compare_metric_values(gated.max_smear_frames, offline.max_smear_frames) {
            MetricComparison::Improved => self.gated_better_rows += 1,
            MetricComparison::Same => {
                if gated.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
                    self.unchanged_rows += 1;
                } else {
                    self.inconclusive_rows += 1;
                }
            }
            MetricComparison::Worsened => self.current_better_rows += 1,
        }

        if offline.max_smear_frames.is_finite() && gated.max_smear_frames.is_finite() {
            self.finite_rows += 1;
            self.offline_smear_sum += offline.max_smear_frames;
            self.gated_smear_sum += gated.max_smear_frames;
            let gated_delta = gated.max_smear_frames - offline.max_smear_frames;
            let improvement_delta = offline.max_smear_frames - gated.max_smear_frames;
            if improvement_delta > self.best_gated_improvement_delta_frames {
                self.best_gated_improvement_delta_frames = improvement_delta;
                self.best_gated_improvement_case_id = audio.case_id.clone();
                self.best_gated_improvement_source = audio.source_path.clone();
                self.best_gated_improvement_ratio = ratio;
            }
            if gated_delta > self.worst_gated_regression_delta_frames {
                self.worst_gated_regression_delta_frames = gated_delta;
                self.worst_gated_regression_case_id = audio.case_id.clone();
                self.worst_gated_regression_source = audio.source_path.clone();
                self.worst_gated_regression_ratio = ratio;
            }
        }

        if draft.max_smear_frames.is_finite() && gated.max_smear_frames.is_finite() {
            let draft_delta = gated.max_smear_frames - draft.max_smear_frames;
            if draft_delta > self.worst_draft_regression_delta_frames {
                self.worst_draft_regression_delta_frames = draft_delta;
                self.worst_draft_regression_case_id = audio.case_id.clone();
                self.worst_draft_regression_source = audio.source_path.clone();
                self.worst_draft_regression_ratio = ratio;
            }
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_compression_short_window_selector_candidate rows={} candidate_path=offline_hq_short_window_selector selected_path=offline_hq_or_short_window baseline_path=offline_hq gate=CurrentMissesOrHighCurrentSmear min_current_misses={} min_current_smear_frames={:.6} accepted_rows={} rejected_rows={} accepted_by_missed_transients_rows={} accepted_by_current_smear_rows={} accepted_by_both_rows={} accepted_candidate_better_rows={} accepted_current_better_rows={} rejected_candidate_better_rows={} rejected_current_better_rows={} gated_better_rows={} current_better_rows={} unchanged_rows={} inconclusive_rows={} finite_rows={} mean_gated_smear_frames={:.6} mean_current_smear_frames={:.6} best_gated_improvement_delta_frames={:.6} best_gated_improvement_case={} best_gated_improvement_source={} best_gated_improvement_ratio={:.6} worst_gated_regression_delta_frames={:.6} worst_gated_regression_case={} worst_gated_regression_source={} worst_gated_regression_ratio={:.6} worst_draft_regression_delta_frames={:.6} worst_draft_regression_case={} worst_draft_regression_source={} worst_draft_regression_ratio={:.6} rejected_candidate_improvement_delta_frames={:.6} accepted_candidate_regression_delta_frames={:.6}",
            self.rows,
            COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
            COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES,
            self.accepted_rows,
            self.rejected_rows,
            self.accepted_by_missed_transients_rows,
            self.accepted_by_current_smear_rows,
            self.accepted_by_both_rows,
            self.accepted_candidate_better_rows,
            self.accepted_current_better_rows,
            self.rejected_candidate_better_rows,
            self.rejected_current_better_rows,
            self.gated_better_rows,
            self.current_better_rows,
            self.unchanged_rows,
            self.inconclusive_rows,
            self.finite_rows,
            finite_ratio(self.gated_smear_sum, self.finite_rows as f64),
            finite_ratio(self.offline_smear_sum, self.finite_rows as f64),
            self.best_gated_improvement_delta_frames,
            self.best_gated_improvement_case_id,
            quoted_report_field(&self.best_gated_improvement_source),
            self.best_gated_improvement_ratio,
            self.worst_gated_regression_delta_frames,
            self.worst_gated_regression_case_id,
            quoted_report_field(&self.worst_gated_regression_source),
            self.worst_gated_regression_ratio,
            self.worst_draft_regression_delta_frames,
            self.worst_draft_regression_case_id,
            quoted_report_field(&self.worst_draft_regression_source),
            self.worst_draft_regression_ratio,
            self.rejected_candidate_improvement_delta_frames,
            self.accepted_candidate_regression_delta_frames,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ExpansionShortWindowSelectorCandidateAccumulator {
    rows: usize,
    accepted_rows: usize,
    rejected_rows: usize,
    accepted_by_missed_transients_rows: usize,
    accepted_by_draft_regression_rows: usize,
    accepted_by_both_rows: usize,
    accepted_candidate_better_rows: usize,
    accepted_current_better_rows: usize,
    rejected_candidate_better_rows: usize,
    rejected_current_better_rows: usize,
    gated_better_rows: usize,
    current_better_rows: usize,
    unchanged_rows: usize,
    inconclusive_rows: usize,
    finite_rows: usize,
    offline_smear_sum: f64,
    gated_smear_sum: f64,
    best_gated_improvement_delta_frames: f64,
    best_gated_improvement_case_id: String,
    best_gated_improvement_source: String,
    best_gated_improvement_ratio: f64,
    worst_gated_regression_delta_frames: f64,
    worst_gated_regression_case_id: String,
    worst_gated_regression_source: String,
    worst_gated_regression_ratio: f64,
    rejected_candidate_improvement_delta_frames: f64,
    accepted_candidate_regression_delta_frames: f64,
}

impl ExpansionShortWindowSelectorCandidateAccumulator {
    fn record(
        &mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        draft: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        offline: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        short_window: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) {
        if ratio <= 1.0 {
            return;
        }

        self.rows += 1;
        let accepts_missed_transients =
            offline.missed_transients >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES;
        let accepts_draft_regression =
            compare_metric_values(offline.max_smear_frames, draft.max_smear_frames)
                == MetricComparison::Worsened;
        let accepted = accepts_missed_transients || accepts_draft_regression;
        if accepted {
            self.accepted_rows += 1;
            if accepts_missed_transients {
                self.accepted_by_missed_transients_rows += 1;
            }
            if accepts_draft_regression {
                self.accepted_by_draft_regression_rows += 1;
            }
            if accepts_missed_transients && accepts_draft_regression {
                self.accepted_by_both_rows += 1;
            }
        } else {
            self.rejected_rows += 1;
        }

        match compare_metric_values(short_window.max_smear_frames, offline.max_smear_frames) {
            MetricComparison::Improved => {
                if accepted {
                    self.accepted_candidate_better_rows += 1;
                } else {
                    self.rejected_candidate_better_rows += 1;
                    if offline.max_smear_frames.is_finite()
                        && short_window.max_smear_frames.is_finite()
                    {
                        self.rejected_candidate_improvement_delta_frames +=
                            offline.max_smear_frames - short_window.max_smear_frames;
                    }
                }
            }
            MetricComparison::Same => {}
            MetricComparison::Worsened => {
                if accepted {
                    self.accepted_current_better_rows += 1;
                    if offline.max_smear_frames.is_finite()
                        && short_window.max_smear_frames.is_finite()
                    {
                        self.accepted_candidate_regression_delta_frames +=
                            short_window.max_smear_frames - offline.max_smear_frames;
                    }
                } else {
                    self.rejected_current_better_rows += 1;
                }
            }
        }

        let gated = if accepted { short_window } else { offline };
        match compare_metric_values(gated.max_smear_frames, offline.max_smear_frames) {
            MetricComparison::Improved => self.gated_better_rows += 1,
            MetricComparison::Same => {
                if gated.max_smear_frames.is_finite() && offline.max_smear_frames.is_finite() {
                    self.unchanged_rows += 1;
                } else {
                    self.inconclusive_rows += 1;
                }
            }
            MetricComparison::Worsened => self.current_better_rows += 1,
        }

        if offline.max_smear_frames.is_finite() && gated.max_smear_frames.is_finite() {
            self.finite_rows += 1;
            self.offline_smear_sum += offline.max_smear_frames;
            self.gated_smear_sum += gated.max_smear_frames;
            let gated_delta = gated.max_smear_frames - offline.max_smear_frames;
            let improvement_delta = offline.max_smear_frames - gated.max_smear_frames;
            if improvement_delta > self.best_gated_improvement_delta_frames {
                self.best_gated_improvement_delta_frames = improvement_delta;
                self.best_gated_improvement_case_id = audio.case_id.clone();
                self.best_gated_improvement_source = audio.source_path.clone();
                self.best_gated_improvement_ratio = ratio;
            }
            if gated_delta > self.worst_gated_regression_delta_frames {
                self.worst_gated_regression_delta_frames = gated_delta;
                self.worst_gated_regression_case_id = audio.case_id.clone();
                self.worst_gated_regression_source = audio.source_path.clone();
                self.worst_gated_regression_ratio = ratio;
            }
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_expansion_short_window_selector_candidate rows={} candidate_path=offline_hq_short_window_selector selected_path=offline_hq_or_short_window baseline_path=offline_hq gate=CurrentMissesOrDraftRegression min_current_misses={} accepted_rows={} rejected_rows={} accepted_by_missed_transients_rows={} accepted_by_draft_regression_rows={} accepted_by_both_rows={} accepted_candidate_better_rows={} accepted_current_better_rows={} rejected_candidate_better_rows={} rejected_current_better_rows={} gated_better_rows={} current_better_rows={} unchanged_rows={} inconclusive_rows={} finite_rows={} mean_gated_smear_frames={:.6} mean_current_smear_frames={:.6} best_gated_improvement_delta_frames={:.6} best_gated_improvement_case={} best_gated_improvement_source={} best_gated_improvement_ratio={:.6} worst_gated_regression_delta_frames={:.6} worst_gated_regression_case={} worst_gated_regression_source={} worst_gated_regression_ratio={:.6} rejected_candidate_improvement_delta_frames={:.6} accepted_candidate_regression_delta_frames={:.6}",
            self.rows,
            COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
            self.accepted_rows,
            self.rejected_rows,
            self.accepted_by_missed_transients_rows,
            self.accepted_by_draft_regression_rows,
            self.accepted_by_both_rows,
            self.accepted_candidate_better_rows,
            self.accepted_current_better_rows,
            self.rejected_candidate_better_rows,
            self.rejected_current_better_rows,
            self.gated_better_rows,
            self.current_better_rows,
            self.unchanged_rows,
            self.inconclusive_rows,
            self.finite_rows,
            finite_ratio(self.gated_smear_sum, self.finite_rows as f64),
            finite_ratio(self.offline_smear_sum, self.finite_rows as f64),
            self.best_gated_improvement_delta_frames,
            self.best_gated_improvement_case_id,
            quoted_report_field(&self.best_gated_improvement_source),
            self.best_gated_improvement_ratio,
            self.worst_gated_regression_delta_frames,
            self.worst_gated_regression_case_id,
            quoted_report_field(&self.worst_gated_regression_source),
            self.worst_gated_regression_ratio,
            self.rejected_candidate_improvement_delta_frames,
            self.accepted_candidate_regression_delta_frames,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ShortWindowSelectorPathAccumulator {
    rows: usize,
    selected_short_window_rows: usize,
    selected_default_rows: usize,
    output_match_rows: usize,
    output_mismatch_rows: usize,
    smear_match_rows: usize,
    smear_mismatch_rows: usize,
    max_abs_smear_delta_frames: f64,
}

impl ShortWindowSelectorPathAccumulator {
    fn record(
        &mut self,
        ratio: f64,
        offline_output: &[f32],
        short_window_output: &[f32],
        selector_output: &[f32],
        offline_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        short_window_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        selector_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) {
        if ratio >= 1.0 {
            return;
        }

        self.rows += 1;
        let accepted = offline_smear.missed_transients
            >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES
            || offline_smear.max_smear_frames
                >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES;
        let (expected_output, expected_smear) = if accepted {
            self.selected_short_window_rows += 1;
            (short_window_output, short_window_smear)
        } else {
            self.selected_default_rows += 1;
            (offline_output, offline_smear)
        };

        if selector_output == expected_output {
            self.output_match_rows += 1;
        } else {
            self.output_mismatch_rows += 1;
        }

        let smear_delta = (selector_smear.max_smear_frames - expected_smear.max_smear_frames).abs();
        if smear_delta <= 1.0e-9
            || (!smear_delta.is_finite() && !expected_smear.max_smear_frames.is_finite())
        {
            self.smear_match_rows += 1;
        } else {
            self.smear_mismatch_rows += 1;
            if smear_delta.is_finite() && smear_delta > self.max_abs_smear_delta_frames {
                self.max_abs_smear_delta_frames = smear_delta;
            }
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_compression_short_window_selector_path rows={} path=offline_hq_compression_short_window_selector baseline_path=offline_hq short_window_path=offline_hq_short_window gate=CurrentMissesOrHighCurrentSmear min_current_misses={} min_current_smear_frames={:.6} selected_short_window_rows={} selected_default_rows={} output_match_rows={} output_mismatch_rows={} smear_match_rows={} smear_mismatch_rows={} max_abs_smear_delta_frames={:.6}",
            self.rows,
            COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
            COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES,
            self.selected_short_window_rows,
            self.selected_default_rows,
            self.output_match_rows,
            self.output_mismatch_rows,
            self.smear_match_rows,
            self.smear_mismatch_rows,
            self.max_abs_smear_delta_frames,
        )
    }
}

#[derive(Default)]
struct MatchedTransientWidthReviewAccumulator {
    rows: usize,
    finite_rows: usize,
    offline_worse_than_draft_rows: usize,
    offline_better_than_draft_rows: usize,
    offline_same_as_draft_rows: usize,
    selector_worse_than_draft_rows: usize,
    selector_better_than_draft_rows: usize,
    selector_same_as_draft_rows: usize,
    selector_better_than_offline_rows: usize,
    selector_worse_than_offline_rows: usize,
    selector_same_as_offline_rows: usize,
    max_offline_vs_draft_delta_frames: f64,
    max_offline_vs_draft_case_id: String,
    max_offline_vs_draft_source: String,
    max_offline_vs_draft_ratio: f64,
    max_selector_vs_draft_delta_frames: f64,
    max_selector_vs_draft_case_id: String,
    max_selector_vs_draft_source: String,
    max_selector_vs_draft_ratio: f64,
    max_selector_residual_smear_frames: f64,
    max_selector_residual_case_id: String,
    max_selector_residual_source: String,
    max_selector_residual_ratio: f64,
    max_selector_residual_input_width_frames: f64,
    max_selector_residual_output_width_frames: f64,
    max_short_window_residual_smear_frames: f64,
}

impl MatchedTransientWidthReviewAccumulator {
    fn record(
        &mut self,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        draft: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        offline: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        short_window: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        selector: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) {
        self.rows += 1;
        let draft_smear = draft.max_matched_smear_frames;
        let offline_smear = offline.max_matched_smear_frames;
        let short_window_smear = short_window.max_matched_smear_frames;
        let selector_smear = selector.max_matched_smear_frames;
        if !draft_smear.is_finite()
            || !offline_smear.is_finite()
            || !short_window_smear.is_finite()
            || !selector_smear.is_finite()
        {
            return;
        }

        self.finite_rows += 1;
        match compare_metric_values(offline_smear, draft_smear) {
            MetricComparison::Improved => self.offline_better_than_draft_rows += 1,
            MetricComparison::Same => self.offline_same_as_draft_rows += 1,
            MetricComparison::Worsened => self.offline_worse_than_draft_rows += 1,
        }
        match compare_metric_values(selector_smear, draft_smear) {
            MetricComparison::Improved => self.selector_better_than_draft_rows += 1,
            MetricComparison::Same => self.selector_same_as_draft_rows += 1,
            MetricComparison::Worsened => self.selector_worse_than_draft_rows += 1,
        }
        match compare_metric_values(selector_smear, offline_smear) {
            MetricComparison::Improved => self.selector_better_than_offline_rows += 1,
            MetricComparison::Same => self.selector_same_as_offline_rows += 1,
            MetricComparison::Worsened => self.selector_worse_than_offline_rows += 1,
        }

        let offline_delta = offline_smear - draft_smear;
        if offline_delta > self.max_offline_vs_draft_delta_frames {
            self.max_offline_vs_draft_delta_frames = offline_delta;
            self.max_offline_vs_draft_case_id = audio.case_id.clone();
            self.max_offline_vs_draft_source = audio.source_path.clone();
            self.max_offline_vs_draft_ratio = ratio;
        }

        let selector_delta = selector_smear - draft_smear;
        if selector_delta > self.max_selector_vs_draft_delta_frames {
            self.max_selector_vs_draft_delta_frames = selector_delta;
            self.max_selector_vs_draft_case_id = audio.case_id.clone();
            self.max_selector_vs_draft_source = audio.source_path.clone();
            self.max_selector_vs_draft_ratio = ratio;
        }

        if selector_smear > self.max_selector_residual_smear_frames {
            self.max_selector_residual_smear_frames = selector_smear;
            self.max_selector_residual_case_id = audio.case_id.clone();
            self.max_selector_residual_source = audio.source_path.clone();
            self.max_selector_residual_ratio = ratio;
            self.max_selector_residual_input_width_frames = selector.max_matched_input_width_frames;
            self.max_selector_residual_output_width_frames =
                selector.max_matched_output_width_frames;
        }
        if short_window_smear > self.max_short_window_residual_smear_frames {
            self.max_short_window_residual_smear_frames = short_window_smear;
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_matched_transient_width_review rows={} finite_rows={} baseline_path=offline_hq selector_path=offline_hq_compression_short_window_selector metric=max_matched_smear_frames offline_worse_than_draft_rows={} offline_better_than_draft_rows={} offline_same_as_draft_rows={} selector_worse_than_draft_rows={} selector_better_than_draft_rows={} selector_same_as_draft_rows={} selector_better_than_offline_rows={} selector_worse_than_offline_rows={} selector_same_as_offline_rows={} max_offline_vs_draft_delta_frames={:.6} max_offline_vs_draft_case={} max_offline_vs_draft_source={} max_offline_vs_draft_ratio={:.6} max_selector_vs_draft_delta_frames={:.6} max_selector_vs_draft_case={} max_selector_vs_draft_source={} max_selector_vs_draft_ratio={:.6} max_selector_residual_smear_frames={:.6} max_selector_residual_case={} max_selector_residual_source={} max_selector_residual_ratio={:.6} max_selector_residual_input_width_frames={:.6} max_selector_residual_output_width_frames={:.6} max_short_window_residual_smear_frames={:.6}",
            self.rows,
            self.finite_rows,
            self.offline_worse_than_draft_rows,
            self.offline_better_than_draft_rows,
            self.offline_same_as_draft_rows,
            self.selector_worse_than_draft_rows,
            self.selector_better_than_draft_rows,
            self.selector_same_as_draft_rows,
            self.selector_better_than_offline_rows,
            self.selector_worse_than_offline_rows,
            self.selector_same_as_offline_rows,
            self.max_offline_vs_draft_delta_frames,
            self.max_offline_vs_draft_case_id,
            quoted_report_field(&self.max_offline_vs_draft_source),
            self.max_offline_vs_draft_ratio,
            self.max_selector_vs_draft_delta_frames,
            self.max_selector_vs_draft_case_id,
            quoted_report_field(&self.max_selector_vs_draft_source),
            self.max_selector_vs_draft_ratio,
            self.max_selector_residual_smear_frames,
            self.max_selector_residual_case_id,
            quoted_report_field(&self.max_selector_residual_source),
            self.max_selector_residual_ratio,
            self.max_selector_residual_input_width_frames,
            self.max_selector_residual_output_width_frames,
            self.max_short_window_residual_smear_frames,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CompressionReviewFeatureRow {
    report_name: &'static str,
    candidate_path: &'static str,
    outcome: &'static str,
    case_id: String,
    source_path: String,
    ratio: f64,
    draft_smear_frames: f64,
    current_smear_frames: f64,
    candidate_smear_frames: f64,
    candidate_delta_frames: f64,
    current_vs_draft_delta_frames: f64,
    candidate_vs_draft_delta_frames: f64,
    input_transients: usize,
    current_output_transients: usize,
    current_matched_transients: usize,
    current_missed_transients: usize,
    candidate_output_transients: usize,
    candidate_matched_transients: usize,
    candidate_missed_transients: usize,
    current_output_input_transient_ratio: f64,
    candidate_output_input_transient_ratio: f64,
    current_max_matched_input_width_frames: f64,
    current_max_matched_output_width_frames: f64,
    candidate_max_matched_input_width_frames: f64,
    candidate_max_matched_output_width_frames: f64,
}

impl CompressionReviewFeatureRow {
    fn from_measurements(
        report_name: &'static str,
        candidate_path: &'static str,
        audio: &DecodedListeningSourceAudio,
        ratio: f64,
        comparison: MetricComparison,
        draft: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        current: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        candidate: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) -> Self {
        Self {
            report_name,
            candidate_path,
            outcome: match comparison {
                MetricComparison::Improved => "CandidateBetter",
                MetricComparison::Worsened => "CurrentBetter",
                MetricComparison::Same => "Unchanged",
            },
            case_id: audio.case_id.clone(),
            source_path: audio.source_path.clone(),
            ratio,
            draft_smear_frames: draft.max_smear_frames,
            current_smear_frames: current.max_smear_frames,
            candidate_smear_frames: candidate.max_smear_frames,
            candidate_delta_frames: candidate.max_smear_frames - current.max_smear_frames,
            current_vs_draft_delta_frames: current.max_smear_frames - draft.max_smear_frames,
            candidate_vs_draft_delta_frames: candidate.max_smear_frames - draft.max_smear_frames,
            input_transients: current.input_transients,
            current_output_transients: current.output_transients,
            current_matched_transients: current.matched_transients,
            current_missed_transients: current.missed_transients,
            candidate_output_transients: candidate.output_transients,
            candidate_matched_transients: candidate.matched_transients,
            candidate_missed_transients: candidate.missed_transients,
            current_output_input_transient_ratio: finite_ratio(
                current.output_transients as f64,
                current.input_transients as f64,
            ),
            candidate_output_input_transient_ratio: finite_ratio(
                candidate.output_transients as f64,
                candidate.input_transients as f64,
            ),
            current_max_matched_input_width_frames: current.max_matched_input_width_frames,
            current_max_matched_output_width_frames: current.max_matched_output_width_frames,
            candidate_max_matched_input_width_frames: candidate.max_matched_input_width_frames,
            candidate_max_matched_output_width_frames: candidate.max_matched_output_width_frames,
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "{} candidate_path={} outcome={} case={} source={} ratio={:.6} draft_smear_frames={:.6} current_smear_frames={:.6} candidate_smear_frames={:.6} candidate_delta_frames={:.6} current_vs_draft_delta_frames={:.6} candidate_vs_draft_delta_frames={:.6} input_transients={} current_output_transients={} current_matched_transients={} current_missed_transients={} candidate_output_transients={} candidate_matched_transients={} candidate_missed_transients={} current_output_input_transient_ratio={:.6} candidate_output_input_transient_ratio={:.6} current_max_matched_input_width_frames={:.6} current_max_matched_output_width_frames={:.6} candidate_max_matched_input_width_frames={:.6} candidate_max_matched_output_width_frames={:.6}",
            self.report_name,
            self.candidate_path,
            self.outcome,
            self.case_id,
            quoted_report_field(&self.source_path),
            self.ratio,
            self.draft_smear_frames,
            self.current_smear_frames,
            self.candidate_smear_frames,
            self.candidate_delta_frames,
            self.current_vs_draft_delta_frames,
            self.candidate_vs_draft_delta_frames,
            self.input_transients,
            self.current_output_transients,
            self.current_matched_transients,
            self.current_missed_transients,
            self.candidate_output_transients,
            self.candidate_matched_transients,
            self.candidate_missed_transients,
            self.current_output_input_transient_ratio,
            self.candidate_output_input_transient_ratio,
            self.current_max_matched_input_width_frames,
            self.current_max_matched_output_width_frames,
            self.candidate_max_matched_input_width_frames,
            self.candidate_max_matched_output_width_frames,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TransientRecoveryGateAccumulator {
    backend: &'static str,
    rows: usize,
    production_input_transients: usize,
    production_matched_transients: usize,
    production_missed_transients: usize,
    full_candidate_input_transients: usize,
    full_candidate_matched_transients: usize,
    full_candidate_missed_transients: usize,
    recovery_matched_transients: usize,
    recovery_missed_transients: usize,
    recovery_missed_rows_improved: usize,
    recovery_missed_rows_same: usize,
    recovery_missed_rows_worsened: usize,
    recovery_max_rows_improved: usize,
    recovery_max_rows_same: usize,
    recovery_max_rows_worsened: usize,
}

impl TransientRecoveryGateAccumulator {
    fn new(backend: &'static str) -> Self {
        Self {
            backend,
            rows: 0,
            production_input_transients: 0,
            production_matched_transients: 0,
            production_missed_transients: 0,
            full_candidate_input_transients: 0,
            full_candidate_matched_transients: 0,
            full_candidate_missed_transients: 0,
            recovery_matched_transients: 0,
            recovery_missed_transients: 0,
            recovery_missed_rows_improved: 0,
            recovery_missed_rows_same: 0,
            recovery_missed_rows_worsened: 0,
            recovery_max_rows_improved: 0,
            recovery_max_rows_same: 0,
            recovery_max_rows_worsened: 0,
        }
    }

    fn record(
        &mut self,
        production: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        full_candidate: &signal_dsp_stretch::StretchTransientSmearMeasurement,
        recovery: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    ) {
        self.rows += 1;
        self.production_input_transients += production.input_transients;
        self.production_matched_transients += production.matched_transients;
        self.production_missed_transients += production.missed_transients;
        self.full_candidate_input_transients += full_candidate.input_transients;
        self.full_candidate_matched_transients += full_candidate.matched_transients;
        self.full_candidate_missed_transients += full_candidate.missed_transients;
        self.recovery_matched_transients += recovery.matched_transients;
        self.recovery_missed_transients += recovery.missed_transients;

        if recovery.missed_transients < production.missed_transients {
            self.recovery_missed_rows_improved += 1;
        } else if recovery.missed_transients > production.missed_transients {
            self.recovery_missed_rows_worsened += 1;
        } else {
            self.recovery_missed_rows_same += 1;
        }

        match compare_metric_values(recovery.max_smear_frames, production.max_smear_frames) {
            MetricComparison::Improved => self.recovery_max_rows_improved += 1,
            MetricComparison::Same => self.recovery_max_rows_same += 1,
            MetricComparison::Worsened => self.recovery_max_rows_worsened += 1,
        }
    }

    fn recovered_misses(&self) -> usize {
        self.production_missed_transients
            .saturating_sub(self.recovery_missed_transients)
    }

    fn full_candidate_input_ratio(&self) -> f64 {
        finite_ratio(
            self.full_candidate_input_transients as f64,
            self.production_input_transients as f64,
        )
    }

    fn target_status(&self) -> &'static str {
        if self.recovered_misses() >= RECOVERY_GATE_MIN_RECOVERED_MISSES
            && self.recovery_missed_rows_worsened <= RECOVERY_GATE_MAX_MISSED_WORSENED_ROWS
            && self.recovery_max_rows_worsened <= RECOVERY_GATE_MAX_SMEAR_WORSENED_ROWS
        {
            "Pass"
        } else {
            "Fail"
        }
    }

    fn global_threshold_status(&self) -> &'static str {
        let ratio = self.full_candidate_input_ratio();
        if ratio.is_finite() && ratio <= RECOVERY_GATE_MAX_GLOBAL_CANDIDATE_INPUT_RATIO {
            "Pass"
        } else {
            "Rejected"
        }
    }

    fn recommendation(&self) -> &'static str {
        match (self.target_status(), self.global_threshold_status()) {
            ("Pass", "Rejected") => "TargetedOutputRecovery",
            ("Pass", "Pass") => "ReviewGlobalThreshold",
            _ => "KeepReportOnly",
        }
    }

    fn format_report_line(&self) -> String {
        format!(
            "decoded_transient_recovery_gate backend={} target_status={} global_threshold_status={} recommendation={} rows={} production_input_transients={} production_matched_transients={} production_missed_transients={} recovery_matched_transients={} recovery_missed_transients={} recovered_misses={} recovery_missed_rows_improved={} recovery_missed_rows_same={} recovery_missed_rows_worsened={} recovery_max_rows_improved={} recovery_max_rows_same={} recovery_max_rows_worsened={} min_recovered_misses={} max_missed_worsened_rows={} max_smear_worsened_rows={} full_candidate_input_transients={} full_candidate_matched_transients={} full_candidate_missed_transients={} full_candidate_input_ratio={:.6} max_global_candidate_input_ratio={:.6}",
            self.backend,
            self.target_status(),
            self.global_threshold_status(),
            self.recommendation(),
            self.rows,
            self.production_input_transients,
            self.production_matched_transients,
            self.production_missed_transients,
            self.recovery_matched_transients,
            self.recovery_missed_transients,
            self.recovered_misses(),
            self.recovery_missed_rows_improved,
            self.recovery_missed_rows_same,
            self.recovery_missed_rows_worsened,
            self.recovery_max_rows_improved,
            self.recovery_max_rows_same,
            self.recovery_max_rows_worsened,
            RECOVERY_GATE_MIN_RECOVERED_MISSES,
            RECOVERY_GATE_MAX_MISSED_WORSENED_ROWS,
            RECOVERY_GATE_MAX_SMEAR_WORSENED_ROWS,
            self.full_candidate_input_transients,
            self.full_candidate_matched_transients,
            self.full_candidate_missed_transients,
            self.full_candidate_input_ratio(),
            RECOVERY_GATE_MAX_GLOBAL_CANDIDATE_INPUT_RATIO,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MetricComparison {
    Improved,
    Same,
    Worsened,
}

fn compare_metric_values(candidate: f64, production: f64) -> MetricComparison {
    if candidate.is_finite() && production.is_finite() {
        if candidate < production {
            MetricComparison::Improved
        } else if candidate > production {
            MetricComparison::Worsened
        } else {
            MetricComparison::Same
        }
    } else if candidate.is_finite() && !production.is_finite() {
        MetricComparison::Improved
    } else if !candidate.is_finite() && production.is_finite() {
        MetricComparison::Worsened
    } else {
        MetricComparison::Same
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TransientAlignmentDiagnostic {
    mean_match_error_frames: f64,
    max_match_error_frames: f64,
    mean_missed_nearest_distance_frames: f64,
    max_missed_nearest_distance_frames: f64,
    max_missed_expected_output_frame: f64,
    max_missed_nearest_output_frame: f64,
    missed_events: Vec<TransientAlignmentMissEvent>,
}

#[derive(Clone, Debug, PartialEq)]
struct TransientAlignmentMissEvent {
    input_frame: usize,
    expected_output_frame: f64,
    nearest_output_frame: Option<f64>,
    nearest_distance_frames: f64,
    input_window_peak: f64,
    input_window_rms: f64,
    expected_output_window_peak: f64,
    expected_output_window_rms: f64,
    nearest_output_window_peak: f64,
    nearest_output_window_rms: f64,
    expected_detector_shape: DetectorShape,
    nearest_detector_shape: DetectorShape,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WindowEnergyStats {
    peak: f64,
    rms: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DetectorShape {
    frame_index: f64,
    energy_score: f64,
    spectral_flux_score: f64,
    combined_score: f64,
    previous_combined_score: f64,
    next_combined_score: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DetectorFrameFeature {
    frame_index: usize,
    energy: f64,
    spectral_flux: f64,
}

fn transient_alignment_diagnostic(
    input: &[f32],
    output: &[f32],
    ratio: f64,
) -> TransientAlignmentDiagnostic {
    if !ratio.is_finite() || ratio <= 0.0 || input.is_empty() || output.is_empty() {
        return transient_alignment_nan();
    }

    let input_events =
        detect_stretch_transients(input, QUALITY_METRIC_WINDOW_SIZE, QUALITY_METRIC_HOP_SIZE);
    let output_events =
        detect_stretch_transients(output, QUALITY_METRIC_WINDOW_SIZE, QUALITY_METRIC_HOP_SIZE);
    let output_detector_shapes = detector_shape_trace(output);
    let tolerance = QUALITY_METRIC_WINDOW_SIZE.max(QUALITY_METRIC_HOP_SIZE * 4) as f64;
    let mut matched_count = 0usize;
    let mut matched_error_sum = 0.0f64;
    let mut matched_error_max = 0.0f64;
    let mut missed_nearest_count = 0usize;
    let mut missed_nearest_sum = 0.0f64;
    let mut missed_nearest_max = 0.0f64;
    let mut max_missed_expected_output_frame = f64::NAN;
    let mut max_missed_nearest_output_frame = f64::NAN;
    let mut missed_events = Vec::new();

    for input_event in input_events {
        let expected_output_frame = input_event.frame_index as f64 * ratio;
        let nearest = nearest_transient_position(&output_events, expected_output_frame);
        match nearest {
            Some((distance, _nearest_frame)) if distance <= tolerance => {
                matched_count += 1;
                matched_error_sum += distance;
                matched_error_max = matched_error_max.max(distance);
            }
            Some((distance, nearest_frame)) => {
                missed_nearest_count += 1;
                missed_nearest_sum += distance;
                let input_window = window_energy_stats(input, input_event.frame_index as f64);
                let expected_output_window = window_energy_stats(output, expected_output_frame);
                let nearest_output_window = window_energy_stats(output, nearest_frame);
                let expected_detector_shape =
                    nearest_detector_shape(&output_detector_shapes, expected_output_frame);
                let nearest_detector_shape =
                    nearest_detector_shape(&output_detector_shapes, nearest_frame);
                missed_events.push(TransientAlignmentMissEvent {
                    input_frame: input_event.frame_index,
                    expected_output_frame,
                    nearest_output_frame: Some(nearest_frame),
                    nearest_distance_frames: distance,
                    input_window_peak: input_window.peak,
                    input_window_rms: input_window.rms,
                    expected_output_window_peak: expected_output_window.peak,
                    expected_output_window_rms: expected_output_window.rms,
                    nearest_output_window_peak: nearest_output_window.peak,
                    nearest_output_window_rms: nearest_output_window.rms,
                    expected_detector_shape,
                    nearest_detector_shape,
                });
                if distance > missed_nearest_max {
                    missed_nearest_max = distance;
                    max_missed_expected_output_frame = expected_output_frame;
                    max_missed_nearest_output_frame = nearest_frame;
                }
            }
            None => {
                let input_window = window_energy_stats(input, input_event.frame_index as f64);
                let expected_output_window = window_energy_stats(output, expected_output_frame);
                let expected_detector_shape =
                    nearest_detector_shape(&output_detector_shapes, expected_output_frame);
                missed_events.push(TransientAlignmentMissEvent {
                    input_frame: input_event.frame_index,
                    expected_output_frame,
                    nearest_output_frame: None,
                    nearest_distance_frames: f64::NAN,
                    input_window_peak: input_window.peak,
                    input_window_rms: input_window.rms,
                    expected_output_window_peak: expected_output_window.peak,
                    expected_output_window_rms: expected_output_window.rms,
                    nearest_output_window_peak: f64::NAN,
                    nearest_output_window_rms: f64::NAN,
                    expected_detector_shape,
                    nearest_detector_shape: detector_shape_nan(),
                });
            }
        }
    }
    missed_events.sort_by(|left, right| {
        right
            .nearest_distance_frames
            .total_cmp(&left.nearest_distance_frames)
    });
    missed_events.truncate(MAX_TRANSIENT_ALIGNMENT_EVENTS_PER_BACKEND);

    TransientAlignmentDiagnostic {
        mean_match_error_frames: if matched_count > 0 {
            matched_error_sum / matched_count as f64
        } else {
            f64::NAN
        },
        max_match_error_frames: if matched_count > 0 {
            matched_error_max
        } else {
            f64::NAN
        },
        mean_missed_nearest_distance_frames: if missed_nearest_count > 0 {
            missed_nearest_sum / missed_nearest_count as f64
        } else {
            f64::NAN
        },
        max_missed_nearest_distance_frames: if missed_nearest_count > 0 {
            missed_nearest_max
        } else {
            f64::NAN
        },
        max_missed_expected_output_frame: if missed_nearest_count > 0 {
            max_missed_expected_output_frame
        } else {
            f64::NAN
        },
        max_missed_nearest_output_frame: if missed_nearest_count > 0 {
            max_missed_nearest_output_frame
        } else {
            f64::NAN
        },
        missed_events,
    }
}

fn transient_alignment_nan() -> TransientAlignmentDiagnostic {
    TransientAlignmentDiagnostic {
        mean_match_error_frames: f64::NAN,
        max_match_error_frames: f64::NAN,
        mean_missed_nearest_distance_frames: f64::NAN,
        max_missed_nearest_distance_frames: f64::NAN,
        max_missed_expected_output_frame: f64::NAN,
        max_missed_nearest_output_frame: f64::NAN,
        missed_events: Vec::new(),
    }
}

fn nearest_transient_position(
    events: &[signal_dsp_stretch::StretchTransientEvent],
    expected_frame: f64,
) -> Option<(f64, f64)> {
    events
        .iter()
        .map(|event| {
            let event_frame = event.frame_index as f64;
            ((event_frame - expected_frame).abs(), event_frame)
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
}

fn window_energy_stats(samples: &[f32], center_frame: f64) -> WindowEnergyStats {
    if samples.is_empty() || !center_frame.is_finite() || center_frame < 0.0 {
        return WindowEnergyStats {
            peak: f64::NAN,
            rms: f64::NAN,
        };
    }

    let center = center_frame.round() as usize;
    let start = center.saturating_sub(TRANSIENT_ALIGNMENT_WINDOW_RADIUS);
    let end = (center + TRANSIENT_ALIGNMENT_WINDOW_RADIUS + 1).min(samples.len());
    if start >= end {
        return WindowEnergyStats {
            peak: f64::NAN,
            rms: f64::NAN,
        };
    }

    let mut peak = 0.0f64;
    let mut square_sum = 0.0f64;
    for sample in &samples[start..end] {
        let value = *sample as f64;
        peak = peak.max(value.abs());
        square_sum += value * value;
    }

    WindowEnergyStats {
        peak,
        rms: (square_sum / (end - start) as f64).sqrt(),
    }
}

fn detector_shape_trace(samples: &[f32]) -> Vec<DetectorShape> {
    let features =
        detector_frame_features(samples, QUALITY_METRIC_WINDOW_SIZE, QUALITY_METRIC_HOP_SIZE);
    if features.len() < 3 {
        return Vec::new();
    }

    let mut energy_rises = Vec::with_capacity(features.len());
    let mut fluxes = Vec::with_capacity(features.len());
    energy_rises.push(0.0);
    fluxes.push(0.0);
    for pair in features.windows(2) {
        energy_rises.push((pair[1].energy - pair[0].energy).max(0.0));
        fluxes.push(pair[1].spectral_flux);
    }

    let energy_scale = mean_plus_stddev(&energy_rises).max(1.0e-12);
    let flux_scale = mean_plus_stddev(&fluxes).max(1.0e-12);
    let mut shapes = Vec::with_capacity(features.len().saturating_sub(2));

    for index in 1..features.len() - 1 {
        let energy_score = energy_rises[index] / energy_scale;
        let spectral_flux_score = fluxes[index] / flux_scale;
        let previous_combined_score =
            energy_rises[index - 1] / energy_scale + fluxes[index - 1] / flux_scale;
        let next_combined_score =
            energy_rises[index + 1] / energy_scale + fluxes[index + 1] / flux_scale;
        shapes.push(DetectorShape {
            frame_index: features[index].frame_index as f64,
            energy_score,
            spectral_flux_score,
            combined_score: energy_score + spectral_flux_score,
            previous_combined_score,
            next_combined_score,
        });
    }

    shapes
}

fn detector_frame_features(
    samples: &[f32],
    window_size: usize,
    hop_size: usize,
) -> Vec<DetectorFrameFeature> {
    if samples.len() < window_size || window_size < 16 || hop_size == 0 {
        return Vec::new();
    }

    let bins = window_size / 2 + 1;
    let window: Vec<f32> = (0..window_size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos())
        .collect();
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(window_size);
    let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];
    let mut previous_magnitudes = vec![0.0f32; bins];
    let mut magnitudes = vec![0.0f32; bins];
    let mut features = Vec::new();

    for start in (0..=samples.len() - window_size).step_by(hop_size) {
        let mut energy = 0.0f64;
        for (slot, (sample, weight)) in buffer.iter_mut().zip(
            samples[start..start + window_size]
                .iter()
                .zip(window.iter()),
        ) {
            let windowed = sample * weight;
            energy += (windowed * windowed) as f64;
            *slot = Complex32::new(windowed, 0.0);
        }
        forward.process(&mut buffer);

        let mut flux = 0.0f64;
        for bin in 0..bins {
            let magnitude = buffer[bin].norm();
            magnitudes[bin] = magnitude;
            flux += (magnitude - previous_magnitudes[bin]).max(0.0) as f64;
        }
        previous_magnitudes.copy_from_slice(&magnitudes);

        features.push(DetectorFrameFeature {
            frame_index: start,
            energy: energy / window_size as f64,
            spectral_flux: flux / bins as f64,
        });
    }

    features
}

fn mean_plus_stddev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    mean + variance.sqrt()
}

fn nearest_detector_shape(shapes: &[DetectorShape], expected_frame: f64) -> DetectorShape {
    if !expected_frame.is_finite() {
        return detector_shape_nan();
    }
    shapes
        .iter()
        .copied()
        .min_by(|left, right| {
            (left.frame_index - expected_frame)
                .abs()
                .total_cmp(&(right.frame_index - expected_frame).abs())
        })
        .unwrap_or_else(detector_shape_nan)
}

fn detector_shape_nan() -> DetectorShape {
    DetectorShape {
        frame_index: f64::NAN,
        energy_score: f64::NAN,
        spectral_flux_score: f64::NAN,
        combined_score: f64::NAN,
        previous_combined_score: f64::NAN,
        next_combined_score: f64::NAN,
    }
}

fn format_transient_metric_detail(
    draft_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    offline_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    draft_strict_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    offline_strict_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    draft_candidate_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    offline_candidate_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    draft_candidate_output_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    offline_candidate_output_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    draft_candidate_recovery_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    offline_candidate_recovery_smear: &signal_dsp_stretch::StretchTransientSmearMeasurement,
    draft_alignment: &TransientAlignmentDiagnostic,
    offline_alignment: &TransientAlignmentDiagnostic,
) -> String {
    format!(
        "draft_input_transients={} draft_output_transients={} draft_matched_transients={} draft_missed_transients={} draft_max_matched_smear_frames={:.6} draft_max_matched_input_frame={:.6} draft_max_matched_output_frame={:.6} draft_max_matched_input_width_frames={:.6} draft_max_matched_output_width_frames={:.6} draft_strict_input_transients={} draft_strict_output_transients={} draft_strict_matched_transients={} draft_strict_missed_transients={} draft_strict_max_smear_frames={:.6} draft_candidate_input_transients={} draft_candidate_output_transients={} draft_candidate_matched_transients={} draft_candidate_missed_transients={} draft_candidate_max_smear_frames={:.6} draft_candidate_output_matched_transients={} draft_candidate_output_missed_transients={} draft_candidate_output_max_smear_frames={:.6} draft_candidate_recovery_matched_transients={} draft_candidate_recovery_missed_transients={} draft_candidate_recovery_max_smear_frames={:.6} draft_mean_match_error_frames={:.6} draft_max_match_error_frames={:.6} draft_mean_missed_nearest_distance_frames={:.6} draft_max_missed_nearest_distance_frames={:.6} draft_max_missed_expected_output_frame={:.6} draft_max_missed_nearest_output_frame={:.6} offline_input_transients={} offline_output_transients={} offline_matched_transients={} offline_missed_transients={} offline_max_matched_smear_frames={:.6} offline_max_matched_input_frame={:.6} offline_max_matched_output_frame={:.6} offline_max_matched_input_width_frames={:.6} offline_max_matched_output_width_frames={:.6} offline_strict_input_transients={} offline_strict_output_transients={} offline_strict_matched_transients={} offline_strict_missed_transients={} offline_strict_max_smear_frames={:.6} offline_candidate_input_transients={} offline_candidate_output_transients={} offline_candidate_matched_transients={} offline_candidate_missed_transients={} offline_candidate_max_smear_frames={:.6} offline_candidate_output_matched_transients={} offline_candidate_output_missed_transients={} offline_candidate_output_max_smear_frames={:.6} offline_candidate_recovery_matched_transients={} offline_candidate_recovery_missed_transients={} offline_candidate_recovery_max_smear_frames={:.6} offline_mean_match_error_frames={:.6} offline_max_match_error_frames={:.6} offline_mean_missed_nearest_distance_frames={:.6} offline_max_missed_nearest_distance_frames={:.6} offline_max_missed_expected_output_frame={:.6} offline_max_missed_nearest_output_frame={:.6}",
        draft_smear.input_transients,
        draft_smear.output_transients,
        draft_smear.matched_transients,
        draft_smear.missed_transients,
        draft_smear.max_matched_smear_frames,
        draft_smear.max_matched_input_frame,
        draft_smear.max_matched_output_frame,
        draft_smear.max_matched_input_width_frames,
        draft_smear.max_matched_output_width_frames,
        draft_strict_smear.input_transients,
        draft_strict_smear.output_transients,
        draft_strict_smear.matched_transients,
        draft_strict_smear.missed_transients,
        draft_strict_smear.max_smear_frames,
        draft_candidate_smear.input_transients,
        draft_candidate_smear.output_transients,
        draft_candidate_smear.matched_transients,
        draft_candidate_smear.missed_transients,
        draft_candidate_smear.max_smear_frames,
        draft_candidate_output_smear.matched_transients,
        draft_candidate_output_smear.missed_transients,
        draft_candidate_output_smear.max_smear_frames,
        draft_candidate_recovery_smear.matched_transients,
        draft_candidate_recovery_smear.missed_transients,
        draft_candidate_recovery_smear.max_smear_frames,
        draft_alignment.mean_match_error_frames,
        draft_alignment.max_match_error_frames,
        draft_alignment.mean_missed_nearest_distance_frames,
        draft_alignment.max_missed_nearest_distance_frames,
        draft_alignment.max_missed_expected_output_frame,
        draft_alignment.max_missed_nearest_output_frame,
        offline_smear.input_transients,
        offline_smear.output_transients,
        offline_smear.matched_transients,
        offline_smear.missed_transients,
        offline_smear.max_matched_smear_frames,
        offline_smear.max_matched_input_frame,
        offline_smear.max_matched_output_frame,
        offline_smear.max_matched_input_width_frames,
        offline_smear.max_matched_output_width_frames,
        offline_strict_smear.input_transients,
        offline_strict_smear.output_transients,
        offline_strict_smear.matched_transients,
        offline_strict_smear.missed_transients,
        offline_strict_smear.max_smear_frames,
        offline_candidate_smear.input_transients,
        offline_candidate_smear.output_transients,
        offline_candidate_smear.matched_transients,
        offline_candidate_smear.missed_transients,
        offline_candidate_smear.max_smear_frames,
        offline_candidate_output_smear.matched_transients,
        offline_candidate_output_smear.missed_transients,
        offline_candidate_output_smear.max_smear_frames,
        offline_candidate_recovery_smear.matched_transients,
        offline_candidate_recovery_smear.missed_transients,
        offline_candidate_recovery_smear.max_smear_frames,
        offline_alignment.mean_match_error_frames,
        offline_alignment.max_match_error_frames,
        offline_alignment.mean_missed_nearest_distance_frames,
        offline_alignment.max_missed_nearest_distance_frames,
        offline_alignment.max_missed_expected_output_frame,
        offline_alignment.max_missed_nearest_output_frame,
    )
}

fn format_transient_alignment_event_lines(
    audio: &DecodedListeningSourceAudio,
    ratio: f64,
    backend: &str,
    alignment: &TransientAlignmentDiagnostic,
) -> Vec<String> {
    alignment
        .missed_events
        .iter()
        .enumerate()
        .map(|(rank, event)| {
            let expected_peak_ratio =
                finite_ratio(event.expected_output_window_peak, event.input_window_peak);
            let expected_rms_ratio =
                finite_ratio(event.expected_output_window_rms, event.input_window_rms);
            let nearest_peak_ratio =
                finite_ratio(event.nearest_output_window_peak, event.input_window_peak);
            let nearest_rms_ratio =
                finite_ratio(event.nearest_output_window_rms, event.input_window_rms);
            let expected_combined_margin =
                event.expected_detector_shape.combined_score
                    - DETECTOR_POLICY.minimum_combined_score;
            let expected_flux_margin =
                event.expected_detector_shape.spectral_flux_score
                    - DETECTOR_POLICY.minimum_spectral_flux_score;
            let expected_local_previous_margin = event.expected_detector_shape.combined_score
                - event.expected_detector_shape.previous_combined_score;
            let expected_local_next_margin = event.expected_detector_shape.combined_score
                - event.expected_detector_shape.next_combined_score;
            let candidate_combined_margin = event.expected_detector_shape.combined_score
                - CANDIDATE_DETECTOR_POLICY.minimum_combined_score;
            let candidate_flux_margin = event.expected_detector_shape.spectral_flux_score
                - CANDIDATE_DETECTOR_POLICY.minimum_spectral_flux_score;
            format!(
                "decoded_transient_alignment_event case={} source={} ratio={:.6} backend={} rank={} alignment_class={} detector_class={} candidate_detector_class={} input_frame={} expected_output_frame={:.6} nearest_output_frame={:.6} nearest_distance_frames={:.6} tolerance_frames={} input_window_peak={:.6} input_window_rms={:.6} expected_output_window_peak={:.6} expected_output_window_rms={:.6} expected_output_peak_ratio={:.6} expected_output_rms_ratio={:.6} expected_detector_frame={:.6} expected_energy_score={:.6} expected_flux_score={:.6} expected_combined_score={:.6} expected_combined_margin={:.6} expected_flux_margin={:.6} expected_local_previous_margin={:.6} expected_local_next_margin={:.6} candidate_combined_margin={:.6} candidate_flux_margin={:.6} expected_previous_combined_score={:.6} expected_next_combined_score={:.6} nearest_output_window_peak={:.6} nearest_output_window_rms={:.6} nearest_output_peak_ratio={:.6} nearest_output_rms_ratio={:.6} nearest_detector_frame={:.6} nearest_energy_score={:.6} nearest_flux_score={:.6} nearest_combined_score={:.6}",
                audio.case_id,
                quoted_report_field(&audio.source_path),
                ratio,
                backend,
                rank + 1,
                classify_transient_alignment_event(event),
                classify_detector_shape(&event.expected_detector_shape),
                classify_candidate_detector_shape(&event.expected_detector_shape),
                event.input_frame,
                event.expected_output_frame,
                event.nearest_output_frame.unwrap_or(f64::NAN),
                event.nearest_distance_frames,
                QUALITY_METRIC_WINDOW_SIZE.max(QUALITY_METRIC_HOP_SIZE * 4),
                event.input_window_peak,
                event.input_window_rms,
                event.expected_output_window_peak,
                event.expected_output_window_rms,
                expected_peak_ratio,
                expected_rms_ratio,
                event.expected_detector_shape.frame_index,
                event.expected_detector_shape.energy_score,
                event.expected_detector_shape.spectral_flux_score,
                event.expected_detector_shape.combined_score,
                expected_combined_margin,
                expected_flux_margin,
                expected_local_previous_margin,
                expected_local_next_margin,
                candidate_combined_margin,
                candidate_flux_margin,
                event.expected_detector_shape.previous_combined_score,
                event.expected_detector_shape.next_combined_score,
                event.nearest_output_window_peak,
                event.nearest_output_window_rms,
                nearest_peak_ratio,
                nearest_rms_ratio,
                event.nearest_detector_shape.frame_index,
                event.nearest_detector_shape.energy_score,
                event.nearest_detector_shape.spectral_flux_score,
                event.nearest_detector_shape.combined_score,
            )
        })
        .collect()
}

fn classify_transient_alignment_event(event: &TransientAlignmentMissEvent) -> &'static str {
    let expected_peak_ratio =
        finite_ratio(event.expected_output_window_peak, event.input_window_peak);
    let expected_rms_ratio = finite_ratio(event.expected_output_window_rms, event.input_window_rms);
    if !expected_peak_ratio.is_finite() && !expected_rms_ratio.is_finite() {
        return "Inconclusive";
    }
    if expected_peak_ratio >= EXPECTED_TRANSIENT_ENERGY_PRESENT_RATIO
        || expected_rms_ratio >= EXPECTED_TRANSIENT_ENERGY_PRESENT_RATIO
    {
        "ExpectedEnergyPresent"
    } else if expected_peak_ratio >= EXPECTED_TRANSIENT_ENERGY_WEAK_RATIO
        || expected_rms_ratio >= EXPECTED_TRANSIENT_ENERGY_WEAK_RATIO
    {
        "ExpectedEnergyWeak"
    } else {
        "ExpectedEnergyMissing"
    }
}

fn classify_detector_shape(shape: &DetectorShape) -> &'static str {
    classify_detector_shape_with_thresholds(
        shape,
        DETECTOR_POLICY.minimum_combined_score,
        DETECTOR_POLICY.minimum_spectral_flux_score,
    )
}

fn classify_candidate_detector_shape(shape: &DetectorShape) -> &'static str {
    classify_detector_shape_with_thresholds(
        shape,
        CANDIDATE_DETECTOR_POLICY.minimum_combined_score,
        CANDIDATE_DETECTOR_POLICY.minimum_spectral_flux_score,
    )
}

fn classify_detector_shape_with_thresholds(
    shape: &DetectorShape,
    combined_threshold: f64,
    flux_threshold: f64,
) -> &'static str {
    if !shape.combined_score.is_finite() || !shape.spectral_flux_score.is_finite() {
        return "Inconclusive";
    }
    if shape.combined_score < combined_threshold {
        "CombinedBelowThreshold"
    } else if shape.spectral_flux_score < flux_threshold {
        "FluxBelowThreshold"
    } else if shape.combined_score < shape.previous_combined_score
        || shape.combined_score <= shape.next_combined_score
    {
        "NotLocalMaximum"
    } else {
        "DetectorWouldPass"
    }
}

fn apply_transient_width_control_candidate(input: &[f32], output: &[f32], ratio: f64) -> Vec<f32> {
    if !ratio.is_finite() || ratio <= 0.0 || ratio >= 1.0 || input.is_empty() || output.is_empty() {
        return output.to_vec();
    }

    let baseline = measure_transient_smear(
        input,
        output,
        ratio,
        QUALITY_METRIC_WINDOW_SIZE,
        QUALITY_METRIC_HOP_SIZE,
    );
    let input_events = signal_dsp_stretch::detect_stretch_transients_with_policy(
        input,
        QUALITY_METRIC_WINDOW_SIZE,
        QUALITY_METRIC_HOP_SIZE,
        DETECTOR_POLICY,
    );
    let output_events = signal_dsp_stretch::detect_stretch_transients_with_policy(
        output,
        QUALITY_METRIC_WINDOW_SIZE,
        QUALITY_METRIC_HOP_SIZE,
        DETECTOR_POLICY,
    );
    let recovery_output_events = signal_dsp_stretch::detect_stretch_transients_with_policy(
        output,
        QUALITY_METRIC_WINDOW_SIZE,
        QUALITY_METRIC_HOP_SIZE,
        CANDIDATE_DETECTOR_POLICY,
    );
    let tolerance = QUALITY_METRIC_WINDOW_SIZE.max(QUALITY_METRIC_HOP_SIZE * 4) as f64;
    let mut controlled = output.to_vec();

    for input_event in input_events {
        let expected_output_frame = input_event.frame_index as f64 * ratio;
        let Some(output_event) = nearest_transient_event(
            &output_events,
            expected_output_frame,
            tolerance,
        )
        .or_else(|| {
            nearest_transient_event(&recovery_output_events, expected_output_frame, tolerance)
        }) else {
            continue;
        };

        let Some(input_bounds) =
            transient_attack_bounds(input, input_event.frame_index, QUALITY_METRIC_WINDOW_SIZE)
        else {
            continue;
        };
        let Some(output_bounds) =
            transient_attack_bounds(output, output_event.frame_index, QUALITY_METRIC_WINDOW_SIZE)
        else {
            continue;
        };
        let target_width = input_bounds.width().saturating_add(2);
        if output_bounds.width() <= target_width {
            continue;
        }

        limit_transient_attack_width(&mut controlled, output_bounds, target_width);
    }

    let candidate = measure_transient_smear(
        input,
        &controlled,
        ratio,
        QUALITY_METRIC_WINDOW_SIZE,
        QUALITY_METRIC_HOP_SIZE,
    );
    if candidate.missed_transients > baseline.missed_transients
        || compare_metric_values(candidate.max_smear_frames, baseline.max_smear_frames)
            == MetricComparison::Worsened
    {
        output.to_vec()
    } else {
        controlled
    }
}

fn width_control_edit_stats(
    baseline: &[f32],
    candidate: &[f32],
    ratio: f64,
) -> WidthControlEditStats {
    let len = baseline.len().min(candidate.len());
    if len == 0 {
        return WidthControlEditStats::default();
    }

    let mut changed_samples = 0usize;
    let mut max_abs_sample_delta = 0.0f64;
    let mut max_abs_sample_delta_event = None;
    let mut max_added_adjacent_step_delta = 0.0f64;
    let mut max_added_adjacent_step_event = None;
    for index in 0..len {
        let sample_delta = (candidate[index] - baseline[index]).abs() as f64;
        if sample_delta > 1.0e-9 {
            changed_samples += 1;
            if sample_delta > max_abs_sample_delta {
                max_abs_sample_delta = sample_delta;
                max_abs_sample_delta_event = Some(WidthControlEditEvent::from_output_frame(
                    index, baseline, candidate, ratio,
                ));
            }
        }
        if index > 0 {
            let baseline_step = (baseline[index] - baseline[index - 1]).abs() as f64;
            let candidate_step = (candidate[index] - candidate[index - 1]).abs() as f64;
            let added_step_delta = candidate_step - baseline_step;
            if added_step_delta > max_added_adjacent_step_delta {
                max_added_adjacent_step_delta = added_step_delta;
                max_added_adjacent_step_event = Some(WidthControlEditEvent::from_output_frame(
                    index, baseline, candidate, ratio,
                ));
            }
        }
    }

    WidthControlEditStats {
        changed_samples,
        max_abs_sample_delta,
        max_abs_sample_delta_event,
        max_added_adjacent_step_delta: max_added_adjacent_step_delta.max(0.0),
        max_added_adjacent_step_event,
    }
}

fn sample_delta_at(baseline: &[f32], candidate: &[f32], index: usize) -> f64 {
    if index >= baseline.len() || index >= candidate.len() {
        f64::NAN
    } else {
        (candidate[index] - baseline[index]).abs() as f64
    }
}

fn adjacent_step_at(samples: &[f32], index: usize) -> f64 {
    if index == 0 || index >= samples.len() {
        f64::NAN
    } else {
        (samples[index] - samples[index - 1]).abs() as f64
    }
}

fn nearest_transient_event(
    events: &[signal_dsp_stretch::StretchTransientEvent],
    expected_frame: f64,
    tolerance_frames: f64,
) -> Option<signal_dsp_stretch::StretchTransientEvent> {
    events
        .iter()
        .copied()
        .filter_map(|event| {
            let distance = (event.frame_index as f64 - expected_frame).abs();
            if distance <= tolerance_frames {
                Some((distance, event))
            } else {
                None
            }
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, event)| event)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TransientAttackBounds {
    left: usize,
    right: usize,
    peak_index: usize,
}

impl TransientAttackBounds {
    fn width(self) -> usize {
        self.right - self.left + 1
    }
}

fn transient_attack_bounds(
    samples: &[f32],
    event_frame: usize,
    search_radius: usize,
) -> Option<TransientAttackBounds> {
    if samples.is_empty() {
        return None;
    }

    let start = event_frame.saturating_sub(search_radius);
    let end = (event_frame + search_radius).min(samples.len().saturating_sub(1));
    if start >= end {
        return None;
    }

    let mut peak_index = start;
    let mut peak = 0.0f32;
    for (offset, sample) in samples[start..=end].iter().enumerate() {
        let magnitude = sample.abs();
        if magnitude > peak {
            peak = magnitude;
            peak_index = start + offset;
        }
    }
    if peak <= 1.0e-6 {
        return None;
    }

    let threshold = peak * 0.5;
    let mut left = peak_index;
    while left > start && samples[left - 1].abs() >= threshold {
        left -= 1;
    }
    let mut right = peak_index;
    while right < end && samples[right + 1].abs() >= threshold {
        right += 1;
    }

    Some(TransientAttackBounds {
        left,
        right,
        peak_index,
    })
}

fn limit_transient_attack_width(
    samples: &mut [f32],
    bounds: TransientAttackBounds,
    target_width: usize,
) {
    if target_width == 0 || bounds.left >= samples.len() || bounds.right >= samples.len() {
        return;
    }

    let peak = samples[bounds.peak_index].abs();
    if peak <= 1.0e-6 {
        return;
    }

    let threshold = peak * 0.5;
    let left_width = target_width / 2;
    let right_width = target_width.saturating_sub(left_width + 1);
    let target_left = bounds
        .peak_index
        .saturating_sub(left_width)
        .max(bounds.left);
    let target_right = (bounds.peak_index + right_width)
        .min(bounds.right)
        .min(samples.len().saturating_sub(1));

    for (index, sample) in samples
        .iter_mut()
        .enumerate()
        .take(bounds.right + 1)
        .skip(bounds.left)
    {
        if (target_left..=target_right).contains(&index) {
            continue;
        }
        if sample.abs() >= threshold {
            *sample = sample.signum() * threshold * 0.999;
        }
    }
}

fn finite_ratio(numerator: f64, denominator: f64) -> f64 {
    if !numerator.is_finite() || !denominator.is_finite() || denominator.abs() <= 1.0e-12 {
        f64::NAN
    } else {
        numerator / denominator
    }
}

fn listening_source_ratios(case_id: &str) -> Result<&'static [f64], String> {
    STRETCH_CORPUS_MANIFEST
        .entries
        .iter()
        .find(|entry| entry.case.case_id == case_id)
        .map(|entry| entry.case.ratios)
        .ok_or_else(|| format!("unknown listening source case {case_id}"))
}

fn format_decoded_stretch_metric_line(
    audio: &DecodedListeningSourceAudio,
    ratio: f64,
    metric: &str,
    draft: f64,
    offline_hq: f64,
    detail: Option<String>,
) -> String {
    let mut line = format!(
        "decoded_stretch_metric case={} source={} ratio={:.6} metric={} draft={:.6} offline_hq={:.6} delta={:.6} outcome={} analyzed_frames={} analysis_limited={}",
        audio.case_id,
        quoted_report_field(&audio.source_path),
        ratio,
        metric,
        draft,
        offline_hq,
        offline_hq - draft,
        decoded_metric_outcome(draft, offline_hq),
        audio.analyzed_frames(),
        audio.analysis_limited,
    );
    if let Some(detail) = detail {
        line.push(' ');
        line.push_str(&detail);
    }
    line
}

fn decoded_metric_outcome(draft: f64, offline_hq: f64) -> &'static str {
    if !draft.is_finite() || !offline_hq.is_finite() {
        "Inconclusive"
    } else if offline_hq < draft {
        "Improved"
    } else if (offline_hq - draft).abs() <= 1.0e-9 {
        "Unchanged"
    } else {
        "Regressed"
    }
}

fn sample_limit(frame_limit: usize, channels: usize) -> Option<usize> {
    if frame_limit == 0 {
        None
    } else {
        Some(frame_limit.saturating_mul(channels))
    }
}

fn frame_analysis_limited(frame_limit: usize, total_frames: usize) -> bool {
    frame_limit > 0 && total_frames > frame_limit
}

fn integer_sample_scale(bits_per_sample: u16) -> f32 {
    if bits_per_sample == 0 {
        return 1.0;
    }
    let magnitude = 1_i64
        .checked_shl(bits_per_sample.saturating_sub(1) as u32)
        .unwrap_or(1) as f32;
    magnitude.max(1.0)
}

fn quoted_report_field(value: &str) -> String {
    format!("{value:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numeric_report_field(line: &str, name: &str) -> f64 {
        let prefix = format!("{name}=");
        line.split_whitespace()
            .find_map(|field| field.strip_prefix(&prefix))
            .unwrap_or_else(|| panic!("missing report field {name}"))
            .parse::<f64>()
            .unwrap_or_else(|error| panic!("invalid report field {name}: {error}"))
    }

    fn transient_smear_measurement(
        max_smear_frames: f64,
    ) -> signal_dsp_stretch::StretchTransientSmearMeasurement {
        signal_dsp_stretch::StretchTransientSmearMeasurement {
            ratio: 0.75,
            input_transients: 1,
            output_transients: 1,
            matched_transients: 1,
            missed_transients: 0,
            mean_smear_frames: max_smear_frames,
            max_smear_frames,
            max_matched_smear_frames: max_smear_frames,
            max_matched_input_frame: 0.0,
            max_matched_output_frame: 0.0,
            max_matched_input_width_frames: 1.0,
            max_matched_output_width_frames: 1.0,
            metric: signal_dsp_stretch::StretchMetricValue::new(
                signal_dsp_stretch::StretchMetric::TransientSmearFrames,
                max_smear_frames,
            ),
        }
    }

    #[test]
    fn parse_args_uses_defaults() {
        let args = parse_args(Vec::<String>::new()).expect("defaults parse");

        assert_eq!(
            args,
            ParseOutcome::Run(Box::new(ReportArgs {
                report_name: DEFAULT_REPORT_NAME.to_string(),
                projection_epoch: DEFAULT_PROJECTION_EPOCH.to_string(),
                output: None,
                external_benchmark_tool: DEFAULT_EXTERNAL_BENCHMARK_TOOL.to_string(),
                external_benchmark_renders: Vec::new(),
                external_benchmark_render_manifests: Vec::new(),
                export_external_benchmark_pack: None,
                external_benchmark_render_plan_status_manifests: Vec::new(),
                listening_source_manifests: Vec::new(),
                decode_listening_sources: false,
                decode_source_frame_limit: DEFAULT_DECODE_SOURCE_FRAME_LIMIT,
                measure_decoded_stretch: false,
                decoded_stretch_report_mode: DecodedStretchReportMode::Full,
                decoded_stretch_frame_limit: DEFAULT_DECODED_STRETCH_FRAME_LIMIT,
                measure_external_benchmark_quality: false,
                external_benchmark_quality_mode: ExternalBenchmarkQualityMode::Full,
                external_benchmark_signal_path: OfflineHighQualityPath::Default,
                export_blind_listening_pack: None,
                export_tail_listening_pack: None,
                export_tail_classifier_validation_pack: None,
                blind_listening_note_manifests: Vec::new(),
            }))
        );
    }

    #[test]
    fn parse_args_accepts_report_fields_and_output() {
        let args = parse_args([
            "--report-name".to_string(),
            "custom".to_string(),
            "--projection-epoch".to_string(),
            "epoch:1".to_string(),
            "--output".to_string(),
            "target/stretch-report.txt".to_string(),
            "--listening-source-manifest".to_string(),
            "target/fma.tsv".to_string(),
            "--decode-listening-sources".to_string(),
            "--decode-source-frame-limit".to_string(),
            "2048".to_string(),
            "--measure-decoded-stretch".to_string(),
            "--decoded-stretch-report-mode".to_string(),
            "expansion-selector".to_string(),
            "--measure-external-benchmark-quality".to_string(),
            "--external-benchmark-quality-mode".to_string(),
            "core".to_string(),
            "--external-benchmark-signal-path".to_string(),
            "expansion-short-window-selector".to_string(),
            "--export-blind-listening-pack".to_string(),
            "target/blind-pack".to_string(),
            "--export-tail-listening-pack".to_string(),
            "target/tail-pack".to_string(),
            "--export-tail-classifier-validation-pack".to_string(),
            "target/tail-classifier-pack".to_string(),
            "--check-blind-listening-notes".to_string(),
            "target/blind-pack/blind-listening-notes.tsv".to_string(),
            "--decoded-stretch-frame-limit".to_string(),
            "1024".to_string(),
            "--external-benchmark-tool".to_string(),
            "rubberband-cli".to_string(),
            "--external-benchmark-render".to_string(),
            "stretch:loop_seam".to_string(),
            "1.5".to_string(),
            "target/rubberband-loop.wav".to_string(),
            "--external-benchmark-render-manifest".to_string(),
            "target/external-renders.tsv".to_string(),
            "--export-external-benchmark-pack".to_string(),
            "target/external-pack".to_string(),
            "--check-external-benchmark-render-plan".to_string(),
            "target/external-pack/external-benchmark-render-plan.tsv".to_string(),
        ])
        .expect("custom args parse");

        assert_eq!(
            args,
            ParseOutcome::Run(Box::new(ReportArgs {
                report_name: "custom".to_string(),
                projection_epoch: "epoch:1".to_string(),
                output: Some(PathBuf::from("target/stretch-report.txt")),
                external_benchmark_tool: "rubberband-cli".to_string(),
                external_benchmark_renders: vec![ExternalBenchmarkRenderArg {
                    case_id: "stretch:loop_seam".to_string(),
                    ratio: "1.5".to_string(),
                    path: PathBuf::from("target/rubberband-loop.wav"),
                    tool_name: None,
                }],
                external_benchmark_render_manifests: vec![PathBuf::from(
                    "target/external-renders.tsv"
                )],
                export_external_benchmark_pack: Some(PathBuf::from("target/external-pack")),
                external_benchmark_render_plan_status_manifests: vec![PathBuf::from(
                    "target/external-pack/external-benchmark-render-plan.tsv"
                )],
                listening_source_manifests: vec![PathBuf::from("target/fma.tsv")],
                decode_listening_sources: true,
                decode_source_frame_limit: 2048,
                measure_decoded_stretch: true,
                decoded_stretch_report_mode: DecodedStretchReportMode::ExpansionSelector,
                decoded_stretch_frame_limit: 1024,
                measure_external_benchmark_quality: true,
                external_benchmark_quality_mode: ExternalBenchmarkQualityMode::Core,
                external_benchmark_signal_path:
                    OfflineHighQualityPath::ExpansionShortWindowSelector,
                export_blind_listening_pack: Some(PathBuf::from("target/blind-pack")),
                export_tail_listening_pack: Some(PathBuf::from("target/tail-pack")),
                export_tail_classifier_validation_pack: Some(PathBuf::from(
                    "target/tail-classifier-pack",
                )),
                blind_listening_note_manifests: vec![PathBuf::from(
                    "target/blind-pack/blind-listening-notes.tsv",
                )],
            }))
        );
    }

    #[test]
    fn parse_args_accepts_help() {
        let args = parse_args(["--help".to_string()]).expect("help parses");

        assert_eq!(args, ParseOutcome::Help);
    }

    #[test]
    fn parse_args_rejects_unknown_argument() {
        let error = parse_args(["--wat".to_string()]).expect_err("unknown arg rejected");

        assert!(error.contains("unknown argument"));
    }

    #[test]
    fn load_external_benchmark_render_reads_wav_metadata() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-report-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&path, 16);
        let args = ReportArgs {
            report_name: DEFAULT_REPORT_NAME.to_string(),
            projection_epoch: DEFAULT_PROJECTION_EPOCH.to_string(),
            output: None,
            external_benchmark_tool: "rubberband-cli".to_string(),
            external_benchmark_renders: vec![ExternalBenchmarkRenderArg {
                case_id: "stretch:loop_seam".to_string(),
                ratio: "1.0".to_string(),
                path: path.clone(),
                tool_name: None,
            }],
            external_benchmark_render_manifests: Vec::new(),
            export_external_benchmark_pack: None,
            external_benchmark_render_plan_status_manifests: Vec::new(),
            listening_source_manifests: Vec::new(),
            decode_listening_sources: false,
            decode_source_frame_limit: DEFAULT_DECODE_SOURCE_FRAME_LIMIT,
            measure_decoded_stretch: false,
            decoded_stretch_report_mode: DecodedStretchReportMode::Full,
            decoded_stretch_frame_limit: DEFAULT_DECODED_STRETCH_FRAME_LIMIT,
            measure_external_benchmark_quality: false,
            external_benchmark_quality_mode: ExternalBenchmarkQualityMode::Full,
            external_benchmark_signal_path: OfflineHighQualityPath::Default,
            export_blind_listening_pack: None,
            export_tail_listening_pack: None,
            export_tail_classifier_validation_pack: None,
            blind_listening_note_manifests: Vec::new(),
        };

        let renders = load_external_benchmark_renders(&args).expect("load external render");

        assert_eq!(renders.len(), 1);
        assert_eq!(renders[0].case_id, "stretch:loop_seam");
        assert_eq!(renders[0].tool_name, "rubberband-cli");
        assert_eq!(renders[0].rendered_frames, 16);
        assert_eq!(renders[0].sample_rate_hz, 48_000);
        assert_eq!(renders[0].channels, 2);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_external_benchmark_render_manifest_reads_tsv_pack() {
        let wav_path = PathBuf::from(format!(
            "target/stretch-corpus-render-manifest-test-{}.wav",
            std::process::id()
        ));
        let manifest_path = PathBuf::from(format!(
            "target/stretch-corpus-render-manifest-test-{}.tsv",
            std::process::id()
        ));
        write_test_wav(&wav_path, 16);
        if let Some(parent) = manifest_path.parent() {
            fs::create_dir_all(parent).expect("create target dir");
        }
        fs::write(
            &manifest_path,
            format!(
                "case_id\tratio\trendered_path\ttool_name\nstretch:vocals\t0.75\t{}\trubberband-cli\n",
                wav_path.display()
            ),
        )
        .expect("write render manifest");
        let args = ReportArgs {
            report_name: DEFAULT_REPORT_NAME.to_string(),
            projection_epoch: DEFAULT_PROJECTION_EPOCH.to_string(),
            output: None,
            external_benchmark_tool: "fallback-tool".to_string(),
            external_benchmark_renders: Vec::new(),
            external_benchmark_render_manifests: vec![manifest_path.clone()],
            export_external_benchmark_pack: None,
            external_benchmark_render_plan_status_manifests: Vec::new(),
            listening_source_manifests: Vec::new(),
            decode_listening_sources: false,
            decode_source_frame_limit: DEFAULT_DECODE_SOURCE_FRAME_LIMIT,
            measure_decoded_stretch: false,
            decoded_stretch_report_mode: DecodedStretchReportMode::Full,
            decoded_stretch_frame_limit: DEFAULT_DECODED_STRETCH_FRAME_LIMIT,
            measure_external_benchmark_quality: false,
            external_benchmark_quality_mode: ExternalBenchmarkQualityMode::Full,
            external_benchmark_signal_path: OfflineHighQualityPath::Default,
            export_blind_listening_pack: None,
            export_tail_listening_pack: None,
            export_tail_classifier_validation_pack: None,
            blind_listening_note_manifests: Vec::new(),
        };

        let renders = load_external_benchmark_renders(&args).expect("load manifest renders");

        assert_eq!(renders.len(), 1);
        assert_eq!(renders[0].case_id, "stretch:vocals");
        assert_eq!(renders[0].ratio, 0.75);
        assert_eq!(renders[0].tool_name, "rubberband-cli");
        assert_eq!(renders[0].rendered_frames, 16);

        let _ = fs::remove_file(wav_path);
        let _ = fs::remove_file(manifest_path);
    }

    #[test]
    fn external_benchmark_render_plan_status_reports_missing_outputs() {
        let present_wav_path = PathBuf::from(format!(
            "target/stretch-corpus-render-plan-present-test-{}.wav",
            std::process::id()
        ));
        let missing_wav_path = PathBuf::from(format!(
            "target/stretch-corpus-render-plan-missing-test-{}.wav",
            std::process::id()
        ));
        let manifest_path = PathBuf::from(format!(
            "target/stretch-corpus-render-plan-status-test-{}.tsv",
            std::process::id()
        ));
        write_test_wav(&present_wav_path, 16);
        if let Some(parent) = manifest_path.parent() {
            fs::create_dir_all(parent).expect("create target dir");
        }
        fs::write(
            &manifest_path,
            format!(
                "case_id\tratio\tsource_wav\trendered_path\ttool_name\nstretch:vocals\t0.75\t{}\t{}\trubberband-cli\nstretch:vocals\t1.25\t{}\t{}\trubberband-cli\n",
                present_wav_path.display(),
                present_wav_path.display(),
                present_wav_path.display(),
                missing_wav_path.display()
            ),
        )
        .expect("write render status manifest");

        let report = format_external_benchmark_render_plan_status(&manifest_path)
            .expect("format render plan status");

        assert!(report.starts_with("external_benchmark_render_plan_status manifest="));
        assert!(report.contains("status=Incomplete"));
        assert!(report.contains("planned_rows=2"));
        assert!(report.contains("present_rows=1"));
        assert!(report.contains("missing_rows=1"));
        assert!(report.contains("invalid_rows=0"));
        assert!(report.contains("capped_missing_rows=1"));
        assert!(report.contains("external_benchmark_render_plan_missing case=stretch:vocals"));
        assert!(report.contains("ratio=1.25"));
        assert!(report.contains(&format!(
            "source_wav={}",
            quoted_report_field(&present_wav_path.display().to_string())
        )));
        assert!(report.contains("tool=\"rubberband-cli\""));
        assert!(report.contains(&quoted_report_field(
            &missing_wav_path.display().to_string()
        )));

        let _ = fs::remove_file(present_wav_path);
        let _ = fs::remove_file(manifest_path);
    }

    #[test]
    fn export_external_benchmark_pack_writes_source_wavs_and_render_plan() {
        let source_path = PathBuf::from(format!(
            "target/stretch-corpus-pack-source-test-{}.wav",
            std::process::id()
        ));
        let export_dir = PathBuf::from(format!(
            "target/stretch-corpus-pack-test-{}",
            std::process::id()
        ));
        write_test_wav(&source_path, 4_096);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: source_path.display().to_string(),
            source_label: "Artist - Song".to_string(),
            license_title: "Attribution".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        };

        let report = export_external_benchmark_pack(&[source], &export_dir, "rubberband-cli", 1024)
            .expect("export comparator pack");
        let manifest_path = export_dir.join("external-benchmark-render-plan.tsv");
        let manifest = fs::read_to_string(&manifest_path).expect("read render plan");

        assert!(report.starts_with("external_benchmark_render_pack export_dir="));
        assert!(report.contains("exported_sources=1"));
        assert!(report.contains("tool=\"rubberband-cli\""));
        assert!(report.contains("external_benchmark_render_plan case=stretch:vocals"));
        assert!(manifest.starts_with("case_id\tratio\tsource_wav\trendered_path\ttool_name\n"));
        assert!(manifest.contains("stretch:vocals\t0.750000\t"));
        assert!(manifest.contains("\trubberband-cli\n"));
        assert!(export_dir.join("sources").exists());
        assert!(export_dir.join("renders").exists());
        assert_eq!(
            fs::read_dir(export_dir.join("sources"))
                .expect("source dir entries")
                .count(),
            1
        );

        let _ = fs::remove_file(source_path);
        let _ = fs::remove_dir_all(export_dir);
    }

    #[test]
    fn external_benchmark_quality_measures_rendered_wav_against_signal_output() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-external-quality-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&path, 4_096);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: path.display().to_string(),
            source_label: "Artist - Song".to_string(),
            license_title: "Attribution".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        };
        let render = ExternalBenchmarkQualityRender {
            case_id: "stretch:vocals".to_string(),
            ratio: 1.0,
            tool_name: "rubberband-cli".to_string(),
            rendered_path: path.display().to_string(),
            source_wav: None,
        };

        let formatted = format_external_benchmark_quality_metrics(
            &[source],
            &[render],
            4_096,
            ExternalBenchmarkQualityMode::Full,
            OfflineHighQualityPath::Default,
        )
        .expect("format external quality metrics");

        assert!(formatted.starts_with("external_benchmark_quality case=stretch:vocals"));
        assert!(formatted.contains("signal_path=Default"));
        assert!(formatted.contains("tool=\"rubberband-cli\""));
        assert!(formatted.contains("status=Measured reason=Ok"));
        assert!(formatted.contains(
            "source_boundary=\"rendered-output-only; no external source or library dependency\""
        ));
        assert!(formatted.contains("sample_rate_match=true"));
        assert!(formatted.contains("source_frames=4096"));
        assert!(formatted.contains("signal_frames=4096"));
        assert!(formatted.contains("external_frames=4096"));
        assert!(formatted.contains("signal_timing_drift_samples=0.000000"));
        assert!(formatted.contains("external_timing_drift_samples=0.000000"));
        assert!(formatted.contains("signal_transient_mean_absolute_offset_frames="));
        assert!(formatted.contains("external_transient_mean_absolute_offset_frames="));
        assert!(formatted.contains("signal_transient_max_crest_growth_db="));
        assert!(formatted.contains("draft_transient_max_crest_growth_db="));
        assert!(formatted.contains("alignment_lag_frames=0"));
        assert!(formatted.contains("aligned_compared_frames=4096"));
        assert!(formatted.contains("aligned_rms_error=0.000000000"));
        assert!(formatted.contains("signal_endpoint_energy_delta_db=0.000000"));
        assert!(formatted.contains("external_endpoint_energy_delta_db=0.000000"));
        assert!(formatted.contains("integrity_limit_id=offline-high-quality-v1"));
        assert!(formatted.contains("signal_integrity_passed=true"));
        assert!(formatted.contains("external_integrity_passed=true"));
        assert!(formatted.contains("signal_measured_endpoint_count=2"));
        assert!(formatted.contains("external_measured_endpoint_count=2"));
        assert!(formatted.contains("signal_added_silence_frames=0"));
        assert!(formatted.contains("external_added_silence_frames=0"));
        assert!(formatted.contains("signal_peak_growth_db=0.000000"));
        assert!(formatted.contains("external_peak_growth_db=0.000000"));
        assert!(formatted.contains("signal_render_seconds="));
        assert!(formatted.contains("signal_cpu_realtime_factor="));
        assert!(formatted.contains("signal_cpu_realtime_factor_basis=rendered-audio-duration"));
        assert!(formatted.contains("signal_heap_baseline_bytes="));
        assert!(formatted.contains("signal_heap_peak_bytes="));
        assert!(formatted.contains("signal_peak_working_memory_bytes="));
        assert!(formatted.contains(
            "signal_peak_working_memory_scope=peak-live-heap-growth-above-pre-render-baseline"
        ));
        assert!(formatted.contains(
            "stretch_render_integrity_limits id=offline-high-quality-v1 max_output_length_drift_frames=0.500000 max_endpoint_energy_delta_db=7.000000 max_added_silence_frames=0 max_peak_growth_db=6.000000"
        ));
        let quality_line = formatted.lines().next().expect("quality row");
        assert!(numeric_report_field(quality_line, "signal_render_seconds") > 0.0);
        assert!(numeric_report_field(quality_line, "signal_cpu_realtime_factor") > 0.0);
        assert!(numeric_report_field(quality_line, "signal_peak_working_memory_bytes") > 0.0);
        assert!(formatted.contains("external_benchmark_feature_delta case=stretch:vocals"));
        assert!(formatted.contains("envelope_correlation=1.000000"));
        assert!(formatted.contains("rms_delta_db=0.000000"));
        assert!(formatted.contains("spectral_centroid_delta_hz=0.000000"));
        assert!(formatted.contains("feature_divergence_score=0.000000"));
        assert!(formatted.contains("external_benchmark_gain_envelope_review rank=1"));
        assert!(formatted.contains("reason=TopFeatureDivergence"));
        assert!(formatted.contains("window_count=1"));
        assert!(formatted.contains("median_window_rms_delta_db=0.000000"));
        assert!(formatted.contains("gain_pattern=CloseGain"));
        assert!(formatted.contains("external_benchmark_level_normalized_review rank=1"));
        assert!(formatted.contains("signal_gain_db_applied=0.000000"));
        assert!(formatted.contains("normalized_feature_divergence_score=0.000000"));
        assert!(formatted.contains("normalization_pattern=MostlyLevelExplained"));
        assert!(formatted.contains("external_benchmark_residual_coherence_review rank=1"));
        assert!(formatted.contains("block_rms_envelope_correlation=1.000000"));
        assert!(formatted.contains("spectral_magnitude_coherence=1.000000"));
        assert!(formatted.contains("residual_pattern=MostlyPhaseOrFineTextureResidual"));
        assert!(formatted.contains("external_benchmark_coherence_target_review rank=1"));
        assert!(formatted.contains("material_scope=VocalTonal"));
        assert!(formatted.contains("target_reason=NoResidual"));
        assert!(formatted.contains("target_score=0.000000"));
        assert!(formatted.contains("external_benchmark_coherence_candidate_review rank=1"));
        assert!(
            formatted.contains("candidate_path=sustained-coherence-long-window-identity-locked")
        );
        assert!(formatted.contains("gate=spectral-magnitude-material-guard"));
        assert!(formatted.contains("gate_decision=Rejected"));
        assert!(formatted.contains("gate_reason=NonSpectralTargetReason"));
        assert!(formatted.contains("product_probe=source-character-v1"));
        assert!(formatted.contains("product_probe_decision="));
        assert!(formatted.contains("product_probe_confidence="));
        assert!(formatted.contains("external_benchmark_coherence_candidate_summary rows=1"));
        assert!(formatted.contains("improved_rows=0 unchanged_rows=1 regressed_rows=0"));
        assert!(formatted.contains(
            "external_benchmark_coherence_candidate_gate_summary gate=spectral-magnitude-material-guard"
        ));
        assert!(formatted.contains("selected_rows=0 rejected_rows=1"));
        assert!(
            formatted.contains("rejected_candidate_better_rows=0 rejected_current_better_rows=0")
        );
        assert!(formatted.contains(
            "external_benchmark_coherence_product_probe_summary probe=source-character-v1"
        ));
        assert!(formatted.contains("promotion_status="));
        assert!(formatted.contains("benchmark_gate_agree_rows="));
        assert!(formatted.contains(
            "external_benchmark_coherence_blend_candidate_summary candidate_path=current-long-window-half-blend"
        ));
        assert!(formatted.contains("external_benchmark_coherence_blend_candidate_review rank=1"));
        assert!(formatted.contains(
            "external_benchmark_coherence_envelope_candidate_summary candidate_path=long-window-current-envelope-match"
        ));
        assert!(formatted.contains("external_benchmark_coherence_envelope_candidate_review rank=1"));
        assert!(formatted.contains(
            "external_benchmark_coherence_expansion_reset_candidate_summary candidate_path=expansion-long-window-transient-reset"
        ));
        assert!(formatted
            .contains("external_benchmark_coherence_expansion_reset_candidate_review rank=1"));
        assert!(formatted.contains(
            "external_benchmark_coherence_stability_adaptive_candidate_summary candidate_path=expansion-long-window-stability-adaptive"
        ));
        assert!(formatted
            .contains("external_benchmark_coherence_stability_adaptive_candidate_review rank=1"));
        assert!(formatted.contains(
            "external_benchmark_coherence_tracked_peak_candidate_summary candidate_path=expansion-long-window-tracked-peak-regions"
        ));
        assert!(
            formatted.contains("external_benchmark_coherence_tracked_peak_candidate_review rank=1")
        );
        assert!(formatted.contains(
            "external_benchmark_coherence_magnitude_slew_candidate_summary candidate_path=expansion-long-window-magnitude-slew"
        ));
        assert!(formatted
            .contains("external_benchmark_coherence_magnitude_slew_candidate_review rank=1"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn external_benchmark_quality_reports_same_event_phase_lock_controls() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-transient-control-test-{}.wav",
            std::process::id()
        ));
        write_transient_test_wav(&path, 48_000);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:drums_percussion".to_string(),
            source_path: path.display().to_string(),
            source_label: "Transient probe".to_string(),
            license_title: "Local test".to_string(),
            license_url: String::new(),
            provenance_url: String::new(),
        };
        let render = ExternalBenchmarkQualityRender {
            case_id: source.case_id.clone(),
            ratio: 0.75,
            tool_name: "control".to_string(),
            rendered_path: path.display().to_string(),
            source_wav: None,
        };

        let formatted = format_external_benchmark_quality_metrics(
            &[source],
            &[render],
            48_000,
            ExternalBenchmarkQualityMode::Core,
            OfflineHighQualityPath::Default,
        )
        .expect("format transient control metrics");

        let control = formatted
            .lines()
            .find(|line| line.starts_with("external_benchmark_transient_control "))
            .expect("same-event control row");
        assert!(control.contains("anchor_input_frame="));
        assert!(control.contains("stability_event_crest_growth_db="));
        assert!(control.contains("tracked_peak_max_crest_growth_db="));
        assert!(control.contains("magnitude_slew_mean_absolute_offset_frames="));
        let tonal = formatted
            .lines()
            .find(|line| line.starts_with("external_benchmark_tonal_texture "))
            .expect("source-relative tonal texture row");
        assert!(tonal.contains("signal_mean_spectral_residual_ratio="));
        assert!(tonal.contains("signal_sideband_delta_vs_external="));
        assert!(tonal.contains("signal_envelope_modulation_delta_vs_external_db="));
        let formant_boundary = formatted
            .lines()
            .find(|line| line.starts_with("external_benchmark_formant_boundary "))
            .expect("source-relative formant and boundary row");
        assert!(formant_boundary.contains("signal_mean_envelope_residual_ratio="));
        assert!(formant_boundary.contains("signal_mean_envelope_centroid_shift_hz="));
        assert!(formant_boundary.contains("signal_max_boundary_step_crest_growth_db="));
        let tail_anchor = formatted
            .lines()
            .find(|line| line.starts_with("external_benchmark_tail_anchor_review "))
            .expect("tail-anchor candidate row");
        assert!(tail_anchor.contains("boundary_improvement_db="));
        assert!(tail_anchor.contains("combined_regression_gate_passed="));
        assert!(tail_anchor.contains("control=source"));
        assert!(formatted
            .lines()
            .any(|line| line.starts_with("external_benchmark_tail_anchor_review control=zero ")));
        assert!(formatted.lines().any(|line| line
            .starts_with("external_benchmark_tail_anchor_review control=multiplicative_zero ")));
        let tail_features = formatted
            .lines()
            .find(|line| line.starts_with("external_benchmark_tail_local_features "))
            .expect("tail-local feature row");
        assert!(tail_features.contains("low_band_energy_share="));
        assert!(tail_features.contains("multiplicative_correction_energy_ratio="));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn external_benchmark_quality_core_mode_skips_feature_reviews() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-external-quality-core-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&path, 4_096);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: path.display().to_string(),
            source_label: "Artist - Song".to_string(),
            license_title: "Attribution".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        };
        let render = ExternalBenchmarkQualityRender {
            case_id: "stretch:vocals".to_string(),
            ratio: 1.0,
            tool_name: "rubberband-cli".to_string(),
            rendered_path: path.display().to_string(),
            source_wav: None,
        };

        let formatted = format_external_benchmark_quality_metrics(
            &[source],
            &[render],
            4_096,
            ExternalBenchmarkQualityMode::Core,
            OfflineHighQualityPath::Default,
        )
        .expect("format core external quality metrics");

        assert!(formatted.starts_with("external_benchmark_quality case=stretch:vocals"));
        assert!(!formatted.contains("external_benchmark_feature_delta "));
        assert!(!formatted.contains("external_benchmark_gain_envelope_review "));
        assert!(!formatted.contains("external_benchmark_level_normalized_review "));
        assert!(!formatted.contains("external_benchmark_residual_coherence_review "));
        assert!(!formatted.contains("external_benchmark_coherence_target_review "));
        assert!(!formatted.contains("external_benchmark_coherence_candidate_review "));
        assert!(!formatted.contains("external_benchmark_coherence_candidate_summary "));
        assert!(!formatted.contains("external_benchmark_coherence_candidate_gate_summary "));
        assert!(!formatted.contains("external_benchmark_coherence_product_probe_summary "));
        assert!(!formatted.contains("external_benchmark_coherence_blend_candidate_summary "));
        assert!(!formatted.contains("external_benchmark_coherence_blend_candidate_review "));
        assert!(!formatted.contains("external_benchmark_coherence_envelope_candidate_summary "));
        assert!(!formatted.contains("external_benchmark_coherence_envelope_candidate_review "));
        assert!(
            !formatted.contains("external_benchmark_coherence_expansion_reset_candidate_summary ")
        );
        assert!(
            !formatted.contains("external_benchmark_coherence_expansion_reset_candidate_review ")
        );
        assert!(!formatted
            .contains("external_benchmark_coherence_stability_adaptive_candidate_summary "));
        assert!(!formatted
            .contains("external_benchmark_coherence_stability_adaptive_candidate_review "));
        assert!(!formatted.contains("external_benchmark_coherence_tracked_peak_candidate_summary "));
        assert!(!formatted.contains("external_benchmark_coherence_tracked_peak_candidate_review "));
        assert!(
            !formatted.contains("external_benchmark_coherence_magnitude_slew_candidate_summary ")
        );
        assert!(
            !formatted.contains("external_benchmark_coherence_magnitude_slew_candidate_review ")
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn external_benchmark_coherence_target_ranking_prefers_normalized_residuals() {
        let strong = external_benchmark_coherence_target_score(0.5, 0.7, 0.95, 0.6, 0.8);
        let weak = external_benchmark_coherence_target_score(0.1, 0.98, 0.99, 0.1, 0.98);

        assert!(strong > weak);
        assert_eq!(
            external_benchmark_coherence_target_material_scope("stretch:full_mix"),
            Some("DensePolyphonic")
        );
        assert_eq!(
            external_benchmark_coherence_target_material_scope("stretch:drums_percussion"),
            None
        );
        assert_eq!(
            classify_external_benchmark_coherence_target_reason(strong, 0.7, 0.95, 0.6, 0.8),
            "SpectralMagnitudeCoherence"
        );
        assert_eq!(
            classify_external_benchmark_coherence_target_reason(0.0, 1.0, 1.0, 0.0, 1.0),
            "NoResidual"
        );
        assert_eq!(external_benchmark_candidate_outcome(1.0, 0.75), "Improved");
        assert_eq!(external_benchmark_candidate_outcome(1.0, 1.0), "Unchanged");
        assert_eq!(external_benchmark_candidate_outcome(1.0, 1.25), "Regressed");
        assert_eq!(
            external_benchmark_coherence_candidate_gate_decision(
                "SpectralMagnitudeCoherence",
                "BassSustain",
                1.25
            ),
            ("Selected", "TargetSpectralMagnitudeCoherence")
        );
        assert_eq!(
            external_benchmark_coherence_candidate_gate_decision(
                "SpectralMagnitudeCoherence",
                "BassSustain",
                1.5
            ),
            ("Rejected", "ExtremeExpansionMaterialGuard")
        );
        assert_eq!(
            external_benchmark_coherence_candidate_gate_decision(
                "SpectralMagnitudeCoherence",
                "DensePolyphonic",
                1.5
            ),
            ("Selected", "TargetSpectralMagnitudeCoherence")
        );
        assert_eq!(
            external_benchmark_coherence_candidate_gate_decision(
                "SampleEnvelopeCoherence",
                "DensePolyphonic",
                1.25
            ),
            ("Rejected", "NonSpectralTargetReason")
        );
        let confident_complex = CoherenceProductObservableProbe {
            low_band_weight: 0.2,
            sustain_body: 0.5,
            rhythmic_activity: 0.2,
            spectral_complexity: 0.5,
            confidence: 0.95,
        };
        assert_eq!(
            coherence_product_observable_probe_decision(&confident_complex, 1.25),
            ("Selected", "ComplexSustainedSource")
        );
        assert_eq!(
            coherence_product_observable_probe_decision(&confident_complex, 1.5),
            ("Selected", "ComplexSustainedSource")
        );
        let low_band_extreme = CoherenceProductObservableProbe {
            low_band_weight: 0.5,
            sustain_body: 0.2,
            rhythmic_activity: 0.2,
            spectral_complexity: 0.5,
            confidence: 0.95,
        };
        assert_eq!(
            coherence_product_observable_probe_decision(&low_band_extreme, 1.5),
            ("Rejected", "ExtremeExpansionSourceGuard")
        );
        let pulse_driven = CoherenceProductObservableProbe {
            rhythmic_activity: 0.7,
            ..confident_complex
        };
        assert_eq!(
            coherence_product_observable_probe_decision(&pulse_driven, 1.25),
            ("Rejected", "PulseDrivenSource")
        );
    }

    #[test]
    fn external_benchmark_quality_uses_selected_signal_path() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-external-quality-path-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&path, 4_096);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: path.display().to_string(),
            source_label: "Artist - Song".to_string(),
            license_title: "Attribution".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        };
        let render = ExternalBenchmarkQualityRender {
            case_id: "stretch:vocals".to_string(),
            ratio: 1.25,
            tool_name: "rubberband-cli".to_string(),
            rendered_path: path.display().to_string(),
            source_wav: None,
        };

        let formatted = format_external_benchmark_quality_metrics(
            &[source],
            &[render],
            4_096,
            ExternalBenchmarkQualityMode::Core,
            OfflineHighQualityPath::ExpansionShortWindowSelector,
        )
        .expect("format selected-path external quality metrics");

        assert!(formatted.starts_with("external_benchmark_quality case=stretch:vocals"));
        assert!(formatted.contains("signal_path=ExpansionShortWindowSelector"));
        assert!(formatted.contains("status=Measured reason=Ok"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn external_benchmark_quality_uses_manifest_source_wav_for_duplicate_cases() {
        let source_a = PathBuf::from(format!(
            "target/stretch-corpus-external-quality-source-a-test-{}.wav",
            std::process::id()
        ));
        let source_b = PathBuf::from(format!(
            "target/stretch-corpus-external-quality-source-b-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&source_a, 4_096);
        write_test_wav(&source_b, 4_096);
        let listening_sources = vec![
            StretchCorpusListeningSource {
                case_id: "stretch:vocals".to_string(),
                source_path: "target/original-a.mp3".to_string(),
                source_label: "Artist - Song A".to_string(),
                license_title: "Attribution".to_string(),
                license_url: "https://example.test/license".to_string(),
                provenance_url: "https://example.test/track-a".to_string(),
            },
            StretchCorpusListeningSource {
                case_id: "stretch:vocals".to_string(),
                source_path: "target/original-b.mp3".to_string(),
                source_label: "Artist - Song B".to_string(),
                license_title: "Attribution".to_string(),
                license_url: "https://example.test/license".to_string(),
                provenance_url: "https://example.test/track-b".to_string(),
            },
        ];
        let renders = vec![
            ExternalBenchmarkQualityRender {
                case_id: "stretch:vocals".to_string(),
                ratio: 1.0,
                tool_name: "rubberband-cli".to_string(),
                rendered_path: source_a.display().to_string(),
                source_wav: Some(source_a.display().to_string()),
            },
            ExternalBenchmarkQualityRender {
                case_id: "stretch:vocals".to_string(),
                ratio: 1.0,
                tool_name: "rubberband-cli".to_string(),
                rendered_path: source_b.display().to_string(),
                source_wav: Some(source_b.display().to_string()),
            },
        ];

        let formatted = format_external_benchmark_quality_metrics(
            &listening_sources,
            &renders,
            4_096,
            ExternalBenchmarkQualityMode::Full,
            OfflineHighQualityPath::Default,
        )
        .expect("format source-wav external quality metrics");

        assert_eq!(
            formatted
                .lines()
                .filter(|line| {
                    line.starts_with("external_benchmark_quality ")
                        && line.contains("status=Measured")
                })
                .count(),
            2
        );
        assert_eq!(
            formatted
                .lines()
                .filter(|line| {
                    line.starts_with("external_benchmark_feature_delta ")
                        && line.contains("status=Measured")
                })
                .count(),
            2
        );
        assert!(formatted.contains(&format!(
            "source={}",
            quoted_report_field(&source_a.display().to_string())
        )));
        assert!(formatted.contains(&format!(
            "source={}",
            quoted_report_field(&source_b.display().to_string())
        )));
        assert!(!formatted.contains("target/original-a.mp3"));
        assert!(!formatted.contains("target/original-b.mp3"));

        let _ = fs::remove_file(source_a);
        let _ = fs::remove_file(source_b);
    }

    #[test]
    fn external_benchmark_quality_skips_ambiguous_case_without_source_wav() {
        let render_path = PathBuf::from(format!(
            "target/stretch-corpus-external-quality-ambiguous-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&render_path, 4_096);
        let listening_sources = vec![
            StretchCorpusListeningSource {
                case_id: "stretch:vocals".to_string(),
                source_path: "target/original-a.mp3".to_string(),
                source_label: "Artist - Song A".to_string(),
                license_title: "Attribution".to_string(),
                license_url: "https://example.test/license".to_string(),
                provenance_url: "https://example.test/track-a".to_string(),
            },
            StretchCorpusListeningSource {
                case_id: "stretch:vocals".to_string(),
                source_path: "target/original-b.mp3".to_string(),
                source_label: "Artist - Song B".to_string(),
                license_title: "Attribution".to_string(),
                license_url: "https://example.test/license".to_string(),
                provenance_url: "https://example.test/track-b".to_string(),
            },
        ];
        let render = ExternalBenchmarkQualityRender {
            case_id: "stretch:vocals".to_string(),
            ratio: 1.0,
            tool_name: "rubberband-cli".to_string(),
            rendered_path: render_path.display().to_string(),
            source_wav: None,
        };

        let formatted = format_external_benchmark_quality_metrics(
            &listening_sources,
            &[render],
            4_096,
            ExternalBenchmarkQualityMode::Full,
            OfflineHighQualityPath::Default,
        )
        .expect("format ambiguous external quality metrics");

        assert!(formatted.contains("status=Skipped reason=AmbiguousListeningSource"));

        let _ = fs::remove_file(render_path);
    }

    #[test]
    fn load_listening_source_manifest_reads_tsv_and_checks_source_path() {
        let audio_path = PathBuf::from(format!(
            "target/stretch-corpus-source-test-{}.mp3",
            std::process::id()
        ));
        if let Some(parent) = audio_path.parent() {
            fs::create_dir_all(parent).expect("create target dir");
        }
        fs::write(&audio_path, b"local test source").expect("write test source");
        let manifest_path = PathBuf::from(format!(
            "target/stretch-corpus-source-test-{}.tsv",
            std::process::id()
        ));
        fs::write(
            &manifest_path,
            format!(
                "case_id\tartist\ttitle\tlicense_title\tlicense_url\ttrack_url\tlocal_path\nstretch:vocals\tArtist\tSong\tAttribution\thttps://example.test/license\thttps://example.test/track\t{}\n",
                audio_path.display()
            ),
        )
        .expect("write source manifest");

        let sources = load_listening_source_manifest(&manifest_path).expect("load source manifest");

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].case_id, "stretch:vocals");
        assert_eq!(sources[0].source_label, "Artist - Song");
        assert_eq!(sources[0].license_title, "Attribution");
        assert_eq!(sources[0].provenance_url, "https://example.test/track");

        let _ = fs::remove_file(audio_path);
        let _ = fs::remove_file(manifest_path);
    }

    #[test]
    fn decoded_listening_source_profile_reads_wav_metrics() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-decode-profile-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&path, 4_096);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: path.display().to_string(),
            source_label: "Artist - Song".to_string(),
            license_title: "Attribution".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        };

        let profile = decode_listening_source_profile(&source, 2_048).expect("decode profile");

        assert_eq!(profile.case_id, "stretch:vocals");
        assert_eq!(profile.sample_rate_hz, 48_000);
        assert_eq!(profile.channels, 2);
        assert_eq!(profile.analyzed_frames, 2_048);
        assert!(profile.analysis_limited);
        assert!(profile.peak > 0.0);
        assert!(profile.rms > 0.0);
        assert!(profile.zero_crossings_per_second > 0.0);

        let formatted = format_decoded_listening_source_profiles(&[source], 2_048)
            .expect("format decoded profiles");

        assert!(formatted.starts_with("decoded_listening_source case=stretch:vocals"));
        assert!(formatted.contains("sample_rate=48000 channels=2"));
        assert!(formatted.contains("analysis_limited=true"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn decoded_stretch_metrics_measure_wav_excerpt() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-decode-stretch-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&path, 4_096);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: path.display().to_string(),
            source_label: "Artist - Song".to_string(),
            license_title: "Attribution".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        };

        let formatted =
            format_decoded_stretch_metrics(&[source], 2_048, DecodedStretchReportMode::Full)
                .expect("format decoded metrics");

        assert!(formatted.contains("decoded_stretch_metric case=stretch:vocals"));
        assert!(formatted.contains("ratio=0.750000 metric=TimingDriftSamples"));
        assert!(formatted.contains("metric=TransientSmearFrames"));
        assert!(formatted.contains("decoded_transient_recovery_gate backend=draft"));
        assert!(formatted.contains("decoded_transient_recovery_gate backend=offline_hq"));
        assert!(formatted.contains("decoded_compression_phase_lock_ablation rows="));
        assert!(formatted.contains("phase_locked_path=offline_hq"));
        assert!(formatted.contains("independent_bins_path=draft"));
        assert!(formatted.contains("decoded_compression_transient_anchor_candidate rows="));
        assert!(formatted.contains("candidate_path=offline_hq_compression_anchor"));
        assert!(formatted.contains("decoded_compression_short_window_candidate rows="));
        assert!(formatted.contains("candidate_path=offline_hq_short_window"));
        assert!(formatted.contains("decoded_expansion_short_window_candidate rows="));
        assert!(formatted.contains("ratio_scope=expansion"));
        assert!(formatted.contains("decoded_expansion_short_window_selector_candidate rows="));
        assert!(formatted.contains("gate=CurrentMissesOrDraftRegression"));
        assert!(formatted.contains("decoded_compression_short_window_selector_candidate rows="));
        assert!(formatted.contains("gate=CurrentMissesOrHighCurrentSmear"));
        assert!(formatted.contains("decoded_compression_short_window_selector_path rows="));
        assert!(formatted.contains("path=offline_hq_compression_short_window_selector"));
        assert!(formatted.contains("decoded_matched_transient_width_review rows="));
        assert!(formatted.contains("metric=max_matched_smear_frames"));
        assert!(formatted.contains("decoded_transient_width_control_candidate rows="));
        assert!(formatted.contains("candidate_path=offline_hq_width_control"));
        assert!(formatted.contains("baseline_path=offline_hq"));
        assert!(formatted.contains("decoded_transient_width_control_edit_gate rows="));
        assert!(formatted.contains("gate=ConservativeEditPressure"));
        assert!(formatted.contains("best_candidate_improvement_delta_frames="));
        assert!(formatted.contains("max_abs_sample_delta="));
        assert!(formatted.contains("max_abs_sample_delta_source="));
        assert!(formatted.contains("max_added_adjacent_step_delta="));
        assert!(formatted.contains("max_added_adjacent_step_source="));
        assert!(formatted.contains("target_status="));
        assert!(formatted.contains("global_threshold_status="));
        assert!(formatted.contains("full_candidate_input_ratio="));
        assert!(formatted.contains("offline_matched_transients="));
        assert!(formatted.contains("offline_max_matched_smear_frames="));
        assert!(formatted.contains("offline_max_matched_input_frame="));
        assert!(formatted.contains("offline_max_matched_output_frame="));
        assert!(formatted.contains("offline_max_matched_input_width_frames="));
        assert!(formatted.contains("offline_max_matched_output_width_frames="));
        assert!(formatted.contains("offline_strict_matched_transients="));
        assert!(formatted.contains("offline_strict_missed_transients="));
        assert!(formatted.contains("offline_strict_max_smear_frames="));
        assert!(formatted.contains("offline_candidate_matched_transients="));
        assert!(formatted.contains("offline_candidate_missed_transients="));
        assert!(formatted.contains("offline_candidate_max_smear_frames="));
        assert!(formatted.contains("offline_candidate_output_matched_transients="));
        assert!(formatted.contains("offline_candidate_output_missed_transients="));
        assert!(formatted.contains("offline_candidate_output_max_smear_frames="));
        assert!(formatted.contains("offline_candidate_recovery_matched_transients="));
        assert!(formatted.contains("offline_candidate_recovery_missed_transients="));
        assert!(formatted.contains("offline_candidate_recovery_max_smear_frames="));
        assert!(formatted.contains("offline_mean_match_error_frames="));
        assert!(formatted.contains("offline_mean_missed_nearest_distance_frames="));
        assert!(formatted.contains("offline_max_missed_expected_output_frame="));
        assert!(formatted.contains("offline_max_missed_nearest_output_frame="));
        assert!(formatted.contains("analysis_limited=true"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn decoded_stretch_expansion_selector_mode_skips_unrelated_reviews() {
        let path = PathBuf::from(format!(
            "target/stretch-corpus-decode-expansion-selector-test-{}.wav",
            std::process::id()
        ));
        write_test_wav(&path, 4_096);
        let source = StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: path.display().to_string(),
            source_label: "Artist - Song".to_string(),
            license_title: "Attribution".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        };

        let formatted = format_decoded_stretch_metrics(
            &[source],
            2_048,
            DecodedStretchReportMode::ExpansionSelector,
        )
        .expect("format expansion selector decoded metrics");

        assert!(formatted.contains("decoded_expansion_short_window_candidate rows="));
        assert!(formatted.contains("decoded_expansion_short_window_selector_candidate rows="));
        assert!(formatted.contains("gate=CurrentMissesOrDraftRegression"));
        assert!(!formatted.contains("decoded_stretch_metric "));
        assert!(!formatted.contains("decoded_compression_short_window_candidate "));
        assert!(!formatted.contains("decoded_matched_transient_width_review "));
        assert!(!formatted.contains("decoded_transient_width_control_candidate "));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn width_control_edit_event_formats_window_damage_fields() {
        let baseline = vec![0.0, 0.0, 1.0, 1.0, 1.0, 0.0];
        let candidate = vec![0.0, 0.0, 1.0, 0.4, 1.0, 0.0];
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:bass".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: candidate.clone(),
            analysis_limited: false,
        };

        let stats = width_control_edit_stats(&baseline, &candidate, 0.75);
        let line = stats
            .max_abs_sample_delta_event
            .expect("sample edit event")
            .with_source(&audio, 0.75, "MaxSampleDelta")
            .format_report_line();

        assert!(line.starts_with("decoded_transient_width_control_edit_event kind=MaxSampleDelta"));
        assert!(line.contains("case=stretch:bass"));
        assert!(line.contains("source_frame=4.000000"));
        assert!(line.contains("output_frame=3"));
        assert!(line.contains("baseline_peak="));
        assert!(line.contains("candidate_peak="));
        assert!(line.contains("baseline_adjacent_step="));
        assert!(line.contains("candidate_adjacent_step="));
    }

    #[test]
    fn width_control_edit_gate_counts_retained_and_rejected_improvements() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:bass".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: vec![0.0; 8],
            analysis_limited: false,
        };
        let offline = transient_smear_measurement(20.0);
        let candidate = transient_smear_measurement(10.0);
        let safe_edit = WidthControlEditStats {
            changed_samples: 2,
            max_abs_sample_delta: WIDTH_CONTROL_EDIT_GATE_MAX_SAMPLE_DELTA * 0.5,
            max_added_adjacent_step_delta: WIDTH_CONTROL_EDIT_GATE_MAX_ADDED_ADJACENT_STEP_DELTA
                * 0.5,
            ..Default::default()
        };
        let risky_edit = WidthControlEditStats {
            changed_samples: 2,
            max_abs_sample_delta: WIDTH_CONTROL_EDIT_GATE_MAX_SAMPLE_DELTA * 0.5,
            max_added_adjacent_step_delta: WIDTH_CONTROL_EDIT_GATE_MAX_ADDED_ADJACENT_STEP_DELTA
                * 2.0,
            ..Default::default()
        };
        let mut gate = TransientWidthControlEditGateAccumulator::default();

        gate.record(&audio, 0.75, &offline, &candidate, &safe_edit);
        gate.record(&audio, 0.75, &offline, &candidate, &risky_edit);
        let line = gate.format_report_line();

        assert!(line.contains("accepted_rows=1"));
        assert!(line.contains("rejected_rows=1"));
        assert!(line.contains("accepted_candidate_better_rows=1"));
        assert!(line.contains("rejected_candidate_better_rows=1"));
        assert!(line.contains("gated_better_rows=1"));
        assert!(line.contains("unchanged_rows=1"));
        assert!(line.contains("rejected_candidate_improvement_delta_frames=10.000000"));
        assert!(line.contains("max_rejected_added_adjacent_step_delta=0.100000000"));
    }

    #[test]
    fn compression_transient_anchor_candidate_counts_metric_outcomes() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:pads_sustains".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: vec![0.0; 8],
            analysis_limited: false,
        };
        let draft = transient_smear_measurement(2.0);
        let offline = transient_smear_measurement(13.0);
        let improved_candidate = transient_smear_measurement(4.0);
        let regressed_candidate = transient_smear_measurement(20.0);
        let mut candidate = CompressionReviewCandidateAccumulator::new(
            "decoded_compression_transient_anchor_candidate",
            "offline_hq_compression_anchor",
        );

        candidate.record(&audio, 0.75, &draft, &offline, &improved_candidate);
        candidate.record(&audio, 0.75, &draft, &offline, &regressed_candidate);
        let line = candidate.format_report_line();

        assert!(line.contains("candidate_better_rows=1"));
        assert!(line.contains("current_better_rows=1"));
        assert!(line.contains("best_candidate_improvement_delta_frames=9.000000"));
        assert!(line.contains("worst_candidate_regression_delta_frames=7.000000"));
        assert!(line.contains("worst_draft_regression_delta_frames=18.000000"));
        assert!(line.contains("baseline_worst_draft_regression_delta_frames=11.000000"));
        assert!(line.contains("baseline_worst_draft_smear_frames=2.000000"));
        assert!(line.contains("baseline_worst_current_smear_frames=13.000000"));
        assert!(line.contains("baseline_worst_candidate_smear_frames=4.000000"));
    }

    #[test]
    fn compression_review_feature_rows_capture_short_window_outcomes() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:drums_percussion".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: vec![0.0; 8],
            analysis_limited: false,
        };
        let draft = transient_smear_measurement(12.0);
        let current = transient_smear_measurement(10.0);
        let improved = transient_smear_measurement(4.0);
        let unchanged = transient_smear_measurement(10.0);
        let worsened = transient_smear_measurement(16.0);
        let mut candidate = CompressionReviewCandidateAccumulator::new(
            "decoded_compression_short_window_candidate",
            "offline_hq_short_window",
        )
        .with_feature_report("decoded_compression_short_window_feature");

        candidate.record(&audio, 0.75, &draft, &current, &improved);
        candidate.record(&audio, 0.75, &draft, &current, &unchanged);
        candidate.record(&audio, 0.75, &draft, &current, &worsened);
        let lines = candidate.format_feature_lines();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("outcome=CandidateBetter"));
        assert!(lines[0].contains("candidate_delta_frames=-6.000000"));
        assert!(lines[0].contains("current_vs_draft_delta_frames=-2.000000"));
        assert!(lines[0].contains("candidate_path=offline_hq_short_window"));
        assert!(lines[1].contains("outcome=CurrentBetter"));
        assert!(lines[1].contains("candidate_delta_frames=6.000000"));
    }

    #[test]
    fn expansion_review_candidate_counts_only_expansion_rows() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:bass".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: vec![0.0; 8],
            analysis_limited: false,
        };
        let draft = transient_smear_measurement(1024.0);
        let current = transient_smear_measurement(49.0);
        let improved = transient_smear_measurement(10.0);
        let worsened = transient_smear_measurement(64.0);
        let mut candidate = CompressionReviewCandidateAccumulator::new_expansion(
            "decoded_expansion_short_window_candidate",
            "offline_hq_short_window",
        )
        .with_feature_report("decoded_expansion_short_window_feature");

        candidate.record(&audio, 0.75, &draft, &current, &improved);
        candidate.record(&audio, 1.25, &draft, &current, &improved);
        candidate.record(&audio, 1.5, &draft, &current, &worsened);
        let line = candidate.format_report_line();
        let feature_lines = candidate.format_feature_lines();

        assert!(line.contains("rows=2"));
        assert!(line.contains("ratio_scope=expansion"));
        assert!(line.contains("candidate_better_rows=1"));
        assert!(line.contains("current_better_rows=1"));
        assert!(line.contains("best_candidate_improvement_delta_frames=39.000000"));
        assert!(line.contains("worst_candidate_regression_delta_frames=15.000000"));
        assert_eq!(feature_lines.len(), 2);
        assert!(feature_lines[0].contains("ratio=1.250000"));
        assert!(feature_lines[1].contains("ratio=1.500000"));
    }

    #[test]
    fn expansion_selector_gate_rejects_current_better_cases() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:bass".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: vec![0.0; 8],
            analysis_limited: false,
        };
        let draft = transient_smear_measurement(1024.0);
        let mut missed_current = transient_smear_measurement(1024.0);
        missed_current.missed_transients = 1;
        let draft_regressed_current = transient_smear_measurement(24.0);
        let clean_current = transient_smear_measurement(49.0);
        let strong_candidate = transient_smear_measurement(6.0);
        let safer_candidate = transient_smear_measurement(0.0);
        let regressed_candidate = transient_smear_measurement(64.0);
        let mut selector = ExpansionShortWindowSelectorCandidateAccumulator::default();

        selector.record(&audio, 1.25, &draft, &missed_current, &strong_candidate);
        selector.record(
            &audio,
            1.25,
            &transient_smear_measurement(8.0),
            &draft_regressed_current,
            &safer_candidate,
        );
        selector.record(&audio, 1.25, &draft, &clean_current, &regressed_candidate);
        let line = selector.format_report_line();

        assert!(line.contains("rows=3"));
        assert!(line.contains("accepted_rows=2"));
        assert!(line.contains("rejected_rows=1"));
        assert!(line.contains("accepted_by_missed_transients_rows=1"));
        assert!(line.contains("accepted_by_draft_regression_rows=1"));
        assert!(line.contains("accepted_candidate_better_rows=2"));
        assert!(line.contains("accepted_current_better_rows=0"));
        assert!(line.contains("rejected_current_better_rows=1"));
        assert!(line.contains("gated_better_rows=2"));
        assert!(line.contains("current_better_rows=0"));
        assert!(line.contains("accepted_candidate_regression_delta_frames=0.000000"));
    }

    #[test]
    fn short_window_selector_gate_counts_accepted_and_rejected_outcomes() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:bass".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: vec![0.0; 8],
            analysis_limited: false,
        };
        let draft = transient_smear_measurement(1_024.0);
        let mut missed_current = transient_smear_measurement(1_024.0);
        missed_current.missed_transients = 1;
        let high_smear_current = transient_smear_measurement(90.0);
        let mild_current = transient_smear_measurement(22.0);
        let moderate_current = transient_smear_measurement(43.0);
        let strong_candidate = transient_smear_measurement(5.0);
        let high_smear_candidate = transient_smear_measurement(49.0);
        let mild_candidate = transient_smear_measurement(4.0);
        let regressed_candidate = transient_smear_measurement(81.0);
        let mut selector = ShortWindowSelectorCandidateAccumulator::default();

        selector.record(&audio, 0.75, &draft, &missed_current, &strong_candidate);
        selector.record(
            &audio,
            0.75,
            &draft,
            &high_smear_current,
            &high_smear_candidate,
        );
        selector.record(&audio, 0.75, &draft, &mild_current, &mild_candidate);
        selector.record(
            &audio,
            0.75,
            &draft,
            &moderate_current,
            &regressed_candidate,
        );
        let line = selector.format_report_line();

        assert!(line.contains("accepted_rows=2"));
        assert!(line.contains("rejected_rows=2"));
        assert!(line.contains("accepted_by_missed_transients_rows=1"));
        assert!(line.contains("accepted_by_current_smear_rows=2"));
        assert!(line.contains("accepted_by_both_rows=1"));
        assert!(line.contains("accepted_candidate_better_rows=2"));
        assert!(line.contains("accepted_current_better_rows=0"));
        assert!(line.contains("rejected_candidate_better_rows=1"));
        assert!(line.contains("rejected_current_better_rows=1"));
        assert!(line.contains("gated_better_rows=2"));
        assert!(line.contains("current_better_rows=0"));
        assert!(line.contains("unchanged_rows=2"));
        assert!(line.contains("mean_gated_smear_frames=29.750000"));
        assert!(line.contains("mean_current_smear_frames=294.750000"));
        assert!(line.contains("best_gated_improvement_delta_frames=1019.000000"));
        assert!(line.contains("worst_gated_regression_delta_frames=0.000000"));
        assert!(line.contains("rejected_candidate_improvement_delta_frames=18.000000"));
        assert!(line.contains("accepted_candidate_regression_delta_frames=0.000000"));
    }

    #[test]
    fn short_window_selector_path_counts_expected_output_selection() {
        let offline_output = vec![0.0, 0.25, 0.5];
        let short_window_output = vec![0.0, 0.5, 1.0];
        let mut missed_current = transient_smear_measurement(128.0);
        missed_current.missed_transients = 1;
        let mild_current = transient_smear_measurement(22.0);
        let selected_short_window = transient_smear_measurement(48.0);
        let selected_default = mild_current.clone();
        let mut accumulator = ShortWindowSelectorPathAccumulator::default();

        accumulator.record(
            0.75,
            &offline_output,
            &short_window_output,
            &short_window_output,
            &missed_current,
            &selected_short_window,
            &selected_short_window,
        );
        accumulator.record(
            0.75,
            &offline_output,
            &short_window_output,
            &offline_output,
            &mild_current,
            &selected_short_window,
            &selected_default,
        );
        let line = accumulator.format_report_line();

        assert!(line.contains("selected_short_window_rows=1"));
        assert!(line.contains("selected_default_rows=1"));
        assert!(line.contains("output_match_rows=2"));
        assert!(line.contains("output_mismatch_rows=0"));
        assert!(line.contains("smear_match_rows=2"));
        assert!(line.contains("smear_mismatch_rows=0"));
    }

    #[test]
    fn matched_width_review_keeps_residual_widening_visible() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:pads_sustains".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 1,
            samples: vec![0.0; 8],
            analysis_limited: false,
        };
        let draft = transient_smear_measurement(2.0);
        let mut offline = transient_smear_measurement(13.0);
        offline.max_matched_input_width_frames = 7.0;
        offline.max_matched_output_width_frames = 20.0;
        let mut short_window = transient_smear_measurement(13.0);
        short_window.max_matched_input_width_frames = 7.0;
        short_window.max_matched_output_width_frames = 20.0;
        let mut selector = transient_smear_measurement(13.0);
        selector.max_matched_input_width_frames = 7.0;
        selector.max_matched_output_width_frames = 20.0;
        let mut accumulator = MatchedTransientWidthReviewAccumulator::default();

        accumulator.record(&audio, 0.75, &draft, &offline, &short_window, &selector);
        let line = accumulator.format_report_line();

        assert!(line.contains("rows=1"));
        assert!(line.contains("finite_rows=1"));
        assert!(line.contains("offline_worse_than_draft_rows=1"));
        assert!(line.contains("selector_worse_than_draft_rows=1"));
        assert!(line.contains("selector_same_as_offline_rows=1"));
        assert!(line.contains("max_offline_vs_draft_delta_frames=11.000000"));
        assert!(line.contains("max_selector_vs_draft_delta_frames=11.000000"));
        assert!(line.contains("max_selector_residual_smear_frames=13.000000"));
        assert!(line.contains("max_selector_residual_input_width_frames=7.000000"));
        assert!(line.contains("max_selector_residual_output_width_frames=20.000000"));
    }

    #[test]
    fn transient_alignment_event_rows_are_capped_and_sorted() {
        let audio = DecodedListeningSourceAudio {
            case_id: "stretch:bass".to_string(),
            source_path: "target/source.wav".to_string(),
            sample_rate_hz: 48_000,
            channels: 2,
            samples: vec![0.0; 4_096],
            analysis_limited: true,
        };
        let alignment = TransientAlignmentDiagnostic {
            mean_match_error_frames: f64::NAN,
            max_match_error_frames: f64::NAN,
            mean_missed_nearest_distance_frames: 2_000.0,
            max_missed_nearest_distance_frames: 3_000.0,
            max_missed_expected_output_frame: 4_000.0,
            max_missed_nearest_output_frame: 7_000.0,
            missed_events: vec![
                TransientAlignmentMissEvent {
                    input_frame: 128,
                    expected_output_frame: 160.0,
                    nearest_output_frame: Some(3_160.0),
                    nearest_distance_frames: 3_000.0,
                    input_window_peak: 0.75,
                    input_window_rms: 0.25,
                    expected_output_window_peak: 0.10,
                    expected_output_window_rms: 0.05,
                    nearest_output_window_peak: 0.70,
                    nearest_output_window_rms: 0.20,
                    expected_detector_shape: DetectorShape {
                        frame_index: 256.0,
                        energy_score: 0.25,
                        spectral_flux_score: 0.75,
                        combined_score: 1.0,
                        previous_combined_score: 0.5,
                        next_combined_score: 0.25,
                    },
                    nearest_detector_shape: DetectorShape {
                        frame_index: 3_072.0,
                        energy_score: 2.0,
                        spectral_flux_score: 2.5,
                        combined_score: 4.5,
                        previous_combined_score: 1.0,
                        next_combined_score: 1.0,
                    },
                },
                TransientAlignmentMissEvent {
                    input_frame: 256,
                    expected_output_frame: 320.0,
                    nearest_output_frame: Some(1_320.0),
                    nearest_distance_frames: 1_000.0,
                    input_window_peak: 0.5,
                    input_window_rms: 0.2,
                    expected_output_window_peak: 0.4,
                    expected_output_window_rms: 0.1,
                    nearest_output_window_peak: 0.6,
                    nearest_output_window_rms: 0.15,
                    expected_detector_shape: DetectorShape {
                        frame_index: 256.0,
                        energy_score: 1.5,
                        spectral_flux_score: 2.5,
                        combined_score: 4.0,
                        previous_combined_score: 2.0,
                        next_combined_score: 5.0,
                    },
                    nearest_detector_shape: DetectorShape {
                        frame_index: 1_280.0,
                        energy_score: 2.0,
                        spectral_flux_score: 2.5,
                        combined_score: 4.5,
                        previous_combined_score: 1.0,
                        next_combined_score: 1.0,
                    },
                },
            ],
        };

        let lines = format_transient_alignment_event_lines(&audio, 1.25, "offline_hq", &alignment);

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("decoded_transient_alignment_event case=stretch:bass"));
        assert!(lines[0].contains(
            "backend=offline_hq rank=1 alignment_class=ExpectedEnergyWeak detector_class=CombinedBelowThreshold candidate_detector_class=CombinedBelowThreshold input_frame=128"
        ));
        assert!(lines[0].contains("expected_output_frame=160.000000"));
        assert!(lines[0].contains("nearest_output_frame=3160.000000"));
        assert!(lines[0].contains("nearest_distance_frames=3000.000000"));
        assert!(lines[0].contains("tolerance_frames=1024"));
        assert!(lines[0].contains("input_window_peak=0.750000"));
        assert!(lines[0].contains("expected_output_window_peak=0.100000"));
        assert!(lines[0].contains("expected_output_peak_ratio=0.133333"));
        assert!(lines[0].contains("expected_detector_frame=256.000000"));
        assert!(lines[0].contains("expected_energy_score=0.250000"));
        assert!(lines[0].contains("expected_flux_score=0.750000"));
        assert!(lines[0].contains("expected_combined_score=1.000000"));
        assert!(lines[0].contains("expected_combined_margin=-2.000000"));
        assert!(lines[0].contains("expected_flux_margin=-1.250000"));
        assert!(lines[0].contains("expected_local_previous_margin=0.500000"));
        assert!(lines[0].contains("expected_local_next_margin=0.750000"));
        assert!(lines[0].contains("candidate_combined_margin=-1.000000"));
        assert!(lines[0].contains("candidate_flux_margin=-0.750000"));
        assert!(lines[0].contains("nearest_output_window_peak=0.700000"));
        assert!(lines[0].contains("nearest_output_peak_ratio=0.933333"));
        assert!(lines[0].contains("nearest_combined_score=4.500000"));
        assert!(lines[1].contains("candidate_detector_class=NotLocalMaximum"));
    }

    fn write_test_wav(path: &PathBuf, frames: usize) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create target dir");
        }
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec).expect("create test wav");
        for frame in 0..frames {
            let sample = if frame % 2 == 0 { 0.25 } else { -0.25 };
            writer.write_sample(sample).expect("write left sample");
            writer.write_sample(sample).expect("write right sample");
        }
        writer.finalize().expect("finalize test wav");
    }

    fn write_transient_test_wav(path: &PathBuf, frames: usize) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create target dir");
        }
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec).expect("create test wav");
        for frame in 0..frames {
            let tonal = (std::f32::consts::TAU * 330.0 * frame as f32 / 48_000.0).sin() * 0.25;
            let transient = if frame % 8_000 < 64 {
                0.9 * (1.0 - (frame % 8_000) as f32 / 64.0)
            } else {
                0.0
            };
            let sample = tonal + transient;
            writer.write_sample(sample).expect("write left sample");
            writer.write_sample(sample).expect("write right sample");
        }
        writer.finalize().expect("finalize test wav");
    }
}
