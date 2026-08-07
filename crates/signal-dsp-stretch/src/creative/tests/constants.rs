use super::*;

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
        "signal-creative-stretch-v4"
    );
    assert_eq!(CREATIVE_STRETCH_DREAM_MIN_RATIO, 4);
    assert_eq!(CREATIVE_STRETCH_DREAM_MAX_RATIO, 16);
    assert_eq!(CREATIVE_STRETCH_CYCLIC_MIN_RATIO, 2);
    assert_eq!(CREATIVE_STRETCH_CYCLIC_MAX_RATIO, 8);
    assert_eq!(
        CreativeStretchCharacter::Dream.ratio_domain(),
        CreativeStretchRatioDomain::Continuous {
            minimum: 4,
            maximum: 16,
        }
    );
    assert_eq!(
        CreativeStretchCharacter::Cyclic.ratio_domain(),
        CreativeStretchRatioDomain::Continuous {
            minimum: 2,
            maximum: 8,
        }
    );
    assert_eq!(CREATIVE_STRETCH_DEFAULT_SPACE.to_bits(), 0.5_f32.to_bits());
    assert_eq!(CREATIVE_STRETCH_MIN_CYCLE, Duration::from_millis(5));
    assert_eq!(CREATIVE_STRETCH_DEFAULT_CYCLE, Duration::from_millis(48));
    assert_eq!(CREATIVE_STRETCH_MAX_CYCLE, Duration::from_millis(90));
    assert_eq!(request.space.to_bits(), 0.5_f32.to_bits());
    assert_eq!(request.cycle, None);
}
