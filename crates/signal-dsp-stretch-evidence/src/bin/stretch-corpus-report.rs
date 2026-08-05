//! Emit the retained Signal stretch comparator and blind-listening evidence.

use std::borrow::Cow;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

#[path = "stretch-corpus-report/alloc_tracker.rs"]
mod alloc_tracker;
#[path = "stretch-corpus-report/listening_pack.rs"]
mod listening_pack;

use alloc_tracker::measure_peak_live_heap;
use listening_pack::{export_blind_listening_pack, format_blind_listening_note_status};
use signal_dsp_stretch::{
    assess_stretch_render_integrity, build_stretch_corpus_comparison_report_with_sources,
    format_stretch_corpus_comparison_report, measure_formant_boundary,
    measure_stretch_render_integrity, measure_tonal_texture, measure_transient_detail,
    output_length_drift_samples, OfflineHighQualityPath, OfflineHighQualityStretcher,
    StretchCorpusAssetRequirement, StretchCorpusListeningSource, StretchExternalBenchmarkRender,
    StretchRenderIntegrityLimits, TimeStretcher, STRETCH_CORPUS_MANIFEST,
};
use symphonia::core::{
    audio::SampleBuffer as SymphoniaSampleBuffer,
    codecs::{DecoderOptions as SymphoniaDecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymphoniaError,
    formats::FormatOptions as SymphoniaFormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions as SymphoniaMetadataOptions,
    probe::Hint as SymphoniaHint,
};

const DEFAULT_REPORT_NAME: &str = "stretch-corpus-v1-offline-evidence";
const DEFAULT_PROJECTION_EPOCH: &str = "projection:deterministic-report-v1";
const DEFAULT_EXTERNAL_BENCHMARK_TOOL: &str = "external-render";
const DEFAULT_FRAME_LIMIT: usize = 48_000 * 60;
const QUALITY_WINDOW_SIZE: usize = 1_024;
const QUALITY_HOP_SIZE: usize = 256;
const INTEGRITY_ENDPOINT_FRAMES: usize = 1_024;
const INTEGRITY_SILENCE_THRESHOLD: f32 = 1.0e-6;
const ALIGNMENT_MAX_LAG_FRAMES: isize = 2_048;
const ALIGNMENT_MAX_COMPARE_FRAMES: usize = 65_536;

#[derive(Debug, PartialEq, Eq)]
struct ReportArgs {
    report_name: String,
    projection_epoch: String,
    output: Option<PathBuf>,
    external_benchmark_tool: String,
    external_benchmark_renders: Vec<ExternalBenchmarkRenderArg>,
    external_benchmark_render_manifests: Vec<PathBuf>,
    listening_source_manifests: Vec<PathBuf>,
    frame_limit: usize,
    measure_external_benchmark_quality: bool,
    signal_path: OfflineHighQualityPath,
    export_blind_listening_pack: Option<PathBuf>,
    blind_listening_note_manifests: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExternalBenchmarkRenderArg {
    case_id: String,
    ratio: String,
    path: PathBuf,
    tool_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkQualityRender {
    case_id: String,
    ratio: f64,
    tool_name: String,
    rendered_path: String,
    source_wav: Option<String>,
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
            listening_source_manifests: Vec::new(),
            frame_limit: DEFAULT_FRAME_LIMIT,
            measure_external_benchmark_quality: false,
            signal_path: OfflineHighQualityPath::Default,
            export_blind_listening_pack: None,
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
        Err(message) => exit_with_error(&message, true),
    };
    let sources =
        load_listening_sources(&args).unwrap_or_else(|message| exit_with_error(&message, false));
    let quality_renders = load_external_benchmark_quality_renders(&args)
        .unwrap_or_else(|message| exit_with_error(&message, false));
    let external_renders = load_external_benchmark_metadata(&quality_renders)
        .unwrap_or_else(|message| exit_with_error(&message, false));

    let report = build_stretch_corpus_comparison_report_with_sources(
        &args.report_name,
        &args.projection_epoch,
        &external_renders,
        &sources,
    );
    let mut formatted = format_stretch_corpus_comparison_report(&report);
    if args.measure_external_benchmark_quality {
        let quality = format_external_benchmark_quality_metrics(
            &sources,
            &quality_renders,
            args.frame_limit,
            args.signal_path,
        )
        .unwrap_or_else(|message| exit_with_error(&message, false));
        if !quality.is_empty() {
            formatted.push('\n');
            formatted.push_str(&quality);
        }
    }
    if let Some(export_dir) = &args.export_blind_listening_pack {
        let pack = export_blind_listening_pack(
            &sources,
            &quality_renders,
            args.frame_limit,
            args.signal_path,
            export_dir,
        )
        .unwrap_or_else(|message| exit_with_error(&message, false));
        formatted.push('\n');
        formatted.push_str(&pack);
    }
    for notes in &args.blind_listening_note_manifests {
        let status = format_blind_listening_note_status(notes)
            .unwrap_or_else(|message| exit_with_error(&message, false));
        formatted.push('\n');
        formatted.push_str(&status);
    }

    if let Some(path) = &args.output {
        fs::write(path, format!("{formatted}\n")).unwrap_or_else(|error| {
            exit_with_error(
                &format!("failed to write {}: {error}", path.display()),
                false,
            )
        });
    } else {
        println!("{formatted}");
    }
}

fn exit_with_error(message: &str, show_usage: bool) -> ! {
    eprintln!("{message}");
    if show_usage {
        eprintln!("{}", usage());
    }
    process::exit(1);
}

enum ParseOutcome {
    // Boxed: `ReportArgs` is ~232 bytes larger than `Help`, so an unboxed
    // variant would make every `ParseOutcome` pay for the argument struct.
    Run(Box<ReportArgs>),
    Help,
}

fn parse_args<I>(args: I) -> Result<ParseOutcome, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = ReportArgs::default();
    let mut iter = args.into_iter();
    while let Some(argument) = iter.next() {
        match argument.as_str() {
            "--help" | "-h" => return Ok(ParseOutcome::Help),
            "--report-name" => parsed.report_name = next_value(&mut iter, "--report-name")?,
            "--projection-epoch" => {
                parsed.projection_epoch = next_value(&mut iter, "--projection-epoch")?;
            }
            "--output" => parsed.output = Some(PathBuf::from(next_value(&mut iter, "--output")?)),
            "--listening-source-manifest" => {
                parsed
                    .listening_source_manifests
                    .push(PathBuf::from(next_value(
                        &mut iter,
                        "--listening-source-manifest",
                    )?))
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
                    })
            }
            "--external-benchmark-render-manifest" => parsed
                .external_benchmark_render_manifests
                .push(PathBuf::from(next_value(
                    &mut iter,
                    "--external-benchmark-render-manifest",
                )?)),
            "--measure-external-benchmark-quality" => {
                parsed.measure_external_benchmark_quality = true;
            }
            "--external-benchmark-signal-path" => {
                parsed.signal_path = parse_offline_high_quality_path(&next_value(
                    &mut iter,
                    "--external-benchmark-signal-path",
                )?)?;
            }
            "--frame-limit" => {
                parsed.frame_limit = next_value(&mut iter, "--frame-limit")?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --frame-limit value: {error}"))?;
            }
            "--export-blind-listening-pack" => {
                parsed.export_blind_listening_pack = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--export-blind-listening-pack",
                )?));
            }
            "--check-blind-listening-notes" => {
                parsed
                    .blind_listening_note_manifests
                    .push(PathBuf::from(next_value(
                        &mut iter,
                        "--check-blind-listening-notes",
                    )?))
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    Ok(ParseOutcome::Run(Box::new(parsed)))
}

