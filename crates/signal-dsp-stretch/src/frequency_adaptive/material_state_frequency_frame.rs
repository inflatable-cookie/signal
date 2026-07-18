use rustfft::{num_complex::Complex64, FftPlanner};

#[cfg(not(debug_assertions))]
mod material_phase;
#[cfg(not(debug_assertions))]
mod sliced_frame;
#[cfg(not(debug_assertions))]
mod sliced_material;

const SAMPLE_RATE_HZ: usize = 48_000;
const FFT_FRAMES: usize = 16_384;
const SOURCE_FRAMES: usize = 8_192;
const PAD_FRAMES: usize = 4_096;
const SUPPORT_FRAMES: [usize; 3] = [4_096, 2_048, 1_024];
const CROSSOVER_HZ: [usize; 2] = [750, 6_000];
const HASH_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scale {
    Long,
    Middle,
    Short,
}

impl Scale {
    fn index(self) -> usize {
        match self {
            Self::Long => 0,
            Self::Middle => 1,
            Self::Short => 2,
        }
    }
}

#[derive(Clone)]
struct Band {
    center: usize,
    scale: Scale,
    taps: Vec<(usize, f64)>,
}

#[derive(Clone)]
struct Representation {
    fft_frames: usize,
    bands: Vec<Band>,
    frame_operator: Vec<f64>,
    common_coefficients: usize,
    common_hop: usize,
    owner_counts: [usize; 3],
    structural_failures: [usize; 4],
    frame_values: [f64; 3],
    filter_hash: u64,
    dual_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct ChannelResult {
    samples: Vec<f64>,
    peak_error: f64,
    rms_error: f64,
    head_error: f64,
    tail_error: f64,
    imaginary_residue: f64,
    conjugate_error: f64,
    non_finite_values: usize,
    output_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct StageAReview {
    geometry: [usize; 5],
    support_frames: [usize; 3],
    crossover_hz: [usize; 2],
    owner_counts: [usize; 3],
    structural_failures: [usize; 4],
    frame_values: [f64; 3],
    maximum_errors: [f64; 6],
    relation_errors: [f64; 4],
    mechanics_failures: [usize; 4],
    reflected_reads: usize,
    non_finite_values: usize,
    hashes: [u64; 4],
}

fn stage_a_review() -> StageAReview {
    let representation = build_representation();
    let source = deterministic_probe();
    let second = source
        .iter()
        .enumerate()
        .map(|(index, sample)| sample * 0.41 + ((index * 29 % 257) as f64 - 128.0) / 1_024.0)
        .collect::<Vec<_>>();

    let controls = [
        source.clone(),
        impulse(0),
        impulse(SOURCE_FRAMES - 1),
        vec![0.0; SOURCE_FRAMES],
    ];
    let mut maximum_errors = [0.0_f64; 6];
    let mut non_finite_values = 0;
    let mut output_hash = HASH_OFFSET;
    let mut silence_failures = 0;
    for control in &controls {
        let result = reconstruct_channel(control, &representation);
        accumulate_errors(&mut maximum_errors, &result);
        non_finite_values += result.non_finite_values;
        hash_u64(&mut output_hash, result.output_hash);
        if control.iter().all(|sample| *sample == 0.0) {
            silence_failures += result
                .samples
                .iter()
                .filter(|sample| **sample != 0.0)
                .count();
        }
    }

    let left = reconstruct_channel(&source, &representation);
    let right = reconstruct_channel(&second, &representation);
    let swapped_left = reconstruct_channel(&second, &representation);
    let swapped_right = reconstruct_channel(&source, &representation);
    let zero = reconstruct_channel(&vec![0.0; SOURCE_FRAMES], &representation);
    let negative = reconstruct_channel(
        &source.iter().map(|sample| -*sample).collect::<Vec<_>>(),
        &representation,
    );
    let scale = 0.375;
    let scaled = reconstruct_channel(
        &source
            .iter()
            .map(|sample| *sample * scale)
            .collect::<Vec<_>>(),
        &representation,
    );

    let hard_pan_error = zero
        .samples
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0, f64::max);
    let swap_error = paired_max_error(&right.samples, &swapped_left.samples)
        .max(paired_max_error(&left.samples, &swapped_right.samples));
    let polarity_error = left
        .samples
        .iter()
        .zip(&negative.samples)
        .map(|(positive, negative)| (positive + negative).abs())
        .fold(0.0, f64::max);
    let scaled_duplicate_error = left
        .samples
        .iter()
        .zip(&scaled.samples)
        .map(|(reference, scaled)| (reference * scale - scaled).abs())
        .fold(0.0, f64::max);
    let relation_errors = [
        hard_pan_error,
        swap_error,
        polarity_error,
        scaled_duplicate_error,
    ];

    let exact_crop_failures = usize::from(left.samples.len() != SOURCE_FRAMES)
        + usize::from(right.samples.len() != SOURCE_FRAMES);
    let boundary_failures = usize::from(
        left.head_error > 1.0e-12
            || left.tail_error > 1.0e-12
            || controls[1][0] != 1.0
            || controls[2][SOURCE_FRAMES - 1] != 1.0,
    );
    let relation_failures = relation_errors
        .iter()
        .filter(|error| **error > 1.0e-12)
        .count();
    let mechanics_failures = [
        exact_crop_failures,
        silence_failures,
        boundary_failures,
        relation_failures,
    ];

    non_finite_values += left.non_finite_values
        + right.non_finite_values
        + swapped_left.non_finite_values
        + swapped_right.non_finite_values
        + zero.non_finite_values
        + negative.non_finite_values
        + scaled.non_finite_values;
    hash_u64(&mut output_hash, left.output_hash);
    hash_u64(&mut output_hash, right.output_hash);
    hash_u64(&mut output_hash, swapped_left.output_hash);
    hash_u64(&mut output_hash, swapped_right.output_hash);
    hash_u64(&mut output_hash, negative.output_hash);
    hash_u64(&mut output_hash, scaled.output_hash);

    let mut review = StageAReview {
        geometry: [
            FFT_FRAMES,
            SOURCE_FRAMES,
            PAD_FRAMES,
            representation.common_coefficients,
            representation.common_hop,
        ],
        support_frames: SUPPORT_FRAMES,
        crossover_hz: CROSSOVER_HZ,
        owner_counts: representation.owner_counts,
        structural_failures: representation.structural_failures,
        frame_values: representation.frame_values,
        maximum_errors,
        relation_errors,
        mechanics_failures,
        reflected_reads: PAD_FRAMES * 2,
        non_finite_values,
        hashes: [
            representation.filter_hash,
            representation.dual_hash,
            output_hash,
            0,
        ],
    };
    review.hashes[3] = review_hash(&review);
    review
}

fn build_representation() -> Representation {
    build_representation_for(FFT_FRAMES, SAMPLE_RATE_HZ, 512)
}

fn build_representation_for(
    fft_frames: usize,
    sample_rate_hz: usize,
    common_hop: usize,
) -> Representation {
    assert_eq!(fft_frames % SUPPORT_FRAMES[0], 0);
    assert_eq!(fft_frames % common_hop, 0);
    let centers = frequency_centers(fft_frames, sample_rate_hz);
    let mut taps = vec![Vec::<(usize, f64)>::new(); centers.len()];
    for bin in 0..fft_frames {
        let right_index = centers.partition_point(|center| *center <= bin) % centers.len();
        let left_index = (right_index + centers.len() - 1) % centers.len();
        let left = centers[left_index];
        let right = centers[right_index];
        let span = if right > left {
            right - left
        } else {
            fft_frames - left + right
        };
        let offset = if bin >= left {
            bin - left
        } else {
            fft_frames - left + bin
        };
        let phase = std::f64::consts::FRAC_PI_2 * offset as f64 / span as f64;
        let left_weight = phase.cos();
        let right_weight = phase.sin();
        if left_weight > f64::EPSILON {
            taps[left_index].push((bin, left_weight));
        }
        if right_weight > f64::EPSILON {
            taps[right_index].push((bin, right_weight));
        }
    }

    let bands = centers
        .into_iter()
        .zip(taps)
        .map(|(center, taps)| Band {
            center,
            scale: scale_for_bin(absolute_bin(center, fft_frames), fft_frames, sample_rate_hz),
            taps,
        })
        .collect::<Vec<_>>();
    let common_coefficients = fft_frames / common_hop;
    let mut frame_operator = vec![0.0_f64; fft_frames];
    let mut coverage = vec![0_usize; fft_frames];
    let mut owner_counts = [0_usize; 3];
    let mut local_collisions = 0;
    for band in &bands {
        owner_counts[band.scale.index()] += 1;
        let mut locals = vec![false; common_coefficients];
        for &(bin, weight) in &band.taps {
            frame_operator[bin] += weight * weight;
            coverage[bin] += 1;
            let local = local_coefficient(bin, band.center, common_coefficients, fft_frames);
            local_collisions += usize::from(locals[local]);
            locals[local] = true;
        }
    }
    let frame_min = frame_operator.iter().copied().fold(f64::INFINITY, f64::min);
    let frame_max = frame_operator.iter().copied().fold(0.0_f64, f64::max);
    let structural_failures = [
        coverage.iter().filter(|count| **count == 0).count(),
        bands
            .iter()
            .filter(|band| band.taps.len() > common_coefficients)
            .count(),
        local_collisions,
        usize::from(owner_counts.contains(&0)),
    ];
    let filter_hash = filter_hash(&bands, common_coefficients);
    let dual_hash = dual_hash(&bands, &frame_operator);

    Representation {
        fft_frames,
        bands,
        frame_operator,
        common_coefficients,
        common_hop,
        owner_counts,
        structural_failures,
        frame_values: [frame_min, frame_max, frame_max / frame_min],
        filter_hash,
        dual_hash,
    }
}

fn frequency_centers(fft_frames: usize, sample_rate_hz: usize) -> Vec<usize> {
    let nyquist = fft_frames / 2;
    let crossover_bins = CROSSOVER_HZ.map(|hz| (hz * fft_frames / sample_rate_hz).min(nyquist));
    let spacing = SUPPORT_FRAMES.map(|support| fft_frames / support);
    let mut positive = vec![0_usize];
    let mut center = 0;
    while center < nyquist {
        let scale = scale_for_bin(center, fft_frames, sample_rate_hz);
        let boundary = match scale {
            Scale::Long => crossover_bins[0],
            Scale::Middle => crossover_bins[1],
            Scale::Short => nyquist,
        };
        center = (center + spacing[scale.index()]).min(boundary);
        if positive.last().copied() != Some(center) {
            positive.push(center);
        }
    }
    let mut centers = positive.clone();
    centers.extend(
        positive[1..positive.len() - 1]
            .iter()
            .rev()
            .map(|bin| fft_frames - bin),
    );
    centers.sort_unstable();
    centers
}

fn scale_for_bin(bin: usize, fft_frames: usize, sample_rate_hz: usize) -> Scale {
    let frequency_hz = bin * sample_rate_hz / fft_frames;
    if frequency_hz < CROSSOVER_HZ[0] {
        Scale::Long
    } else if frequency_hz < CROSSOVER_HZ[1] {
        Scale::Middle
    } else {
        Scale::Short
    }
}

fn reconstruct_channel(input: &[f64], representation: &Representation) -> ChannelResult {
    let mut planner = FftPlanner::<f64>::new();
    let forward_full = planner.plan_fft_forward(representation.fft_frames);
    let inverse_full = planner.plan_fft_inverse(representation.fft_frames);
    let forward_band = planner.plan_fft_forward(representation.common_coefficients);
    let inverse_band = planner.plan_fft_inverse(representation.common_coefficients);
    let mut spectrum = (0..representation.fft_frames)
        .map(|index| {
            let logical = index as isize - PAD_FRAMES as isize;
            Complex64::new(reflected_sample(input, logical), 0.0)
        })
        .collect::<Vec<_>>();
    forward_full.process(&mut spectrum);

    let mut reconstructed = vec![Complex64::new(0.0, 0.0); representation.fft_frames];
    let mut non_finite_values = 0;
    for band in &representation.bands {
        let mut coefficients = vec![Complex64::new(0.0, 0.0); representation.common_coefficients];
        for &(bin, weight) in &band.taps {
            coefficients[local_coefficient(
                bin,
                band.center,
                representation.common_coefficients,
                representation.fft_frames,
            )] = spectrum[bin] * weight;
        }
        inverse_band.process(&mut coefficients);
        let scale = 1.0 / representation.common_coefficients as f64;
        for coefficient in &mut coefficients {
            *coefficient *= scale;
            non_finite_values += usize::from(!coefficient.re.is_finite());
            non_finite_values += usize::from(!coefficient.im.is_finite());
        }
        forward_band.process(&mut coefficients);
        for &(bin, weight) in &band.taps {
            let dual = weight / representation.frame_operator[bin];
            reconstructed[bin] += coefficients[local_coefficient(
                bin,
                band.center,
                representation.common_coefficients,
                representation.fft_frames,
            )] * dual;
        }
    }

    let mut conjugate_error = 0.0_f64;
    for bin in 0..representation.fft_frames {
        let mirror = if bin == 0 {
            0
        } else {
            representation.fft_frames - bin
        };
        conjugate_error =
            conjugate_error.max((reconstructed[bin] - reconstructed[mirror].conj()).norm());
    }
    inverse_full.process(&mut reconstructed);
    let inverse_scale = 1.0 / representation.fft_frames as f64;
    let crop = &reconstructed[PAD_FRAMES..PAD_FRAMES + input.len()];
    let mut samples = Vec::with_capacity(input.len());
    let mut errors = Vec::with_capacity(input.len());
    let mut imaginary_residue = 0.0_f64;
    let mut output_hash = HASH_OFFSET;
    for (source, output) in input.iter().zip(crop) {
        let output = *output * inverse_scale;
        non_finite_values += usize::from(!output.re.is_finite());
        non_finite_values += usize::from(!output.im.is_finite());
        imaginary_residue = imaginary_residue.max(output.im.abs());
        samples.push(output.re);
        errors.push((source - output.re).abs());
        hash_u64(&mut output_hash, output.re.to_bits());
        hash_u64(&mut output_hash, output.im.to_bits());
    }
    let peak_error = errors.iter().copied().fold(0.0_f64, f64::max);
    let rms_error =
        (errors.iter().map(|error| error * error).sum::<f64>() / input.len().max(1) as f64).sqrt();
    ChannelResult {
        samples,
        peak_error,
        rms_error,
        head_error: errors.first().copied().unwrap_or(0.0),
        tail_error: errors.last().copied().unwrap_or(0.0),
        imaginary_residue,
        conjugate_error,
        non_finite_values,
        output_hash,
    }
}

fn local_coefficient(
    bin: usize,
    center: usize,
    coefficient_count: usize,
    fft_frames: usize,
) -> usize {
    circular_delta(bin, center, fft_frames).rem_euclid(coefficient_count as isize) as usize
}

fn circular_delta(bin: usize, center: usize, fft_frames: usize) -> isize {
    let raw = bin as isize - center as isize;
    if raw > fft_frames as isize / 2 {
        raw - fft_frames as isize
    } else if raw < -(fft_frames as isize / 2) {
        raw + fft_frames as isize
    } else {
        raw
    }
}

fn absolute_bin(bin: usize, fft_frames: usize) -> usize {
    if bin <= fft_frames / 2 {
        bin
    } else {
        fft_frames - bin
    }
}

fn reflected_sample(input: &[f64], index: isize) -> f64 {
    if input.is_empty() {
        return 0.0;
    }
    let period = (input.len() * 2) as isize;
    let wrapped = index.rem_euclid(period) as usize;
    let source = if wrapped < input.len() {
        wrapped
    } else {
        input.len() * 2 - 1 - wrapped
    };
    input[source]
}

fn deterministic_probe() -> Vec<f64> {
    (0..SOURCE_FRAMES)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE_HZ as f64;
            (std::f64::consts::TAU * 55.0 * time).sin() * 0.23
                + (std::f64::consts::TAU * 440.0 * time + 0.31).sin() * 0.19
                + (std::f64::consts::TAU * 4_000.0 * time + 0.73).sin() * 0.11
                + ((index * 73 % 509) as f64 - 254.0) / 4_096.0
        })
        .collect()
}

