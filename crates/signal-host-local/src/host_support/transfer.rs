use signal_hardware::HardwareStreamConfig;
use signal_primitives::AudioBuffer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LocalAudioTransferOutcome {
    pub(crate) copied_samples: usize,
    pub(crate) zero_filled_samples: usize,
    pub(crate) dropped_samples: usize,
}

pub(crate) struct LocalAudioTransferResult {
    pub(crate) outcome: LocalAudioTransferOutcome,
    pub(crate) output_peak: f32,
}

pub(crate) fn transfer_runtime_output_to_host_buffer(
    runtime_output: &AudioBuffer,
    stream: &HardwareStreamConfig,
    policy: signal_runtime::RuntimeHostAudioTransferPolicy,
) -> LocalAudioTransferResult {
    let callback_frames = stream.buffer_size.min(policy.max_callback_frames);
    let host_channels = usize::from(stream.output_channels.min(policy.max_transfer_channels));
    let runtime_channels = runtime_output.channel_count().0;
    let copied_frames = callback_frames.min(runtime_output.frames().0);
    let copied_channels = host_channels.min(runtime_channels);
    let mut host_buffer = vec![0.0_f32; callback_frames.saturating_mul(host_channels)];
    let runtime_samples = runtime_output.samples();

    for frame_index in 0..copied_frames {
        for channel_index in 0..copied_channels {
            let runtime_index = frame_index
                .saturating_mul(runtime_channels)
                .saturating_add(channel_index);
            let host_index = frame_index
                .saturating_mul(host_channels)
                .saturating_add(channel_index);
            host_buffer[host_index] = runtime_samples[runtime_index];
        }
    }

    let copied_samples = copied_frames.saturating_mul(copied_channels);
    let callback_samples = callback_frames.saturating_mul(host_channels);
    let dropped_frame_samples = runtime_output
        .frames()
        .0
        .saturating_sub(copied_frames)
        .saturating_mul(runtime_channels);
    let dropped_channel_samples =
        copied_frames.saturating_mul(runtime_channels.saturating_sub(copied_channels));
    let output_peak = host_buffer
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    LocalAudioTransferResult {
        outcome: LocalAudioTransferOutcome {
            copied_samples,
            zero_filled_samples: callback_samples.saturating_sub(copied_samples),
            dropped_samples: dropped_frame_samples.saturating_add(dropped_channel_samples),
        },
        output_peak,
    }
}

pub(crate) fn scale_audio_buffer(buffer: &AudioBuffer, gain: f32) -> AudioBuffer {
    let mut scaled = buffer.clone();
    for sample in scaled.samples_mut() {
        *sample *= gain;
    }
    scaled
}