fn next_value<I>(iter: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .ok_or_else(|| format!("missing value for {name}"))
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
            "invalid --external-benchmark-signal-path value: {value}; expected default, compression-short-window-selector, or expansion-short-window-selector"
        )),
    }
}

fn usage() -> &'static str {
    "usage: stretch-corpus-report [--report-name NAME] [--projection-epoch EPOCH] [--listening-source-manifest TSV] [--external-benchmark-render CASE RATIO WAV] [--external-benchmark-render-manifest TSV] [--external-benchmark-tool NAME] [--measure-external-benchmark-quality] [--external-benchmark-signal-path default|compression-short-window-selector|expansion-short-window-selector] [--frame-limit N] [--export-blind-listening-pack DIR] [--check-blind-listening-notes TSV] [--output PATH]"
}

fn load_external_benchmark_quality_renders(
    args: &ReportArgs,
) -> Result<Vec<ExternalBenchmarkQualityRender>, String> {
    let mut renders = args
        .external_benchmark_renders
        .iter()
        .map(|render| {
            Ok(ExternalBenchmarkQualityRender {
                case_id: render.case_id.clone(),
                ratio: parse_ratio(&render.ratio)?,
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
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(manifest)
            .map_err(|error| format!("failed to open {}: {error}", manifest.display()))?;
        let headers = reader
            .headers()
            .map_err(|error| format!("failed to read {} headers: {error}", manifest.display()))?
            .clone();
        for row in reader.records() {
            let record =
                row.map_err(|error| format!("failed to read {} row: {error}", manifest.display()))?;
            renders.push(ExternalBenchmarkQualityRender {
                case_id: required_field(manifest, &headers, &record, "case_id")?.to_string(),
                ratio: parse_ratio(required_field(manifest, &headers, &record, "ratio")?)?,
                rendered_path: required_any_field(
                    manifest,
                    &headers,
                    &record,
                    &["rendered_path", "path"],
                )?
                .to_string(),
                tool_name: field(&headers, &record, "tool_name")
                    .or_else(|| field(&headers, &record, "tool"))
                    .unwrap_or(&args.external_benchmark_tool)
                    .to_string(),
                source_wav: field(&headers, &record, "source_wav").map(str::to_string),
            });
        }
    }
    Ok(renders)
}

fn load_external_benchmark_metadata(
    renders: &[ExternalBenchmarkQualityRender],
) -> Result<Vec<StretchExternalBenchmarkRender>, String> {
    renders
        .iter()
        .map(|render| {
            let reader = hound::WavReader::open(&render.rendered_path).map_err(|error| {
                format!(
                    "failed to open external render {}: {error}",
                    render.rendered_path
                )
            })?;
            let spec = reader.spec();
            Ok(StretchExternalBenchmarkRender {
                case_id: render.case_id.clone(),
                ratio: render.ratio,
                pitch_shift_semitones: None,
                tool_name: render.tool_name.clone(),
                rendered_path: render.rendered_path.clone(),
                rendered_frames: reader.duration() as usize,
                sample_rate_hz: spec.sample_rate,
                channels: spec.channels,
            })
        })
        .collect()
}

fn parse_ratio(value: &str) -> Result<f64, String> {
    let ratio = value
        .parse::<f64>()
        .map_err(|error| format!("invalid external benchmark ratio {value}: {error}"))?;
    if ratio.is_finite() && ratio > 0.0 {
        Ok(ratio)
    } else {
        Err(format!(
            "invalid external benchmark ratio {value}: expected positive finite value"
        ))
    }
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
        if !STRETCH_CORPUS_MANIFEST.entries.iter().any(|entry| {
            entry.case.case_id == case_id
                && entry.asset_requirement == StretchCorpusAssetRequirement::OperatorProvidedAudio
        }) {
            return Err(format!(
                "{} uses unsupported listening case id {case_id}",
                manifest.display()
            ));
        }
        let source_path =
            required_any_field(manifest, &headers, &record, &["source_path", "local_path"])?;
        if !Path::new(source_path).exists() {
            return Err(format!(
                "{} references missing source {source_path}",
                manifest.display()
            ));
        }
        sources.push(StretchCorpusListeningSource {
            case_id: case_id.to_string(),
            source_path: source_path.to_string(),
            source_label: field(&headers, &record, "source_label")
                .unwrap_or("operator listening source")
                .to_string(),
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

fn required_any_field<'a>(
    manifest: &Path,
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
    manifest: &Path,
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

#[derive(Clone, Debug, PartialEq)]
struct DecodedListeningSourceAudio {
    source_path: String,
    sample_rate_hz: u32,
    channels: u16,
    samples: Vec<f32>,
}

impl DecodedListeningSourceAudio {
    fn analyzed_frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    fn mono_samples(&self) -> Vec<f32> {
        let channels = self.channels as usize;
        self.samples
            .chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    }
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
        return Err(format!("invalid source WAV {}", path.display()));
    }
    let limit = sample_limit(frame_limit, spec.channels as usize).unwrap_or(usize::MAX);
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .take(limit)
            .map(|sample| {
                sample.map_err(|error| format!("failed to read {}: {error}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = integer_sample_scale(spec.bits_per_sample);
            reader
                .samples::<i32>()
                .take(limit)
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / scale)
                        .map_err(|error| format!("failed to read {}: {error}", path.display()))
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    decoded_audio(source, spec.sample_rate, spec.channels, samples)
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
    let mut frames = 0usize;
    loop {
        if frame_limit > 0 && frames >= frame_limit {
            break;
        }
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => return Err(format!("failed to decode {}: {error}", path.display())),
        };
        let spec = *decoded.spec();
        let actual_channels = spec.channels.count();
        if actual_channels == 0 || spec.rate == 0 {
            continue;
        }
        let channels = actual_channels.clamp(1, 2);
        sample_rate_hz.get_or_insert(spec.rate);
        output_channels.get_or_insert(channels as u16);
        let mut buffer = SymphoniaSampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buffer.copy_interleaved_ref(decoded);
        for frame in buffer.samples().chunks(actual_channels) {
            if frame_limit > 0 && frames >= frame_limit {
                break;
            }
            samples.extend(frame.iter().take(channels));
            frames += 1;
        }
    }
    decoded_audio(
        source,
        sample_rate_hz
            .ok_or_else(|| format!("source {} produced no sample rate", path.display()))?,
        output_channels.ok_or_else(|| format!("source {} produced no channels", path.display()))?,
        samples,
    )
}

fn decoded_audio(
    source: &StretchCorpusListeningSource,
    sample_rate_hz: u32,
    channels: u16,
    samples: Vec<f32>,
) -> Result<DecodedListeningSourceAudio, String> {
    if sample_rate_hz == 0 || channels == 0 || samples.len() < channels as usize {
        return Err(format!(
            "source {} produced no decodable audio",
            source.source_path
        ));
    }
    Ok(DecodedListeningSourceAudio {
        source_path: source.source_path.clone(),
        sample_rate_hz,
        channels,
        samples,
    })
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

fn decode_external_benchmark_render_audio(
    render: &ExternalBenchmarkQualityRender,
) -> Result<ExternalBenchmarkDecodedAudio, String> {
    let path = PathBuf::from(&render.rendered_path);
    let mut reader = hound::WavReader::open(&path)
        .map_err(|error| format!("failed to open external render {}: {error}", path.display()))?;
    let spec = reader.spec();
    if spec.channels == 0 || spec.sample_rate == 0 {
        return Err(format!("invalid external render WAV {}", path.display()));
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
    let channels = spec.channels as usize;
    let mono_samples = samples
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect();
    Ok(ExternalBenchmarkDecodedAudio {
        sample_rate_hz: spec.sample_rate,
        channels: spec.channels,
        samples,
        mono_samples,
    })
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

fn format_external_benchmark_quality_metrics(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    signal_path: OfflineHighQualityPath,
) -> Result<String, String> {
    let limits = StretchRenderIntegrityLimits::offline_high_quality();
    let mut lines = Vec::new();
    for render in renders {
        let source = match source_for_external_quality_render(sources, render) {
            ExternalBenchmarkQualitySource::Found(source) => source,
            ExternalBenchmarkQualitySource::Missing => {
                lines.push(format_quality_skip(
                    render,
                    signal_path,
                    "MissingListeningSource",
                ));
                continue;
            }
            ExternalBenchmarkQualitySource::Ambiguous => {
                lines.push(format_quality_skip(
                    render,
                    signal_path,
                    "AmbiguousListeningSource",
                ));
                continue;
            }
        };
        let source_audio = decode_listening_source_audio(source.as_ref(), frame_limit)?;
        let external = decode_external_benchmark_render_audio(render)?;
        if source_audio.sample_rate_hz != external.sample_rate_hz {
            lines.push(format_quality_skip(
                render,
                signal_path,
                "SampleRateMismatch",
            ));
            continue;
        }
        let source_mono = source_audio.mono_samples();
        let ((signal, render_seconds), heap) = measure_peak_live_heap(|| {
            let started = Instant::now();
            let output = OfflineHighQualityStretcher::with_path(render.ratio, signal_path)
                .stretch_mono(&source_mono)
                .expect("render fits the offline output bound");
            (output, started.elapsed().as_secs_f64())
        });
        let signal_transient = measure_transient_detail(
            &source_mono,
            &signal,
            render.ratio,
            QUALITY_WINDOW_SIZE,
            QUALITY_HOP_SIZE,
        );
        let external_transient = measure_transient_detail(
            &source_mono,
            &external.mono_samples,
            render.ratio,
            QUALITY_WINDOW_SIZE,
            QUALITY_HOP_SIZE,
        );
        let signal_tonal = measure_tonal_texture(&source_mono, &signal, render.ratio);
        let external_tonal =
            measure_tonal_texture(&source_mono, &external.mono_samples, render.ratio);
        let signal_formant = measure_formant_boundary(
            &source_mono,
            &signal,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let external_formant = measure_formant_boundary(
            &source_mono,
            &external.mono_samples,
            render.ratio,
            source_audio.sample_rate_hz,
        );
        let signal_integrity = measure_stretch_render_integrity(
            &source_mono,
            &signal,
            render.ratio,
            INTEGRITY_ENDPOINT_FRAMES,
            INTEGRITY_SILENCE_THRESHOLD,
        );
        let external_integrity = measure_stretch_render_integrity(
            &source_mono,
            &external.mono_samples,
            render.ratio,
            INTEGRITY_ENDPOINT_FRAMES,
            INTEGRITY_SILENCE_THRESHOLD,
        );
        let aligned = align_and_measure_error(&signal, &external.mono_samples);
        let rendered_seconds = signal.len() as f64 / source_audio.sample_rate_hz as f64;
        lines.push(format!(
            "external_benchmark_quality case={} source={} signal_path={:?} render={} tool={} ratio={:.6} status=Measured source_sample_rate_hz={} external_channels={} source_frames={} signal_frames={} external_frames={} signal_timing_drift_samples={:.6} external_timing_drift_samples={:.6} alignment_lag_frames={} aligned_frames={} aligned_correlation={:.6} aligned_rms_error={:.6} aligned_rms_error_ratio={:.6} signal_transient_matches={} external_transient_matches={} signal_transient_mean_absolute_offset_frames={:.6} external_transient_mean_absolute_offset_frames={:.6} signal_transient_max_crest_growth_db={:.6} external_transient_max_crest_growth_db={:.6} signal_tonal_residual_ratio={:.6} external_tonal_residual_ratio={:.6} signal_added_sideband_ratio={:.6} external_added_sideband_ratio={:.6} signal_formant_residual_ratio={:.6} external_formant_residual_ratio={:.6} signal_formant_centroid_shift_hz={:.6} external_formant_centroid_shift_hz={:.6} signal_boundary_step_growth_db={:.6} external_boundary_step_growth_db={:.6} signal_integrity_passed={} external_integrity_passed={} signal_endpoint_energy_delta_db={:.6} external_endpoint_energy_delta_db={:.6} signal_added_silence_frames={} external_added_silence_frames={} signal_peak_growth_db={:.6} external_peak_growth_db={:.6} signal_render_seconds={:.6} signal_cpu_realtime_factor={:.6} signal_peak_working_memory_bytes={}",
            render.case_id,
            quoted_report_field(&source_audio.source_path),
            signal_path,
            quoted_report_field(&render.rendered_path),
            quoted_report_field(&render.tool_name),
            render.ratio,
            source_audio.sample_rate_hz,
            external.channels,
            source_audio.analyzed_frames(),
            signal.len(),
            external.frames(),
            output_length_drift_samples(source_mono.len(), signal.len(), render.ratio),
            output_length_drift_samples(source_mono.len(), external.frames(), render.ratio),
            aligned.lag_frames,
            aligned.compared_frames,
            aligned.correlation,
            aligned.rms_error,
            finite_ratio(aligned.rms_error, aligned.external_rms),
            signal_transient.matched_transients,
            external_transient.matched_transients,
            signal_transient.mean_absolute_timing_offset_frames,
            external_transient.mean_absolute_timing_offset_frames,
            signal_transient.max_transient_crest_growth_db,
            external_transient.max_transient_crest_growth_db,
            signal_tonal.mean_spectral_residual_ratio,
            external_tonal.mean_spectral_residual_ratio,
            signal_tonal.mean_added_sideband_ratio,
            external_tonal.mean_added_sideband_ratio,
            signal_formant.mean_envelope_residual_ratio,
            external_formant.mean_envelope_residual_ratio,
            signal_formant.mean_envelope_centroid_shift_hz,
            external_formant.mean_envelope_centroid_shift_hz,
            signal_formant.max_boundary_step_crest_growth_db,
            external_formant.max_boundary_step_crest_growth_db,
            assess_stretch_render_integrity(signal_integrity, limits).passed,
            assess_stretch_render_integrity(external_integrity, limits).passed,
            signal_integrity.endpoint_energy_delta_db,
            external_integrity.endpoint_energy_delta_db,
            signal_integrity.added_silence_frames,
            external_integrity.added_silence_frames,
            signal_integrity.peak_growth_db,
            external_integrity.peak_growth_db,
            render_seconds,
            finite_ratio(render_seconds, rendered_seconds),
            heap.peak_growth_bytes,
        ));
    }
    Ok(lines.join("\n"))
}

fn format_quality_skip(
    render: &ExternalBenchmarkQualityRender,
    signal_path: OfflineHighQualityPath,
    reason: &str,
) -> String {
    format!(
        "external_benchmark_quality case={} signal_path={:?} render={} tool={} ratio={:.6} status=Skipped reason={reason}",
        render.case_id,
        signal_path,
        quoted_report_field(&render.rendered_path),
        quoted_report_field(&render.tool_name),
        render.ratio,
    )
}

#[derive(Clone, Debug, PartialEq)]
struct AlignedErrorMeasurement {
    lag_frames: isize,
    compared_frames: usize,
    correlation: f64,
    rms_error: f64,
    external_rms: f64,
}

fn align_and_measure_error(signal: &[f32], external: &[f32]) -> AlignedErrorMeasurement {
    let mut best_lag = 0isize;
    let mut best_correlation = f64::NEG_INFINITY;
    for lag in -ALIGNMENT_MAX_LAG_FRAMES..=ALIGNMENT_MAX_LAG_FRAMES {
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
            best_lag = lag;
            best_correlation = correlation;
        }
    }
    let Some((signal_start, external_start, frames)) = aligned_ranges(signal, external, best_lag)
    else {
        return AlignedErrorMeasurement {
            lag_frames: 0,
            compared_frames: 0,
            correlation: f64::NAN,
            rms_error: f64::NAN,
            external_rms: f64::NAN,
        };
    };
    let mut error_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    for (signal_sample, external_sample) in signal[signal_start..signal_start + frames]
        .iter()
        .zip(&external[external_start..external_start + frames])
    {
        let error = *signal_sample as f64 - *external_sample as f64;
        error_square_sum += error * error;
        external_square_sum += (*external_sample as f64) * (*external_sample as f64);
    }
    AlignedErrorMeasurement {
        lag_frames: best_lag,
        compared_frames: frames,
        correlation: best_correlation,
        rms_error: (error_square_sum / frames as f64).sqrt(),
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
        .min(ALIGNMENT_MAX_COMPARE_FRAMES);
    (frames > 0).then_some((signal_start, external_start, frames))
}

fn normalized_correlation(signal: &[f32], external: &[f32]) -> f64 {
    let mut dot = 0.0;
    let mut signal_square_sum = 0.0;
    let mut external_square_sum = 0.0;
    for (signal_sample, external_sample) in signal.iter().zip(external) {
        dot += *signal_sample as f64 * *external_sample as f64;
        signal_square_sum += (*signal_sample as f64) * (*signal_sample as f64);
        external_square_sum += (*external_sample as f64) * (*external_sample as f64);
    }
    finite_ratio(dot, (signal_square_sum * external_square_sum).sqrt())
}

fn finite_ratio(numerator: f64, denominator: f64) -> f64 {
    if numerator.is_finite() && denominator.is_finite() && denominator.abs() > 1.0e-12 {
        numerator / denominator
    } else {
        f64::NAN
    }
}

fn sample_limit(frame_limit: usize, channels: usize) -> Option<usize> {
    (frame_limit > 0).then(|| frame_limit.saturating_mul(channels))
}

fn integer_sample_scale(bits_per_sample: u16) -> f32 {
    if bits_per_sample == 0 {
        1.0
    } else {
        2_f32.powi(i32::from(bits_per_sample) - 1)
    }
}

fn quoted_report_field(value: &str) -> String {
    format!("{:?}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_keeps_only_retained_quality_and_listening_surfaces() {
        let ParseOutcome::Run(args) = parse_args([
            "--measure-external-benchmark-quality".to_string(),
            "--external-benchmark-signal-path".to_string(),
            "expansion-short-window-selector".to_string(),
            "--frame-limit".to_string(),
            "48000".to_string(),
        ])
        .expect("parse") else {
            panic!("expected run");
        };
        assert!(args.measure_external_benchmark_quality);
        assert_eq!(
            args.signal_path,
            OfflineHighQualityPath::ExpansionShortWindowSelector
        );
        assert_eq!(args.frame_limit, 48_000);
    }

    #[test]
    fn alignment_recovers_known_lag() {
        let mut signal = vec![0.0; 128];
        let mut external = vec![0.0; 128];
        for index in 16..96 {
            signal[index] = (index as f32 * 0.17).sin();
            external[index + 5] = signal[index];
        }
        let aligned = align_and_measure_error(&signal, &external);
        assert_eq!(aligned.lag_frames, 5);
        assert!(aligned.correlation > 0.999);
    }
}
