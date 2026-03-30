use super::super::super::*;

pub(super) fn filled_stereo_buffer(sample_rate_hz: u32, frames: usize, value: f32) -> AudioBuffer {
    let mut buffer = AudioBuffer::new(
        SampleRate(sample_rate_hz),
        ChannelLayout::Stereo,
        FrameCount(frames),
    );
    buffer.samples_mut().fill(value);
    buffer
}
