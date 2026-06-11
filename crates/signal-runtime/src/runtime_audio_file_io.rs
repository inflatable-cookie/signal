use super::runtime_recording_capture::RuntimeRecordingCaptureActiveSession;
use super::*;
use hound::{SampleFormat as HoundSampleFormat, WavSpec, WavWriter};

pub(super) fn commit_recording_capture_wav(
    active: &RuntimeRecordingCaptureActiveSession,
) -> Result<u32, RuntimeError> {
    if let Some(parent) = Path::new(&active.capture_path).parent() {
        fs::create_dir_all(parent).map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!("failed to create recording capture directory: {error}"),
            )
        })?;
    }

    let spec = WavSpec {
        channels: active.channel_count as u16,
        sample_rate: active.sample_rate_hz,
        bits_per_sample: 32,
        sample_format: HoundSampleFormat::Float,
    };
    let mut writer = WavWriter::create(&active.capture_path, spec).map_err(|error| {
        RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!("failed to create recording capture wav: {error}"),
        )
    })?;
    for sample in &active.samples {
        writer.write_sample(*sample).map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!("failed to write recording capture sample: {error}"),
            )
        })?;
    }
    writer.finalize().map_err(|error| {
        RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!("failed to finalize recording capture wav: {error}"),
        )
    })?;

    Ok(active.buffered_frame_count.min(u32::MAX as u64) as u32)
}
