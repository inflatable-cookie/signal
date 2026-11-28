use std::{fs, path::Path};

use hound::SampleFormat as HoundSampleFormat;
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
use symphonia::core::{
    codecs::{
        audio::{AudioDecoderOptions as SymphoniaDecoderOptions, CODEC_ID_NULL_AUDIO},
        CodecParameters as SymphoniaCodecParameters,
    },
    errors::Error as SymphoniaError,
    formats::{probe::Hint as SymphoniaHint, FormatOptions as SymphoniaFormatOptions},
    io::MediaSourceStream,
    meta::MetadataOptions as SymphoniaMetadataOptions,
};

use crate::interfaces::{RuntimeError, RuntimeErrorKind};

pub(crate) fn decode_runtime_wav_asset(
    path: &Path,
    asset_id: &str,
) -> Result<AudioBuffer, RuntimeError> {
    let mut reader = hound::WavReader::open(path).map_err(|error| {
        RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!(
                "failed to open cached wav media asset `{asset_id}` at {}: {error}",
                path.display()
            ),
        )
    })?;
    let spec = reader.spec();
    let samples = match spec.sample_format {
        HoundSampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| {
                sample.map_err(|error| {
                    RuntimeError::new(
                        RuntimeErrorKind::ResourceUnavailable,
                        format!("failed to decode float sample from `{asset_id}`: {error}"),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        HoundSampleFormat::Int => {
            let scale = if spec.bits_per_sample == 0 {
                1.0
            } else {
                let magnitude = 1_i64
                    .checked_shl(spec.bits_per_sample.saturating_sub(1) as u32)
                    .unwrap_or(1) as f32;
                if magnitude == 0.0 {
                    1.0
                } else {
                    magnitude
                }
            };
            reader
                .samples::<i32>()
                .map(|sample| {
                    sample.map(|value| value as f32 / scale).map_err(|error| {
                        RuntimeError::new(
                            RuntimeErrorKind::ResourceUnavailable,
                            format!("failed to decode integer sample from `{asset_id}`: {error}"),
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let channels = match spec.channels {
        1 => ChannelLayout::Mono,
        2 => ChannelLayout::Stereo,
        count => ChannelLayout::Count(signal_primitives::ChannelCount(count as usize)),
    };
    Ok(AudioBuffer::from_interleaved(
        SampleRate(spec.sample_rate),
        channels,
        samples,
    ))
}

pub(crate) fn decode_runtime_media_asset_with_symphonia(
    path: &Path,
    asset_id: &str,
) -> Result<AudioBuffer, RuntimeError> {
    let file = fs::File::open(path).map_err(|error| {
        RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!(
                "failed to open cached media asset `{asset_id}` at {}: {error}",
                path.display()
            ),
        )
    })?;
    let media_stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = SymphoniaHint::new();
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            media_stream,
            SymphoniaFormatOptions::default(),
            SymphoniaMetadataOptions::default(),
        )
        .map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "failed to probe cached media asset `{asset_id}` at {}: {error}",
                    path.display()
                ),
            )
        })?;

    let (track_id, codec_params) = format
        .tracks()
        .iter()
        .find_map(|track| match &track.codec_params {
            Some(SymphoniaCodecParameters::Audio(params))
                if params.codec != CODEC_ID_NULL_AUDIO =>
            {
                Some((track.id, params))
            }
            _ => None,
        })
        .ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::UnsupportedCapability,
                format!(
                    "cached media asset `{asset_id}` at {} has no supported audio track",
                    path.display()
                ),
            )
        })?;
    // Symphonia 0.6 trims encoder delay and padding by default; 0.5 did not.
    // Keep the untrimmed output so decoded frames stay unchanged.
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(
            codec_params,
            &SymphoniaDecoderOptions::default().gapless(false),
        )
        .map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "failed to create cached media decoder for `{asset_id}` at {}: {error}",
                    path.display()
                ),
            )
        })?;

    let mut sample_rate = None;
    let mut channel_count = None;
    let mut samples = Vec::new();
    let mut interleaved = Vec::<f32>::new();
    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
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
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to read cached media packet for `{asset_id}` at {}: {error}",
                        path.display()
                    ),
                ));
            }
        };
        if packet.track_id != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to decode cached media asset `{asset_id}` at {}: {error}",
                        path.display()
                    ),
                ));
            }
        };
        let actual_channels = decoded.spec().channels().count();
        let output_channels = actual_channels.clamp(1, 2);
        if sample_rate.is_none() {
            sample_rate = Some(decoded.spec().rate());
            channel_count = Some(output_channels);
        }

        decoded.copy_to_vec_interleaved(&mut interleaved);
        if actual_channels <= output_channels {
            samples.extend_from_slice(&interleaved);
        } else {
            for frame in interleaved.chunks(actual_channels) {
                for sample in frame.iter().take(output_channels) {
                    samples.push(*sample);
                }
            }
        }
    }

    let sample_rate = sample_rate.ok_or_else(|| {
        RuntimeError::new(
            RuntimeErrorKind::UnsupportedCapability,
            format!(
                "cached media asset `{asset_id}` at {} produced no decodable audio frames",
                path.display()
            ),
        )
    })?;
    let channels = match channel_count.unwrap_or(1) {
        1 => ChannelLayout::Mono,
        2 => ChannelLayout::Stereo,
        count => ChannelLayout::Count(signal_primitives::ChannelCount(count)),
    };
    Ok(AudioBuffer::from_interleaved(
        SampleRate(sample_rate),
        channels,
        samples,
    ))
}
