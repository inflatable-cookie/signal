use std::borrow::Cow;
use std::path::PathBuf;

use signal_dsp_stretch::{StretchCorpusListeningSource, StretchExternalBenchmarkRender};

use crate::args::ReportArgs;
use crate::listening::decode_wav_samples;
use crate::manifest::{field, required_any_field, required_field};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExternalBenchmarkQualityRender {
    pub case_id: String,
    pub ratio: f64,
    pub tool_name: String,
    pub rendered_path: String,
    pub source_wav: Option<String>,
}

pub(crate) fn load_external_benchmark_quality_renders(
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

pub(crate) fn load_external_benchmark_metadata(
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExternalBenchmarkDecodedAudio {
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub mono_samples: Vec<f32>,
}

impl ExternalBenchmarkDecodedAudio {
    pub fn frames(&self) -> usize {
        self.mono_samples.len()
    }
}

pub(crate) fn decode_external_benchmark_render_audio(
    render: &ExternalBenchmarkQualityRender,
) -> Result<ExternalBenchmarkDecodedAudio, String> {
    let path = PathBuf::from(&render.rendered_path);
    let mut reader = hound::WavReader::open(&path)
        .map_err(|error| format!("failed to open external render {}: {error}", path.display()))?;
    let spec = reader.spec();
    if spec.channels == 0 || spec.sample_rate == 0 {
        return Err(format!("invalid external render WAV {}", path.display()));
    }
    let samples = decode_wav_samples(&mut reader, &path, None)?;
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

pub(crate) enum ExternalBenchmarkQualitySource<'a> {
    Found(Cow<'a, StretchCorpusListeningSource>),
    Missing,
    Ambiguous,
}

pub(crate) fn source_for_external_quality_render<'a>(
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
