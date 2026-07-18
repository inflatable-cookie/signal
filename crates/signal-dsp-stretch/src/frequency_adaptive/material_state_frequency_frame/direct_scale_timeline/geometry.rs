use std::sync::Arc;

use rustfft::{Fft, FftPlanner};

use super::*;

const MAX_HOP: usize = 480;
const MAX_OWNED_BINS: usize = 631;
pub(super) const CAPACITY: MemoryCounts = MemoryCounts {
    source_samples: 12 * MAX_HOP * CHANNEL_CAPACITY,
    pending_complex: PENDING_TICKS * CHANNEL_CAPACITY * MAX_OWNED_BINS,
    guidance_values: GUIDANCE_TICKS * MAX_OWNED_BINS,
    phase_values: 2 * CHANNEL_CAPACITY * MAX_OWNED_BINS,
    region_records: 2 * CHANNEL_CAPACITY * MAX_OWNED_BINS,
    output_samples: 8 * MAX_HOP * CHANNEL_CAPACITY,
    transform_complex: 14 * MAX_HOP * CHANNEL_CAPACITY,
    scratch_complex: 16 * MAX_HOP,
};

pub(super) struct ScalePlan {
    pub scale: Scale,
    pub length: usize,
    pub window: Vec<f64>,
    pub forward: Arc<dyn Fft<f64>>,
    pub inverse: Arc<dyn Fft<f64>>,
}

pub(super) struct Prepared {
    pub sample_rate: usize,
    pub channels: usize,
    pub hop: usize,
    pub lengths: [usize; 3],
    pub owned_bins: [usize; 3],
    pub plans: [ScalePlan; 3],
    pub memory: MemoryCounts,
    pub planner_scratch: usize,
    pub source_ring: Vec<f64>,
    pub pending: Vec<Complex64>,
    pub guidance: Vec<f64>,
    pub phase: Vec<f64>,
    pub regions: Vec<RegionRecord>,
    pub output_ring: Vec<f64>,
    pub transform: Vec<Complex64>,
    pub scratch: Vec<Complex64>,
    pub has_state: bool,
    pub region_slot: usize,
}

pub(super) fn prepare(
    sample_rate: usize,
    channels: usize,
    ratio: f64,
    discontinuity: bool,
) -> Result<Prepared, PrepareError> {
    if !PROOF_RATES.contains(&sample_rate) {
        return Err(PrepareError::Unsupported(UnsupportedGeometry::SampleRate));
    }
    if !(1..=CHANNEL_CAPACITY).contains(&channels) {
        return Err(PrepareError::Unsupported(UnsupportedGeometry::ChannelCount));
    }
    if !ratio.is_finite() || !(MIN_RATIO..=MAX_RATIO).contains(&ratio) {
        return Err(PrepareError::Unsupported(UnsupportedGeometry::Ratio));
    }
    if discontinuity {
        return Err(PrepareError::Unsupported(
            UnsupportedGeometry::Discontinuity,
        ));
    }

    let hop = sample_rate / 100;
    let lengths = [8 * hop, 4 * hop, 2 * hop];
    let owned_bins = Scale::ALL.map(|scale| count_owned_bins(sample_rate, lengths, scale));
    let total_owned = owned_bins.iter().sum::<usize>();
    let memory = memory_counts(hop, channels, total_owned);
    validate_capacity(memory)?;

    let mut planner = FftPlanner::<f64>::new();
    let plans = Scale::ALL.map(|scale| {
        let length = lengths[scale.index()];
        ScalePlan {
            scale,
            length,
            window: normalized_window(length, hop),
            forward: planner.plan_fft_forward(length),
            inverse: planner.plan_fft_inverse(length),
        }
    });
    let planner_scratch = plans
        .iter()
        .flat_map(|plan| {
            [
                plan.forward.get_inplace_scratch_len(),
                plan.inverse.get_inplace_scratch_len(),
            ]
        })
        .max()
        .unwrap_or(0);
    if planner_scratch > memory.scratch_complex {
        return Err(PrepareError::Capacity(CapacityExceeded::ScratchComplex));
    }

    Ok(Prepared {
        sample_rate,
        channels,
        hop,
        lengths,
        owned_bins,
        plans,
        memory,
        planner_scratch,
        source_ring: vec![0.0; memory.source_samples],
        pending: vec![Complex64::default(); memory.pending_complex],
        guidance: vec![0.0; memory.guidance_values],
        phase: vec![0.0; memory.phase_values],
        regions: vec![RegionRecord::default(); memory.region_records],
        output_ring: vec![0.0; memory.output_samples],
        transform: vec![Complex64::default(); memory.transform_complex],
        scratch: vec![Complex64::default(); memory.scratch_complex],
        has_state: false,
        region_slot: 0,
    })
}

