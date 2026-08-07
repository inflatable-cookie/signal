use crate::creative_cyclic::{
    render_continuous as render_cyclic, CandidateError as CyclicCandidateError,
    Request as CyclicRequest,
};
use crate::creative_direct_renewal_dream::{
    render as render_dream, CandidateError as DreamCandidateError,
    CandidateRequest as DreamCandidateRequest, ADMISSION_SEED,
};
use signal_primitives::{Sample, SampleRate};
use std::time::Duration;

/// Semantic behavior version of the public creative-stretch renderer.
pub const CREATIVE_STRETCH_ENGINE_VERSION: &str = "signal-creative-stretch-v4";

/// Smallest output/input ratio supported by the `Dream` character.
pub const CREATIVE_STRETCH_DREAM_MIN_RATIO: usize = 4;

/// Largest output/input ratio supported by the `Dream` character.
pub const CREATIVE_STRETCH_DREAM_MAX_RATIO: usize = 16;

/// Smallest output/input ratio supported by the `Cyclic` character.
pub const CREATIVE_STRETCH_CYCLIC_MIN_RATIO: usize = 2;

/// Largest output/input ratio supported by the `Cyclic` character.
pub const CREATIVE_STRETCH_CYCLIC_MAX_RATIO: usize = 8;

/// Default preserve-to-widen value used by [`CreativeStretchRequest::new`].
pub const CREATIVE_STRETCH_DEFAULT_SPACE: f32 = 0.5;

/// Shortest admitted `Cyclic` cycle duration.
pub const CREATIVE_STRETCH_MIN_CYCLE: Duration = Duration::from_millis(5);

/// Default admitted `Cyclic` cycle duration.
pub const CREATIVE_STRETCH_DEFAULT_CYCLE: Duration = Duration::from_millis(48);

/// Longest admitted `Cyclic` cycle duration.
pub const CREATIVE_STRETCH_MAX_CYCLE: Duration = Duration::from_millis(90);

const MAX_EXACT_INTEGER_U128: u128 = (1_u128 << 53) - 1;

/// Creative character requested from Signal's offline renderer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreativeStretchCharacter {
    /// Smooth, fused, musical spectral smear.
    Dream,
    /// Commanded Akai-style cyclic repetition.
    Cyclic,
}

/// Output/input ratio domain admitted for one creative character.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreativeStretchRatioDomain {
    /// Every exact target frame count inside the inclusive ratio bounds.
    Continuous {
        /// Inclusive minimum output/input ratio.
        minimum: usize,
        /// Inclusive maximum output/input ratio.
        maximum: usize,
    },
    /// Exact integer output/input ratios.
    Exact(&'static [usize]),
}

impl CreativeStretchCharacter {
    /// Output/input ratio domain admitted for this character.
    pub const fn ratio_domain(self) -> CreativeStretchRatioDomain {
        match self {
            Self::Dream => CreativeStretchRatioDomain::Continuous {
                minimum: CREATIVE_STRETCH_DREAM_MIN_RATIO,
                maximum: CREATIVE_STRETCH_DREAM_MAX_RATIO,
            },
            Self::Cyclic => CreativeStretchRatioDomain::Continuous {
                minimum: CREATIVE_STRETCH_CYCLIC_MIN_RATIO,
                maximum: CREATIVE_STRETCH_CYCLIC_MAX_RATIO,
            },
        }
    }
}

/// One whole-buffer offline creative-stretch request.
///
/// `target_frames` is authoritative and must fall inside the selected
/// character's [`CreativeStretchCharacter::ratio_domain`]. This request
/// allocates and must not be rendered on the audio thread.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreativeStretchRequest<'a> {
    /// Finite mono or interleaved-stereo samples.
    pub input: &'a [Sample],
    /// Source and output channel count. Only mono and stereo are supported.
    pub channels: u16,
    /// Source and output sample rate.
    pub sample_rate: SampleRate,
    /// Exact requested output frame count.
    pub target_frames: usize,
    /// Semantic creative character.
    pub character: CreativeStretchCharacter,
    /// Dream preserve-to-widen control in the inclusive range `0.0..=1.0`.
    ///
    /// Cyclic requests must retain [`CREATIVE_STRETCH_DEFAULT_SPACE`].
    pub space: f32,
    /// Optional Cyclic cycle duration.
    ///
    /// `None` selects [`CREATIVE_STRETCH_DEFAULT_CYCLE`]. Dream requests must
    /// leave this as `None`.
    pub cycle: Option<Duration>,
}

