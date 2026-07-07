//! Emit deterministic Signal stretch corpus evidence reports.

use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process;

use rustfft::{num_complex::Complex32, FftPlanner};
use signal_dsp_stretch::{
    build_stretch_corpus_comparison_report_with_sources, detect_stretch_transients,
    format_stretch_corpus_comparison_report, measure_transient_smear,
    measure_transient_smear_with_output_recovery_policy, measure_transient_smear_with_policies,
    measure_transient_smear_with_policy, output_length_drift_samples, OfflineHighQualityStretcher,
    PhaseVocoderStretcher, StretchCorpusAssetRequirement, StretchCorpusListeningSource,
    StretchExternalBenchmarkRender, StretchTransientDetectorPolicy, TimeStretcher,
    STRETCH_CORPUS_MANIFEST,
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
const DEFAULT_DECODE_SOURCE_FRAME_LIMIT: usize = 48_000 * 60;
const DEFAULT_DECODED_STRETCH_FRAME_LIMIT: usize = 48_000 * 10;
const QUALITY_METRIC_WINDOW_SIZE: usize = 1_024;
const QUALITY_METRIC_HOP_SIZE: usize = 256;
const COMPRESSION_SHORT_WINDOW_REVIEW_WINDOW_SIZE: usize = 1_024;
const COMPRESSION_SHORT_WINDOW_REVIEW_ANALYSIS_HOP: usize =
    COMPRESSION_SHORT_WINDOW_REVIEW_WINDOW_SIZE / 4;
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
const SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES: usize = 1;
const SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES: f64 = 64.0;
const EXTERNAL_BENCHMARK_ALIGNMENT_MAX_LAG_FRAMES: isize = 2_048;
const EXTERNAL_BENCHMARK_ALIGNMENT_MAX_COMPARE_FRAMES: usize = 65_536;
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
    listening_source_manifests: Vec<PathBuf>,
    decode_listening_sources: bool,
    decode_source_frame_limit: usize,
    measure_decoded_stretch: bool,
    decoded_stretch_frame_limit: usize,
    measure_external_benchmark_quality: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExternalBenchmarkRenderArg {
    case_id: String,
    ratio: String,
    path: PathBuf,
}

impl Default for ReportArgs {
    fn default() -> Self {
        Self {
            report_name: DEFAULT_REPORT_NAME.to_string(),
            projection_epoch: DEFAULT_PROJECTION_EPOCH.to_string(),
            output: None,
            external_benchmark_tool: DEFAULT_EXTERNAL_BENCHMARK_TOOL.to_string(),
            external_benchmark_renders: Vec::new(),
            listening_source_manifests: Vec::new(),
            decode_listening_sources: false,
            decode_source_frame_limit: DEFAULT_DECODE_SOURCE_FRAME_LIMIT,
            measure_decoded_stretch: false,
            decoded_stretch_frame_limit: DEFAULT_DECODED_STRETCH_FRAME_LIMIT,
            measure_external_benchmark_quality: false,
        }
    }
}

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(ParseOutcome::Run(args)) => args,
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
        match format_decoded_stretch_metrics(&listening_sources, args.decoded_stretch_frame_limit) {
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
        match format_external_benchmark_quality_metrics(
            &listening_sources,
            &external_renders,
            args.decoded_stretch_frame_limit,
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
    Run(ReportArgs),
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
                    });
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
            "--measure-external-benchmark-quality" => {
                parsed.measure_external_benchmark_quality = true;
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
    Ok(ParseOutcome::Run(parsed))
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
    "usage: stretch-corpus-report [--report-name NAME] [--projection-epoch EPOCH] [--listening-source-manifest TSV] [--decode-listening-sources] [--decode-source-frame-limit N] [--measure-decoded-stretch] [--measure-external-benchmark-quality] [--decoded-stretch-frame-limit N] [--external-benchmark-tool NAME] [--external-benchmark-render CASE RATIO WAV] [--output PATH]"
}

fn load_external_benchmark_renders(
    args: &ReportArgs,
) -> Result<Vec<StretchExternalBenchmarkRender>, String> {
    args.external_benchmark_renders
        .iter()
        .map(|render| load_external_benchmark_render(args, render))
        .collect()
}

fn load_external_benchmark_render(
    args: &ReportArgs,
    render: &ExternalBenchmarkRenderArg,
) -> Result<StretchExternalBenchmarkRender, String> {
    let ratio = render
        .ratio
        .parse::<f64>()
        .map_err(|error| format!("invalid external benchmark ratio {}: {error}", render.ratio))?;
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(format!(
            "invalid external benchmark ratio {}: expected positive finite value",
            render.ratio
        ));
    }

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
        tool_name: args.external_benchmark_tool.clone(),
        rendered_path: render.path.display().to_string(),
        rendered_frames,
        sample_rate_hz: spec.sample_rate,
        channels,
    })
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
    let mut compression_short_window_selector_candidate =
        ShortWindowSelectorCandidateAccumulator::default();
    let mut width_control_candidate = TransientWidthControlCandidateAccumulator::default();
    let mut width_control_edit_gate = TransientWidthControlEditGateAccumulator::default();
    for source in sources {
        let audio = decode_listening_source_audio(source, frame_limit)?;
        let mono = audio.mono_samples();
        for &ratio in listening_source_ratios(&source.case_id)? {
            let mut draft = PhaseVocoderStretcher::new(ratio);
            let draft_output = draft.stretch_mono(&mono);
            let mut offline = OfflineHighQualityStretcher::new(ratio);
            let offline_output = offline.stretch_mono(&mono);

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
                COMPRESSION_SHORT_WINDOW_REVIEW_WINDOW_SIZE,
                COMPRESSION_SHORT_WINDOW_REVIEW_ANALYSIS_HOP,
            );
            let offline_short_window_output = offline_short_window.stretch_mono(&mono);
            let offline_short_window_smear = measure_transient_smear(
                &mono,
                &offline_short_window_output,
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
            compression_short_window_selector_candidate.record(
                &audio,
                ratio,
                &draft_smear,
                &offline_smear,
                &offline_short_window_smear,
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
    if compression_short_window_selector_candidate.rows > 0 {
        lines.push(compression_short_window_selector_candidate.format_report_line());
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
    alignment_lag_frames: isize,
    aligned_compared_frames: usize,
    aligned_correlation: f64,
    aligned_rms_error: f64,
    aligned_peak_error: f64,
    signal_rms: f64,
    external_rms: f64,
    aligned_rms_error_ratio: f64,
}

impl ExternalBenchmarkQualityMeasurement {
    fn format_report_line(&self) -> String {
        format!(
            "external_benchmark_quality case={} source={} ratio={:.6} tool={} render={} status={} reason={} source_boundary={} sample_rate_match={} source_sample_rate={} external_sample_rate={} external_channels={} source_frames={} signal_frames={} external_frames={} signal_timing_drift_samples={:.6} external_timing_drift_samples={:.6} timing_drift_delta_samples={:.6} signal_transient_smear_frames={:.6} external_transient_smear_frames={:.6} transient_smear_delta_frames={:.6} alignment_lag_frames={} aligned_compared_frames={} aligned_correlation={:.6} aligned_rms_error={:.9} aligned_peak_error={:.9} signal_rms={:.9} external_rms={:.9} aligned_rms_error_ratio={:.6}",
            self.case_id,
            quoted_report_field(&self.source_path),
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
            self.alignment_lag_frames,
            self.aligned_compared_frames,
            self.aligned_correlation,
            self.aligned_rms_error,
            self.aligned_peak_error,
            self.signal_rms,
            self.external_rms,
            self.aligned_rms_error_ratio,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalBenchmarkDecodedAudio {
    sample_rate_hz: u32,
    channels: u16,
    mono_samples: Vec<f32>,
}

impl ExternalBenchmarkDecodedAudio {
    fn frames(&self) -> usize {
        self.mono_samples.len()
    }
}

fn format_external_benchmark_quality_metrics(
    sources: &[StretchCorpusListeningSource],
    renders: &[StretchExternalBenchmarkRender],
    frame_limit: usize,
) -> Result<String, String> {
    let mut lines = Vec::new();
    for render in renders {
        let source = match sources
            .iter()
            .find(|source| source.case_id == render.case_id)
        {
            Some(source) => source,
            None => {
                lines.push(format_external_benchmark_quality_skip_line(
                    render,
                    "",
                    "MissingListeningSource",
                    0,
                    0,
                    0,
                    0,
                ));
                continue;
            }
        };
        let source_audio = decode_listening_source_audio(source, frame_limit)?;
        let external_audio = decode_external_benchmark_render_audio(render)?;
        if external_audio.frames() == 0 {
            lines.push(format_external_benchmark_quality_skip_line(
                render,
                &source.source_path,
                "NoComparatorAudio",
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
                source_audio.sample_rate_hz,
                external_audio.sample_rate_hz,
                external_audio.channels,
                source_audio.analyzed_frames(),
            ));
            continue;
        }

        let source_mono = source_audio.mono_samples();
        let mut signal = OfflineHighQualityStretcher::new(render.ratio);
        let signal_output = signal.stretch_mono(&source_mono);
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
        let signal_timing_drift =
            output_length_drift_samples(source_mono.len(), signal_output.len(), render.ratio);
        let external_timing_drift = output_length_drift_samples(
            source_mono.len(),
            external_audio.mono_samples.len(),
            render.ratio,
        );
        let aligned = align_and_measure_error(&signal_output, &external_audio.mono_samples);

        lines.push(
            ExternalBenchmarkQualityMeasurement {
                case_id: render.case_id.clone(),
                source_path: source.source_path.clone(),
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
                alignment_lag_frames: aligned.lag_frames,
                aligned_compared_frames: aligned.compared_frames,
                aligned_correlation: aligned.correlation,
                aligned_rms_error: aligned.rms_error,
                aligned_peak_error: aligned.peak_error,
                signal_rms: aligned.signal_rms,
                external_rms: aligned.external_rms,
                aligned_rms_error_ratio: finite_ratio(aligned.rms_error, aligned.external_rms),
            }
            .format_report_line(),
        );
    }
    Ok(lines.join("\n"))
}

fn format_external_benchmark_quality_skip_line(
    render: &StretchExternalBenchmarkRender,
    source_path: &str,
    reason: &'static str,
    source_sample_rate_hz: u32,
    external_sample_rate_hz: u32,
    external_channels: u16,
    source_frames: usize,
) -> String {
    ExternalBenchmarkQualityMeasurement {
        case_id: render.case_id.clone(),
        source_path: source_path.to_string(),
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
        alignment_lag_frames: 0,
        aligned_compared_frames: 0,
        aligned_correlation: f64::NAN,
        aligned_rms_error: f64::NAN,
        aligned_peak_error: f64::NAN,
        signal_rms: f64::NAN,
        external_rms: f64::NAN,
        aligned_rms_error_ratio: f64::NAN,
    }
    .format_report_line()
}

fn decode_external_benchmark_render_audio(
    render: &StretchExternalBenchmarkRender,
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
        mono_samples,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct AlignedErrorMeasurement {
    lag_frames: isize,
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

impl CompressionReviewCandidateAccumulator {
    fn new(report_name: &'static str, candidate_path: &'static str) -> Self {
        Self {
            report_name,
            candidate_path,
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
        if ratio >= 1.0 {
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
            "{} rows={} candidate_path={} baseline_path=offline_hq candidate_better_rows={} current_better_rows={} unchanged_rows={} inconclusive_rows={} finite_rows={} mean_candidate_smear_frames={:.6} mean_current_smear_frames={:.6} best_candidate_improvement_delta_frames={:.6} best_candidate_improvement_case={} best_candidate_improvement_source={} best_candidate_improvement_ratio={:.6} worst_candidate_regression_delta_frames={:.6} worst_candidate_regression_case={} worst_candidate_regression_source={} worst_candidate_regression_ratio={:.6} worst_draft_regression_delta_frames={:.6} worst_draft_regression_case={} worst_draft_regression_source={} worst_draft_regression_ratio={:.6} baseline_worst_draft_regression_delta_frames={:.6} baseline_worst_draft_regression_case={} baseline_worst_draft_regression_source={} baseline_worst_draft_regression_ratio={:.6} baseline_worst_draft_smear_frames={:.6} baseline_worst_current_smear_frames={:.6} baseline_worst_candidate_smear_frames={:.6}",
            self.report_name,
            self.rows,
            self.candidate_path,
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
            offline.missed_transients >= SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES;
        let accepts_current_smear =
            offline.max_smear_frames >= SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES;
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
            SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
            SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES,
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
            ParseOutcome::Run(ReportArgs {
                report_name: DEFAULT_REPORT_NAME.to_string(),
                projection_epoch: DEFAULT_PROJECTION_EPOCH.to_string(),
                output: None,
                external_benchmark_tool: DEFAULT_EXTERNAL_BENCHMARK_TOOL.to_string(),
                external_benchmark_renders: Vec::new(),
                listening_source_manifests: Vec::new(),
                decode_listening_sources: false,
                decode_source_frame_limit: DEFAULT_DECODE_SOURCE_FRAME_LIMIT,
                measure_decoded_stretch: false,
                decoded_stretch_frame_limit: DEFAULT_DECODED_STRETCH_FRAME_LIMIT,
                measure_external_benchmark_quality: false,
            })
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
            "--measure-external-benchmark-quality".to_string(),
            "--decoded-stretch-frame-limit".to_string(),
            "1024".to_string(),
            "--external-benchmark-tool".to_string(),
            "rubberband-cli".to_string(),
            "--external-benchmark-render".to_string(),
            "stretch:loop_seam".to_string(),
            "1.5".to_string(),
            "target/rubberband-loop.wav".to_string(),
        ])
        .expect("custom args parse");

        assert_eq!(
            args,
            ParseOutcome::Run(ReportArgs {
                report_name: "custom".to_string(),
                projection_epoch: "epoch:1".to_string(),
                output: Some(PathBuf::from("target/stretch-report.txt")),
                external_benchmark_tool: "rubberband-cli".to_string(),
                external_benchmark_renders: vec![ExternalBenchmarkRenderArg {
                    case_id: "stretch:loop_seam".to_string(),
                    ratio: "1.5".to_string(),
                    path: PathBuf::from("target/rubberband-loop.wav"),
                }],
                listening_source_manifests: vec![PathBuf::from("target/fma.tsv")],
                decode_listening_sources: true,
                decode_source_frame_limit: 2048,
                measure_decoded_stretch: true,
                decoded_stretch_frame_limit: 1024,
                measure_external_benchmark_quality: true,
            })
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
            }],
            listening_source_manifests: Vec::new(),
            decode_listening_sources: false,
            decode_source_frame_limit: DEFAULT_DECODE_SOURCE_FRAME_LIMIT,
            measure_decoded_stretch: false,
            decoded_stretch_frame_limit: DEFAULT_DECODED_STRETCH_FRAME_LIMIT,
            measure_external_benchmark_quality: false,
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
        let render = StretchExternalBenchmarkRender {
            case_id: "stretch:vocals".to_string(),
            ratio: 1.0,
            pitch_shift_semitones: None,
            tool_name: "rubberband-cli".to_string(),
            rendered_path: path.display().to_string(),
            rendered_frames: 4_096,
            sample_rate_hz: 48_000,
            channels: 2,
        };

        let formatted = format_external_benchmark_quality_metrics(&[source], &[render], 4_096)
            .expect("format external quality metrics");

        assert!(formatted.starts_with("external_benchmark_quality case=stretch:vocals"));
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
        assert!(formatted.contains("alignment_lag_frames=0"));
        assert!(formatted.contains("aligned_compared_frames=4096"));
        assert!(formatted.contains("aligned_rms_error=0.000000000"));

        let _ = fs::remove_file(path);
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
            format_decoded_stretch_metrics(&[source], 2_048).expect("format decoded metrics");

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
        assert!(formatted.contains("decoded_compression_short_window_selector_candidate rows="));
        assert!(formatted.contains("gate=CurrentMissesOrHighCurrentSmear"));
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
}
