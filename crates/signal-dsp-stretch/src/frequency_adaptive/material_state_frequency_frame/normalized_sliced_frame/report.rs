use super::*;

mod expected;
mod overflow;
use expected::{expected_memory, expected_row};
use overflow::overflow_failures;

pub(super) fn stage_a_review() -> StageAReview {
    let rates = PROOF_RATES
        .into_iter()
        .map(|sample_rate| rate_review(&prepare(sample_rate).expect("proof geometry")))
        .collect();
    let mut review = StageAReview {
        rates,
        overflow_failures: overflow_failures(),
        hash: 0,
    };
    review.hash = review_hash(&review);
    review
}

fn rate_review(geometry: &Geometry) -> RateReview {
    let expected = expected_row(geometry.sample_rate);
    let identity_lengths = [
        1,
        geometry.outer_advance - 1,
        geometry.outer_advance,
        geometry.outer_advance + 1,
        3 * geometry.outer_advance + 17,
    ];
    let bounded_lengths = [
        geometry.outer_advance + 17,
        4 * geometry.outer_advance + 29,
        12 * geometry.outer_advance + 31,
    ];
    let mut renderer = Renderer::new(geometry);
    let mut maximum_errors = [0.0_f64; 7];
    let mut mechanics_errors = [0.0_f64; 4];
    let mut mechanics_failures = [0_usize; 5];
    let mut non_finite_values = 0;
    let mut output_hash = HASH_OFFSET;

    for length in identity_lengths {
        let source = deterministic_probe(length, geometry.sample_rate);
        let result = renderer.render(&source);
        accumulate_errors(&mut maximum_errors, &result);
        mechanics_failures[0] += usize::from(result.samples.len() != length);
        mechanics_failures[1] += result.coverage_failures;
        mechanics_failures[3] +=
            usize::from(result.work != geometry.per_slice_work.scaled(result.slice_count));
        non_finite_values += result.non_finite_values;
        hash_u64(&mut output_hash, result.hash);
    }

    let mechanics_length = geometry.outer_advance + 17;
    let source = deterministic_probe(mechanics_length, geometry.sample_rate);
    let second = second_probe(&source);
    let silence = vec![0.0; mechanics_length];
    let negative = source.iter().map(|sample| -*sample).collect::<Vec<_>>();
    let scale = 0.375;
    let scaled = source
        .iter()
        .map(|sample| sample * scale)
        .collect::<Vec<_>>();
    let mut head = vec![0.0; mechanics_length];
    head[0] = 1.0;
    let mut tail = vec![0.0; mechanics_length];
    tail[mechanics_length - 1] = 1.0;

    let left = renderer.render(&source);
    let left_repeat = renderer.render(&source);
    let right = renderer.render(&second);
    let right_repeat = renderer.render(&second);
    let zero = renderer.render(&silence);
    let inverted = renderer.render(&negative);
    let reduced = renderer.render(&scaled);
    let head_result = renderer.render(&head);
    let tail_result = renderer.render(&tail);
    mechanics_errors[0] = maximum_abs(&zero.samples);
    mechanics_errors[1] = paired_max_error(&left.samples, &left_repeat.samples)
        .max(paired_max_error(&right.samples, &right_repeat.samples));
    mechanics_errors[2] = left
        .samples
        .iter()
        .zip(&inverted.samples)
        .map(|(positive, negative)| (positive + negative).abs())
        .fold(0.0_f64, f64::max);
    mechanics_errors[3] = left
        .samples
        .iter()
        .zip(&reduced.samples)
        .map(|(reference, duplicate)| (reference * scale - duplicate).abs())
        .fold(0.0_f64, f64::max);
    mechanics_failures[2] += usize::from(
        (head_result.samples[0] - 1.0).abs() > 1.0e-12
            || (tail_result.samples[mechanics_length - 1] - 1.0).abs() > 1.0e-12,
    );
    mechanics_failures[4] +=
        usize::from(left.hash != left_repeat.hash) + usize::from(right.hash != right_repeat.hash);
    for result in [
        &left,
        &left_repeat,
        &right,
        &right_repeat,
        &zero,
        &inverted,
        &reduced,
        &head_result,
        &tail_result,
    ] {
        non_finite_values += result.non_finite_values;
        hash_u64(&mut output_hash, result.hash);
    }

    let bounded_slices =
        bounded_lengths.map(|length| required_slice_count(length, geometry.outer_advance));
    let bounded_tokens = bounded_lengths.map(|length| boundary_token_review(length, geometry));
    let total_work = bounded_slices.map(|slices| geometry.per_slice_work.scaled(slices));
    let owner_counts = geometry.representation.owner_counts;
    let structural_failures = [
        geometry.representation.structural_failures[..3]
            .iter()
            .sum(),
        usize::from(
            [
                geometry.fft_frames,
                geometry.outer_advance,
                geometry.hop,
                geometry.representation.common_coefficients,
            ] != expected.0
                || geometry.supports != expected.1,
        ),
        usize::from(
            [
                geometry.representation.bands.len(),
                geometry.positive_atoms,
                geometry.tap_records,
            ] != expected.2,
        ),
        usize::from(geometry.tap_records != 2 * geometry.fft_frames - expected.2[0]),
        usize::from(geometry.memory != expected_memory(geometry)),
        usize::from(geometry.representation.frame_values[0] <= 0.0),
        usize::from(
            geometry
                .representation
                .bands
                .iter()
                .any(|band| band.taps.len() > COEFFICIENT_CAPACITY),
        ),
        usize::from(owner_counts != expected.3),
    ];
    let mut geometry_hash = HASH_OFFSET;
    hash_u64(&mut geometry_hash, geometry.representation.filter_hash);
    hash_u64(&mut geometry_hash, geometry.representation.dual_hash);
    hash_memory(&mut geometry_hash, geometry.memory);
    hash_work(&mut geometry_hash, geometry.per_slice_work);

    RateReview {
        sample_rate: geometry.sample_rate,
        geometry: [
            geometry.fft_frames,
            geometry.outer_advance,
            geometry.hop,
            geometry.representation.common_coefficients,
        ],
        supports: geometry.supports,
        atom_counts: [
            geometry.representation.bands.len(),
            geometry.positive_atoms,
            geometry.tap_records,
        ],
        owner_counts,
        structural_failures,
        maximum_errors,
        mechanics_errors,
        mechanics_failures,
        bounded_lengths,
        bounded_slices,
        bounded_tokens,
        memory: geometry.memory,
        per_slice_work: geometry.per_slice_work,
        total_work,
        non_finite_values,
        hashes: [geometry_hash, output_hash, token_hash(&bounded_tokens)],
    }
}

