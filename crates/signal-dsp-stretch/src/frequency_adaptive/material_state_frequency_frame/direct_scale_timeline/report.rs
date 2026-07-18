use super::{geometry::*, render::*, *};

pub(super) fn stage_review() -> StageReview {
    let rates = PROOF_RATES.into_iter().map(rate_review).collect::<Vec<_>>();
    let mut review = StageReview {
        rates,
        overflow_failures: overflow_failures(),
        unsupported_failures: unsupported_failures(),
        hash: 0,
    };
    review.hash = review_hash(&review);
    review
}

fn rate_review(sample_rate: usize) -> RateReview {
    let mut prepared =
        prepare(sample_rate, CHANNEL_CAPACITY, 1.0, false).expect("Rule 31Z proof geometry");
    let length = 12 * prepared.hop + 17;
    let source = deterministic_probe(length, sample_rate);
    let scale_reviews = Scale::ALL
        .into_iter()
        .map(|scale| scale_review(&mut prepared, &source, scale))
        .collect::<Vec<_>>();
    let masked_diagnostics = masked_diagnostics(&mut prepared, length);
    let unity_failures = unity_failures(&source);
    let schedule_failures = schedule_failures(sample_rate, prepared.hop);
    let structural_failures = storage_failures(&prepared)
        + usize::from(prepared.owned_bins.iter().sum::<usize>() > 631)
        + usize::from(prepared.planner_scratch > prepared.memory.scratch_complex)
        + scale_reviews
            .iter()
            .filter(|review| review.scale == Scale::Short && sample_rate == 8_000)
            .map(|review| usize::from(review.owned_bins != 0))
            .sum::<usize>()
        + scale_reviews
            .iter()
            .map(|review| {
                usize::from(review.work != expected_work(length, review.length, prepared.hop))
            })
            .sum::<usize>();
    let work = scale_reviews
        .iter()
        .fold(WorkCounts::default(), |mut total, review| {
            total.forward_transforms += review.work.forward_transforms;
            total.inverse_transforms += review.work.inverse_transforms;
            total.window_visits += review.work.window_visits;
            total.coefficient_visits += review.work.coefficient_visits;
            total.conjugate_visits += review.work.conjugate_visits;
            total
        });
    let mut hash = HASH_OFFSET;
    hash_usize(&mut hash, sample_rate);
    hash_usize(&mut hash, prepared.hop);
    for value in prepared.lengths.into_iter().chain(prepared.owned_bins) {
        hash_usize(&mut hash, value);
    }
    hash_memory(&mut hash, prepared.memory);
    hash_usize(&mut hash, prepared.planner_scratch);
    hash_work(&mut hash, work);
    for review in &scale_reviews {
        hash_u64(&mut hash, review.hash);
    }
    for row in &masked_diagnostics {
        hash_u64(&mut hash, row.hash);
    }
    for value in [unity_failures, schedule_failures, structural_failures] {
        hash_usize(&mut hash, value);
    }

    RateReview {
        sample_rate,
        hop: prepared.hop,
        lengths: prepared.lengths,
        owned_bins: prepared.owned_bins,
        memory: prepared.memory,
        planner_scratch: prepared.planner_scratch,
        scale_reviews,
        masked_diagnostics,
        unity_failures,
        schedule_failures,
        structural_failures,
        work,
        hash,
    }
}

fn scale_review(prepared: &mut Prepared, source: &[f64], scale: Scale) -> ScaleReview {
    let mut output = vec![0.0; source.len()];
    let metrics = prepared.render_scale(source, &mut output, 0, scale, false);
    let reconstruction_error = paired_max_error(source, &output);
    let mut hash = HASH_OFFSET;
    for sample in &output {
        hash_u64(&mut hash, sample.to_bits());
    }
    hash_work(&mut hash, metrics.work);
    ScaleReview {
        scale,
        length: prepared.lengths[scale.index()],
        owned_bins: prepared.owned_bins[scale.index()],
        partition_error: partition_error(&prepared.plans[scale.index()].window, prepared.hop),
        reconstruction_error,
        imaginary_residue: metrics.imaginary_residue,
        conjugate_error: metrics.conjugate_error,
        non_finite_values: metrics.non_finite_values,
        work: metrics.work,
        hash,
    }
}

fn masked_diagnostics(prepared: &mut Prepared, length: usize) -> Vec<MaskedDiagnostic> {
    let sample_rate = prepared.sample_rate;
    let mut controls = vec![
        ("silence", vec![0.0; length]),
        ("impulse", impulse(length, length / 2)),
        ("noise", deterministic_noise(length)),
        ("tone-375", tone(length, sample_rate, 375.0)),
        ("tone-750", tone(length, sample_rate, 750.0)),
        ("tone-3000", tone(length, sample_rate, 3_000.0)),
    ];
    if sample_rate >= 44_100 {
        controls.push(("tone-6000", tone(length, sample_rate, 6_000.0)));
        controls.push(("tone-9000", tone(length, sample_rate, 9_000.0)));
    }
    controls
        .into_iter()
        .map(|(name, source)| masked_diagnostic(prepared, name, &source))
        .collect()
}