fn impulse(index: usize) -> Vec<f64> {
    let mut result = vec![0.0; SOURCE_FRAMES];
    result[index] = 1.0;
    result
}

fn accumulate_errors(maximum: &mut [f64; 6], result: &ChannelResult) {
    let values = [
        result.peak_error,
        result.rms_error,
        result.head_error,
        result.tail_error,
        result.imaginary_residue,
        result.conjugate_error,
    ];
    for (slot, value) in maximum.iter_mut().zip(values) {
        *slot = slot.max(value);
    }
}

fn paired_max_error(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max)
}

fn filter_hash(bands: &[Band], common_coefficients: usize) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_usize(&mut hash, common_coefficients);
    for band in bands {
        hash_usize(&mut hash, band.center);
        hash_usize(&mut hash, band.scale.index());
        for &(bin, weight) in &band.taps {
            hash_usize(&mut hash, bin);
            hash_u64(&mut hash, weight.to_bits());
        }
    }
    hash
}

fn dual_hash(bands: &[Band], frame_operator: &[f64]) -> u64 {
    let mut hash = HASH_OFFSET;
    for band in bands {
        for &(bin, weight) in &band.taps {
            hash_usize(&mut hash, bin);
            hash_u64(&mut hash, (weight / frame_operator[bin]).to_bits());
        }
    }
    hash
}

