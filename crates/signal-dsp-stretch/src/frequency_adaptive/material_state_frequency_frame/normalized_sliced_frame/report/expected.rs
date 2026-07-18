use super::*;

pub(super) fn expected_row(sample_rate: usize) -> ([usize; 4], [usize; 3], [usize; 3], [usize; 3]) {
    match sample_rate {
        8_000 => (
            [2_560, 1_280, 80, 32],
            [640, 320, 160],
            [380, 191, 4_740],
            [119, 261, 0],
        ),
        44_100 => (
            [14_112, 7_056, 441, 32],
            [3_528, 1_764, 882],
            [1_182, 592, 27_042],
            [119, 420, 643],
        ),
        48_000 => (
            [15_360, 7_680, 480, 32],
            [3_840, 1_920, 960],
            [1_260, 631, 29_460],
            [119, 420, 721],
        ),
        _ => unreachable!("proof rate"),
    }
}

pub(super) fn expected_memory(geometry: &Geometry) -> MemoryCounts {
    let signed = geometry.representation.bands.len();
    let positive = geometry.positive_atoms;
    MemoryCounts {
        coefficient_complex: 8 * CHANNEL_CAPACITY * signed * COEFFICIENT_CAPACITY,
        transform_complex: 2 * geometry.fft_frames
            + COEFFICIENT_CAPACITY
            + geometry.scratch_capacity,
        outer_samples: 2 * CHANNEL_CAPACITY * geometry.fft_frames,
        guidance_values: MATERIAL_HALO_FRAMES * positive * (CHANNEL_CAPACITY + 3),
        phase_values: 6 * CHANNEL_CAPACITY * positive,
        region_records: 2 * positive,
        static_values: 2 * geometry.fft_frames,
        tap_records: geometry.tap_records,
        band_records: signed,
    }
}
