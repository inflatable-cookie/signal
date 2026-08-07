use super::*;

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
fn public_cyclic_matches_private_renderer_across_the_domain() {
    let mono = mono_input(64);
    let stereo = stereo_input(64);
    let cycles = [
        (Some(CREATIVE_STRETCH_MIN_CYCLE), 5_000),
        (None, 48_000),
        (Some(CREATIVE_STRETCH_MAX_CYCLE), 90_000),
    ];

    for target_frames in cyclic_parity_targets(mono.len()) {
        for &(cycle, cycle_us) in &cycles {
            let mut mono_request = CreativeStretchRequest::new(
                &mono,
                1,
                SAMPLE_RATE,
                target_frames,
                CreativeStretchCharacter::Cyclic,
            );
            if let Some(cycle) = cycle {
                mono_request = mono_request.with_cycle(cycle);
            }
            let public_mono =
                render_creative_stretch(mono_request).expect("public Cyclic mono render");
            let private_mono = cyclic_private_render(&mono, 1, target_frames, cycle_us);
            assert_eq!(public_mono, private_mono);
            assert_eq!(public_mono.len(), target_frames);
            assert!(public_mono.iter().all(|sample| sample.is_finite()));

            let mut stereo_request = CreativeStretchRequest::new(
                &stereo,
                2,
                SAMPLE_RATE,
                target_frames,
                CreativeStretchCharacter::Cyclic,
            );
            if let Some(cycle) = cycle {
                stereo_request = stereo_request.with_cycle(cycle);
            }
            let public_stereo =
                render_creative_stretch(stereo_request).expect("public Cyclic stereo render");
            let private_stereo = cyclic_private_render(&stereo, 2, target_frames, cycle_us);
            assert_eq!(public_stereo, private_stereo);
            assert_eq!(public_stereo.len(), target_frames * 2);
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
        for target_frames in [mono.len() * 5 / 2, mono.len() * 5, mono.len() * 15 / 2] {
            let output = render_creative_stretch(CreativeStretchRequest::new(
                &input,
                2,
                SAMPLE_RATE,
                target_frames,
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
}