fn masked_diagnostic(
    prepared: &mut Prepared,
    control: &'static str,
    source: &[f64],
) -> MaskedDiagnostic {
    let mut output = vec![0.0; source.len()];
    let mut scale_output = vec![0.0; source.len()];
    let mut non_finite_values = 0;
    for scale in Scale::ALL {
        let metrics = prepared.render_scale(source, &mut scale_output, 0, scale, true);
        non_finite_values += metrics.non_finite_values;
        for (target, value) in output.iter_mut().zip(&scale_output) {
            *target += value;
        }
    }
    non_finite_values += output.iter().filter(|sample| !sample.is_finite()).count();
    let residual = source
        .iter()
        .zip(&output)
        .map(|(source, output)| source - output)
        .collect::<Vec<_>>();
    let input_rms = rms(source);
    let output_rms = rms(&output);
    let gain_delta_db = if input_rms == 0.0 && output_rms == 0.0 {
        0.0
    } else {
        20.0 * (output_rms / input_rms).log10()
    };
    let boundary_error = residual
        .first()
        .copied()
        .unwrap_or(0.0)
        .abs()
        .max(residual.last().copied().unwrap_or(0.0).abs());
    let mut hash = HASH_OFFSET;
    for sample in &output {
        hash_u64(&mut hash, sample.to_bits());
    }
    MaskedDiagnostic {
        control,
        peak_residual: maximum_abs(&residual),
        rms_residual: rms(&residual),
        gain_delta_db,
        timing_frames: best_lag(source, &output, prepared.hop),
        boundary_error,
        non_finite_values,
        hash,
    }
}

fn storage_failures(prepared: &Prepared) -> usize {
    usize::from(prepared.source_ring.len() != prepared.memory.source_samples)
        + usize::from(prepared.pending.len() != prepared.memory.pending_complex)
        + usize::from(prepared.guidance.len() != prepared.memory.guidance_values)
        + usize::from(prepared.phase.len() != prepared.memory.phase_values)
        + usize::from(prepared.regions.len() != prepared.memory.region_records)
        + usize::from(prepared.output_ring.len() != prepared.memory.output_samples)
        + usize::from(prepared.transform.len() != prepared.memory.transform_complex)
        + usize::from(prepared.scratch.len() != prepared.memory.scratch_complex)
}

fn unity_failures(source: &[f64]) -> usize {
    let mut output = vec![f64::NAN; source.len()];
    unity_bypass(source, &mut output);
    source
        .iter()
        .zip(&output)
        .filter(|(source, output)| source.to_bits() != output.to_bits())
        .count()
}

fn schedule_failures(sample_rate: usize, hop: usize) -> usize {
    let source = 17 * hop + 13;
    [0.25, 0.75, 1.5, 4.0]
        .into_iter()
        .map(|ratio| {
            let target = (source as f64 * ratio).round() as usize;
            let Some((first, last)) = synthesis_tick_range(target, hop) else {
                return 1;
            };
            let mut failures = usize::from(first != -3)
                + usize::from(last != ((target + 4 * hop - 1) / hop) as isize);
            let mut previous = None;
            for tick in first - 9..=last + 9 {
                let center =
                    source_center(tick, hop, source, target).expect("nonempty Rule 31Z schedule");
                let expected =
                    (tick as f64 * hop as f64 * source as f64 / target as f64).round() as isize;
                failures += usize::from(center != expected);
                if let Some(previous) = previous {
                    failures += usize::from(center <= previous);
                }
                previous = Some(center);
            }
            failures
        })
        .sum::<usize>()
        + usize::from(sample_rate / 100 != hop)
}

pub(super) fn overflow_failures() -> usize {
    let fields = [
        (CapacityExceeded::SourceSamples, 0),
        (CapacityExceeded::PendingComplex, 1),
        (CapacityExceeded::GuidanceValues, 2),
        (CapacityExceeded::PhaseValues, 3),
        (CapacityExceeded::RegionRecords, 4),
        (CapacityExceeded::OutputSamples, 5),
        (CapacityExceeded::TransformComplex, 6),
        (CapacityExceeded::ScratchComplex, 7),
    ];
    let mut failures = usize::from(validate_capacity(CAPACITY).is_err());
    for (expected, field) in fields {
        let mut request = CAPACITY;
        match field {
            0 => request.source_samples += 1,
            1 => request.pending_complex += 1,
            2 => request.guidance_values += 1,
            3 => request.phase_values += 1,
            4 => request.region_records += 1,
            5 => request.output_samples += 1,
            6 => request.transform_complex += 1,
            7 => request.scratch_complex += 1,
            _ => unreachable!(),
        }
        failures +=
            usize::from(validate_capacity(request) != Err(PrepareError::Capacity(expected)));
    }
    failures
}

