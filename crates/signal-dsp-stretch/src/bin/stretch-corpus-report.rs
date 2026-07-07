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
const MAX_TRANSIENT_ALIGNMENT_EVENTS_PER_BACKEND: usize = 3;
const TRANSIENT_ALIGNMENT_WINDOW_RADIUS: usize = QUALITY_METRIC_WINDOW_SIZE;
const EXPECTED_TRANSIENT_ENERGY_PRESENT_RATIO: f64 = 0.50;
const EXPECTED_TRANSIENT_ENERGY_WEAK_RATIO: f64 = 0.10;
const RECOVERY_GATE_MIN_RECOVERED_MISSES: usize = 1;
const RECOVERY_GATE_MAX_MISSED_WORSENED_ROWS: usize = 0;
const RECOVERY_GATE_MAX_SMEAR_WORSENED_ROWS: usize = 0;
const RECOVERY_GATE_MAX_GLOBAL_CANDIDATE_INPUT_RATIO: f64 = 2.0;
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
    "usage: stretch-corpus-report [--report-name NAME] [--projection-epoch EPOCH] [--listening-source-manifest TSV] [--decode-listening-sources] [--decode-source-frame-limit N] [--measure-decoded-stretch] [--decoded-stretch-frame-limit N] [--external-benchmark-tool NAME] [--external-benchmark-render CASE RATIO WAV] [--output PATH]"
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
            draft_recovery_gate.record(
                &draft_smear,
                &draft_candidate_smear,
                &draft_candidate_recovery_smear,
            );
            offline_recovery_gate.record(
                &offline_smear,
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
    Ok(lines.join("\n"))
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
        "draft_input_transients={} draft_output_transients={} draft_matched_transients={} draft_missed_transients={} draft_candidate_input_transients={} draft_candidate_output_transients={} draft_candidate_matched_transients={} draft_candidate_missed_transients={} draft_candidate_max_smear_frames={:.6} draft_candidate_output_matched_transients={} draft_candidate_output_missed_transients={} draft_candidate_output_max_smear_frames={:.6} draft_candidate_recovery_matched_transients={} draft_candidate_recovery_missed_transients={} draft_candidate_recovery_max_smear_frames={:.6} draft_mean_match_error_frames={:.6} draft_max_match_error_frames={:.6} draft_mean_missed_nearest_distance_frames={:.6} draft_max_missed_nearest_distance_frames={:.6} draft_max_missed_expected_output_frame={:.6} draft_max_missed_nearest_output_frame={:.6} offline_input_transients={} offline_output_transients={} offline_matched_transients={} offline_missed_transients={} offline_candidate_input_transients={} offline_candidate_output_transients={} offline_candidate_matched_transients={} offline_candidate_missed_transients={} offline_candidate_max_smear_frames={:.6} offline_candidate_output_matched_transients={} offline_candidate_output_missed_transients={} offline_candidate_output_max_smear_frames={:.6} offline_candidate_recovery_matched_transients={} offline_candidate_recovery_missed_transients={} offline_candidate_recovery_max_smear_frames={:.6} offline_mean_match_error_frames={:.6} offline_max_match_error_frames={:.6} offline_mean_missed_nearest_distance_frames={:.6} offline_max_missed_nearest_distance_frames={:.6} offline_max_missed_expected_output_frame={:.6} offline_max_missed_nearest_output_frame={:.6}",
        draft_smear.input_transients,
        draft_smear.output_transients,
        draft_smear.matched_transients,
        draft_smear.missed_transients,
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
        assert!(formatted.contains("target_status="));
        assert!(formatted.contains("global_threshold_status="));
        assert!(formatted.contains("full_candidate_input_ratio="));
        assert!(formatted.contains("offline_matched_transients="));
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
