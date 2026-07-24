use crate::creative_cyclic::{
    render as render_cyclic, CandidateError as CyclicCandidateError, Request as CyclicRequest,
};
use crate::creative_direct_renewal_dream::{
    render as render_dream, CandidateError as DreamCandidateError,
    CandidateRequest as DreamCandidateRequest, ADMISSION_SEED,
};
use signal_primitives::{Sample, SampleRate};
use std::time::Duration;

/// Semantic behavior version of the public creative-stretch renderer.
pub const CREATIVE_STRETCH_ENGINE_VERSION: &str = "signal-creative-stretch-v3";

/// Smallest output/input ratio supported by the `Dream` character.
pub const CREATIVE_STRETCH_DREAM_MIN_RATIO: usize = 4;

/// Largest output/input ratio supported by the `Dream` character.
pub const CREATIVE_STRETCH_DREAM_MAX_RATIO: usize = 16;

/// Exact output/input ratios supported by the `Cyclic` character.
pub const CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS: [usize; 3] = [2, 4, 8];

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
            Self::Cyclic => {
                CreativeStretchRatioDomain::Exact(&CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS)
            }
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
    if request.input.len() % channels != 0 {
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
    let supported =
        match request.character {
            CreativeStretchCharacter::Dream => {
                let minimum = source_frames
                    .checked_mul(CREATIVE_STRETCH_DREAM_MIN_RATIO)
                    .ok_or(CreativeStretchError::SizeOverflow)?;
                let maximum = source_frames
                    .checked_mul(CREATIVE_STRETCH_DREAM_MAX_RATIO)
                    .ok_or(CreativeStretchError::SizeOverflow)?;
                (minimum..=maximum).contains(&request.target_frames)
            }
            CreativeStretchCharacter::Cyclic => CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS
                .iter()
                .any(|ratio| {
                    source_frames
                        .checked_mul(*ratio)
                        .is_some_and(|expected| expected == request.target_frames)
                }),
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

    fn dream_parity_targets(source_frames: usize) -> [usize; 9] {
        [
            source_frames * 4,
            source_frames * 4 + 1,
            source_frames * 9 / 2,
            source_frames * 6,
            source_frames * 8,
            source_frames * 10,
            source_frames * 31 / 2,
            source_frames * 16 - 1,
            source_frames * 16,
        ]
    }

    fn dream_private_render(
        input: &[Sample],
        channels: usize,
        target_frames: usize,
        space: f32,
    ) -> Vec<Sample> {
        render_dream(DreamCandidateRequest {
            input,
            channels,
            sample_rate: SAMPLE_RATE.0,
            target_frames,
            seed: ADMISSION_SEED,
            space,
        })
        .expect("private reference render")
    }

    fn cyclic_private_render(
        input: &[Sample],
        channels: usize,
        target_frames: usize,
        cycle_us: u32,
    ) -> Vec<Sample> {
        render_cyclic(CyclicRequest {
            input,
            channels,
            sample_rate: SAMPLE_RATE.0,
            target_frames,
            cycle_us,
        })
        .expect("private Cyclic reference render")
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
            "signal-creative-stretch-v3"
        );
        assert_eq!(CREATIVE_STRETCH_DREAM_MIN_RATIO, 4);
        assert_eq!(CREATIVE_STRETCH_DREAM_MAX_RATIO, 16);
        assert_eq!(CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS, [2, 4, 8]);
        assert_eq!(
            CreativeStretchCharacter::Dream.ratio_domain(),
            CreativeStretchRatioDomain::Continuous {
                minimum: 4,
                maximum: 16,
            }
        );
        assert_eq!(
            CreativeStretchCharacter::Cyclic.ratio_domain(),
            CreativeStretchRatioDomain::Exact(&[2, 4, 8])
        );
        assert_eq!(CREATIVE_STRETCH_DEFAULT_SPACE.to_bits(), 0.5_f32.to_bits());
        assert_eq!(CREATIVE_STRETCH_MIN_CYCLE, Duration::from_millis(5));
        assert_eq!(CREATIVE_STRETCH_DEFAULT_CYCLE, Duration::from_millis(48));
        assert_eq!(CREATIVE_STRETCH_MAX_CYCLE, Duration::from_millis(90));
        assert_eq!(request.space.to_bits(), 0.5_f32.to_bits());
        assert_eq!(request.cycle, None);
    }

    #[test]
    fn public_dream_validation_covers_every_exact_target_in_the_domain() {
        for source_frames in [1, 2, 3, 257] {
            let input = mono_input(source_frames);
            let minimum = source_frames * CREATIVE_STRETCH_DREAM_MIN_RATIO;
            let maximum = source_frames * CREATIVE_STRETCH_DREAM_MAX_RATIO;

            for target_frames in minimum..=maximum {
                let request = CreativeStretchRequest::new(
                    &input,
                    1,
                    SAMPLE_RATE,
                    target_frames,
                    CreativeStretchCharacter::Dream,
                );
                assert_eq!(validate_request(request), Ok(None));
            }

            for target_frames in [minimum - 1, maximum + 1] {
                let request = CreativeStretchRequest::new(
                    &input,
                    1,
                    SAMPLE_RATE,
                    target_frames,
                    CreativeStretchCharacter::Dream,
                );
                assert_eq!(
                    validate_request(request),
                    Err(CreativeStretchError::UnsupportedTargetFrames)
                );
            }
        }
    }

    #[test]
    fn public_dream_mono_matches_private_renderer_across_the_domain() {
        let input = mono_input(64);
        for target_frames in dream_parity_targets(input.len()) {
            let public = render_creative_stretch(CreativeStretchRequest::new(
                &input,
                1,
                SAMPLE_RATE,
                target_frames,
                CreativeStretchCharacter::Dream,
            ))
            .expect("public mono render");
            let private = dream_private_render(&input, 1, target_frames, 0.5);

            assert_eq!(public, private);
            assert_eq!(public.len(), target_frames);
            assert!(public.iter().all(|sample| sample.is_finite()));
        }
    }

    #[test]
    fn public_dream_stereo_matches_private_renderer_across_the_domain() {
        let input = stereo_input(64);
        let source_frames = input.len() / 2;
        for target_frames in dream_parity_targets(source_frames) {
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
                let private = dream_private_render(&input, 2, target_frames, space);

                assert_eq!(public, private);
                assert_eq!(public.len(), target_frames * 2);
                assert!(public.iter().all(|sample| sample.is_finite()));
            }
        }
    }

    #[test]
    fn public_cyclic_matches_private_renderer_at_every_ratio_and_cycle_anchor() {
        let mono = mono_input(64);
        let stereo = stereo_input(64);
        let cycles = [
            (Some(CREATIVE_STRETCH_MIN_CYCLE), 5_000),
            (None, 48_000),
            (Some(CREATIVE_STRETCH_MAX_CYCLE), 90_000),
        ];

        for ratio in CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS {
            for (cycle, cycle_us) in cycles {
                let mono_target = mono.len() * ratio;
                let mut mono_request = CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono_target,
                    CreativeStretchCharacter::Cyclic,
                );
                if let Some(cycle) = cycle {
                    mono_request = mono_request.with_cycle(cycle);
                }
                let public_mono =
                    render_creative_stretch(mono_request).expect("public Cyclic mono render");
                let private_mono = cyclic_private_render(&mono, 1, mono_target, cycle_us);
                assert_eq!(public_mono, private_mono);
                assert_eq!(public_mono.len(), mono_target);
                assert!(public_mono.iter().all(|sample| sample.is_finite()));

                let stereo_target = stereo.len() / 2 * ratio;
                let mut stereo_request = CreativeStretchRequest::new(
                    &stereo,
                    2,
                    SAMPLE_RATE,
                    stereo_target,
                    CreativeStretchCharacter::Cyclic,
                );
                if let Some(cycle) = cycle {
                    stereo_request = stereo_request.with_cycle(cycle);
                }
                let public_stereo =
                    render_creative_stretch(stereo_request).expect("public Cyclic stereo render");
                let private_stereo = cyclic_private_render(&stereo, 2, stereo_target, cycle_us);
                assert_eq!(public_stereo, private_stereo);
                assert_eq!(public_stereo.len(), stereo_target * 2);
                assert!(public_stereo.iter().all(|sample| sample.is_finite()));
            }
        }
    }

    #[test]
    fn public_cyclic_cycle_canonicalization_is_integer_round_half_up() {
        let input = mono_input(64);
        let target_frames = input.len() * 2;
        let cases = [
            (Duration::from_nanos(5_000_499), 5_000),
            (Duration::from_nanos(5_000_500), 5_001),
        ];

        for (cycle, cycle_us) in cases {
            let public = render_creative_stretch(
                CreativeStretchRequest::new(
                    &input,
                    1,
                    SAMPLE_RATE,
                    target_frames,
                    CreativeStretchCharacter::Cyclic,
                )
                .with_cycle(cycle),
            )
            .expect("public Cyclic rounded-cycle render");
            let private = cyclic_private_render(&input, 1, target_frames, cycle_us);
            assert_eq!(public, private);
        }
    }

    #[test]
    fn public_cyclic_preserves_linked_stereo_relations() {
        let mono = mono_input(64);
        for anti_phase in [false, true] {
            let mut input = Vec::with_capacity(mono.len() * 2);
            for sample in &mono {
                input.push(*sample);
                input.push(if anti_phase { -*sample } else { *sample });
            }
            let output = render_creative_stretch(CreativeStretchRequest::new(
                &input,
                2,
                SAMPLE_RATE,
                mono.len() * 8,
                CreativeStretchCharacter::Cyclic,
            ))
            .expect("public Cyclic linked-stereo render");
            for frame in output.chunks_exact(2) {
                let relation_error = if anti_phase {
                    frame[0] + frame[1]
                } else {
                    frame[0] - frame[1]
                };
                assert!(relation_error.abs() <= 1e-6);
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

        let cyclic_request = CreativeStretchRequest::new(
            &input,
            1,
            SAMPLE_RATE,
            input.len() * 4,
            CreativeStretchCharacter::Cyclic,
        );
        let first_cyclic = render_creative_stretch(cyclic_request).expect("first Cyclic render");
        let second_cyclic = render_creative_stretch(cyclic_request).expect("second Cyclic render");
        assert_eq!(first_cyclic, second_cyclic);

        let empty_cyclic = render_creative_stretch(CreativeStretchRequest::new(
            &[],
            1,
            SAMPLE_RATE,
            0,
            CreativeStretchCharacter::Cyclic,
        ))
        .expect("empty Cyclic render");
        assert!(empty_cyclic.is_empty());
    }

    #[test]
    fn every_private_error_has_the_frozen_public_mapping() {
        let mappings = [
            (
                DreamCandidateError::InvalidChannels,
                CreativeStretchError::InvalidChannelCount,
            ),
            (
                DreamCandidateError::InvalidSampleRate,
                CreativeStretchError::UnsupportedSampleRate,
            ),
            (
                DreamCandidateError::PartialFrame,
                CreativeStretchError::PartialFrame,
            ),
            (
                DreamCandidateError::NonFiniteInput,
                CreativeStretchError::NonFiniteInput,
            ),
            (
                DreamCandidateError::InvalidSpace,
                CreativeStretchError::InvalidSpace,
            ),
            (
                DreamCandidateError::EmptyInput,
                CreativeStretchError::EmptyInput,
            ),
            (
                DreamCandidateError::ZeroTarget,
                CreativeStretchError::ZeroTargetFrames,
            ),
            (
                DreamCandidateError::UnsupportedRatio,
                CreativeStretchError::UnsupportedTargetFrames,
            ),
            (
                DreamCandidateError::SizeOverflow,
                CreativeStretchError::SizeOverflow,
            ),
            (
                DreamCandidateError::AllocationFailed,
                CreativeStretchError::AllocationFailed,
            ),
            (
                DreamCandidateError::NonFiniteProcessing,
                CreativeStretchError::NonFiniteOutput,
            ),
        ];

        for (private, public) in mappings {
            assert_eq!(CreativeStretchError::from(private), public);
        }

        let input = mono_input(64);
        let request = CreativeStretchRequest::new(
            &input,
            1,
            SAMPLE_RATE,
            input.len() * 2,
            CreativeStretchCharacter::Cyclic,
        );
        let cyclic_mappings = [
            (
                CyclicCandidateError::InvalidChannels,
                CreativeStretchError::InvalidChannelCount,
            ),
            (
                CyclicCandidateError::PartialFrame,
                CreativeStretchError::PartialFrame,
            ),
            (
                CyclicCandidateError::InvalidSampleRate,
                CreativeStretchError::UnsupportedSampleRate,
            ),
            (
                CyclicCandidateError::NonFiniteInput,
                CreativeStretchError::NonFiniteInput,
            ),
            (
                CyclicCandidateError::InvalidCycle,
                CreativeStretchError::InvalidCycle,
            ),
            (
                CyclicCandidateError::InvalidEmptyTarget,
                CreativeStretchError::ZeroTargetFrames,
            ),
            (
                CyclicCandidateError::UnsupportedCompression,
                CreativeStretchError::UnsupportedTargetFrames,
            ),
            (
                CyclicCandidateError::UnsupportedRatio,
                CreativeStretchError::UnsupportedTargetFrames,
            ),
            (
                CyclicCandidateError::ExactIntegerLimit,
                CreativeStretchError::SizeOverflow,
            ),
            (
                CyclicCandidateError::ArithmeticOverflow,
                CreativeStretchError::SizeOverflow,
            ),
            (
                CyclicCandidateError::AllocationOverflow,
                CreativeStretchError::SizeOverflow,
            ),
        ];
        for (private, public) in cyclic_mappings {
            assert_eq!(map_cyclic_error(private, request), public);
        }

        let empty_request =
            CreativeStretchRequest::new(&[], 1, SAMPLE_RATE, 2, CreativeStretchCharacter::Cyclic);
        assert_eq!(
            map_cyclic_error(CyclicCandidateError::InvalidEmptyTarget, empty_request),
            CreativeStretchError::EmptyInput
        );
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
                    mono.len() * 17,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::UnsupportedTargetFrames,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 4,
                    CreativeStretchCharacter::Dream,
                )
                .with_cycle(CREATIVE_STRETCH_DEFAULT_CYCLE),
                CreativeStretchError::UnsupportedCharacterControl,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 2,
                    CreativeStretchCharacter::Cyclic,
                )
                .with_space(0.0),
                CreativeStretchError::UnsupportedCharacterControl,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 2,
                    CreativeStretchCharacter::Cyclic,
                )
                .with_space(f32::NAN),
                CreativeStretchError::UnsupportedCharacterControl,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 2,
                    CreativeStretchCharacter::Cyclic,
                )
                .with_cycle(Duration::from_nanos(4_999_999)),
                CreativeStretchError::InvalidCycle,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 2,
                    CreativeStretchCharacter::Cyclic,
                )
                .with_cycle(Duration::from_nanos(90_000_001)),
                CreativeStretchError::InvalidCycle,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 16,
                    CreativeStretchCharacter::Cyclic,
                ),
                CreativeStretchError::UnsupportedTargetFrames,
            ),
            (
                CreativeStretchRequest::new(
                    &mono,
                    1,
                    SAMPLE_RATE,
                    mono.len() * 2,
                    CreativeStretchCharacter::Dream,
                ),
                CreativeStretchError::UnsupportedTargetFrames,
            ),
        ];

        for (request, expected) in cases {
            assert_eq!(render_creative_stretch(request), Err(expected));
        }
    }

    #[test]
    fn cyclic_sixteen_is_rejected_by_preallocation_validation() {
        let input = mono_input(64);
        let request = CreativeStretchRequest::new(
            &input,
            1,
            SAMPLE_RATE,
            input.len() * 16,
            CreativeStretchCharacter::Cyclic,
        );
        assert_eq!(
            validate_request(request),
            Err(CreativeStretchError::UnsupportedTargetFrames)
        );
        assert_eq!(
            render_creative_stretch(request),
            Err(CreativeStretchError::UnsupportedTargetFrames)
        );
    }
}