impl<'a> CreativeStretchRequest<'a> {
    /// Construct a request with default character controls.
    pub fn new(
        input: &'a [Sample],
        channels: u16,
        sample_rate: SampleRate,
        target_frames: usize,
        character: CreativeStretchCharacter,
    ) -> Self {
        Self {
            input,
            channels,
            sample_rate,
            target_frames,
            character,
            space: CREATIVE_STRETCH_DEFAULT_SPACE,
            cycle: None,
        }
    }

    /// Set the preserve-to-widen control.
    ///
    /// Invalid values are reported by [`render_creative_stretch`]; they are
    /// never clamped.
    pub fn with_space(mut self, space: f32) -> Self {
        self.space = space;
        self
    }

    /// Set the Cyclic cycle duration.
    ///
    /// Dream requests reject this control. Invalid Cyclic durations are
    /// reported by [`render_creative_stretch`]; they are never clamped.
    pub fn with_cycle(mut self, cycle: Duration) -> Self {
        self.cycle = Some(cycle);
        self
    }
}

/// Failure returned by [`render_creative_stretch`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreativeStretchError {
    /// Channel count was not mono or stereo.
    InvalidChannelCount,
    /// Sample rate was outside `8000..=192000`.
    UnsupportedSampleRate,
    /// Interleaved input ended with a partial frame.
    PartialFrame,
    /// Input contained a non-finite sample.
    NonFiniteInput,
    /// `space` was non-finite or outside `0.0..=1.0`.
    InvalidSpace,
    /// Cyclic `cycle` was outside the inclusive `5..=90 ms` range.
    InvalidCycle,
    /// A character-local control was supplied to the wrong character.
    UnsupportedCharacterControl,
    /// Non-zero output was requested from empty input.
    EmptyInput,
    /// A non-empty input requested zero output frames.
    ZeroTargetFrames,
    /// Target frames fell outside the selected character's admitted ratio domain.
    UnsupportedTargetFrames,
    /// Request geometry or output size exceeded the supported integer range.
    SizeOverflow,
    /// Output allocation failed.
    AllocationFailed,
    /// Rendering produced a non-finite intermediate or output value.
    NonFiniteOutput,
}

impl From<DreamCandidateError> for CreativeStretchError {
    fn from(error: DreamCandidateError) -> Self {
        match error {
            DreamCandidateError::InvalidChannels => Self::InvalidChannelCount,
            DreamCandidateError::InvalidSampleRate => Self::UnsupportedSampleRate,
            DreamCandidateError::PartialFrame => Self::PartialFrame,
            DreamCandidateError::NonFiniteInput => Self::NonFiniteInput,
            DreamCandidateError::InvalidSpace => Self::InvalidSpace,
            DreamCandidateError::EmptyInput => Self::EmptyInput,
            DreamCandidateError::ZeroTarget => Self::ZeroTargetFrames,
            DreamCandidateError::UnsupportedRatio => Self::UnsupportedTargetFrames,
            DreamCandidateError::SizeOverflow => Self::SizeOverflow,
            DreamCandidateError::AllocationFailed => Self::AllocationFailed,
            DreamCandidateError::NonFiniteProcessing => Self::NonFiniteOutput,
        }
    }
}

fn canonical_cycle_us(cycle: Duration) -> Result<u32, CreativeStretchError> {
    if !(CREATIVE_STRETCH_MIN_CYCLE..=CREATIVE_STRETCH_MAX_CYCLE).contains(&cycle) {
        return Err(CreativeStretchError::InvalidCycle);
    }
    u32::try_from((cycle.as_nanos() + 500) / 1_000).map_err(|_| CreativeStretchError::SizeOverflow)
}

