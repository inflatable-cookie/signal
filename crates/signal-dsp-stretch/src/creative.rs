use crate::creative_direct_renewal_dream::{
    render, CandidateError, CandidateRequest, ADMISSION_SEED,
};
use signal_primitives::{Sample, SampleRate};

/// Semantic behavior version of the public creative-stretch renderer.
pub const CREATIVE_STRETCH_ENGINE_VERSION: &str = "signal-creative-stretch-v1";

/// Exact output/input ratios supported by [`render_creative_stretch`].
pub const CREATIVE_STRETCH_SUPPORTED_RATIOS: [usize; 3] = [4, 8, 16];

/// Default preserve-to-widen value used by [`CreativeStretchRequest::new`].
pub const CREATIVE_STRETCH_DEFAULT_SPACE: f32 = 0.5;

/// Creative character requested from Signal's offline renderer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreativeStretchCharacter {
    /// Smooth, fused, musical spectral smear.
    Dream,
}

/// One whole-buffer offline creative-stretch request.
///
/// `target_frames` is authoritative and must equal the source frame count
/// multiplied by `4`, `8`, or `16`. This request allocates and must not be
/// rendered on the audio thread.
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
    /// Semantic creative character. Only [`CreativeStretchCharacter::Dream`]
    /// is currently admitted.
    pub character: CreativeStretchCharacter,
    /// Preserve-to-widen control in the inclusive range `0.0..=1.0`.
    pub space: f32,
}

impl<'a> CreativeStretchRequest<'a> {
    /// Construct a request with [`CREATIVE_STRETCH_DEFAULT_SPACE`].
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
    /// Non-zero output was requested from empty input.
    EmptyInput,
    /// A non-empty input requested zero output frames.
    ZeroTargetFrames,
    /// Target frames did not resolve to exact `4x`, `8x`, or `16x`.
    UnsupportedTargetFrames,
    /// Request geometry or output size exceeded the supported integer range.
    SizeOverflow,
    /// Output allocation failed.
    AllocationFailed,
    /// Rendering produced a non-finite intermediate or output value.
    NonFiniteOutput,
}

impl From<CandidateError> for CreativeStretchError {
    fn from(error: CandidateError) -> Self {
        match error {
            CandidateError::InvalidChannels => Self::InvalidChannelCount,
            CandidateError::InvalidSampleRate => Self::UnsupportedSampleRate,
            CandidateError::PartialFrame => Self::PartialFrame,
            CandidateError::NonFiniteInput => Self::NonFiniteInput,
            CandidateError::InvalidSpace => Self::InvalidSpace,
            CandidateError::EmptyInput => Self::EmptyInput,
            CandidateError::ZeroTarget => Self::ZeroTargetFrames,
            CandidateError::UnsupportedRatio => Self::UnsupportedTargetFrames,
            CandidateError::SizeOverflow => Self::SizeOverflow,
            CandidateError::AllocationFailed => Self::AllocationFailed,
            CandidateError::NonFiniteProcessing => Self::NonFiniteOutput,
        }
    }
}

