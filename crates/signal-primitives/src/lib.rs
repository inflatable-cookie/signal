//! Foundational audio, time, channel, and simple control primitives for the
//! Signal workspace.
//!
//! These types are intentionally small and copy-friendly so DSP, analysis,
//! graph, and runtime crates can share one vocabulary for sample rate, frame
//! counts, channel layout, and interleaved buffers.
//!
//! ```no_run
//! use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate, Seconds};
//!
//! let sample_rate = SampleRate(48_000);
//! let frames = sample_rate.seconds_to_frames(Seconds(0.25));
//! let audio = AudioBuffer::new(sample_rate, ChannelLayout::Stereo, FrameCount(128));
//!
//! assert_eq!(frames.0, 12_000);
//! assert_eq!(audio.channel_count().0, 2);
//! ```

pub type Sample = f32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleRate(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameCount(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelCount(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Seconds(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct FrequencyHz(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct GainLinear(pub Sample);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepSegment {
    start_frame: usize,
    frame_count: usize,
    value: Sample,
}

impl SampleRate {
    pub fn as_f32(self) -> f32 {
        self.0 as f32
    }

    pub fn seconds_to_frames(self, seconds: Seconds) -> FrameCount {
        if self.0 == 0 {
            return FrameCount(0);
        }

        FrameCount((seconds.0.max(0.0) * self.as_f32()).round() as usize)
    }

    pub fn frames_to_seconds(self, frames: FrameCount) -> Seconds {
        if self.0 == 0 {
            return Seconds(0.0);
        }

        Seconds(frames.0 as f32 / self.as_f32())
    }
}

impl FrameCount {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl ChannelCount {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl Seconds {
    pub fn as_f32(self) -> f32 {
        self.0
    }
}

impl FrequencyHz {
    pub fn as_f32(self) -> f32 {
        self.0
    }

    pub fn normalized(self, sample_rate: SampleRate) -> f32 {
        if sample_rate.0 == 0 {
            return 0.0;
        }

        (self.0 / sample_rate.as_f32()).clamp(0.0, 0.5)
    }
}

impl GainLinear {
    pub fn as_sample(self) -> Sample {
        self.0
    }
}

impl StepSegment {
    pub fn new(start_frame: usize, frame_count: usize, value: Sample) -> Self {
        Self {
            start_frame,
            frame_count,
            value,
        }
    }

    pub fn start_frame(self) -> usize {
        self.start_frame
    }

    pub fn frame_count(self) -> usize {
        self.frame_count
    }

    pub fn end_frame(self) -> usize {
        self.start_frame.saturating_add(self.frame_count)
    }

    pub fn value(self) -> Sample {
        self.value
    }

    pub fn contains(self, frame_index: usize) -> bool {
        frame_index >= self.start_frame && frame_index < self.end_frame()
    }
}

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

    pub fn clear(&mut self) {
        self.data.fill(0.0);
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
    use super::{AudioBuffer, ChannelLayout, FrequencyHz, SampleRate, Seconds, StepSegment};

    #[test]
    fn mono_mixdown_averages_channels() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.25, 0.75],
        );

        assert_eq!(audio.to_mono(), vec![0.0, 0.5]);
    }

    #[test]
    fn sample_rate_converts_between_seconds_and_frames() {
        let sample_rate = SampleRate(48_000);
        let frames = sample_rate.seconds_to_frames(Seconds(0.25));

        assert_eq!(frames.0, 12_000);
        assert_eq!(sample_rate.frames_to_seconds(frames), Seconds(0.25));
    }

    #[test]
    fn step_segment_tracks_sample_accurate_range() {
        let segment = StepSegment::new(32, 16, 0.75);

        assert_eq!(segment.start_frame(), 32);
        assert_eq!(segment.end_frame(), 48);
        assert!(segment.contains(32));
        assert!(segment.contains(47));
        assert!(!segment.contains(48));
        assert_eq!(segment.value(), 0.75);
    }

    #[test]
    fn frequency_normalization_is_clamped_to_nyquist() {
        let sample_rate = SampleRate(48_000);

        assert_eq!(FrequencyHz(12_000.0).normalized(sample_rate), 0.25);
        assert_eq!(FrequencyHz(96_000.0).normalized(sample_rate), 0.5);
    }
}
