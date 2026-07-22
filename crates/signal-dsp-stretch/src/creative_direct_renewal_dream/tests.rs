macro_rules! direct_renewal_dream_tests {
    () => {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::fs::{self, OpenOptions};
        use std::io::Write;
        use std::path::PathBuf;
        use std::process::Command;
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use std::sync::Mutex;

        use rustfft::num_complex::Complex64;

        use crate::creative_direct_renewal_dream::{
            address, boundary_envelope, cubic_coefficients, fft_size, frequency_weight, high_53,
            interpolated_sample, mix64, nearest_power_of_two_ties_up, periodic_hann, phase,
            planned_working_bytes, render, renew_mono, renew_stereo,
            round_half_up_two_thirds, validate_dimensions,
            CandidateError, CandidateRequest, RenderPlan, ADMISSION_SEED, BASE_TAG, BIN_TAG,
            DIRECT_RENEWAL_DREAM_ENGINE_VERSION, DIRECT_RENEWAL_DREAM_RECEIPT_SCHEMA,
            DIRECT_RENEWAL_DREAM_SUMMARY_SCHEMA, FRAME_TAG, MAX_EXACT_INTEGER, SPACE_TAG, TEST_TAG,
        };

        const RATIOS: [usize; 3] = [4, 8, 16];
        const SAMPLE_RATE: u32 = 48_000;
        const SYNTHETIC_SOURCE_FRAMES: usize = 96_000;

        static ALLOCATION_MEASURING: AtomicBool = AtomicBool::new(false);
        static PROCESSING_STARTED: AtomicBool = AtomicBool::new(false);
        static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
        static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);
        static PROCESSING_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
        static ALLOCATION_LOCK: Mutex<()> = Mutex::new(());

        struct DirectRenewalDreamAllocator;

        unsafe impl GlobalAlloc for DirectRenewalDreamAllocator {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                let pointer = unsafe { System.alloc(layout) };
                if !pointer.is_null() {
                    direct_renewal_dream_record_allocation(layout.size());
                }
                pointer
            }

            unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
                let pointer = unsafe { System.alloc_zeroed(layout) };
                if !pointer.is_null() {
                    direct_renewal_dream_record_allocation(layout.size());
                }
                pointer
            }

            unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
                direct_renewal_dream_record_deallocation(layout.size());
                unsafe { System.dealloc(pointer, layout) };
            }

            unsafe fn realloc(
                &self,
                pointer: *mut u8,
                layout: Layout,
                new_size: usize,
            ) -> *mut u8 {
                let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
                if !new_pointer.is_null() {
                    if new_size >= layout.size() {
                        direct_renewal_dream_record_allocation(new_size - layout.size());
                    } else {
                        direct_renewal_dream_record_deallocation(layout.size() - new_size);
                    }
                }
                new_pointer
            }
        }

        #[global_allocator]
        static DIRECT_RENEWAL_DREAM_ALLOCATOR: DirectRenewalDreamAllocator =
            DirectRenewalDreamAllocator;

        fn direct_renewal_dream_record_allocation(bytes: usize) {
            let live = LIVE_BYTES.fetch_add(bytes, Ordering::Relaxed) + bytes;
            if !ALLOCATION_MEASURING.load(Ordering::Relaxed) {
                return;
            }
            if PROCESSING_STARTED.load(Ordering::Relaxed) {
                PROCESSING_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            }
            PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
        }

        fn direct_renewal_dream_record_deallocation(bytes: usize) {
            LIVE_BYTES.fetch_sub(bytes, Ordering::Relaxed);
        }

        pub(crate) fn direct_renewal_dream_processing_started() {
            PROCESSING_STARTED.store(true, Ordering::SeqCst);
        }

        struct AllocationMeasurement {
            baseline: usize,
        }

        impl AllocationMeasurement {
            fn begin() -> Self {
                let baseline = LIVE_BYTES.load(Ordering::SeqCst);
                PEAK_BYTES.store(baseline, Ordering::SeqCst);
                PROCESSING_ALLOCATIONS.store(0, Ordering::SeqCst);
                PROCESSING_STARTED.store(false, Ordering::SeqCst);
                ALLOCATION_MEASURING.store(true, Ordering::SeqCst);
                Self { baseline }
            }

            fn finish(self) -> (usize, usize) {
                ALLOCATION_MEASURING.store(false, Ordering::SeqCst);
                let peak = PEAK_BYTES.load(Ordering::SeqCst);
                let processing_allocations = PROCESSING_ALLOCATIONS.load(Ordering::SeqCst);
                (peak.saturating_sub(self.baseline), processing_allocations)
            }
        }

        #[derive(Clone, Copy)]
        struct RenderSpec {
            ratios: [usize; 3],
            min_sample_rate: u32,
            max_sample_rate: u32,
            max_abs_sample: f32,
        }

        #[derive(Clone, Copy)]
        struct EvidenceSpec {
            structural_rows: usize,
            structural_renders: usize,
            synthetic_rows: usize,
            synthetic_renders: usize,
            admission_seed: u64,
        }

        #[derive(Clone, Copy)]
        struct MemorySpec {
            max_working_bytes: usize,
            duration_independent: bool,
        }

        #[derive(Clone, Copy)]
        struct RunSpec {
            owner_deadline_seconds: u64,
            test_threads: usize,
            retries: usize,
            conformance_rounds: usize,
        }

        const RENDER_SPEC: RenderSpec = RenderSpec {
            ratios: RATIOS,
            min_sample_rate: 8_000,
            max_sample_rate: 192_000,
            max_abs_sample: 8.0,
        };
        const EVIDENCE_SPEC: EvidenceSpec = EvidenceSpec {
            structural_rows: 169,
            structural_renders: 49,
            synthetic_rows: 88,
            synthetic_renders: 76,
            admission_seed: ADMISSION_SEED,
        };
        const MEMORY_SPEC: MemorySpec = MemorySpec {
            max_working_bytes: 32 * 1024 * 1024,
            duration_independent: true,
        };
        const RUN_SPEC: RunSpec = RunSpec {
            owner_deadline_seconds: 600,
            test_threads: 1,
            retries: 0,
            conformance_rounds: 2,
        };

        type OwnerFn = fn() -> Result<(), String>;

        #[derive(Clone, Copy)]
        struct GateOwner {
            id: &'static str,
            test_name: &'static str,
            owner: OwnerFn,
            rows: usize,
            renders: usize,
            worst_output_frames: usize,
            assertion_mask: u64,
            receipt_field_mask: u64,
            deadline_seconds: u64,
            construction_oracle: bool,
        }

        const ALL_ASSERTIONS: u64 = 0x0000_0000_0000_ffff;
        const ALL_RECEIPT_FIELDS: u64 = 0x0000_0000_0000_3fff;

        const GATE_OWNERS: [GateOwner; 15] = [
            gate(
                "S01",
                "direct_renewal_dream_structural_request_preallocation",
                owner_s01,
                16,
                2,
                65_536,
                false,
            ),
            gate(
                "S02",
                "direct_renewal_dream_structural_transform_map",
                owner_s02,
                30,
                0,
                0,
                true,
            ),
            gate(
                "S03",
                "direct_renewal_dream_structural_window_interpolation_gain",
                owner_s03,
                18,
                0,
                0,
                true,
            ),
            gate(
                "S04",
                "direct_renewal_dream_structural_counter_mono_spectrum",
                owner_s04,
                20,
                0,
                0,
                true,
            ),
            gate(
                "S05",
                "direct_renewal_dream_structural_linked_stereo_space",
                owner_s05,
                24,
                0,
                0,
                true,
            ),
            gate(
                "S06",
                "direct_renewal_dream_structural_blend_boundary_crop",
                owner_s06,
                18,
                18,
                524_304,
                false,
            ),
            gate(
                "S07",
                "direct_renewal_dream_structural_edge_silence_matrix",
                owner_s07,
                24,
                24,
                524_304,
                false,
            ),
            gate(
                "S08",
                "direct_renewal_dream_structural_determinism_seed",
                owner_s08,
                3,
                3,
                32_768,
                false,
            ),
            gate(
                "S09",
                "direct_renewal_dream_structural_allocation_memory",
                owner_s09,
                4,
                2,
                4_194_304,
                false,
            ),
            gate(
                "S10",
                "direct_renewal_dream_structural_single_timeline_private_surface",
                owner_s10,
                12,
                0,
                0,
                true,
            ),
            gate(
                "Y01",
                "direct_renewal_dream_synthetic_integrity_crest_discontinuity",
                owner_y01,
                30,
                30,
                26_880_000,
                false,
            ),
            gate(
                "Y02",
                "direct_renewal_dream_synthetic_pitch_diagnostic",
                owner_y02,
                21,
                9,
                13_824_000,
                false,
            ),
            gate(
                "Y03",
                "direct_renewal_dream_synthetic_impulse_diagnostic",
                owner_y03,
                6,
                6,
                9_216_000,
                false,
            ),
            gate(
                "Y04",
                "direct_renewal_dream_synthetic_periodicity_modulation_gap",
                owner_y04,
                9,
                9,
                13_824_000,
                false,
            ),
            gate(
                "Y05",
                "direct_renewal_dream_synthetic_linked_stereo_inventory",
                owner_y05,
                22,
                22,
                18_816_000,
                false,
            ),
        ];

        const fn gate(
            id: &'static str,
            test_name: &'static str,
            owner: OwnerFn,
            rows: usize,
            renders: usize,
            worst_output_frames: usize,
            construction_oracle: bool,
        ) -> GateOwner {
            GateOwner {
                id,
                test_name,
                owner,
                rows,
                renders,
                worst_output_frames,
                assertion_mask: ALL_ASSERTIONS,
                receipt_field_mask: ALL_RECEIPT_FIELDS,
                deadline_seconds: RUN_SPEC.owner_deadline_seconds,
                construction_oracle,
            }
        }

        fn mono_request(input: &[f32], ratio: usize, seed: u64) -> CandidateRequest<'_> {
            CandidateRequest {
                input,
                channels: 1,
                sample_rate: SAMPLE_RATE,
                target_frames: input.len() * ratio,
                seed,
                space: 0.5,
            }
        }

        fn stereo_request(
            input: &[f32],
            ratio: usize,
            seed: u64,
            space: f32,
        ) -> CandidateRequest<'_> {
            CandidateRequest {
                input,
                channels: 2,
                sample_rate: SAMPLE_RATE,
                target_frames: input.len() / 2 * ratio,
                seed,
                space,
            }
        }

        fn tone(frames: usize, channels: usize, frequency: f64) -> Vec<f32> {
            let mut output = Vec::with_capacity(frames * channels);
            for frame in 0..frames {
                let sample = (0.5
                    * (std::f64::consts::TAU * frequency * frame as f64 / SAMPLE_RATE as f64).sin())
                    as f32;
                for _ in 0..channels {
                    output.push(sample);
                }
            }
            output
        }

        fn finite_exact_endpoints(output: &[f32], frames: usize, channels: usize) {
            assert_eq!(output.len(), frames * channels);
            assert!(output.iter().all(|sample| sample.is_finite()));
            assert!(output
                .iter()
                .all(|sample| sample.abs() <= RENDER_SPEC.max_abs_sample));
            for channel in 0..channels {
                assert_eq!(output[channel].to_bits(), 0.0_f32.to_bits());
                assert_eq!(
                    output[(frames - 1) * channels + channel].to_bits(),
                    0.0_f32.to_bits()
                );
            }
        }

        #[test]
        fn direct_renewal_dream_construction_manifest() {
            assert_eq!(GATE_OWNERS.len(), 15);
            assert_eq!(RENDER_SPEC.ratios, [4, 8, 16]);
            assert_eq!(RENDER_SPEC.min_sample_rate, 8_000);
            assert_eq!(RENDER_SPEC.max_sample_rate, 192_000);
            assert_eq!(EVIDENCE_SPEC.admission_seed, 0x0123_4567_89ab_cdef);
            assert_eq!(RUN_SPEC.test_threads, 1);
            assert_eq!(RUN_SPEC.retries, 0);
            assert_eq!(RUN_SPEC.conformance_rounds, 2);
            assert!(MEMORY_SPEC.duration_independent);
            assert_eq!(MEMORY_SPEC.max_working_bytes, 32 * 1024 * 1024);
            assert_eq!(
                GATE_OWNERS[..10]
                    .iter()
                    .map(|owner| owner.rows)
                    .sum::<usize>(),
                EVIDENCE_SPEC.structural_rows
            );
            assert_eq!(
                GATE_OWNERS[..10]
                    .iter()
                    .map(|owner| owner.renders)
                    .sum::<usize>(),
                EVIDENCE_SPEC.structural_renders
            );
            assert_eq!(
                GATE_OWNERS[10..]
                    .iter()
                    .map(|owner| owner.rows)
                    .sum::<usize>(),
                EVIDENCE_SPEC.synthetic_rows
            );
            assert_eq!(
                GATE_OWNERS[10..]
                    .iter()
                    .map(|owner| owner.renders)
                    .sum::<usize>(),
                EVIDENCE_SPEC.synthetic_renders
            );
            for (index, owner) in GATE_OWNERS.iter().enumerate() {
                let expected = if index < 10 {
                    format!("S{:02}", index + 1)
                } else {
                    format!("Y{:02}", index - 9)
                };
                assert_eq!(owner.id, expected);
                assert!(owner.test_name.starts_with("direct_renewal_dream_"));
                assert_ne!(owner.owner as usize, 0);
                assert_ne!(owner.assertion_mask, 0);
                assert_ne!(owner.receipt_field_mask, 0);
                assert_eq!(owner.deadline_seconds, 600);
                assert!(owner.worst_output_frames <= 26_880_000);
                if owner.construction_oracle {
                    (owner.owner)().unwrap();
                }
            }
            let ledger = include_str!("creative_direct_renewal_dream/regression_manifest.tsv");
            let mut ledger_lines = ledger.lines();
            assert_eq!(
                ledger_lines.next(),
                Some("owner\ttest_name\trow_index\trow_id\trender_count\towner_output_frames_bound")
            );
            let ledger_rows = ledger_lines.map(|line| line.split('\t').collect::<Vec<_>>()).collect::<Vec<_>>();
            assert_eq!(ledger_rows.len(), EVIDENCE_SPEC.structural_rows + EVIDENCE_SPEC.synthetic_rows);
            for owner in GATE_OWNERS {
                let rows = ledger_rows.iter().filter(|row| row[0] == owner.id).collect::<Vec<_>>();
                assert_eq!(rows.len(), owner.rows);
                assert_eq!(rows.iter().map(|row| row[4].parse::<usize>().unwrap()).sum::<usize>(), owner.renders);
                for (row_index, row) in rows.into_iter().enumerate() {
                    assert_eq!(row.len(), 6);
                    assert_eq!(row[1], owner.test_name);
                    assert_eq!(row[2].parse::<usize>().unwrap(), row_index);
                    assert!(!row[3].is_empty());
                    assert_eq!(row[5].parse::<usize>().unwrap(), owner.worst_output_frames);
                }
            }
            assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
            for source in SourceKind::ALL {
                assert_eq!(hash_f32(&source.generate()), source.expected_hash());
            }
        }

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
            request.target_frames = mono.len() * 5;
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
            let _allocation_guard = ALLOCATION_LOCK.lock().unwrap();
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
            let module = include_str!("creative_direct_renewal_dream/mod.rs");
            let plan = include_str!("creative_direct_renewal_dream/plan.rs");
            let analysis = include_str!("creative_direct_renewal_dream/analysis.rs");
            let stereo = include_str!("creative_direct_renewal_dream/stereo.rs");
            let synthesis = include_str!("creative_direct_renewal_dream/synthesis.rs");
            let lib = include_str!("lib.rs");
            let cargo = include_str!("../Cargo.toml");
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
                "signal-creative-direct-renewal-dream-v1"
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

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum SourceKind {
            LowTone,
            MidTone,
            Chord,
            HarmonicPad,
            Impulse,
            ImpulseTrain,
            SilenceGap,
            UniformNoise,
            RademacherNoise,
            AmplitudeModulatedNoise,
        }

        impl SourceKind {
            const ALL: [Self; 10] = [
                Self::LowTone,
                Self::MidTone,
                Self::Chord,
                Self::HarmonicPad,
                Self::Impulse,
                Self::ImpulseTrain,
                Self::SilenceGap,
                Self::UniformNoise,
                Self::RademacherNoise,
                Self::AmplitudeModulatedNoise,
            ];

            fn id(self) -> &'static str {
                match self {
                    Self::LowTone => "low-tone",
                    Self::MidTone => "mid-tone",
                    Self::Chord => "chord",
                    Self::HarmonicPad => "harmonic-pad",
                    Self::Impulse => "impulse",
                    Self::ImpulseTrain => "impulse-train",
                    Self::SilenceGap => "silence-gap",
                    Self::UniformNoise => "uniform-noise",
                    Self::RademacherNoise => "rademacher-noise",
                    Self::AmplitudeModulatedNoise => "amplitude-modulated-noise",
                }
            }

            fn expected_hash(self) -> &'static str {
                match self {
                    Self::AmplitudeModulatedNoise => "ba6b9c244618939769e7283fac92f198690238db0c96d99c280892ee358ab31b",
                    Self::Chord => "b7c85b6faed8d670fd7eefa66f7be6a89df0f7c5c3a4444146a2d083a70792e7",
                    Self::HarmonicPad => "732895709a05fa724d9dd76a03bc22c64865b84ba93ba351e49354b31f95e96c",
                    Self::ImpulseTrain => "47314d3121745479660fb0d0350b41aec987074f75a503181805e5f4545e8138",
                    Self::Impulse => "fc73433e0fab2786572b6a98bd0cc9f86145960581d77e3dbc7d1bfa6abca57b",
                    Self::LowTone => "2c6d1c766ce73ac75000f8e9cbd6238fafbf180c64baa33d62a55fb9517f32e1",
                    Self::MidTone => "36397e016a1d00a5bf1884d049a1454ab7342965ffd3cf21179474610a218b33",
                    Self::RademacherNoise => "c1ae606691767937990e38a314ceadeee6c7cb0a9da63c7ed3d3a3ef31b838b5",
                    Self::SilenceGap => "1c17fdc3cecd09cfcc403c39a9c7aadb75c41239433c20863cb967fbcef0013e",
                    Self::UniformNoise => "cde1917d6afdfe3dfb260da2a6273a243e261032bb9fe6624e49020089ee9923",
                }
            }

            fn support(self) -> (usize, usize) {
                match self {
                    Self::Impulse => (48_000, 48_001),
                    Self::ImpulseTrain => (19_200, 77_798),
                    _ => (24_000, 72_000),
                }
            }

            fn generate(self) -> Vec<f32> {
                let mut samples = vec![0.0_f32; SYNTHETIC_SOURCE_FRAMES];
                match self {
                    Self::Impulse => samples[48_000] = 1.0,
                    Self::ImpulseTrain => {
                        for (frame, value) in [(19_200, 1.0), (38_937, -0.8), (58_103, 0.65), (77_797, -0.5)] {
                            samples[frame] = value;
                        }
                    }
                    _ => {
                        for (frame, sample) in samples.iter_mut().enumerate() {
                            if !(24_000..72_000).contains(&frame) {
                                continue;
                            }
                            let raw = match self {
                                Self::LowTone => sinusoid(frame, 110.0, 0.5),
                                Self::MidTone => sinusoid(frame, 440.0, 0.5),
                                Self::Chord => [110.0, 164.813_778, 220.0, 277.182_631, 329.627_557]
                                    .into_iter()
                                    .map(|frequency| sinusoid(frame, frequency, 0.1))
                                    .sum(),
                                Self::HarmonicPad | Self::SilenceGap => (1..=8)
                                    .map(|partial| {
                                        (0.35 / partial as f64)
                                            * (2.0
                                                * std::f64::consts::PI
                                                * 110.0
                                                * partial as f64
                                                * frame as f64
                                                / SAMPLE_RATE as f64)
                                                .sin()
                                    })
                                    .sum(),
                                Self::UniformNoise => {
                                    let unit = high_53(mix64(frame as u64 ^ TEST_TAG)) as f64
                                        / (1_u64 << 53) as f64;
                                    0.5 * (2.0 * unit - 1.0)
                                }
                                Self::RademacherNoise => rademacher_sign(frame) * 0.5,
                                Self::AmplitudeModulatedNoise => {
                                    rademacher_sign(frame)
                                        * 0.5
                                        * (0.5
                                            + 0.375
                                                * (2.0
                                                    * std::f64::consts::PI
                                                    * 1.7
                                                    * frame as f64
                                                    / SAMPLE_RATE as f64)
                                                    .sin())
                                }
                                Self::Impulse | Self::ImpulseTrain => unreachable!(),
                            };
                            let gap = self == Self::SilenceGap && (42_000..54_000).contains(&frame);
                            let weight = support_weight(frame);
                            *sample = if gap {
                                0.0
                            } else {
                                (raw * weight) as f32
                            };
                        }
                    }
                }
                samples
            }
        }

        fn sinusoid(frame: usize, frequency: f64, amplitude: f64) -> f64 {
            amplitude
                * (2.0 * std::f64::consts::PI * frequency * frame as f64
                    / SAMPLE_RATE as f64)
                    .sin()
        }

        fn rademacher_sign(frame: usize) -> f64 {
            if mix64(frame as u64 ^ TEST_TAG) >> 63 == 1 { 1.0 } else { -1.0 }
        }

        fn support_weight(frame: usize) -> f64 {
            match frame {
                24_000..=26_047 => {
                    0.5 - 0.5 * (std::f64::consts::PI * (frame - 24_000) as f64 / 2_047.0).cos()
                }
                26_048..=69_951 => 1.0,
                69_952..=71_999 => {
                    0.5 - 0.5 * (std::f64::consts::PI * (71_999 - frame) as f64 / 2_047.0).cos()
                }
                _ => 0.0,
            }
        }

        fn sha256_hex(bytes: &[u8]) -> String {
            const INITIAL: [u32; 8] = [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ];
            const K: [u32; 64] = [
                0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
                0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
                0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
                0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
                0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
                0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
                0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
                0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
            ];
            let bit_len = (bytes.len() as u64).wrapping_mul(8);
            let mut padded = Vec::with_capacity((bytes.len() + 72) & !63);
            padded.extend_from_slice(bytes);
            padded.push(0x80);
            while padded.len() % 64 != 56 { padded.push(0); }
            padded.extend_from_slice(&bit_len.to_be_bytes());
            let mut hash = INITIAL;
            for chunk in padded.chunks_exact(64) {
                let mut words = [0_u32; 64];
                for (index, word) in words[..16].iter_mut().enumerate() {
                    *word = u32::from_be_bytes(chunk[index * 4..index * 4 + 4].try_into().unwrap());
                }
                for index in 16..64 {
                    let s0 = words[index - 15].rotate_right(7) ^ words[index - 15].rotate_right(18) ^ (words[index - 15] >> 3);
                    let s1 = words[index - 2].rotate_right(17) ^ words[index - 2].rotate_right(19) ^ (words[index - 2] >> 10);
                    words[index] = words[index - 16].wrapping_add(s0).wrapping_add(words[index - 7]).wrapping_add(s1);
                }
                let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = hash;
                for index in 0..64 {
                    let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                    let choice = (e & f) ^ (!e & g);
                    let temp1 = h.wrapping_add(s1).wrapping_add(choice).wrapping_add(K[index]).wrapping_add(words[index]);
                    let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                    let majority = (a & b) ^ (a & c) ^ (b & c);
                    let temp2 = s0.wrapping_add(majority);
                    h = g; g = f; f = e; e = d.wrapping_add(temp1);
                    d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
                }
                for (state, value) in hash.iter_mut().zip([a, b, c, d, e, f, g, h]) {
                    *state = state.wrapping_add(value);
                }
            }
            hash.iter().map(|word| format!("{word:08x}")).collect()
        }

        fn hash_f32(samples: &[f32]) -> String {
            let mut bytes = Vec::with_capacity(samples.len() * size_of::<f32>());
            for sample in samples { bytes.extend_from_slice(&sample.to_le_bytes()); }
            sha256_hex(&bytes)
        }

        fn checkpoint_identity() -> String {
            let output = Command::new("git").args(["rev-parse", "HEAD"]).output().unwrap();
            assert!(output.status.success());
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        }

        fn json_string(value: &str) -> String {
            let mut output = String::from("\"");
            for character in value.chars() {
                match character {
                    '\\' => output.push_str("\\\\"),
                    '"' => output.push_str("\\\""),
                    '\n' => output.push_str("\\n"),
                    '\r' => output.push_str("\\r"),
                    '\t' => output.push_str("\\t"),
                    other => output.push(other),
                }
            }
            output.push('"');
            output
        }

        struct Receipt<'a> {
            owner: &'a str,
            row_index: usize,
            row_id: &'a str,
            status: &'a str,
            render_count: usize,
            output_frames: usize,
            input_hash: &'a str,
            output_hash: &'a str,
            assertions: Vec<String>,
            diagnostics: Vec<String>,
        }

        fn receipt_directory(owner: &str) -> PathBuf {
            let stage = std::env::var("DIRECT_RENEWAL_STAGE").unwrap_or_else(|_| "synthetic".into());
            let round = std::env::var("DIRECT_RENEWAL_ROUND").unwrap_or_else(|_| "0".into());
            PathBuf::from("target/creative-stretch-direct-renewal-31-66")
                .join(checkpoint_identity())
                .join(stage)
                .join(round)
                .join(owner)
        }

        fn write_receipt(receipt: Receipt<'_>) {
            let directory = receipt_directory(receipt.owner);
            fs::create_dir_all(&directory).unwrap();
            let path = directory.join("rows.jsonl");
            let mut file = OpenOptions::new().create(true).append(true).open(path).unwrap();
            let assertions = receipt.assertions.iter().map(|item| json_string(item)).collect::<Vec<_>>().join(",");
            let diagnostics = receipt.diagnostics.iter().map(|item| json_string(item)).collect::<Vec<_>>().join(",");
            writeln!(
                file,
                "{{\"schema\":{},\"checkpoint\":{},\"stage\":{},\"round\":{},\"owner\":{},\"row_index\":{},\"row_id\":{},\"status\":{},\"render_count\":{},\"output_frames\":{},\"input_sha256\":{},\"output_sha256\":{},\"assertions\":[{}],\"diagnostics\":[{}]}}",
                json_string(DIRECT_RENEWAL_DREAM_RECEIPT_SCHEMA),
                json_string(&checkpoint_identity()),
                json_string(&std::env::var("DIRECT_RENEWAL_STAGE").unwrap_or_else(|_| "synthetic".into())),
                json_string(&std::env::var("DIRECT_RENEWAL_ROUND").unwrap_or_else(|_| "0".into())),
                json_string(receipt.owner), receipt.row_index, json_string(receipt.row_id),
                json_string(receipt.status), receipt.render_count, receipt.output_frames,
                json_string(receipt.input_hash), json_string(receipt.output_hash), assertions, diagnostics
            ).unwrap();
            file.flush().unwrap();
            file.sync_all().unwrap();
        }

        fn write_summary(owner: &str, expected_rows: usize, completed_rows: usize, expected_renders: usize, completed_renders: usize, status: &str, errors: &[String]) {
            let directory = receipt_directory(owner);
            fs::create_dir_all(&directory).unwrap();
            let mut file = OpenOptions::new().create(true).truncate(true).write(true).open(directory.join("summary.json")).unwrap();
            let errors = errors.iter().map(|error| json_string(error)).collect::<Vec<_>>().join(",");
            writeln!(file, "{{\"schema\":{},\"checkpoint\":{},\"owner\":{},\"status\":{},\"expected_rows\":{},\"complete_rows\":{},\"expected_renders\":{},\"complete_renders\":{},\"errors\":[{}]}}", json_string(DIRECT_RENEWAL_DREAM_SUMMARY_SCHEMA), json_string(&checkpoint_identity()), json_string(owner), json_string(status), expected_rows, completed_rows, expected_renders, completed_renders, errors).unwrap();
            file.flush().unwrap();
            file.sync_all().unwrap();
        }

        fn mapped_support(source: SourceKind, ratio: usize) -> (usize, usize) {
            let (start, end) = source.support();
            (start * ratio, end * ratio)
        }

        fn hard_integrity(output: &[f32], ratio: usize, channels: usize) -> Result<(), String> {
            let frames = SYNTHETIC_SOURCE_FRAMES * ratio;
            if output.len() != frames * channels { return Err("exact-length".into()); }
            if output.iter().any(|sample| !sample.is_finite()) { return Err("finite".into()); }
            if output.iter().any(|sample| sample.abs() > RENDER_SPEC.max_abs_sample) { return Err("max-abs".into()); }
            for channel in 0..channels {
                if output[channel].to_bits() != 0.0_f32.to_bits() || output[(frames - 1) * channels + channel].to_bits() != 0.0_f32.to_bits() {
                    return Err("exact-zero-endpoints".into());
                }
            }
            Ok(())
        }

        fn no_dropout(output: &[f32], source: SourceKind, ratio: usize) -> bool {
            let (start, end) = mapped_support(source, ratio);
            let window_frames = fft_size(SAMPLE_RATE) / 2;
            if end.saturating_sub(start) < window_frames { return true; }
            (start..=end - window_frames).step_by(window_frames).all(|window_start| {
                let window_end = window_start + window_frames;
                let authored_gap = source == SourceKind::SilenceGap
                    && window_start >= 42_000 * ratio
                    && window_end <= 54_000 * ratio;
                authored_gap
                    || output[window_start..window_end]
                        .iter()
                        .any(|sample| *sample != 0.0)
            })
        }

        fn rms(samples: &[f32]) -> f64 {
            if samples.is_empty() { return 0.0; }
            (samples.iter().map(|sample| (*sample as f64).powi(2)).sum::<f64>() / samples.len() as f64).sqrt()
        }

        fn difference_crest_db(samples: &[f32]) -> f64 {
            if samples.len() < 2 { return 0.0; }
            let mut maximum = 0.0_f64;
            let mut energy = 0.0_f64;
            for pair in samples.windows(2) {
                let difference = pair[1] as f64 - pair[0] as f64;
                maximum = maximum.max(difference.abs());
                energy += difference * difference;
            }
            let difference_rms = (energy / (samples.len() - 1) as f64).sqrt();
            if maximum == 0.0 { 0.0 } else { 20.0 * (maximum / difference_rms).log10() }
        }

        const DIFFERENCE_CREST_REFERENCE: [[f64; 3]; 10] = [
            [9.905726, 10.894208, 10.519063],
            [9.556341, 10.436868, 10.905863],
            [11.802457, 12.936199, 12.552822],
            [11.915677, 13.552276, 14.040544],
            [21.672820, 21.668892, 21.489501],
            [16.312956, 16.540906, 17.186745],
            [13.453803, 15.084147, 15.790229],
            [14.905336, 15.539239, 15.680264],
            [14.348783, 15.440176, 15.456703],
            [16.083822, 16.292147, 16.221062],
        ];

        fn finish_owner(owner: &str, errors: Vec<String>, rows: usize, renders: usize) -> Result<(), String> {
            write_summary(owner, rows, rows, renders, renders, if errors.is_empty() { "pass" } else { "fail" }, &errors);
            if errors.is_empty() { Ok(()) } else { Err(errors.join("; ")) }
        }

        fn owner_y01() -> Result<(), String> {
            let owner = "Y01";
            let mut errors = Vec::new();
            let mut row_index = 0;
            for (source_index, source) in SourceKind::ALL.into_iter().enumerate() {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for (ratio_index, ratio) in RATIOS.into_iter().enumerate() {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let mut row_errors = Vec::new();
                    if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                    if !no_dropout(&output, source, ratio) { row_errors.push("dropout".into()); }
                    let (start, end) = if source == SourceKind::Impulse { (0, output.len()) } else { mapped_support(source, ratio) };
                    let crest = difference_crest_db(&output[start..end]);
                    if !crest.is_finite() { row_errors.push("finite-difference-crest".into()); }
                    let output_hash = hash_f32(&output);
                    let row_id = format!("{}-{ratio}x", source.id());
                    write_receipt(Receipt {
                        owner, row_index, row_id: &row_id,
                        status: if row_errors.is_empty() { "pass" } else { "fail" },
                        render_count: 1, output_frames: output.len(), input_hash: &input_hash,
                        output_hash: &output_hash,
                        assertions: vec!["exact-length-finite-endpoints-max8".into(), "no-H-dropout".into()],
                        diagnostics: vec![format!("difference_crest_db={crest:.9}"), format!("reference_delta_db={:.9}", crest - DIFFERENCE_CREST_REFERENCE[source_index][ratio_index])],
                    });
                    errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                    row_index += 1;
                }
            }
            finish_owner(owner, errors, 30, 30)
        }

        fn pitch_spectrum(output: &[f32], ratio: usize) -> (Vec<Complex64>, usize) {
            let support_start = 24_000 * ratio;
            let support_end = 72_000 * ratio;
            let quarter = (support_end - support_start) / 4;
            let measured = &output[support_start + quarter..support_end - quarter];
            let padded_len = (measured.len() * 8).next_power_of_two();
            let mut spectrum = vec![Complex64::new(0.0, 0.0); padded_len];
            for (index, sample) in measured.iter().enumerate() {
                let window = 0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / measured.len() as f64).cos();
                spectrum[index].re = *sample as f64 * window;
            }
            FftPlanner::<f64>::new().plan_fft_forward(padded_len).process(&mut spectrum);
            (spectrum, padded_len)
        }

        fn estimate_pitch(spectrum: &[Complex64], fft_len: usize, expected: f64) -> f64 {
            let frequency_per_bin = SAMPLE_RATE as f64 / fft_len as f64;
            let first = ((expected - 4.0) / frequency_per_bin).ceil().max(1.0) as usize;
            let last = ((expected + 4.0) / frequency_per_bin).floor() as usize;
            let peak = (first..=last).max_by(|left, right| spectrum[*left].norm_sqr().total_cmp(&spectrum[*right].norm_sqr())).unwrap();
            let left = spectrum[peak - 1].norm().max(f64::MIN_POSITIVE).ln();
            let center = spectrum[peak].norm().max(f64::MIN_POSITIVE).ln();
            let right = spectrum[peak + 1].norm().max(f64::MIN_POSITIVE).ln();
            let denominator = left - 2.0 * center + right;
            let offset = if denominator == 0.0 { 0.0 } else { 0.5 * (left - right) / denominator };
            (peak as f64 + offset) * frequency_per_bin
        }

        fn owner_y02() -> Result<(), String> {
            let owner = "Y02";
            let cases: [(SourceKind, &[f64]); 3] = [
                (SourceKind::LowTone, &[110.0]),
                (SourceKind::MidTone, &[440.0]),
                (SourceKind::Chord, &[110.0, 164.813_778, 220.0, 277.182_631, 329.627_557]),
            ];
            let mut errors = Vec::new();
            let mut row_index = 0;
            for (source, frequencies) in cases {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for ratio in RATIOS {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let output_hash = hash_f32(&output);
                    let (spectrum, fft_len) = pitch_spectrum(&output, ratio);
                    for (frequency_index, frequency) in frequencies.iter().enumerate() {
                        let estimate = estimate_pitch(&spectrum, fft_len, *frequency);
                        let error_hz = (estimate - frequency).abs();
                        let mut row_errors = Vec::new();
                        if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                        if !estimate.is_finite() || !error_hz.is_finite() { row_errors.push("finite-pitch".into()); }
                        let row_id = format!("{}-{ratio}x-{frequency:.6}hz", source.id());
                        write_receipt(Receipt {
                            owner, row_index, row_id: &row_id,
                            status: if row_errors.is_empty() { "pass" } else { "fail" },
                            render_count: usize::from(frequency_index == 0), output_frames: output.len(),
                            input_hash: &input_hash, output_hash: &output_hash,
                            assertions: vec!["finite-pitch-diagnostic".into()],
                            diagnostics: vec![format!("estimated_hz={estimate:.9}"), format!("error_hz={error_hz:.9}")],
                        });
                        errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                        row_index += 1;
                    }
                }
            }
            finish_owner(owner, errors, 21, 9)
        }

        fn shortest_energy_width(samples: &[f32], fraction: f64) -> usize {
            let total = samples.iter().map(|sample| (*sample as f64).powi(2)).sum::<f64>();
            if total == 0.0 { return 0; }
            let target = total * fraction;
            let mut best = samples.len();
            let mut end = 0;
            let mut accumulated = 0.0;
            for start in 0..samples.len() {
                while end < samples.len() && accumulated < target {
                    accumulated += (samples[end] as f64).powi(2);
                    end += 1;
                }
                if accumulated >= target { best = best.min(end - start); }
                if start < end { accumulated -= (samples[start] as f64).powi(2); }
            }
            best
        }

        fn energy_centroid(samples: &[f32]) -> f64 {
            let mut weighted = 0.0;
            let mut total = 0.0;
            for (index, sample) in samples.iter().enumerate() {
                let energy = (*sample as f64).powi(2);
                weighted += index as f64 * energy;
                total += energy;
            }
            if total == 0.0 { 0.0 } else { weighted / total }
        }

        fn active_regions(samples: &[f32]) -> (usize, Option<f64>) {
            let windows = samples.windows(480).step_by(240).map(rms).collect::<Vec<_>>();
            let Some((peak_index, peak)) = windows.iter().copied().enumerate().max_by(|left, right| left.1.total_cmp(&right.1)) else { return (0, None); };
            if peak == 0.0 { return (0, None); }
            let threshold = peak * 10.0_f64.powf(-30.0 / 20.0);
            let active = windows.iter().enumerate().filter_map(|(index, value)| (*value >= threshold).then_some(index * 240)).collect::<Vec<_>>();
            let mut regions: Vec<(usize, usize, f64)> = Vec::new();
            for start in active {
                let value = windows[start / 240];
                if regions.last().is_none_or(|(_, last, _)| start.saturating_sub(*last) >= 2_400) {
                    regions.push((start, start, value));
                } else {
                    let region = regions.last_mut().unwrap();
                    region.1 = start;
                    region.2 = region.2.max(value);
                }
            }
            let peak_start = peak_index * 240;
            let primary_region = regions.iter().position(|(start, last, _)| *start <= peak_start && peak_start <= *last).unwrap_or(0);
            let secondary = regions.iter().enumerate().filter(|(index, _)| *index != primary_region).map(|(_, (_, _, value))| 20.0 * (value / peak).log10()).max_by(f64::total_cmp);
            (regions.len(), secondary)
        }

        fn owner_y03() -> Result<(), String> {
            let owner = "Y03";
            let mut errors = Vec::new();
            let mut row_index = 0;
            for source in [SourceKind::Impulse, SourceKind::ImpulseTrain] {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for ratio in RATIOS {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let mut row_errors = Vec::new();
                    if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                    let width = shortest_energy_width(&output, 0.95);
                    let centroid = energy_centroid(&output);
                    let expected = (48_000.5 * output.len() as f64 / SYNTHETIC_SOURCE_FRAMES as f64) - 0.5;
                    let centroid_error = (centroid - expected).abs();
                    let (regions, secondary) = active_regions(&output);
                    if !centroid.is_finite() || !centroid_error.is_finite() { row_errors.push("finite-impulse-diagnostic".into()); }
                    let output_hash = hash_f32(&output);
                    let row_id = format!("{}-{ratio}x", source.id());
                    write_receipt(Receipt {
                        owner, row_index, row_id: &row_id,
                        status: if row_errors.is_empty() { "pass" } else { "fail" }, render_count: 1,
                        output_frames: output.len(), input_hash: &input_hash, output_hash: &output_hash,
                        assertions: vec!["finite-impulse-diagnostics".into()],
                        diagnostics: vec![format!("width95={width}"), format!("centroid_error={centroid_error:.9}"), format!("active_regions={regions}"), format!("secondary_db={}", secondary.map_or("null".into(), |value| format!("{value:.9}")))],
                    });
                    errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                    row_index += 1;
                }
            }
            finish_owner(owner, errors, 6, 6)
        }

        fn linear_autocorrelation_max(samples: &[f32]) -> f64 {
            let mean = samples.iter().map(|sample| *sample as f64).sum::<f64>() / samples.len() as f64;
            let fft_len = (samples.len() * 2 - 1).next_power_of_two();
            let mut spectrum = vec![Complex64::new(0.0, 0.0); fft_len];
            for (bin, sample) in spectrum.iter_mut().zip(samples) { bin.re = *sample as f64 - mean; }
            let mut planner = FftPlanner::<f64>::new();
            planner.plan_fft_forward(fft_len).process(&mut spectrum);
            for bin in &mut spectrum { *bin = Complex64::new(bin.norm_sqr(), 0.0); }
            planner.plan_fft_inverse(fft_len).process(&mut spectrum);
            let lag_zero = spectrum[0].re;
            (960..=48_000).map(|lag| (spectrum[lag].re / lag_zero).abs()).fold(0.0, f64::max)
        }

        fn block_rms_cv(samples: &[f32]) -> f64 {
            let values = samples.windows(2_400).step_by(1_200).map(rms).collect::<Vec<_>>();
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / values.len() as f64;
            variance.sqrt() / mean
        }

        fn owner_y04() -> Result<(), String> {
            const AUTOCORRELATION: [f64; 3] = [0.017218163, 0.017727693, 0.017090511];
            const UNIFORM_CV: [f64; 3] = [0.387747959, 0.460013282, 0.492971808];
            const MID_CV: [f64; 3] = [0.617268653, 0.679139581, 0.708639858];
            let owner = "Y04";
            let mut errors = Vec::new();
            let mut row_index = 0;
            for source in [SourceKind::UniformNoise, SourceKind::MidTone, SourceKind::SilenceGap] {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for (ratio_index, ratio) in RATIOS.into_iter().enumerate() {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let (start, end) = mapped_support(source, ratio);
                    let mut row_errors = Vec::new();
                    if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                    let diagnostic = match source {
                        SourceKind::UniformNoise => {
                            let autocorrelation = linear_autocorrelation_max(&output[start..end]);
                            let cv = block_rms_cv(&output[start..end]);
                            if autocorrelation > AUTOCORRELATION[ratio_index] + 0.05 { row_errors.push("autocorrelation".into()); }
                            if cv > UNIFORM_CV[ratio_index] + 0.05 { row_errors.push("uniform-cv".into()); }
                            vec![format!("autocorrelation={autocorrelation:.9}"), format!("block_rms_cv={cv:.9}")]
                        }
                        SourceKind::MidTone => {
                            let cv = block_rms_cv(&output[start..end]);
                            if cv > MID_CV[ratio_index] + 0.05 { row_errors.push("mid-cv".into()); }
                            vec![format!("block_rms_cv={cv:.9}")]
                        }
                        SourceKind::SilenceGap => {
                            let gap = rms(&output[42_000 * ratio..54_000 * ratio]);
                            let active = rms(&output[start..end]);
                            let gap_db = 20.0 * (gap / active).log10();
                            if !gap_db.is_finite() { row_errors.push("finite-gap-rms".into()); }
                            vec![format!("gap_relative_db={gap_db:.9}")]
                        }
                        _ => unreachable!(),
                    };
                    let output_hash = hash_f32(&output);
                    let row_id = format!("{}-{ratio}x", source.id());
                    write_receipt(Receipt {
                        owner, row_index, row_id: &row_id,
                        status: if row_errors.is_empty() { "pass" } else { "fail" }, render_count: 1,
                        output_frames: output.len(), input_hash: &input_hash, output_hash: &output_hash,
                        assertions: vec!["periodicity-modulation-gap".into()], diagnostics: diagnostic,
                    });
                    errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                    row_index += 1;
                }
            }
            finish_owner(owner, errors, 9, 9)
        }

        #[derive(Clone, Copy)]
        enum StereoFixture { Duplicate, Base, CommonNegated, Mixed, SwappedMixed, AntiPhase, DelayedPad }

        impl StereoFixture {
            fn id(self) -> &'static str {
                match self { Self::Duplicate => "duplicate", Self::Base => "base", Self::CommonNegated => "common-negated", Self::Mixed => "mixed", Self::SwappedMixed => "swapped-mixed", Self::AntiPhase => "anti-phase", Self::DelayedPad => "delayed-pad" }
            }

            fn generate(self) -> Vec<f32> {
                let mid = SourceKind::MidTone.generate();
                let pad = SourceKind::HarmonicPad.generate();
                let chord = SourceKind::Chord.generate();
                let noise = SourceKind::UniformNoise.generate();
                let mut output = Vec::with_capacity(SYNTHETIC_SOURCE_FRAMES * 2);
                for frame in 0..SYNTHETIC_SOURCE_FRAMES {
                    let delayed_pad = frame.checked_sub(37).map_or(0.0, |index| pad[index]);
                    let delayed_chord = frame.checked_sub(37).map_or(0.0, |index| chord[index]);
                    let pair = match self {
                        Self::Duplicate | Self::Base => (mid[frame], mid[frame]),
                        Self::CommonNegated => (-mid[frame], -mid[frame]),
                        Self::AntiPhase => (mid[frame], -mid[frame]),
                        Self::DelayedPad => (pad[frame], delayed_pad),
                        Self::Mixed => (chord[frame] + 0.2 * noise[frame], delayed_chord - 0.2 * noise[frame]),
                        Self::SwappedMixed => (delayed_chord - 0.2 * noise[frame], chord[frame] + 0.2 * noise[frame]),
                    };
                    output.extend_from_slice(&[pair.0, pair.1]);
                }
                output
            }
        }

        fn channel_pair(interleaved: &[f32]) -> (Vec<f32>, Vec<f32>) {
            let mut left = Vec::with_capacity(interleaved.len() / 2);
            let mut right = Vec::with_capacity(interleaved.len() / 2);
            for frame in interleaved.chunks_exact(2) { left.push(frame[0]); right.push(frame[1]); }
            (left, right)
        }

        fn band_energies(channel: &[f32]) -> [f64; 5] {
            let len = channel.len();
            let mut spectrum = channel.iter().map(|sample| Complex64::new(*sample as f64, 0.0)).collect::<Vec<_>>();
            FftPlanner::<f64>::new().plan_fft_forward(len).process(&mut spectrum);
            let mut bands = [0.0_f64; 5];
            for (bin, value) in spectrum.iter().enumerate().take(len / 2 + 1) {
                let frequency = bin as f64 * SAMPLE_RATE as f64 / len as f64;
                let weight = if bin == 0 || (len % 2 == 0 && bin == len / 2) { 1.0 } else { 2.0 };
                let energy = weight * value.norm_sqr();
                bands[0] += energy;
                if frequency <= 80.0 { bands[1] += energy; }
                let band = if frequency < 250.0 { 2 } else if frequency < 1_500.0 { 3 } else { 4 };
                bands[band] += energy;
            }
            bands
        }

        fn stereo_metrics(interleaved: &[f32]) -> ([f64; 4], f64, f64) {
            let (left, right) = channel_pair(interleaved);
            let left_energy = band_energies(&left);
            let right_energy = band_energies(&right);
            let balances = std::array::from_fn(|index| {
                let energy_index = [0, 2, 3, 4][index];
                if left_energy[energy_index] == 0.0 && right_energy[energy_index] == 0.0 { 0.0 }
                else { 10.0 * (right_energy[energy_index] / left_energy[energy_index]).log10() }
            });
            let low_fraction = (left_energy[1] + right_energy[1]) / (left_energy[0] + right_energy[0]);
            let side_energy = interleaved.chunks_exact(2).map(|frame| ((frame[0] as f64 - frame[1] as f64) * 0.5).powi(2)).sum::<f64>();
            (balances, low_fraction, side_energy)
        }

        fn time_relation_residual(fixture: StereoFixture, output: &[f32], _space: f32) -> f64 {
            output.chunks_exact(2).map(|frame| match fixture {
                StereoFixture::Duplicate | StereoFixture::Base | StereoFixture::CommonNegated => (frame[0] - frame[1]).abs() as f64,
                StereoFixture::AntiPhase => (frame[0] + frame[1]).abs() as f64,
                _ => 0.0,
            }).fold(0.0, f64::max)
        }

        fn owner_y05() -> Result<(), String> {
            let owner = "Y05";
            let mut rows = Vec::new();
            for ratio in RATIOS { for space in [0.0_f32, 0.5, 1.0] { rows.push((StereoFixture::Duplicate, ratio, space)); } }
            rows.extend([(StereoFixture::Base, 8, 0.5), (StereoFixture::CommonNegated, 8, 0.5), (StereoFixture::Mixed, 8, 0.5), (StereoFixture::SwappedMixed, 8, 0.5)]);
            for space in [0.0_f32, 0.5, 1.0] { rows.push((StereoFixture::AntiPhase, 8, space)); }
            for ratio in RATIOS { for space in [0.0_f32, 1.0] { rows.push((StereoFixture::DelayedPad, ratio, space)); } }
            assert_eq!(rows.len(), 22);
            let mut errors = Vec::new();
            let mut duplicate_balances: Vec<(usize, f32, [f64; 4])> = Vec::new();
            let mut anti_balances: Vec<(f32, [f64; 4])> = Vec::new();
            for (row_index, (fixture, ratio, space)) in rows.into_iter().enumerate() {
                let input = fixture.generate();
                let input_hash = hash_f32(&input);
                let (source_balances, source_low_fraction, source_side_energy) = stereo_metrics(&input);
                let output = render(stereo_request(&input, ratio, ADMISSION_SEED, space)).map_err(|error| format!("{error:?}"))?;
                let (candidate_balances, candidate_low_fraction, candidate_side_energy) = stereo_metrics(&output);
                let relation_residual = time_relation_residual(fixture, &output, space);
                let mut row_errors = Vec::new();
                if let Err(error) = hard_integrity(&output, ratio, 2) { row_errors.push(error); }
                for band in 0..4 {
                    if !source_balances[band].is_finite() || !candidate_balances[band].is_finite() { row_errors.push(format!("finite-balance-{band}")); continue; }
                    if (candidate_balances[band] - source_balances[band]).abs() > 0.75 { row_errors.push(format!("balance-error-{band}")); }
                    if source_balances[band].abs() >= 0.5 && source_balances[band].signum() != candidate_balances[band].signum() { row_errors.push(format!("dominance-reversal-{band}")); }
                }
                if space == 0.0 && matches!(fixture, StereoFixture::Duplicate | StereoFixture::Base | StereoFixture::CommonNegated | StereoFixture::AntiPhase) && relation_residual > 1.0e-6 { row_errors.push("source-relation".into()); }
                if !source_low_fraction.is_finite() || !candidate_low_fraction.is_finite() || !source_side_energy.is_finite() || !candidate_side_energy.is_finite() { row_errors.push("finite-stereo-diagnostic".into()); }
                if matches!(fixture, StereoFixture::Duplicate) { duplicate_balances.push((ratio, space, candidate_balances)); }
                if matches!(fixture, StereoFixture::AntiPhase) { anti_balances.push((space, candidate_balances)); }
                let output_hash = hash_f32(&output);
                let row_id = format!("{}-{ratio}x-space-{space:.1}", fixture.id());
                write_receipt(Receipt {
                    owner, row_index, row_id: &row_id,
                    status: if row_errors.is_empty() { "pass" } else { "fail" }, render_count: 1,
                    output_frames: output.len() / 2, input_hash: &input_hash, output_hash: &output_hash,
                    assertions: vec!["stereo-integrity".into(), "balance-bands".into(), "dominance".into()],
                    diagnostics: vec![format!("source_balance_db={source_balances:?}"), format!("candidate_balance_db={candidate_balances:?}"), format!("relation_residual={relation_residual:.12}"), format!("source_low_fraction={source_low_fraction:.12}"), format!("candidate_low_fraction={candidate_low_fraction:.12}"), format!("source_side_energy={source_side_energy:.12}"), format!("candidate_side_energy={candidate_side_energy:.12}")],
                });
                errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
            }
            for ratio in RATIOS {
                let trio = duplicate_balances.iter().filter(|(candidate_ratio, _, _)| *candidate_ratio == ratio).take(3).collect::<Vec<_>>();
                for band in 0..4 {
                    let minimum = trio.iter().map(|(_, _, value)| value[band]).fold(f64::INFINITY, f64::min);
                    let maximum = trio.iter().map(|(_, _, value)| value[band]).fold(f64::NEG_INFINITY, f64::max);
                    if maximum - minimum > 0.5 { errors.push(format!("duplicate-{ratio}x:balance-spread-{band}")); }
                }
            }
            for band in 0..4 {
                let minimum = anti_balances.iter().map(|(_, value)| value[band]).fold(f64::INFINITY, f64::min);
                let maximum = anti_balances.iter().map(|(_, value)| value[band]).fold(f64::NEG_INFINITY, f64::max);
                if maximum - minimum > 0.5 { errors.push(format!("anti-phase:balance-spread-{band}")); }
            }
            finish_owner(owner, errors, 22, 22)
        }

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
    };
}
