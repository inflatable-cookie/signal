use super::*;

pub(super) fn overflow_failures() -> usize {
    let geometry = prepare(48_000).expect("maximum proof geometry");
    let base = CapacityRequest {
        signed_atoms: SIGNED_ATOM_CAPACITY,
        positive_atoms: POSITIVE_ATOM_CAPACITY,
        coefficients: COEFFICIENT_CAPACITY,
        regions: REGION_CAPACITY,
        source_slices: SOURCE_SLICE_CAPACITY,
        output_slices: OUTPUT_SLICE_CAPACITY,
        scratch_used: geometry.scratch_capacity,
        scratch_capacity: geometry.scratch_capacity,
    };
    let checks = [
        (
            prepare(44_101).err(),
            Some(PrepareError::Unsupported(
                geometry::UnsupportedGeometry::HundredthFrame,
            )),
        ),
        (
            prepare(32_000).err(),
            Some(PrepareError::Unsupported(
                geometry::UnsupportedGeometry::ProofRate,
            )),
        ),
        capacity_error(
            CapacityRequest {
                signed_atoms: SIGNED_ATOM_CAPACITY + 1,
                ..base
            },
            geometry::CapacityExceeded::SignedAtoms,
        ),
        capacity_error(
            CapacityRequest {
                positive_atoms: POSITIVE_ATOM_CAPACITY + 1,
                ..base
            },
            geometry::CapacityExceeded::PositiveAtoms,
        ),
        capacity_error(
            CapacityRequest {
                coefficients: COEFFICIENT_CAPACITY + 1,
                ..base
            },
            geometry::CapacityExceeded::Coefficients,
        ),
        capacity_error(
            CapacityRequest {
                regions: REGION_CAPACITY + 1,
                ..base
            },
            geometry::CapacityExceeded::Regions,
        ),
        capacity_error(
            CapacityRequest {
                source_slices: SOURCE_SLICE_CAPACITY + 1,
                ..base
            },
            geometry::CapacityExceeded::SourceSlices,
        ),
        capacity_error(
            CapacityRequest {
                output_slices: OUTPUT_SLICE_CAPACITY + 1,
                ..base
            },
            geometry::CapacityExceeded::OutputSlices,
        ),
        capacity_error(
            CapacityRequest {
                scratch_used: geometry.scratch_capacity + 1,
                ..base
            },
            geometry::CapacityExceeded::Scratch,
        ),
    ];
    checks
        .into_iter()
        .filter(|(actual, expected)| actual != expected)
        .count()
}

fn capacity_error(
    request: CapacityRequest,
    expected: geometry::CapacityExceeded,
) -> (Option<PrepareError>, Option<PrepareError>) {
    (
        validate_capacity(request).err(),
        Some(PrepareError::Capacity(expected)),
    )
}