pub(super) fn unsupported_failures() -> usize {
    let cases = [
        (
            prepare(96_000, 2, 1.0, false).err(),
            PrepareError::Unsupported(UnsupportedGeometry::SampleRate),
        ),
        (
            prepare(48_000, 0, 1.0, false).err(),
            PrepareError::Unsupported(UnsupportedGeometry::ChannelCount),
        ),
        (
            prepare(48_000, 3, 1.0, false).err(),
            PrepareError::Unsupported(UnsupportedGeometry::ChannelCount),
        ),
        (
            prepare(48_000, 2, f64::NAN, false).err(),
            PrepareError::Unsupported(UnsupportedGeometry::Ratio),
        ),
        (
            prepare(48_000, 2, 0.24, false).err(),
            PrepareError::Unsupported(UnsupportedGeometry::Ratio),
        ),
        (
            prepare(48_000, 2, 4.01, false).err(),
            PrepareError::Unsupported(UnsupportedGeometry::Ratio),
        ),
        (
            prepare(48_000, 2, 1.0, true).err(),
            PrepareError::Unsupported(UnsupportedGeometry::Discontinuity),
        ),
        (
            source_center(0, 480, 0, 1).err(),
            PrepareError::Unsupported(UnsupportedGeometry::TargetLength),
        ),
    ];
    cases
        .into_iter()
        .filter(|(actual, expected)| *actual != Some(*expected))
        .count()
}

fn review_hash(review: &StageReview) -> u64 {
    let mut hash = HASH_OFFSET;
    for rate in &review.rates {
        hash_u64(&mut hash, rate.hash);
    }
    hash_usize(&mut hash, review.overflow_failures);
    hash_usize(&mut hash, review.unsupported_failures);
    hash
}

fn deterministic_probe(length: usize, sample_rate: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            let time = index as f64 / sample_rate as f64;
            (std::f64::consts::TAU * 375.0 * time + 0.11).sin() * 0.23
                + (std::f64::consts::TAU * 3_000.0 * time + 0.37).sin() * 0.19
                + (std::f64::consts::TAU * 7_500.0 * time + 0.73).sin() * 0.11
                + ((index * 73 % 509) as f64 - 254.0) / 8_192.0
        })
        .collect()
}

fn deterministic_noise(length: usize) -> Vec<f64> {
    let mut state = 0x1234_5678_u64;
    (0..length)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            ((state >> 32) as u32 as f64 / u32::MAX as f64 - 0.5) * 0.4
        })
        .collect()
}

fn tone(length: usize, sample_rate: usize, frequency: f64) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (std::f64::consts::TAU * frequency * index as f64 / sample_rate as f64).sin() * 0.25
        })
        .collect()
}

fn impulse(length: usize, at: usize) -> Vec<f64> {
    let mut result = vec![0.0; length];
    result[at] = 1.0;
    result
}

fn frame_count(target: usize, length: usize, hop: usize) -> usize {
    let (first, last) = {
        let half_hops = length / (2 * hop);
        (
            -(half_hops as isize) + 1,
            ((target + length / 2 - 1) / hop) as isize,
        )
    };
    (last - first + 1) as usize
}

fn expected_work(target: usize, length: usize, hop: usize) -> WorkCounts {
    let frames = frame_count(target, length, hop);
    WorkCounts {
        forward_transforms: frames,
        inverse_transforms: frames,
        window_visits: 2 * frames * length,
        coefficient_visits: frames * length,
        conjugate_visits: frames * (length / 2 + 1),
    }
}

fn paired_max_error(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0_f64, f64::max)
}

fn maximum_abs(values: &[f64]) -> f64 {
    values
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max)
}

fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
}

fn best_lag(source: &[f64], output: &[f64], radius: usize) -> isize {
    if maximum_abs(source) == 0.0 {
        return 0;
    }
    (-(radius as isize)..=radius as isize)
        .map(|lag| {
            let correlation = source
                .iter()
                .enumerate()
                .filter_map(|(index, source)| {
                    let shifted = index as isize + lag;
                    (0..output.len() as isize)
                        .contains(&shifted)
                        .then(|| source * output[shifted as usize])
                })
                .sum::<f64>();
            (lag, correlation)
        })
        .max_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| right.0.abs().cmp(&left.0.abs()))
                .then_with(|| right.0.cmp(&left.0))
        })
        .map(|(lag, _)| lag)
        .unwrap_or(0)
}
