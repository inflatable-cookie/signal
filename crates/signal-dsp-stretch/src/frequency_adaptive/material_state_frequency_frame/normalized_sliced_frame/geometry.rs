use rustfft::FftPlanner;

use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::{
    build_representation_for_geometry, Representation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UnsupportedGeometry {
    HundredthFrame,
    ProofRate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CapacityExceeded {
    SignedAtoms,
    PositiveAtoms,
    Coefficients,
    Regions,
    SourceSlices,
    OutputSlices,
    Scratch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PrepareError {
    Unsupported(UnsupportedGeometry),
    Capacity(CapacityExceeded),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CapacityRequest {
    pub signed_atoms: usize,
    pub positive_atoms: usize,
    pub coefficients: usize,
    pub regions: usize,
    pub source_slices: usize,
    pub output_slices: usize,
    pub scratch_used: usize,
    pub scratch_capacity: usize,
}

pub(super) struct Geometry {
    pub sample_rate: usize,
    pub hop: usize,
    pub fft_frames: usize,
    pub outer_advance: usize,
    pub supports: [usize; 3],
    pub representation: Representation,
    pub positive_atoms: usize,
    pub tap_records: usize,
    pub scratch_capacity: usize,
    pub memory: MemoryCounts,
    pub per_slice_work: WorkCounts,
}

pub(super) fn prepare(sample_rate: usize) -> Result<Geometry, PrepareError> {
    if !sample_rate.is_multiple_of(100) {
        return Err(PrepareError::Unsupported(
            UnsupportedGeometry::HundredthFrame,
        ));
    }
    if !PROOF_RATES.contains(&sample_rate) {
        return Err(PrepareError::Unsupported(UnsupportedGeometry::ProofRate));
    }

    let hop = sample_rate / 100;
    let fft_frames = 32 * hop;
    let outer_advance = 16 * hop;
    let supports = [8 * hop, 4 * hop, 2 * hop];
    let representation =
        build_representation_for_geometry(fft_frames, sample_rate, hop, supports, [750, 6_000]);
    let signed_atoms = representation.bands.len();
    let positive_atoms = representation
        .bands
        .iter()
        .filter(|band| band.center <= fft_frames / 2)
        .count();
    let tap_records = representation
        .bands
        .iter()
        .map(|band| band.taps.len())
        .sum();

    let mut planner = FftPlanner::<f64>::new();
    let scratch_capacity = [
        planner.plan_fft_forward(fft_frames),
        planner.plan_fft_inverse(fft_frames),
        planner.plan_fft_forward(COEFFICIENT_CAPACITY),
        planner.plan_fft_inverse(COEFFICIENT_CAPACITY),
    ]
    .into_iter()
    .map(|fft| fft.get_inplace_scratch_len())
    .max()
    .unwrap_or(0);
    validate_capacity(CapacityRequest {
        signed_atoms,
        positive_atoms,
        coefficients: representation.common_coefficients,
        regions: positive_atoms,
        source_slices: SOURCE_SLICE_CAPACITY,
        output_slices: OUTPUT_SLICE_CAPACITY,
        scratch_used: scratch_capacity,
        scratch_capacity,
    })?;

    let memory = MemoryCounts {
        coefficient_complex: (SOURCE_SLICE_CAPACITY + OUTPUT_SLICE_CAPACITY)
            * CHANNEL_CAPACITY
            * signed_atoms
            * COEFFICIENT_CAPACITY,
        transform_complex: 2 * fft_frames + COEFFICIENT_CAPACITY + scratch_capacity,
        outer_samples: 2 * CHANNEL_CAPACITY * fft_frames,
        guidance_values: MATERIAL_HALO_FRAMES * positive_atoms * (CHANNEL_CAPACITY + 3),
        phase_values: 6 * CHANNEL_CAPACITY * positive_atoms,
        region_records: 2 * positive_atoms,
        static_values: 2 * fft_frames,
        tap_records,
        band_records: signed_atoms,
    };
    let per_slice_work = WorkCounts {
        full_transforms: 2,
        band_transforms: 2 * signed_atoms,
        tap_visits: 2 * tap_records,
        coefficient_visits: 2 * signed_atoms * COEFFICIENT_CAPACITY,
        sample_visits: 4 * fft_frames,
        conjugate_visits: fft_frames / 2 + 1,
    };
    Ok(Geometry {
        sample_rate,
        hop,
        fft_frames,
        outer_advance,
        supports,
        representation,
        positive_atoms,
        tap_records,
        scratch_capacity,
        memory,
        per_slice_work,
    })
}

pub(super) fn validate_capacity(request: CapacityRequest) -> Result<(), PrepareError> {
    let exceeded = if request.signed_atoms > SIGNED_ATOM_CAPACITY {
        Some(CapacityExceeded::SignedAtoms)
    } else if request.positive_atoms > POSITIVE_ATOM_CAPACITY {
        Some(CapacityExceeded::PositiveAtoms)
    } else if request.coefficients > COEFFICIENT_CAPACITY {
        Some(CapacityExceeded::Coefficients)
    } else if request.regions > REGION_CAPACITY {
        Some(CapacityExceeded::Regions)
    } else if request.source_slices > SOURCE_SLICE_CAPACITY {
        Some(CapacityExceeded::SourceSlices)
    } else if request.output_slices > OUTPUT_SLICE_CAPACITY {
        Some(CapacityExceeded::OutputSlices)
    } else if request.scratch_used > request.scratch_capacity {
        Some(CapacityExceeded::Scratch)
    } else {
        None
    };
    exceeded.map_or(Ok(()), |kind| Err(PrepareError::Capacity(kind)))
}