pub(super) fn memory_counts(hop: usize, channels: usize, owned: usize) -> MemoryCounts {
    MemoryCounts {
        source_samples: 12 * hop * channels,
        pending_complex: PENDING_TICKS * channels * owned,
        guidance_values: GUIDANCE_TICKS * owned,
        phase_values: 2 * channels * owned,
        region_records: 2 * channels * owned,
        output_samples: 8 * hop * channels,
        transform_complex: 14 * hop * channels,
        scratch_complex: 16 * hop,
    }
}

pub(super) fn validate_capacity(request: MemoryCounts) -> Result<(), PrepareError> {
    let exceeded = if request.source_samples > CAPACITY.source_samples {
        Some(CapacityExceeded::SourceSamples)
    } else if request.pending_complex > CAPACITY.pending_complex {
        Some(CapacityExceeded::PendingComplex)
    } else if request.guidance_values > CAPACITY.guidance_values {
        Some(CapacityExceeded::GuidanceValues)
    } else if request.phase_values > CAPACITY.phase_values {
        Some(CapacityExceeded::PhaseValues)
    } else if request.region_records > CAPACITY.region_records {
        Some(CapacityExceeded::RegionRecords)
    } else if request.output_samples > CAPACITY.output_samples {
        Some(CapacityExceeded::OutputSamples)
    } else if request.transform_complex > CAPACITY.transform_complex {
        Some(CapacityExceeded::TransformComplex)
    } else if request.scratch_complex > CAPACITY.scratch_complex {
        Some(CapacityExceeded::ScratchComplex)
    } else {
        None
    };
    exceeded.map_or(Ok(()), |kind| Err(PrepareError::Capacity(kind)))
}

pub(super) fn normalized_window(length: usize, hop: usize) -> Vec<f64> {
    let scale = 2.0 * hop as f64 / length as f64;
    (0..length)
        .map(|index| {
            let hann = 0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos();
            (scale * hann).sqrt()
        })
        .collect()
}

pub(super) fn owns_frequency(scale: Scale, frequency: f64, nyquist: f64) -> bool {
    match scale {
        Scale::Long => frequency < 750.0,
        Scale::Middle => frequency >= 750.0 && (frequency < 6_000.0 || nyquist < 6_000.0),
        Scale::Short => nyquist >= 6_000.0 && frequency >= 6_000.0,
    }
}

pub(super) fn owned_start_bin(sample_rate: usize, length: usize, scale: Scale) -> usize {
    let threshold = match scale {
        Scale::Long => return 0,
        Scale::Middle => 750,
        Scale::Short => 6_000,
    };
    (threshold * length).div_ceil(sample_rate)
}

fn count_owned_bins(sample_rate: usize, lengths: [usize; 3], scale: Scale) -> usize {
    let length = lengths[scale.index()];
    let nyquist = sample_rate as f64 * 0.5;
    (0..=length / 2)
        .filter(|bin| {
            let frequency = *bin as f64 * sample_rate as f64 / length as f64;
            owns_frequency(scale, frequency, nyquist)
        })
        .count()
}

pub(super) fn synthesis_tick_range(target: usize, hop: usize) -> Option<(isize, isize)> {
    if target == 0 {
        return None;
    }
    let first = -3_isize;
    let last = ((target + 4 * hop - 1) / hop) as isize;
    Some((first, last))
}

pub(super) fn source_center(
    output_tick: isize,
    hop: usize,
    source: usize,
    target: usize,
) -> Result<isize, PrepareError> {
    if source == 0 || target == 0 {
        return Err(PrepareError::Unsupported(UnsupportedGeometry::TargetLength));
    }
    let output = output_tick as f64 * hop as f64;
    Ok((output * source as f64 / target as f64).round() as isize)
}

pub(super) fn reflected(source: &[f64], index: isize) -> f64 {
    if source.len() == 1 {
        return source[0];
    }
    let period = 2 * source.len() as isize;
    let folded = index.rem_euclid(period);
    let resolved = if folded < source.len() as isize {
        folded
    } else {
        period - folded - 1
    };
    source[resolved as usize]
}