fn accumulate_errors(maximum: &mut [f64; 7], result: &render::RenderResult) {
    for (slot, value) in maximum.iter_mut().zip([
        result.peak_error,
        result.rms_error,
        result.head_error,
        result.tail_error,
        result.imaginary_residue,
        result.conjugate_error,
        result.partition_error,
    ]) {
        *slot = slot.max(value);
    }
}

fn deterministic_probe(length: usize, sample_rate: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            let time = index as f64 / sample_rate as f64;
            (std::f64::consts::TAU * 55.0 * time).sin() * 0.23
                + (std::f64::consts::TAU * 440.0 * time + 0.31).sin() * 0.19
                + (std::f64::consts::TAU * 1_700.0 * time + 0.73).sin() * 0.11
                + ((index * 73 % 509) as f64 - 254.0) / 4_096.0
        })
        .collect()
}

fn second_probe(source: &[f64]) -> Vec<f64> {
    source
        .iter()
        .enumerate()
        .map(|(index, sample)| sample * 0.41 + ((index * 29 % 257) as f64 - 128.0) / 1_024.0)
        .collect()
}

fn maximum_abs(samples: &[f64]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f64, f64::max)
}

fn token_hash(tokens: &[TokenReview]) -> u64 {
    let mut hash = HASH_OFFSET;
    for token in tokens {
        for value in [
            token.expected_updates,
            token.updates,
            token.final_value,
            token.duplicate_updates,
            token.reset_failures,
            token.capacity_failures,
            token.slice_creations,
            token.slice_retirements,
            token.active_high_water,
            token.boundary_crossings,
        ] {
            hash_usize(&mut hash, value);
        }
    }
    hash
}
