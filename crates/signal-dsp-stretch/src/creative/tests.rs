use super::*;
use crate::StretchBackendTier;

const SAMPLE_RATE: SampleRate = SampleRate(8_000);

/// Contract `085`, 2026-07-27: creative renders are uncacheable. The
/// surface must not grow a key, receipt, or artifact vocabulary without
/// the enumeration Contract `046` requires of the transparent identity.
#[test]
fn creative_surface_carries_no_cache_or_artifact_vocabulary() {
    // Scan the production module only: this owner names the forbidden
    // identifiers itself in the sibling test module.
    let source = include_str!("mod.rs");
    for forbidden in [
        "CacheIdentity",
        "cache_key",
        "canonical_key",
        "stable_hash",
        "PromotionReceipt",
        "OfflineStretchArtifact",
        "StretchOfflineChunk",
    ] {
        assert!(
            !source.contains(forbidden),
            "creative surface mentions `{forbidden}`; Contract `085` declares \
             creative renders uncacheable, so a cache surface needs a contract \
             change and a named consumer first"
        );
    }
}

/// No stretch tier describes a creative render, so no cache identity can
/// name one. Adding a tier variant breaks this match and this owner.
#[test]
fn no_stretch_tier_describes_a_creative_render() {
    fn is_transparent_tier(tier: StretchBackendTier) -> bool {
        match tier {
            StretchBackendTier::Repitch
            | StretchBackendTier::RealtimePreview
            | StretchBackendTier::OfflineHighQuality => true,
        }
    }

    for tier in [
        StretchBackendTier::Repitch,
        StretchBackendTier::RealtimePreview,
        StretchBackendTier::OfflineHighQuality,
    ] {
        assert!(is_transparent_tier(tier));
        let token = tier.cache_key_token();
        for creative_word in ["creative", "dream", "cyclic"] {
            assert!(
                !token.contains(creative_word),
                "tier token `{token}` names a creative render"
            );
        }
    }
}

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

fn mono_input(frames: usize) -> Vec<Sample> {
    (0..frames)
        .map(|frame| {
            (0.4 * (std::f64::consts::TAU * 220.0 * frame as f64 / f64::from(SAMPLE_RATE.0)).sin())
                as Sample
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

fn cyclic_parity_targets(source_frames: usize) -> [usize; 12] {
    [
        source_frames * 2,
        source_frames * 2 + 1,
        source_frames * 5 / 2,
        source_frames * 3,
        source_frames * 4 - 1,
        source_frames * 4,
        source_frames * 4 + 1,
        source_frames * 5,
        source_frames * 6,
        source_frames * 15 / 2,
        source_frames * 8 - 1,
        source_frames * 8,
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
