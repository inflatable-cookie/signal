use super::*;

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
