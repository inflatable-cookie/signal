        fn owner_s01() -> Result<(), String> {
            let empty = CandidateRequest {
                input: &[],
                channels: 1,
                sample_rate: SAMPLE_RATE,
                target_frames: 0,
                seed: ADMISSION_SEED,
                space: 0.0,
            };
            assert_eq!(render(empty).unwrap(), Vec::<f32>::new());

            let mono = tone(4_096, 1, 440.0);
            let mono_output = render(mono_request(&mono, 4, ADMISSION_SEED)).unwrap();
            finite_exact_endpoints(&mono_output, 16_384, 1);
            let stereo = tone(4_096, 2, 440.0);
            let stereo_output = render(stereo_request(&stereo, 16, ADMISSION_SEED, 0.5)).unwrap();
            finite_exact_endpoints(&stereo_output, 65_536, 2);

            let mut request = mono_request(&mono, 4, ADMISSION_SEED);
            request.channels = 0;
            assert_eq!(render(request), Err(CandidateError::InvalidChannels));
            request.channels = 3;
            assert_eq!(render(request), Err(CandidateError::InvalidChannels));
            let partial = CandidateRequest {
                input: &[0.0, 0.0, 0.0],
                channels: 2,
                sample_rate: SAMPLE_RATE,
                target_frames: 4,
                seed: 0,
                space: 0.0,
            };
            assert_eq!(render(partial), Err(CandidateError::PartialFrame));
            request = mono_request(&mono, 4, ADMISSION_SEED);
            request.sample_rate = 7_999;
            assert_eq!(render(request), Err(CandidateError::InvalidSampleRate));
            request.sample_rate = 192_001;
            assert_eq!(render(request), Err(CandidateError::InvalidSampleRate));
            let non_finite = [f32::NAN, f32::INFINITY];
            assert_eq!(
                render(mono_request(&non_finite, 4, ADMISSION_SEED)),
                Err(CandidateError::NonFiniteInput)
            );
            request = mono_request(&mono, 4, ADMISSION_SEED);
            request.space = f32::NAN;
            assert_eq!(render(request), Err(CandidateError::InvalidSpace));
            request.space = -f32::EPSILON;
            assert_eq!(render(request), Err(CandidateError::InvalidSpace));
            request.space = f32::from_bits(1.0_f32.to_bits() + 1);
            assert_eq!(render(request), Err(CandidateError::InvalidSpace));
            request = empty;
            request.target_frames = 4;
            assert_eq!(render(request), Err(CandidateError::EmptyInput));
            request = mono_request(&mono, 4, ADMISSION_SEED);
            request.target_frames = 0;
            assert_eq!(render(request), Err(CandidateError::ZeroTarget));
            request.target_frames = mono.len() * 3;
            assert_eq!(render(request), Err(CandidateError::UnsupportedRatio));
            assert_eq!(
                validate_dimensions(MAX_EXACT_INTEGER + 1, 0),
                Err(CandidateError::SizeOverflow)
            );
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_request_preallocation() {
            owner_s01().unwrap();
        }

        #[allow(clippy::manual_div_ceil)]
        fn owner_s02() -> Result<(), String> {
            assert_eq!(fft_size(8_000), 8_192);
            assert_eq!(fft_size(11_025), 8_192);
            assert_eq!(fft_size(44_100), 32_768);
            assert_eq!(fft_size(48_000), 32_768);
            assert_eq!(fft_size(96_000), 65_536);
            assert_eq!(fft_size(192_000), 131_072);
            assert_eq!(round_half_up_two_thirds(48_000), 32_000);
            assert_eq!(nearest_power_of_two_ties_up(6), 8);
            assert_eq!(nearest_power_of_two_ties_up(5), 4);

            let input = vec![0.0; SYNTHETIC_SOURCE_FRAMES];
            for ratio in RATIOS {
                let request = mono_request(&input, ratio, ADMISSION_SEED);
                let plan = RenderPlan::new(&request).unwrap();
                assert_eq!(plan.blocks, (plan.target_frames + plan.hop - 1) / plan.hop);
                let indices = [0, 1, plan.blocks / 2, plan.blocks - 1, plan.blocks];
                let mut previous = f64::NEG_INFINITY;
                for index in indices {
                    let center = plan.source_center(index).unwrap();
                    let output_frame = index * plan.hop;
                    let expected = ((2 * output_frame + 1) as u128 * plan.source_frames as u128)
                        as f64
                        / (2 * plan.target_frames) as f64
                        - 0.5;
                    assert_eq!(center.to_bits(), expected.to_bits());
                    assert!(center > previous);
                    previous = center;
                }
                for index in 1..=plan.blocks {
                    assert!(
                        plan.source_center(index).unwrap() > plan.source_center(index - 1).unwrap()
                    );
                }
            }
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_transform_map() {
            owner_s02().unwrap();
        }

        fn owner_s03() -> Result<(), String> {
            let (window, gain) = periodic_hann(32_768);
            assert_eq!(window[0], 0.0);
            assert!(window[1] > 0.0);
            assert_eq!(window[32_768 / 2], 1.0);
            assert!(window[32_768 / 4] > window[1]);
            assert!(window[32_768 - 1] > 0.0);
            assert!(gain.is_finite() && gain > 1.0);
            for rate in [8_000, 48_000, 192_000] {
                let (_, rate_gain) = periodic_hann(fft_size(rate));
                assert!((rate_gain - (8.0_f64 / 3.0).sqrt()).abs() < 1.0e-12);
            }
            for fraction in [
                0.0,
                0.25,
                0.5,
                0.75,
                f32::from_bits(1.0_f32.to_bits() - 1) as f64,
            ] {
                let coefficients = cubic_coefficients(fraction);
                assert!((coefficients.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
                assert!(coefficients.iter().all(|value| value.is_finite()));
            }
            let nodes = [1.0_f32, 2.0, 3.0, 4.0];
            assert_eq!(interpolated_sample(&nodes, 1, 0, 4, 0.0), 1.0);
            assert_eq!(interpolated_sample(&nodes, 1, 0, 4, 1.0), 2.0);
            assert_eq!(interpolated_sample(&nodes, 1, 0, 4, 2.0), 3.0);
            assert_eq!(interpolated_sample(&nodes, 1, 0, 4, 3.0), 4.0);
            assert_eq!(interpolated_sample(&nodes, 1, 0, 4, -8.0), 0.0);
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_window_interpolation_gain() {
            owner_s03().unwrap();
        }

        fn owner_s04() -> Result<(), String> {
            assert_eq!(mix64(0), 0x0000_0000_0000_0000);
            assert_eq!(mix64(1), 0x5692_161d_100b_05e5);
            assert_eq!(mix64(u64::MAX), 0xb4d0_55fc_f2cb_bd7b);
            assert_eq!(FRAME_TAG, 0x454d_4152_4657_4e52);
            assert_eq!(BIN_TAG, 0x3030_4e49_4257_4e52);
            assert_eq!(BASE_TAG, 0x3045_5341_4257_4e52);
            assert_eq!(SPACE_TAG, 0x4543_4150_5357_4e52);
            assert_eq!(TEST_TAG, 0x3054_5345_5457_4e52);
            let base = address(ADMISSION_SEED, 7, 11, BASE_TAG);
            let space = address(ADMISSION_SEED, 7, 11, SPACE_TAG);
            assert_eq!(base, 0xaefa_0630_73d5_e350);
            assert_eq!(space, 0x4afa_5260_8a58_dffb);
            assert_eq!(high_53(base), 0x0015_df40_c60e_7abc);
            assert_eq!(high_53(space), 0x0009_5f4a_4c11_4b1b);
            assert!(phase(base) >= -std::f64::consts::PI && phase(base) < std::f64::consts::PI);
            assert!(phase(space) >= -std::f64::consts::PI && phase(space) < std::f64::consts::PI);

            let mut spectrum = vec![Complex32::new(0.0, 0.0); 8];
            spectrum[0] = Complex32::new(4.0, 1.0);
            spectrum[1] = Complex32::new(1.0, 2.0);
            spectrum[4] = Complex32::new(-3.0, 1.0);
            renew_mono(&mut spectrum, 7, ADMISSION_SEED);
            assert_eq!(spectrum[0], Complex32::new(4.0, 0.0));
            assert_eq!(spectrum[4], Complex32::new(-3.0, 0.0));
            assert!((spectrum[1].norm() - 5.0_f32.sqrt()).abs() < 1.0e-5);
            assert_eq!(spectrum[7], spectrum[1].conj());
            assert_eq!(spectrum[6], spectrum[2].conj());
            assert_ne!(
                address(ADMISSION_SEED, 7, 11, BASE_TAG),
                address(ADMISSION_SEED, 8, 11, BASE_TAG)
            );
            assert_ne!(
                address(ADMISSION_SEED, 7, 11, BASE_TAG),
                address(ADMISSION_SEED, 7, 12, BASE_TAG)
            );
            assert_ne!(
                address(ADMISSION_SEED, 7, 11, BASE_TAG),
                address(ADMISSION_SEED ^ 1, 7, 11, BASE_TAG)
            );
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_counter_mono_spectrum() {
            owner_s04().unwrap();
        }

        fn owner_s05() -> Result<(), String> {
            assert_eq!(frequency_weight(0.0), 0.0);
            assert_eq!(frequency_weight(250.0), 0.0);
            assert!((frequency_weight(875.0) - 0.5).abs() < 1.0e-12);
            assert_eq!(frequency_weight(1_500.0), 1.0);
            assert_eq!(frequency_weight(24_000.0), 1.0);

            for space in [0.0_f32, 0.5, 1.0] {
                let mut left = vec![Complex32::new(0.0, 0.0); 8];
                let mut right = vec![Complex32::new(0.0, 0.0); 8];
                left[1] = Complex32::new(1.0, 2.0);
                right[1] = Complex32::new(-2.0, 0.5);
                let left_norm = left[1].norm();
                let right_norm = right[1].norm();
                renew_stereo(&mut left, &mut right, 3, ADMISSION_SEED, space, 48_000);
                assert!((left[1].norm() - left_norm).abs() < 1.0e-5);
                assert!((right[1].norm() - right_norm).abs() < 1.0e-5);
                assert_eq!(left[7], left[1].conj());
                assert_eq!(right[7], right[1].conj());
            }

            for relation in 0..4 {
                for space in [0.0_f32, 1.0] {
                    let mut left = vec![Complex32::new(0.0, 0.0); 16];
                    let mut right = vec![Complex32::new(0.0, 0.0); 16];
                    left[3] = Complex32::new(0.75, -0.25);
                    right[3] = match relation {
                        0 => left[3],
                        1 => -left[3],
                        2 => Complex32::new(-left[3].re, -left[3].im),
                        _ => Complex32::new(-0.1, 0.6),
                    };
                    let source_left = left[3];
                    let source_right = right[3];
                    renew_stereo(&mut left, &mut right, 4, ADMISSION_SEED, space, 48_000);
                    if space == 0.0 {
                        let left_ratio = left[3] / source_left;
                        let right_ratio = right[3] / source_right;
                        assert!((left_ratio - right_ratio).norm() < 1.0e-6);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_linked_stereo_space() {
            owner_s05().unwrap();
        }

        fn owner_s06() -> Result<(), String> {
            for ratio in RATIOS {
                let hop = fft_size(SAMPLE_RATE) / 2;
                for frames in [1, hop - 1, hop, hop + 1, 2 * hop, 2 * hop + 1] {
                    let input = tone(frames, 1, 440.0);
                    let request = mono_request(&input, ratio, ADMISSION_SEED);
                    let plan = RenderPlan::new(&request).unwrap();
                    let output = render(request).unwrap();
                    finite_exact_endpoints(&output, frames * ratio, 1);
                    assert_eq!(boundary_envelope(0, &plan), 0.0);
                    assert_eq!(boundary_envelope(plan.target_frames - 1, &plan), 0.0);
                    if plan.head_extent > 1 {
                        assert!(boundary_envelope(1, &plan) > 0.0);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_blend_boundary_crop() {
            owner_s06().unwrap();
        }

        fn edge_fixture(kind: usize, frames: usize) -> Vec<f32> {
            let mut input = vec![0.0_f32; frames];
            match kind {
                0 => {}
                1 => input[0] = 0.5,
                2 => input.fill(0.25),
                3 => input[frames / 2] = 1.0,
                4 => input.fill(-0.25),
                5 => input = tone(frames, 1, 220.0),
                6 => input[frames / 3] = 1.0,
                _ => input = tone(frames, 1, 880.0),
            }
            input
        }

        fn owner_s07() -> Result<(), String> {
            let fft = fft_size(SAMPLE_RATE);
            let lengths = [1, fft / 2, fft / 2, fft / 2, fft, fft, fft + 1, fft + 1];
            for (kind, frames) in lengths.into_iter().enumerate() {
                let input = edge_fixture(kind, frames);
                for ratio in RATIOS {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).unwrap();
                    finite_exact_endpoints(&output, frames * ratio, 1);
                    if kind == 0 {
                        assert!(output
                            .iter()
                            .all(|sample| sample.to_bits() == 0.0_f32.to_bits()));
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_edge_silence_matrix() {
            owner_s07().unwrap();
        }

        fn owner_s08() -> Result<(), String> {
            let input = tone(4_096, 1, 440.0);
            let first = render(mono_request(&input, 8, ADMISSION_SEED)).unwrap();
            let second = render(mono_request(&input, 8, ADMISSION_SEED)).unwrap();
            assert_eq!(first, second);
            let changed = render(mono_request(&input, 8, ADMISSION_SEED ^ 1)).unwrap();
            assert_ne!(first, changed);
            let extreme = render(mono_request(&input, 8, u64::MAX)).unwrap();
            finite_exact_endpoints(&extreme, 32_768, 1);
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_determinism_seed() {
            owner_s08().unwrap();
        }

        fn owner_s09() -> Result<(), String> {
            let low = tone(fft_size(8_000), 1, 440.0);
            let low_request = CandidateRequest {
                input: &low,
                channels: 1,
                sample_rate: 8_000,
                target_frames: low.len() * 4,
                seed: ADMISSION_SEED,
                space: 0.0,
            };
            let low_plan = RenderPlan::new(&low_request).unwrap();
            assert_eq!(low_plan.fft_size, 8_192);
            let low_working_bytes = planned_working_bytes(&low_plan);
            assert!(low_working_bytes <= MEMORY_SPEC.max_working_bytes);
            let high_frames = fft_size(192_000);
            let high = tone(high_frames, 2, 440.0);
            let high_request = CandidateRequest {
                input: &high,
                channels: 2,
                sample_rate: 192_000,
                target_frames: high_frames * 4,
                seed: ADMISSION_SEED,
                space: 0.5,
            };
            let high_plan = RenderPlan::new(&high_request).unwrap();
            assert_eq!(high_plan.fft_size, 131_072);
            let high_working_bytes = planned_working_bytes(&high_plan);
            assert!(high_working_bytes <= MEMORY_SPEC.max_working_bytes);
            let measurement = AllocationMeasurement::begin();
            let output = render(high_request).unwrap();
            let (peak_growth, processing_allocations) = measurement.finish();
            finite_exact_endpoints(&output, high_frames * 4, 2);
            assert_eq!(processing_allocations, 0);
            let working_peak = peak_growth.saturating_sub(output.capacity() * size_of::<f32>());
            assert!(working_peak <= MEMORY_SPEC.max_working_bytes);

            let longer = tone(high_frames * 8, 2, 440.0);
            let longer_request = CandidateRequest {
                input: &longer,
                channels: 2,
                sample_rate: 192_000,
                target_frames: high_frames * 8 * 4,
                seed: ADMISSION_SEED,
                space: 0.5,
            };
            let longer_plan = RenderPlan::new(&longer_request).unwrap();
            assert_eq!(planned_working_bytes(&longer_plan), high_working_bytes);
            let measurement = AllocationMeasurement::begin();
            let longer_output = render(longer_request).unwrap();
            let (longer_peak_growth, longer_processing_allocations) = measurement.finish();
            finite_exact_endpoints(&longer_output, high_frames * 8 * 4, 2);
            assert_eq!(longer_processing_allocations, 0);
            let longer_working_peak = longer_peak_growth
                .saturating_sub(longer_output.capacity() * size_of::<f32>());
            assert!(longer_working_peak <= MEMORY_SPEC.max_working_bytes);
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_allocation_memory() {
            owner_s09().unwrap();
        }

        fn owner_s10() -> Result<(), String> {
            let module = include_str!("../mod.rs");
            let plan = include_str!("../plan.rs");
            let analysis = include_str!("../analysis.rs");
            let stereo = include_str!("../stereo.rs");
            let synthesis = include_str!("../synthesis.rs");
            let lib = include_str!("../../lib.rs");
            let cargo = include_str!("../../../Cargo.toml");
            let combined = [module, plan, analysis, stereo, synthesis].concat();
            for forbidden in [
                "previous_phase",
                "peak_track",
                "transient_state",
                "material_separator",
                "alignment_search",
                "recurrence",
                "limiter",
                "post_render_gain",
            ] {
                assert!(
                    !combined.contains(forbidden),
                    "forbidden owner: {forbidden}"
                );
            }
            assert!(lib.contains(
                "#[cfg_attr(test, macro_use)]\nmod creative_direct_renewal_dream;"
            ));
            assert!(!lib.contains("pub mod creative_direct_renewal_dream"));
            assert!(!cargo.contains("creative-direct-renewal"));
            assert!(!combined.contains("std::fs"));
            assert_eq!(
                DIRECT_RENEWAL_DREAM_ENGINE_VERSION,
                "signal-creative-direct-renewal-dream-v2"
            );
            assert_eq!(
                DIRECT_RENEWAL_DREAM_RECEIPT_SCHEMA,
                "signal.creative-direct-renewal.receipt.v1"
            );
            assert_eq!(
                DIRECT_RENEWAL_DREAM_SUMMARY_SCHEMA,
                "signal.creative-direct-renewal.summary.v1"
            );
            Ok(())
        }

        #[test]
        fn direct_renewal_dream_structural_single_timeline_private_surface() {
            owner_s10().unwrap();
        }
