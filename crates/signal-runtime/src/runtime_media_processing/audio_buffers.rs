use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};

pub(crate) fn hash_audio_buffer(buffer: &AudioBuffer) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for sample in buffer.samples() {
        hash ^= u64::from(sample.to_bits());
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= buffer.frames().0 as u64;
    hash = hash.wrapping_mul(1099511628211);
    hash ^= buffer.channel_count().0 as u64;
    hash
}

pub(crate) fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
}

pub(crate) fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let mean_square = samples
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>()
        / samples.len() as f64;
    mean_square.sqrt() as f32
}

pub(crate) fn adapt_audio_buffer_layout(
    input: &AudioBuffer,
    target_layout: ChannelLayout,
) -> AudioBuffer {
    if input.channels() == target_layout {
        return input.clone();
    }

    match (input.channels(), target_layout) {
        (ChannelLayout::Mono, ChannelLayout::Stereo) => {
            let mut samples = Vec::with_capacity(input.samples().len().saturating_mul(2));
            for sample in input.samples() {
                samples.push(*sample);
                samples.push(*sample);
            }
            AudioBuffer::from_interleaved(input.sample_rate(), target_layout, samples)
        }
        (ChannelLayout::Stereo, ChannelLayout::Mono) => {
            AudioBuffer::from_interleaved(input.sample_rate(), target_layout, input.to_mono())
        }
        _ => AudioBuffer::new(input.sample_rate(), target_layout, input.frames()),
    }
}

pub(crate) fn mix_audio_buffer(target: &mut AudioBuffer, source: &AudioBuffer) {
    let adapted = if target.channels() == source.channels() {
        source.clone()
    } else {
        adapt_audio_buffer_layout(source, target.channels())
    };
    for (dst, src) in target.samples_mut().iter_mut().zip(adapted.samples()) {
        *dst += *src;
    }
}

pub(crate) fn write_offline_render_block(
    target: &mut Option<AudioBuffer>,
    total_frames: usize,
    frame_offset: usize,
    block: &AudioBuffer,
) {
    let buffer = target.get_or_insert_with(|| {
        AudioBuffer::new(
            block.sample_rate(),
            block.channels(),
            FrameCount(total_frames),
        )
    });
    let channel_count = buffer.channel_count().0;
    let start = frame_offset.saturating_mul(channel_count);
    let end = start
        .saturating_add(block.samples().len())
        .min(buffer.samples().len());
    buffer.samples_mut()[start..end].copy_from_slice(&block.samples()[..end - start]);
}

pub(crate) fn resample_audio_buffer_linear(
    input: &AudioBuffer,
    target_sample_rate: SampleRate,
) -> AudioBuffer {
    if input.sample_rate() == target_sample_rate {
        return input.clone();
    }
    if input.frames().0 == 0 || input.sample_rate().0 == 0 || target_sample_rate.0 == 0 {
        return AudioBuffer::new(target_sample_rate, input.channels(), FrameCount(0));
    }
    let output_frames = ((input.frames().0 as u64)
        .saturating_mul(target_sample_rate.0 as u64)
        .saturating_add(input.sample_rate().0 as u64 / 2)
        / input.sample_rate().0 as u64) as usize;
    let mut output = AudioBuffer::new(
        target_sample_rate,
        input.channels(),
        FrameCount(output_frames),
    );
    let channel_count = output.channel_count().0;
    let source_frame_ratio = input.sample_rate().0 as f64 / target_sample_rate.0 as f64;
    for frame_index in 0..output_frames {
        let source_frame = frame_index as f64 * source_frame_ratio;
        for channel_index in 0..channel_count {
            output.samples_mut()[frame_index * channel_count + channel_index] =
                sample_audio_buffer_linear(input, source_frame, channel_index, input.frames().0);
        }
    }
    output
}

pub(crate) fn sample_audio_buffer_linear(
    buffer: &AudioBuffer,
    source_frame: f64,
    channel_index: usize,
    frame_count: usize,
) -> f32 {
    if frame_count == 0 || channel_index >= buffer.channel_count().0 || source_frame.is_nan() {
        return 0.0;
    }
    let max_frame = frame_count.saturating_sub(1);
    let base_frame = source_frame.floor().max(0.0) as usize;
    if base_frame > max_frame {
        return 0.0;
    }
    let next_frame = (base_frame + 1).min(max_frame);
    let frac = (source_frame - base_frame as f64).clamp(0.0, 1.0) as f32;
    let channel_count = buffer.channel_count().0;
    let base_index = base_frame * channel_count + channel_index;
    let next_index = next_frame * channel_count + channel_index;
    let base = buffer.samples().get(base_index).copied().unwrap_or(0.0);
    let next = buffer.samples().get(next_index).copied().unwrap_or(base);
    base + ((next - base) * frac)
}