/// Render one exact-ratio creative stretch through Signal's admitted `Dream`
/// renderer.
///
/// The result contains exactly `request.target_frames * request.channels`
/// finite interleaved samples. Unsupported requests return a typed error;
/// this function never falls back to the transparent stretcher.
pub fn render_creative_stretch(
    request: CreativeStretchRequest<'_>,
) -> Result<Vec<Sample>, CreativeStretchError> {
    match request.character {
        CreativeStretchCharacter::Dream => render(CandidateRequest {
            input: request.input,
            channels: usize::from(request.channels),
            sample_rate: request.sample_rate.0,
            target_frames: request.target_frames,
            seed: ADMISSION_SEED,
            space: request.space,
        })
        .map_err(CreativeStretchError::from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: SampleRate = SampleRate(8_000);

    fn mono_input(frames: usize) -> Vec<Sample> {
        (0..frames)
            .map(|frame| {
                (0.4 * (std::f64::consts::TAU * 220.0 * frame as f64 / f64::from(SAMPLE_RATE.0))
                    .sin()) as Sample
            })
            .collect()
    }

    fn stereo_input(frames: usize) -> Vec<Sample> {
        let mut input = Vec::with_capacity(frames * 2);
        for frame in 0..frames {
            let time = frame as f64 / f64::from(SAMPLE_RATE.0);
            input.push((0.35 * (std::f64::consts::TAU * 180.0 * time).sin()) as Sample);
            input.push((0.25 * (std::f64::consts::TAU * 310.0 * time).sin()) as Sample);
        }
        input
    }

    fn private_render(
        input: &[Sample],
        channels: usize,
        target_frames: usize,
        space: f32,
    ) -> Vec<Sample> {
        render(CandidateRequest {
            input,
            channels,
            sample_rate: SAMPLE_RATE.0,
            target_frames,
            seed: ADMISSION_SEED,
            space,
        })
        .expect("private reference render")
    }

    #[test]
    fn public_constants_and_request_default_are_frozen() {
        let input = mono_input(64);
        let request = CreativeStretchRequest::new(
            &input,
            1,
            SAMPLE_RATE,
            input.len() * 8,
            CreativeStretchCharacter::Dream,
        );

        assert_eq!(
            CREATIVE_STRETCH_ENGINE_VERSION,
            "signal-creative-stretch-v1"
        );
        assert_eq!(CREATIVE_STRETCH_SUPPORTED_RATIOS, [4, 8, 16]);
        assert_eq!(CREATIVE_STRETCH_DEFAULT_SPACE.to_bits(), 0.5_f32.to_bits());
        assert_eq!(request.space.to_bits(), 0.5_f32.to_bits());
    }

    #[test]
    fn public_mono_matches_private_renderer_at_every_ratio() {
        let input = mono_input(64);
        for ratio in CREATIVE_STRETCH_SUPPORTED_RATIOS {
            let target_frames = input.len() * ratio;
            let public = render_creative_stretch(CreativeStretchRequest::new(
                &input,
                1,
                SAMPLE_RATE,
                target_frames,
                CreativeStretchCharacter::Dream,
            ))
            .expect("public mono render");
            let private = private_render(&input, 1, target_frames, 0.5);

            assert_eq!(public, private);
            assert_eq!(public.len(), target_frames);
            assert!(public.iter().all(|sample| sample.is_finite()));
        }
    }

    #[test]
    fn public_stereo_matches_private_renderer_at_every_ratio_and_space() {
        let input = stereo_input(64);
        for ratio in CREATIVE_STRETCH_SUPPORTED_RATIOS {
            let target_frames = input.len() / 2 * ratio;
            for space in [0.0, 0.5, 1.0] {
                let public = render_creative_stretch(
                    CreativeStretchRequest::new(
                        &input,
                        2,
                        SAMPLE_RATE,
                        target_frames,
                        CreativeStretchCharacter::Dream,
                    )
                    .with_space(space),
                )
                .expect("public stereo render");
                let private = private_render(&input, 2, target_frames, space);

                assert_eq!(public, private);
                assert_eq!(public.len(), target_frames * 2);
                assert!(public.iter().all(|sample| sample.is_finite()));
            }
        }
    }

    #[test]
    fn public_request_is_deterministic_and_empty_zero_succeeds() {
        let input = mono_input(64);
        let request = CreativeStretchRequest::new(
            &input,
            1,
            SAMPLE_RATE,
            input.len() * 4,
            CreativeStretchCharacter::Dream,
        );
        let first = render_creative_stretch(request).expect("first render");
        let second = render_creative_stretch(request).expect("second render");
        assert_eq!(first, second);

        let empty = render_creative_stretch(CreativeStretchRequest::new(
            &[],
            1,
            SAMPLE_RATE,
            0,
            CreativeStretchCharacter::Dream,
        ))
        .expect("empty render");
        assert!(empty.is_empty());
    }

    #[test]
    fn every_private_error_has_the_frozen_public_mapping() {
        let mappings = [
            (
                CandidateError::InvalidChannels,
                CreativeStretchError::InvalidChannelCount,
            ),
            (
                CandidateError::InvalidSampleRate,
                CreativeStretchError::UnsupportedSampleRate,
            ),
            (
                CandidateError::PartialFrame,
                CreativeStretchError::PartialFrame,
            ),
            (
                CandidateError::NonFiniteInput,
                CreativeStretchError::NonFiniteInput,
            ),
            (
                CandidateError::InvalidSpace,
                CreativeStretchError::InvalidSpace,
            ),
            (CandidateError::EmptyInput, CreativeStretchError::EmptyInput),
            (
                CandidateError::ZeroTarget,
                CreativeStretchError::ZeroTargetFrames,
            ),
            (
                CandidateError::UnsupportedRatio,
                CreativeStretchError::UnsupportedTargetFrames,
            ),
            (
                CandidateError::SizeOverflow,
                CreativeStretchError::SizeOverflow,
            ),
            (
                CandidateError::AllocationFailed,
                CreativeStretchError::AllocationFailed,
            ),
            (
                CandidateError::NonFiniteProcessing,
                CreativeStretchError::NonFiniteOutput,
            ),
        ];

        for (private, public) in mappings {
            assert_eq!(CreativeStretchError::from(private), public);
        }
    }

    #[test]
    fn public_entry_reports_request_errors_without_clamping_or_fallback() {
        let mono = mono_input(64);
        let cases = [
            (
                CreativeStretchRequest::new(
                    &mono,
                    0,
                    SAMPLE_RATE,
                    mono.len() * 4,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::InvalidChannelCount,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SampleRate(7_999),
                    mono.len() * 4,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::UnsupportedSampleRate,
            ),
            (
                CreativeStretchRequest::new(
                    &mono[..63],
                    2,
                    SAMPLE_RATE,
                    63 * 4,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::PartialFrame,
            ),
            (
                CreativeStretchRequest::new(
                    &[f32::NAN],
                    1,
                    SAMPLE_RATE,
                    4,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::NonFiniteInput,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 4,
                    CreativeStretchCharacter::Dream,
                )
                .with_space(1.01),
                CreativeStretchError::InvalidSpace,
            ),
            (
                CreativeStretchRequest::new(
                    &[],
                    1,
                    SAMPLE_RATE,
                    4,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::EmptyInput,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    0,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::ZeroTargetFrames,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 6,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::UnsupportedTargetFrames,
            ),
        ];

        for (request, expected) in cases {
            assert_eq!(render_creative_stretch(request), Err(expected));
        }
    }
}
