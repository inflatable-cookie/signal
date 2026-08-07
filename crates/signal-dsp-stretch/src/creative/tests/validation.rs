use super::*;

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
fn public_cyclic_validation_covers_every_exact_target_in_the_domain() {
    for source_frames in [1, 2, 3, 257] {
        let input = mono_input(source_frames);
        let minimum = source_frames * CREATIVE_STRETCH_CYCLIC_MIN_RATIO;
        let maximum = source_frames * CREATIVE_STRETCH_CYCLIC_MAX_RATIO;

        for target_frames in minimum..=maximum {
            let request = CreativeStretchRequest::new(
                &input,
                1,
                SAMPLE_RATE,
                target_frames,
                CreativeStretchCharacter::Cyclic,
            );
            assert_eq!(validate_request(request), Ok(Some(48_000)));
        }

        for target_frames in [minimum - 1, maximum + 1] {
            let request = CreativeStretchRequest::new(
                &input,
                1,
                SAMPLE_RATE,
                target_frames,
                CreativeStretchCharacter::Cyclic,
            );
            assert_eq!(
                validate_request(request),
                Err(CreativeStretchError::UnsupportedTargetFrames)
            );
        }
    }
}
