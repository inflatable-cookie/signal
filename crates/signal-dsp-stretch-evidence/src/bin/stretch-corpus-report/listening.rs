use std::fs::File;
use std::path::{Path, PathBuf};

use signal_dsp_stretch::{
    StretchCorpusAssetRequirement, StretchCorpusListeningSource, STRETCH_CORPUS_MANIFEST,
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

use crate::args::ReportArgs;
use crate::manifest::{field, required_any_field, required_field};

pub(crate) fn load_listening_sources(
    args: &ReportArgs,
) -> Result<Vec<StretchCorpusListeningSource>, String> {
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DecodedListeningSourceAudio {
    pub source_path: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

impl DecodedListeningSourceAudio {
    pub fn analyzed_frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    pub fn mono_samples(&self) -> Vec<f32> {
        let channels = self.channels as usize;
        self.samples
            .chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    }
}

pub(crate) fn decode_listening_source_audio(
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
    let samples = decode_wav_samples(&mut reader, path, Some(limit))?;
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

pub(crate) fn decode_wav_samples(
    reader: &mut hound::WavReader<std::io::BufReader<std::fs::File>>,
    path: &Path,
    sample_limit: Option<usize>,
) -> Result<Vec<f32>, String> {
    let spec = reader.spec();
    let take = sample_limit.unwrap_or(usize::MAX);
    match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .take(take)
            .map(|sample| {
                sample.map_err(|error| format!("failed to read {}: {error}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>(),
        hound::SampleFormat::Int => {
            let scale = integer_sample_scale(spec.bits_per_sample);
            reader
                .samples::<i32>()
                .take(take)
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / scale)
                        .map_err(|error| format!("failed to read {}: {error}", path.display()))
                })
                .collect::<Result<Vec<_>, _>>()
        }
    }
}

fn sample_limit(frame_limit: usize, channels: usize) -> Option<usize> {
    (frame_limit > 0).then(|| frame_limit.saturating_mul(channels))
}

pub(crate) fn integer_sample_scale(bits_per_sample: u16) -> f32 {
    if bits_per_sample == 0 {
        1.0
    } else {
        2_f32.powi(i32::from(bits_per_sample) - 1)
    }
}
