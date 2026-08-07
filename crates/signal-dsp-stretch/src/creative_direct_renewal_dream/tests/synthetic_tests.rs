        #[test]
        fn direct_renewal_dream_synthetic_integrity_crest_discontinuity() {
            owner_y01().unwrap();
        }

        #[test]
        fn direct_renewal_dream_synthetic_pitch_diagnostic() {
            owner_y02().unwrap();
        }

        #[test]
        fn direct_renewal_dream_synthetic_impulse_diagnostic() {
            owner_y03().unwrap();
        }

        #[test]
        fn direct_renewal_dream_synthetic_periodicity_modulation_gap() {
            owner_y04().unwrap();
        }

        #[test]
        fn direct_renewal_dream_synthetic_linked_stereo_inventory() {
            owner_y05().unwrap();
        }

        #[test]
        fn continuous_direct_renewal_dream_structural_target_domain() {
            for source_frames in 1..=64 {
                assert_eq!(
                    validate_dimensions(source_frames, 4 * source_frames - 1),
                    Err(CandidateError::UnsupportedRatio)
                );
                for target_frames in 4 * source_frames..=16 * source_frames {
                    assert_eq!(
                        validate_dimensions(source_frames, target_frames),
                        Ok(()),
                        "source_frames={source_frames} target_frames={target_frames}"
                    );
                }
                assert_eq!(
                    validate_dimensions(source_frames, 16 * source_frames + 1),
                    Err(CandidateError::UnsupportedRatio)
                );
            }
        }

        #[test]
        fn continuous_direct_renewal_dream_synthetic_interior_ratios() {
            let source_frames = 4_096;
            let mono = tone(source_frames, 1, 440.0);
            let stereo = tone(source_frames, 2, 440.0);
            for (numerator, denominator) in [(9, 2), (6, 1), (10, 1), (31, 2)] {
                let target_frames = source_frames * numerator / denominator;
                let mono_request = CandidateRequest {
                    input: &mono,
                    channels: 1,
                    sample_rate: SAMPLE_RATE,
                    target_frames,
                    seed: ADMISSION_SEED,
                    space: 0.5,
                };
                let mono_output = render(mono_request).unwrap();
                finite_exact_endpoints(&mono_output, target_frames, 1);
                assert_eq!(render(mono_request).unwrap(), mono_output);

                let stereo_request = CandidateRequest {
                    input: &stereo,
                    channels: 2,
                    sample_rate: SAMPLE_RATE,
                    target_frames,
                    seed: ADMISSION_SEED,
                    space: 0.0,
                };
                let stereo_output = render(stereo_request).unwrap();
                finite_exact_endpoints(&stereo_output, target_frames, 2);
                assert_eq!(render(stereo_request).unwrap(), stereo_output);
                assert!(stereo_output
                    .chunks_exact(2)
                    .all(|frame| frame[0].to_bits() == frame[1].to_bits()));
            }
        }
