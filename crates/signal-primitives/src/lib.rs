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

#![warn(missing_docs)]

/// Floating-point audio sample value.
pub type Sample = f32;

/// Error returned when an [`AudioBuffer`] cannot be constructed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioBufferConstructionError {
    /// The requested channel layout has a channel count of zero.
    InvalidChannelLayout {
        /// The invalid channel count that was provided.
        channel_count: usize,
    },
    /// The interleaved sample slice length is not an exact multiple of the channel count.
    LossyInterleavedSampleCount {
        /// The channel count used to validate the sample slice.
        channel_count: usize,
        /// The number of samples in the slice that could not be evenly divided.
        sample_count: usize,
    },
}

/// Sample rate in Hz.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleRate(pub u32);

/// Number of audio frames (samples per channel) in a block or buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameCount(pub usize);

/// Number of audio channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelCount(pub usize);

/// Duration in seconds.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Seconds(pub f32);

/// Frequency in Hz.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct FrequencyHz(pub f32);

/// Linear (non-dB) gain multiplier.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct GainLinear(pub Sample);

/// A constant-value segment of a step envelope, expressed in sample-accurate frame offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepSegment {
    start_frame: usize,
    frame_count: usize,
    value: Sample,
}

impl SampleRate {
    /// Return the sample rate as an `f32`.
    pub fn as_f32(self) -> f32 {
        self.0 as f32
    }

    /// Convert a duration in seconds to a frame count at this sample rate.
    ///
    /// Negative durations are clamped to zero. Returns [`FrameCount(0)`] if the
    /// sample rate is zero.
    pub fn seconds_to_frames(self, seconds: Seconds) -> FrameCount {
        if self.0 == 0 {
            return FrameCount(0);
        }

        FrameCount((seconds.0.max(0.0) * self.as_f32()).round() as usize)
    }

    /// Convert a frame count to a duration in seconds at this sample rate.
    ///
    /// Returns [`Seconds(0.0)`] if the sample rate is zero.
    pub fn frames_to_seconds(self, frames: FrameCount) -> Seconds {
        if self.0 == 0 {
            return Seconds(0.0);
        }

        Seconds(frames.0 as f32 / self.as_f32())
    }
}

impl FrameCount {
    /// Return the frame count as a `usize`.
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl ChannelCount {
    /// Return the channel count as a `usize`.
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl Seconds {
    /// Return the duration as an `f32`.
    pub fn as_f32(self) -> f32 {
        self.0
    }
}

impl FrequencyHz {
    /// Return the frequency as an `f32`.
    pub fn as_f32(self) -> f32 {
        self.0
    }

    /// Return the frequency normalized to the range `[0.0, 0.5]` (Nyquist = 0.5).
    ///
    /// Returns `0.0` if the sample rate is zero.
    pub fn normalized(self, sample_rate: SampleRate) -> f32 {
        if sample_rate.0 == 0 {
            return 0.0;
        }

        (self.0 / sample_rate.as_f32()).clamp(0.0, 0.5)
    }
}

impl GainLinear {
    /// Return the gain as a [`Sample`] value.
    pub fn as_sample(self) -> Sample {
        self.0
    }
}

impl StepSegment {
    /// Create a new segment starting at `start_frame`, spanning `frame_count` frames, held at `value`.
    pub fn new(start_frame: usize, frame_count: usize, value: Sample) -> Self {
        Self {
            start_frame,
            frame_count,
            value,
        }
    }

    /// First frame index included in this segment.
    pub fn start_frame(self) -> usize {
        self.start_frame
    }

    /// Number of frames in this segment.
    pub fn frame_count(self) -> usize {
        self.frame_count
    }

    /// One past the last frame index in this segment (exclusive upper bound).
    pub fn end_frame(self) -> usize {
        self.start_frame.saturating_add(self.frame_count)
    }

    /// The held sample value for this segment.
    pub fn value(self) -> Sample {
        self.value
    }

    /// Return `true` if `frame_index` falls within `[start_frame, end_frame)`.
    pub fn contains(self, frame_index: usize) -> bool {
        frame_index >= self.start_frame && frame_index < self.end_frame()
    }
}

/// Named or arbitrary channel layout for an audio buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChannelLayout {
    /// Single channel.
    Mono,
    /// Two channels (left + right).
    Stereo,
    /// Arbitrary channel count.
    Count(ChannelCount),
}

impl ChannelLayout {
    /// Return the number of channels described by this layout.
    pub fn channels(self) -> ChannelCount {
        match self {
            Self::Mono => ChannelCount(1),
            Self::Stereo => ChannelCount(2),
            Self::Count(count) => count,
        }
    }

    /// Collapse `Count(1)` and `Count(2)` to their named equivalents; leave others unchanged.
    pub fn normalized(self) -> Self {
        match self.channels().0 {
            1 => Self::Mono,
            2 => Self::Stereo,
            _ => self,
        }
    }

