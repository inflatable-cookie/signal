use super::*;

/// `render_creative_stretch` returns samples and nothing else. A caller
/// gets no key, receipt, or plan it could cache against.
#[test]
fn creative_render_returns_samples_without_a_cache_handle() {
    let input = mono_input(64);
    let rendered: Vec<Sample> = render_creative_stretch(CreativeStretchRequest::new(
        &input,
        1,
        SAMPLE_RATE,
        input.len() * 4,
        CreativeStretchCharacter::Dream,
    ))
    .expect("admitted Dream request renders");
    assert_eq!(rendered.len(), input.len() * 4);
}
