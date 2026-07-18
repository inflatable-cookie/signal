use rustfft::num_complex::Complex64;

use super::{hash::*, HASH_OFFSET};

mod geometry;
mod render;
mod report;

const PROOF_RATES: [usize; 3] = [8_000, 44_100, 48_000];
const CHANNEL_CAPACITY: usize = 2;
const MIN_RATIO: f64 = 0.25;
const MAX_RATIO: f64 = 4.0;
const GUIDANCE_TICKS: usize = 19;
const PENDING_TICKS: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scale {
    Long,
    Middle,
    Short,
}

impl Scale {
    const ALL: [Self; 3] = [Self::Long, Self::Middle, Self::Short];

    fn index(self) -> usize {
        match self {
            Self::Long => 0,
            Self::Middle => 1,
            Self::Short => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MemoryCounts {
    source_samples: usize,
    pending_complex: usize,
    guidance_values: usize,
    phase_values: usize,
    region_records: usize,
    output_samples: usize,
    transform_complex: usize,
    scratch_complex: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct WorkCounts {
    forward_transforms: usize,
    inverse_transforms: usize,
    window_visits: usize,
    coefficient_visits: usize,
    conjugate_visits: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RegionRecord {
    peak: usize,
    owner: usize,
    supported: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnsupportedGeometry {
    SampleRate,
    ChannelCount,
    Ratio,
    TargetLength,
    Discontinuity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CapacityExceeded {
    SourceSamples,
    PendingComplex,
    GuidanceValues,
    PhaseValues,
    RegionRecords,
    OutputSamples,
    TransformComplex,
    ScratchComplex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrepareError {
    Unsupported(UnsupportedGeometry),
    Capacity(CapacityExceeded),
}

#[derive(Clone, Debug, PartialEq)]
struct ScaleReview {
    scale: Scale,
    length: usize,
    owned_bins: usize,
    partition_error: f64,
    reconstruction_error: f64,
    imaginary_residue: f64,
    conjugate_error: f64,
    non_finite_values: usize,
    work: WorkCounts,
    hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct MaskedDiagnostic {
    control: &'static str,
    peak_residual: f64,
    rms_residual: f64,
    gain_delta_db: f64,
    timing_frames: isize,
    boundary_error: f64,
    non_finite_values: usize,
    hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct RateReview {
    sample_rate: usize,
    hop: usize,
    lengths: [usize; 3],
    owned_bins: [usize; 3],
    memory: MemoryCounts,
    planner_scratch: usize,
    scale_reviews: Vec<ScaleReview>,
    masked_diagnostics: Vec<MaskedDiagnostic>,
    unity_failures: usize,
    schedule_failures: usize,
    structural_failures: usize,
    work: WorkCounts,
    hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct StageReview {
    rates: Vec<RateReview>,
    overflow_failures: usize,
    unsupported_failures: usize,
    hash: u64,
}

fn hash_memory(hash: &mut u64, memory: MemoryCounts) {
    for value in [
        memory.source_samples,
        memory.pending_complex,
        memory.guidance_values,
        memory.phase_values,
        memory.region_records,
        memory.output_samples,
        memory.transform_complex,
        memory.scratch_complex,
    ] {
        hash_usize(hash, value);
    }
}

fn hash_work(hash: &mut u64, work: WorkCounts) {
    for value in [
        work.forward_transforms,
        work.inverse_transforms,
        work.window_visits,
        work.coefficient_visits,
        work.conjugate_visits,
    ] {
        hash_usize(hash, value);
    }
}

#[cfg(test)]
mod tests {
    use super::{report::*, *};

    #[test]
    fn direct_scale_timeline_rule_31z_geometry_and_capacity_pass() {
        let review = stage_review();
        eprintln!("direct_scale_timeline_rule_31z {review:#?}");
        assert_eq!(review.rates.len(), 3, "{review:#?}");
        assert_eq!(review.overflow_failures, 0, "{review:#?}");
        assert_eq!(review.unsupported_failures, 0, "{review:#?}");
        assert_eq!(review.rates[0].owned_bins, [60, 131, 0]);
        assert_eq!(review.rates[1].owned_bins, [60, 210, 322]);
        assert_eq!(review.rates[2].owned_bins, [60, 210, 361]);
        assert_eq!(
            review.rates[2].memory,
            MemoryCounts {
                source_samples: 11_520,
                pending_complex: 12_620,
                guidance_values: 11_989,
                phase_values: 2_524,
                region_records: 2_524,
                output_samples: 7_680,
                transform_complex: 13_440,
                scratch_complex: 7_680,
            }
        );
        assert!(review
            .rates
            .iter()
            .all(|rate| rate.structural_failures == 0));
        assert!(review.rates.iter().all(|rate| rate.schedule_failures == 0));
        assert!(review.rates.iter().all(|rate| rate.unity_failures == 0));
    }

    #[test]
    fn direct_scale_timeline_rule_31z_per_scale_reconstruction_passes() {
        let review = stage_review();
        for rate in &review.rates {
            for scale in &rate.scale_reviews {
                assert!(scale.partition_error <= 1.0e-12, "{scale:#?}");
                assert!(scale.reconstruction_error <= 1.0e-12, "{scale:#?}");
                assert!(scale.imaginary_residue <= 1.0e-12, "{scale:#?}");
                assert!(scale.conjugate_error <= 1.0e-12, "{scale:#?}");
                assert_eq!(scale.non_finite_values, 0, "{scale:#?}");
            }
        }
    }

    #[test]
    fn direct_scale_timeline_rule_31z_masked_sum_is_diagnostic_and_repeats() {
        let first = stage_review();
        let second = stage_review();
        eprintln!("direct_scale_timeline_rule_31z_masked {first:#?}");
        assert_eq!(first, second);
        for rate in &first.rates {
            assert!(rate.masked_diagnostics.iter().all(|row| {
                row.peak_residual.is_finite()
                    && row.rms_residual.is_finite()
                    && row.gain_delta_db.is_finite()
                    && row.boundary_error.is_finite()
                    && row.non_finite_values == 0
            }));
            assert_eq!(rate.masked_diagnostics[0].control, "silence");
            assert_eq!(rate.masked_diagnostics[0].peak_residual, 0.0);
        }
    }

    #[test]
    fn direct_scale_timeline_rule_31z_rejects_every_excess_and_unsupported_request() {
        assert_eq!(overflow_failures(), 0);
        assert_eq!(unsupported_failures(), 0);
    }
}
