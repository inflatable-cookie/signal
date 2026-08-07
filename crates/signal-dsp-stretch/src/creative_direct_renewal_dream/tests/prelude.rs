        use std::alloc::{GlobalAlloc, Layout, System};
        use std::cell::Cell;
        use std::fs::{self, OpenOptions};
        use std::io::Write;
        use std::path::PathBuf;
        use std::process::Command;

        use rustfft::num_complex::{Complex32, Complex64};
        use rustfft::FftPlanner;

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

        // The measuring state is thread-scoped on purpose. The allocator hook
        // below is process-global, so process-global counters would attribute
        // every other test thread's allocations to whichever thread happens to
        // be measuring. `Cell` with const init registers no destructor, so
        // reading it from inside the allocator cannot re-enter it.
        thread_local! {
            static ALLOCATION_MEASURING: Cell<bool> = const { Cell::new(false) };
            static PROCESSING_STARTED: Cell<bool> = const { Cell::new(false) };
            static LIVE_BYTES: Cell<usize> = const { Cell::new(0) };
            static PEAK_BYTES: Cell<usize> = const { Cell::new(0) };
            static PROCESSING_ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
        }

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
            let _ = ALLOCATION_MEASURING.try_with(|measuring| {
                if !measuring.get() {
                    return;
                }
                let live = LIVE_BYTES.with(|value| {
                    let updated = value.get().saturating_add(bytes);
                    value.set(updated);
                    updated
                });
                PEAK_BYTES.with(|peak| peak.set(peak.get().max(live)));
                if PROCESSING_STARTED.with(Cell::get) {
                    PROCESSING_ALLOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
                }
            });
        }

        fn direct_renewal_dream_record_deallocation(bytes: usize) {
            let _ = ALLOCATION_MEASURING.try_with(|measuring| {
                if !measuring.get() {
                    return;
                }
                LIVE_BYTES.with(|value| value.set(value.get().saturating_sub(bytes)));
            });
        }

        pub(crate) fn direct_renewal_dream_processing_started() {
            let _ = PROCESSING_STARTED.try_with(|started| started.set(true));
        }

        struct AllocationMeasurement;

        impl AllocationMeasurement {
            /// Measure allocations on the calling thread only. Live bytes start
            /// at zero, so the reported peak is growth attributable to the
            /// measured region rather than a global high-water mark.
            fn begin() -> Self {
                LIVE_BYTES.with(|value| value.set(0));
                PEAK_BYTES.with(|peak| peak.set(0));
                PROCESSING_ALLOCATIONS.with(|count| count.set(0));
                PROCESSING_STARTED.with(|started| started.set(false));
                ALLOCATION_MEASURING.with(|measuring| measuring.set(true));
                Self
            }

            fn finish(self) -> (usize, usize) {
                ALLOCATION_MEASURING.with(|measuring| measuring.set(false));
                let peak = PEAK_BYTES.with(Cell::get);
                let processing_allocations = PROCESSING_ALLOCATIONS.with(Cell::get);
                (peak, processing_allocations)
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
