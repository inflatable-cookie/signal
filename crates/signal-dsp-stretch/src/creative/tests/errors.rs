use super::*;

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
            CreativeStretchRequest::new(&[], 1, SAMPLE_RATE, 4, CreativeStretchCharacter::Dream),
            CreativeStretchError::EmptyInput,
        ),
        (
            CreativeStretchRequest::new(&mono, 1, SAMPLE_RATE, 0, CreativeStretchCharacter::Dream),
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
