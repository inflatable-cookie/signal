use super::{render::*, *};

pub(super) fn stage_a_review() -> SlicedStageAReview {
    let representation = build_representation();
    let renderer = SlicedRenderer::new(&representation);
    let mut maximum_errors = [0.0_f64; 7];
    let mut relation_errors = [0.0_f64; 4];
    let mut mechanics_failures = [0_usize; 6];
    let mut identity_slice_counts = [0_usize; 5];
    let mut boundedness = [[0_usize; 4]; 3];
    let mut non_finite_values = 0;
    let mut output_hash = HASH_OFFSET;

    for (slot, length) in identity_slice_counts.iter_mut().zip(IDENTITY_LENGTHS) {
        let source = deterministic_probe_for(length);
        let second = second_probe(&source);
        let left = renderer.render(&source);
        let right = renderer.render(&second);
        let swapped_right = renderer.render(&second);
        let swapped_left = renderer.render(&source);
        let scale = 0.375;
        let negative_input = source.iter().map(|sample| -*sample).collect::<Vec<_>>();
        let scaled_input = source
            .iter()
            .map(|sample| sample * scale)
            .collect::<Vec<_>>();
        let silence_input = vec![0.0; length];
        let head_input = impulse_for(length, 0);
        let tail_input = impulse_for(length, length - 1);
        let negative = renderer.render(&negative_input);
        let scaled = renderer.render(&scaled_input);
        let silence = renderer.render(&silence_input);
        let head = renderer.render(&head_input);
        let tail = renderer.render(&tail_input);
        let results = [
            &left,
            &right,
            &swapped_right,
            &swapped_left,
            &negative,
            &scaled,
            &silence,
            &head,
            &tail,
        ];
        *slot = left.slice_count;
        for (result, input) in results.into_iter().zip([
            source.as_slice(),
            second.as_slice(),
            second.as_slice(),
            source.as_slice(),
            negative_input.as_slice(),
            scaled_input.as_slice(),
            silence_input.as_slice(),
            head_input.as_slice(),
            tail_input.as_slice(),
        ]) {
            accumulate_result(&mut maximum_errors, result, input);
            mechanics_failures[0] += usize::from(result.samples.len() != length);
            mechanics_failures[1] += result.coverage_failures;
            non_finite_values += result.non_finite_values;
            hash_u64(&mut output_hash, result.output_hash);
        }
        mechanics_failures[2] += silence
            .samples
            .iter()
            .filter(|sample| **sample != 0.0)
            .count();
        mechanics_failures[3] += usize::from(
            (head.samples[0] - 1.0).abs() > 1.0e-12
                || (tail.samples[length - 1] - 1.0).abs() > 1.0e-12,
        );
        relation_errors[0] = relation_errors[0].max(max_abs(&silence.samples));
        relation_errors[1] = relation_errors[1]
            .max(paired_max_error(&right.samples, &swapped_right.samples))
            .max(paired_max_error(&left.samples, &swapped_left.samples));
        relation_errors[2] = relation_errors[2].max(
            left.samples
                .iter()
                .zip(&negative.samples)
                .map(|(positive, negative)| (positive + negative).abs())
                .fold(0.0_f64, f64::max),
        );
        relation_errors[3] = relation_errors[3].max(
            left.samples
                .iter()
                .zip(&scaled.samples)
                .map(|(reference, duplicate)| (reference * scale - duplicate).abs())
                .fold(0.0_f64, f64::max),
        );
        mechanics_failures[5] += usize::from(
            left.output_hash != swapped_left.output_hash
                || right.output_hash != swapped_right.output_hash,
        );
        for (row, bounded_length) in boundedness.iter_mut().zip(BOUNDED_LENGTHS) {
            if length == bounded_length {
                *row = boundedness_row(&left);
            }
        }
    }
    mechanics_failures[4] = relation_errors
        .iter()
        .filter(|error| **error > 1.0e-12)
        .count();

    let bounded_probe = renderer.render(&deterministic_probe_for(BOUNDED_LENGTHS[1]));
    boundedness[1] = boundedness_row(&bounded_probe);
    non_finite_values += bounded_probe.non_finite_values;
    hash_u64(&mut output_hash, bounded_probe.output_hash);
    let per_slice_operations = per_slice_operations(&representation);
    let mut review = SlicedStageAReview {
        geometry: [
            FFT_FRAMES,
            OUTER_ADVANCE,
            representation.common_hop,
            representation.common_coefficients,
        ],
        support_frames: SUPPORT_FRAMES,
        crossover_hz: CROSSOVER_HZ,
        owner_counts: representation.owner_counts,
        structural_failures: representation.structural_failures,
        identity_lengths: IDENTITY_LENGTHS,
        identity_slice_counts,
        maximum_errors,
        relation_errors,
        mechanics_failures,
        boundedness,
        per_slice_operations,
        non_finite_values,
        hashes: [
            representation.filter_hash,
            representation.dual_hash,
            output_hash,
            0,
        ],
    };
    review.hashes[3] = sliced_review_hash(&review);
    review
}

fn boundedness_row(result: &SliceResult) -> [usize; 4] {
    [
        result.slice_count,
        result.maximum_live_slices,
        result.peak_live_coefficients,
        result.counted_operations,
    ]
}

fn accumulate_result(maximum: &mut [f64; 7], result: &SliceResult, input: &[f64]) {
    let mut peak = 0.0_f64;
    let mut square_sum = 0.0;
    for (source, output) in input.iter().zip(&result.samples) {
        let error = (source - output).abs();
        peak = peak.max(error);
        square_sum += error * error;
    }
    let values = [
        peak,
        (square_sum / input.len().max(1) as f64).sqrt(),
        input
            .first()
            .zip(result.samples.first())
            .map_or(0.0, |(a, b)| (a - b).abs()),
        input
            .last()
            .zip(result.samples.last())
            .map_or(0.0, |(a, b)| (a - b).abs()),
        result.imaginary_residue,
        result.conjugate_error,
        result.partition_error,
    ];
    for (slot, value) in maximum.iter_mut().zip(values) {
        *slot = slot.max(value);
    }
}

fn deterministic_probe_for(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE_HZ as f64;
            (std::f64::consts::TAU * 55.0 * time).sin() * 0.23
                + (std::f64::consts::TAU * 440.0 * time + 0.31).sin() * 0.19
                + (std::f64::consts::TAU * 4_000.0 * time + 0.73).sin() * 0.11
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

fn impulse_for(length: usize, index: usize) -> Vec<f64> {
    let mut result = vec![0.0; length];
    result[index] = 1.0;
    result
}

fn max_abs(samples: &[f64]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f64, f64::max)
}
