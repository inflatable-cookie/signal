use super::*;

#[test]
fn cyclic_boundaries_are_rejected_by_preallocation_validation() {
    let input = mono_input(64);
    for target_frames in [input.len() * 2 - 1, input.len() * 8 + 1, input.len() * 16] {
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
        assert_eq!(
            render_creative_stretch(request),
            Err(CreativeStretchError::UnsupportedTargetFrames)
        );
    }
}
