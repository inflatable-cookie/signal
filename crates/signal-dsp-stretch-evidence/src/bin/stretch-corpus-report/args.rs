use std::path::PathBuf;
use std::process;

use signal_dsp_stretch::OfflineHighQualityPath;

pub(crate) const DEFAULT_REPORT_NAME: &str = "stretch-corpus-v1-offline-evidence";
pub(crate) const DEFAULT_PROJECTION_EPOCH: &str = "projection:deterministic-report-v1";
pub(crate) const DEFAULT_EXTERNAL_BENCHMARK_TOOL: &str = "external-render";
pub(crate) const DEFAULT_FRAME_LIMIT: usize = 48_000 * 60;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ReportArgs {
    pub report_name: String,
    pub projection_epoch: String,
    pub output: Option<PathBuf>,
    pub external_benchmark_tool: String,
    pub external_benchmark_renders: Vec<ExternalBenchmarkRenderArg>,
    pub external_benchmark_render_manifests: Vec<PathBuf>,
    pub listening_source_manifests: Vec<PathBuf>,
    pub frame_limit: usize,
    pub measure_external_benchmark_quality: bool,
    pub signal_path: OfflineHighQualityPath,
    pub export_blind_listening_pack: Option<PathBuf>,
    pub blind_listening_note_manifests: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExternalBenchmarkRenderArg {
    pub case_id: String,
    pub ratio: String,
    pub path: PathBuf,
    pub tool_name: Option<String>,
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

pub(crate) fn exit_with_error(message: &str, show_usage: bool) -> ! {
    eprintln!("{message}");
    if show_usage {
        eprintln!("{}", usage());
    }
    process::exit(1);
}

pub(crate) enum ParseOutcome {
    // Boxed: `ReportArgs` is ~232 bytes larger than `Help`, so an unboxed
    // variant would make every `ParseOutcome` pay for the argument struct.
    Run(Box<ReportArgs>),
    Help,
}

pub(crate) fn parse_args<I>(args: I) -> Result<ParseOutcome, String>
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

pub(crate) fn usage() -> &'static str {
    "usage: stretch-corpus-report [--report-name NAME] [--projection-epoch EPOCH] [--listening-source-manifest TSV] [--external-benchmark-render CASE RATIO WAV] [--external-benchmark-render-manifest TSV] [--external-benchmark-tool NAME] [--measure-external-benchmark-quality] [--external-benchmark-signal-path default|compression-short-window-selector|expansion-short-window-selector] [--frame-limit N] [--export-blind-listening-pack DIR] [--check-blind-listening-notes TSV] [--output PATH]"
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
}
