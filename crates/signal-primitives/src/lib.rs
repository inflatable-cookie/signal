//! Foundational audio, time, and channel primitives for the Signal workspace.

pub type Sample = f32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleRate(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameCount(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelCount(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChannelLayout {
    Mono,
    Stereo,
    Count(ChannelCount),
}

impl ChannelLayout {
    pub fn channels(self) -> ChannelCount {
        match self {
            Self::Mono => ChannelCount(1),
            Self::Stereo => ChannelCount(2),
            Self::Count(count) => count,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioBuffer {
    sample_rate: SampleRate,
    channels: ChannelLayout,
    frames: FrameCount,
    data: Vec<Sample>,
}

impl AudioBuffer {
    pub fn new(sample_rate: SampleRate, channels: ChannelLayout, frames: FrameCount) -> Self {
        let len = channels.channels().0.saturating_mul(frames.0);
        Self {
            sample_rate,
            channels,
            frames,
            data: vec![0.0; len],
        }
    }

    pub fn from_interleaved(
        sample_rate: SampleRate,
        channels: ChannelLayout,
        data: Vec<Sample>,
    ) -> Self {
        let channel_count = channels.channels().0;
        let frames = if channel_count == 0 {
            0
        } else {
            data.len() / channel_count
        };

        Self {
            sample_rate,
            channels,
            frames: FrameCount(frames),
            data,
        }
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn channels(&self) -> ChannelLayout {
        self.channels
    }

    pub fn frames(&self) -> FrameCount {
        self.frames
    }

    pub fn channel_count(&self) -> ChannelCount {
        self.channels.channels()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn samples(&self) -> &[Sample] {
        &self.data
    }

    pub fn samples_mut(&mut self) -> &mut [Sample] {
        &mut self.data
    }

    pub fn to_mono(&self) -> Vec<Sample> {
        let channels = self.channel_count().0;
        if channels == 0 || self.data.is_empty() {
            return Vec::new();
        }

        if channels == 1 {
            return self.data.clone();
        }

        let scale = 1.0 / channels as Sample;
        self.data
            .chunks_exact(channels)
            .map(|frame| frame.iter().copied().sum::<Sample>() * scale)
            .collect()
    }

    pub fn seconds_per_frame(&self) -> f32 {
        if self.sample_rate.0 == 0 {
            return 0.0;
        }
        1.0 / self.sample_rate.0 as f32
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioBuffer, ChannelLayout, SampleRate};

    #[test]
    fn mono_mixdown_averages_channels() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.25, 0.75],
        );

        assert_eq!(audio.to_mono(), vec![0.0, 0.5]);
    }
}
