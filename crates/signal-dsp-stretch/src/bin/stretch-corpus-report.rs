//! Emit deterministic Signal stretch corpus evidence reports.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use signal_dsp_stretch::{
    build_stretch_corpus_comparison_report_with_sources, format_stretch_corpus_comparison_report,
    StretchCorpusAssetRequirement, StretchCorpusListeningSource, StretchExternalBenchmarkRender,
    STRETCH_CORPUS_MANIFEST,
};

const DEFAULT_REPORT_NAME: &str = "stretch-corpus-v1-offline-evidence";
const DEFAULT_PROJECTION_EPOCH: &str = "projection:deterministic-report-v1";
const DEFAULT_EXTERNAL_BENCHMARK_TOOL: &str = "external-render";

#[derive(Debug, PartialEq, Eq)]
struct ReportArgs {
    report_name: String,
    projection_epoch: String,
    output: Option<PathBuf>,
    external_benchmark_tool: String,
    external_benchmark_renders: Vec<ExternalBenchmarkRenderArg>,
    listening_source_manifests: Vec<PathBuf>,
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
    let formatted = format_stretch_corpus_comparison_report(&report);

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
    "usage: stretch-corpus-report [--report-name NAME] [--projection-epoch EPOCH] [--listening-source-manifest TSV] [--external-benchmark-tool NAME] [--external-benchmark-render CASE RATIO WAV] [--output PATH]"
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
