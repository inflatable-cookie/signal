use super::{Review, HASH_OFFSET, SOURCE_FRAMES};

pub(super) fn window(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect()
}

pub(super) fn reflected(input: &[f64], mut index: isize) -> f64 {
    if input.is_empty() {
        return 0.0;
    }
    let end = input.len() as isize - 1;
    while index < 0 || index > end {
        index = if index < 0 {
            -index - 1
        } else {
            2 * end - index + 1
        };
    }
    input[index as usize]
}

pub(super) fn controls() -> Vec<Vec<f64>> {
    let mut seed = 0x1234_5678_u64;
    vec![
        (0..SOURCE_FRAMES)
            .map(|i| (std::f64::consts::TAU * 997.0 * i as f64 / 48_000.0).sin())
            .collect(),
        (0..SOURCE_FRAMES)
            .map(|i| {
                ((i as f64 / SOURCE_FRAMES as f64).powi(2) * 400.0 * std::f64::consts::TAU).sin()
            })
            .collect(),
        (0..SOURCE_FRAMES)
            .map(|i| if i % 997 == 0 { 1.0 } else { 0.0 })
            .collect(),
        (0..SOURCE_FRAMES)
            .map(|i| usize::from(i == 0 || i + 1 == SOURCE_FRAMES) as f64)
            .collect(),
        (0..SOURCE_FRAMES)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                (seed as i64 as f64) / i64::MAX as f64
            })
            .collect(),
        vec![0.0; SOURCE_FRAMES],
    ]
}

pub(super) fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}

pub(super) fn review_hash(review: &Review) -> u64 {
    let mut state = HASH_OFFSET;
    for value in review.hashes[..5].iter().copied() {
        hash(&mut state, value);
    }
    for value in review.maximum_errors {
        hash(&mut state, value.to_bits());
    }
    state
}