fn validate_request(
    request: CreativeStretchRequest<'_>,
) -> Result<Option<u32>, CreativeStretchError> {
    let channels = usize::from(request.channels);
    if !matches!(channels, 1 | 2) {
        return Err(CreativeStretchError::InvalidChannelCount);
    }
    if !request.input.len().is_multiple_of(channels) {
        return Err(CreativeStretchError::PartialFrame);
    }
    if !(8_000..=192_000).contains(&request.sample_rate.0) {
        return Err(CreativeStretchError::UnsupportedSampleRate);
    }
    if request.input.iter().any(|sample| !sample.is_finite()) {
        return Err(CreativeStretchError::NonFiniteInput);
    }

    let cycle_us = match request.character {
        CreativeStretchCharacter::Dream => {
            if request.cycle.is_some() {
                return Err(CreativeStretchError::UnsupportedCharacterControl);
            }
            if !request.space.is_finite() || !(0.0..=1.0).contains(&request.space) {
                return Err(CreativeStretchError::InvalidSpace);
            }
            None
        }
        CreativeStretchCharacter::Cyclic => {
            if request.space.to_bits() != CREATIVE_STRETCH_DEFAULT_SPACE.to_bits() {
                return Err(CreativeStretchError::UnsupportedCharacterControl);
            }
            Some(canonical_cycle_us(
                request.cycle.unwrap_or(CREATIVE_STRETCH_DEFAULT_CYCLE),
            )?)
        }
    };

    let source_frames = request.input.len() / channels;
    if source_frames == 0 || request.target_frames == 0 {
        return if source_frames == 0 && request.target_frames == 0 {
            Ok(cycle_us)
        } else if source_frames == 0 {
            Err(CreativeStretchError::EmptyInput)
        } else {
            Err(CreativeStretchError::ZeroTargetFrames)
        };
    }
    if source_frames as u128 > MAX_EXACT_INTEGER_U128
        || request.target_frames as u128 > MAX_EXACT_INTEGER_U128
    {
        return Err(CreativeStretchError::SizeOverflow);
    }
    let supported = match request.character {
        CreativeStretchCharacter::Dream => {
            let minimum = source_frames
                .checked_mul(CREATIVE_STRETCH_DREAM_MIN_RATIO)
                .ok_or(CreativeStretchError::SizeOverflow)?;
            let maximum = source_frames
                .checked_mul(CREATIVE_STRETCH_DREAM_MAX_RATIO)
                .ok_or(CreativeStretchError::SizeOverflow)?;
            (minimum..=maximum).contains(&request.target_frames)
        }
        CreativeStretchCharacter::Cyclic => {
            let minimum = source_frames
                .checked_mul(CREATIVE_STRETCH_CYCLIC_MIN_RATIO)
                .ok_or(CreativeStretchError::SizeOverflow)?;
            let maximum = source_frames
                .checked_mul(CREATIVE_STRETCH_CYCLIC_MAX_RATIO)
                .ok_or(CreativeStretchError::SizeOverflow)?;
            (minimum..=maximum).contains(&request.target_frames)
        }
    };
    if !supported {
        return Err(CreativeStretchError::UnsupportedTargetFrames);
    }
    request
        .target_frames
        .checked_mul(channels)
        .and_then(|samples| samples.checked_mul(std::mem::size_of::<Sample>()))
        .ok_or(CreativeStretchError::SizeOverflow)?;
    Ok(cycle_us)
}

fn map_cyclic_error(
    error: CyclicCandidateError,
    request: CreativeStretchRequest<'_>,
) -> CreativeStretchError {
    match error {
        CyclicCandidateError::InvalidChannels => CreativeStretchError::InvalidChannelCount,
        CyclicCandidateError::PartialFrame => CreativeStretchError::PartialFrame,
        CyclicCandidateError::InvalidSampleRate => CreativeStretchError::UnsupportedSampleRate,
        CyclicCandidateError::NonFiniteInput => CreativeStretchError::NonFiniteInput,
        CyclicCandidateError::InvalidCycle => CreativeStretchError::InvalidCycle,
        CyclicCandidateError::InvalidEmptyTarget if request.input.is_empty() => {
            CreativeStretchError::EmptyInput
        }
        CyclicCandidateError::InvalidEmptyTarget => CreativeStretchError::ZeroTargetFrames,
        CyclicCandidateError::UnsupportedCompression | CyclicCandidateError::UnsupportedRatio => {
            CreativeStretchError::UnsupportedTargetFrames
        }
        CyclicCandidateError::ExactIntegerLimit
        | CyclicCandidateError::ArithmeticOverflow
        | CyclicCandidateError::AllocationOverflow => CreativeStretchError::SizeOverflow,
    }
}

/// Render one creative stretch through Signal's admitted character renderer.
///
/// The result contains exactly `request.target_frames * request.channels`
/// finite interleaved samples. Unsupported requests return a typed error;
/// this function never falls back to the transparent stretcher.
pub fn render_creative_stretch(
    request: CreativeStretchRequest<'_>,
) -> Result<Vec<Sample>, CreativeStretchError> {
    let cycle_us = validate_request(request)?;
    match request.character {
        CreativeStretchCharacter::Dream => render_dream(DreamCandidateRequest {
            input: request.input,
            channels: usize::from(request.channels),
            sample_rate: request.sample_rate.0,
            target_frames: request.target_frames,
            seed: ADMISSION_SEED,
            space: request.space,
        })
        .map_err(CreativeStretchError::from),
        CreativeStretchCharacter::Cyclic => render_cyclic(CyclicRequest {
            input: request.input,
            channels: usize::from(request.channels),
            sample_rate: request.sample_rate.0,
            target_frames: request.target_frames,
            cycle_us: cycle_us.expect("validated Cyclic request carries cycle"),
        })
        .map_err(|error| map_cyclic_error(error, request)),
    }
}

#[cfg(test)]
mod tests;