    /// Normalize the layout and return an error if the channel count is zero.
    pub fn validate(self) -> Result<Self, AudioBufferConstructionError> {
        let normalized = self.normalized();
        let channel_count = normalized.channels().0;
        if channel_count == 0 {
            return Err(AudioBufferConstructionError::InvalidChannelLayout { channel_count });
        }
        Ok(normalized)
    }
}

/// Owned interleaved PCM audio buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioBuffer {
    sample_rate: SampleRate,
    channels: ChannelLayout,
    frames: FrameCount,
    data: Vec<Sample>,
}

impl AudioBuffer {
    /// Allocate a silent buffer for the given sample rate, channel layout, and frame count.
    ///
    /// # Panics
    ///
    /// Panics if `channels` resolves to a zero channel count.
    pub fn new(sample_rate: SampleRate, channels: ChannelLayout, frames: FrameCount) -> Self {
        Self::try_new(sample_rate, channels, frames)
            .expect("audio buffer construction should use a non-zero channel layout")
    }

    /// Allocate a silent buffer, returning an error instead of panicking on invalid input.
    pub fn try_new(
        sample_rate: SampleRate,
        channels: ChannelLayout,
        frames: FrameCount,
    ) -> Result<Self, AudioBufferConstructionError> {
        let channels = channels.validate()?;
        let len = channels.channels().0.saturating_mul(frames.0);
        Ok(Self {
            sample_rate,
            channels,
            frames,
            data: vec![0.0; len],
        })
    }

    /// Wrap an existing interleaved sample slice.
    ///
    /// # Panics
    ///
    /// Panics if `channels` resolves to a zero channel count, or if `data.len()` is not an exact
    /// multiple of the channel count.
    pub fn from_interleaved(
        sample_rate: SampleRate,
        channels: ChannelLayout,
        data: Vec<Sample>,
    ) -> Self {
        Self::try_from_interleaved(sample_rate, channels, data).expect(
            "interleaved audio buffer construction should use a non-zero channel layout and an exact frame multiple",
        )
    }

    /// Wrap an existing interleaved sample slice, returning an error instead of panicking on
    /// invalid input.
    pub fn try_from_interleaved(
        sample_rate: SampleRate,
        channels: ChannelLayout,
        data: Vec<Sample>,
    ) -> Result<Self, AudioBufferConstructionError> {
        let channels = channels.validate()?;
        let channel_count = channels.channels().0;
        if !data.len().is_multiple_of(channel_count) {
            return Err(AudioBufferConstructionError::LossyInterleavedSampleCount {
                channel_count,
                sample_count: data.len(),
            });
        }
        let frames = data.len() / channel_count;

        Ok(Self {
            sample_rate,
            channels,
            frames: FrameCount(frames),
            data,
        })
    }

    /// Sample rate of this buffer.
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Channel layout of this buffer.
    pub fn channels(&self) -> ChannelLayout {
        self.channels
    }

    /// Number of frames (samples per channel) in this buffer.
    pub fn frames(&self) -> FrameCount {
        self.frames
    }

    /// Number of channels in this buffer.
    pub fn channel_count(&self) -> ChannelCount {
        self.channels.channels()
    }

    /// Return `true` if the buffer contains no samples.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Interleaved sample data as a slice.
    pub fn samples(&self) -> &[Sample] {
        &self.data
    }

    /// Interleaved sample data as a mutable slice.
    pub fn samples_mut(&mut self) -> &mut [Sample] {
        &mut self.data
    }

    /// Zero all samples in the buffer.
    pub fn clear(&mut self) {
        self.data.fill(0.0);
    }

    /// Mix all channels down to mono by averaging, returning a new sample vector.
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

    /// Duration of a single frame in seconds (reciprocal of the sample rate).
    ///
    /// Returns `0.0` if the sample rate is zero.
    pub fn seconds_per_frame(&self) -> f32 {
        if self.sample_rate.0 == 0 {
            return 0.0;
        }
        1.0 / self.sample_rate.0 as f32
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AudioBuffer, AudioBufferConstructionError, ChannelCount, ChannelLayout, FrequencyHz,
        SampleRate, Seconds, StepSegment,
    };

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

    #[test]
    fn try_new_rejects_zero_channel_layout() {
        let error = AudioBuffer::try_new(
            SampleRate(48_000),
            ChannelLayout::Count(ChannelCount(0)),
            super::FrameCount(16),
        )
        .unwrap_err();

        assert_eq!(
            error,
            AudioBufferConstructionError::InvalidChannelLayout { channel_count: 0 }
        );
    }

    #[test]
    fn try_from_interleaved_rejects_lossy_sample_count() {
        let error = AudioBuffer::try_from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![0.0, 1.0, 2.0],
        )
        .unwrap_err();

        assert_eq!(
            error,
            AudioBufferConstructionError::LossyInterleavedSampleCount {
                channel_count: 2,
                sample_count: 3,
            }
        );
    }
}