fn review_hash(review: &StageAReview) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in review.geometry {
        hash_usize(&mut hash, value);
    }
    for value in review.support_frames {
        hash_usize(&mut hash, value);
    }
    for value in review.crossover_hz {
        hash_usize(&mut hash, value);
    }
    for value in review.owner_counts {
        hash_usize(&mut hash, value);
    }
    for value in review.structural_failures {
        hash_usize(&mut hash, value);
    }
    for value in review.frame_values {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in review.maximum_errors {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in review.relation_errors {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in review.mechanics_failures {
        hash_usize(&mut hash, value);
    }
    hash_usize(&mut hash, review.reflected_reads);
    hash_usize(&mut hash, review.non_finite_values);
    for value in &review.hashes[..3] {
        hash_u64(&mut hash, *value);
    }
    hash
}

fn hash_usize(hash: &mut u64, value: usize) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_u64(hash: &mut u64, value: u64) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency_adaptive_material_frame_stage_a_passes_identity_and_mechanics() {
        let review = stage_a_review();
        assert_eq!(review.geometry, [16_384, 8_192, 4_096, 32, 512]);
        assert_eq!(review.support_frames, [4_096, 2_048, 1_024]);
        assert_eq!(review.crossover_hz, [750, 6_000]);
        assert!(
            review.owner_counts.iter().all(|count| *count > 0),
            "{review:?}"
        );
        assert_eq!(review.structural_failures, [0; 4], "{review:?}");
        assert!(review.frame_values[0] > 0.0, "{review:?}");
        assert!(review.frame_values[1].is_finite(), "{review:?}");
        assert!(review.frame_values[2] <= 1.0 + 1.0e-12, "{review:?}");
        assert!(review.maximum_errors[0] <= 1.0e-12, "{review:?}");
        assert!(review.maximum_errors[1] <= 1.0e-13, "{review:?}");
        assert!(review.maximum_errors[2] <= 1.0e-12, "{review:?}");
        assert!(review.maximum_errors[3] <= 1.0e-12, "{review:?}");
        assert!(review.maximum_errors[4] <= 1.0e-12, "{review:?}");
        assert!(review.maximum_errors[5] <= 1.0e-12, "{review:?}");
        assert!(
            review.relation_errors.iter().all(|error| *error <= 1.0e-12),
            "{review:?}"
        );
        assert_eq!(review.mechanics_failures, [0; 4], "{review:?}");
        assert_eq!(review.reflected_reads, 8_192);
        assert_eq!(review.non_finite_values, 0, "{review:?}");
        assert!(review.hashes.iter().all(|hash| *hash != 0), "{review:?}");
        eprintln!("frequency_adaptive_material_frame_stage_a {review:?}");
    }

    #[test]
    fn frequency_adaptive_material_frame_stage_a_is_deterministic() {
        let first = stage_a_review();
        let repeated = stage_a_review();
        assert_eq!(first, repeated);
    }
}
