        fn owner_y01() -> Result<(), String> {
            let owner = "Y01";
            let mut errors = Vec::new();
            let mut row_index = 0;
            for (source_index, source) in SourceKind::ALL.into_iter().enumerate() {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for (ratio_index, ratio) in RATIOS.into_iter().enumerate() {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let mut row_errors = Vec::new();
                    if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                    if !no_dropout(&output, source, ratio) { row_errors.push("dropout".into()); }
                    let (start, end) = if source == SourceKind::Impulse { (0, output.len()) } else { mapped_support(source, ratio) };
                    let crest = difference_crest_db(&output[start..end]);
                    if !crest.is_finite() { row_errors.push("finite-difference-crest".into()); }
                    let output_hash = hash_f32(&output);
                    let row_id = format!("{}-{ratio}x", source.id());
                    write_receipt(Receipt {
                        owner, row_index, row_id: &row_id,
                        status: if row_errors.is_empty() { "pass" } else { "fail" },
                        render_count: 1, output_frames: output.len(), input_hash: &input_hash,
                        output_hash: &output_hash,
                        assertions: vec!["exact-length-finite-endpoints-max8".into(), "no-H-dropout".into()],
                        diagnostics: vec![format!("difference_crest_db={crest:.9}"), format!("reference_delta_db={:.9}", crest - DIFFERENCE_CREST_REFERENCE[source_index][ratio_index])],
                    });
                    errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                    row_index += 1;
                }
            }
            finish_owner(owner, errors, 30, 30)
        }

        fn pitch_spectrum(output: &[f32], ratio: usize) -> (Vec<Complex64>, usize) {
            let support_start = 24_000 * ratio;
            let support_end = 72_000 * ratio;
            let quarter = (support_end - support_start) / 4;
            let measured = &output[support_start + quarter..support_end - quarter];
            let padded_len = (measured.len() * 8).next_power_of_two();
            let mut spectrum = vec![Complex64::new(0.0, 0.0); padded_len];
            for (index, sample) in measured.iter().enumerate() {
                let window = 0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / measured.len() as f64).cos();
                spectrum[index].re = *sample as f64 * window;
            }
            FftPlanner::<f64>::new().plan_fft_forward(padded_len).process(&mut spectrum);
            (spectrum, padded_len)
        }

        fn estimate_pitch(spectrum: &[Complex64], fft_len: usize, expected: f64) -> f64 {
            let frequency_per_bin = SAMPLE_RATE as f64 / fft_len as f64;
            let first = ((expected - 4.0) / frequency_per_bin).ceil().max(1.0) as usize;
            let last = ((expected + 4.0) / frequency_per_bin).floor() as usize;
            let peak = (first..=last).max_by(|left, right| spectrum[*left].norm_sqr().total_cmp(&spectrum[*right].norm_sqr())).unwrap();
            let left = spectrum[peak - 1].norm().max(f64::MIN_POSITIVE).ln();
            let center = spectrum[peak].norm().max(f64::MIN_POSITIVE).ln();
            let right = spectrum[peak + 1].norm().max(f64::MIN_POSITIVE).ln();
            let denominator = left - 2.0 * center + right;
            let offset = if denominator == 0.0 { 0.0 } else { 0.5 * (left - right) / denominator };
            (peak as f64 + offset) * frequency_per_bin
        }

        fn owner_y02() -> Result<(), String> {
            let owner = "Y02";
            let cases: [(SourceKind, &[f64]); 3] = [
                (SourceKind::LowTone, &[110.0]),
                (SourceKind::MidTone, &[440.0]),
                (SourceKind::Chord, &[110.0, 164.813_778, 220.0, 277.182_631, 329.627_557]),
            ];
            let mut errors = Vec::new();
            let mut row_index = 0;
            for (source, frequencies) in cases {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for ratio in RATIOS {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let output_hash = hash_f32(&output);
                    let (spectrum, fft_len) = pitch_spectrum(&output, ratio);
                    for (frequency_index, frequency) in frequencies.iter().enumerate() {
                        let estimate = estimate_pitch(&spectrum, fft_len, *frequency);
                        let error_hz = (estimate - frequency).abs();
                        let mut row_errors = Vec::new();
                        if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                        if !estimate.is_finite() || !error_hz.is_finite() { row_errors.push("finite-pitch".into()); }
                        let row_id = format!("{}-{ratio}x-{frequency:.6}hz", source.id());
                        write_receipt(Receipt {
                            owner, row_index, row_id: &row_id,
                            status: if row_errors.is_empty() { "pass" } else { "fail" },
                            render_count: usize::from(frequency_index == 0), output_frames: output.len(),
                            input_hash: &input_hash, output_hash: &output_hash,
                            assertions: vec!["finite-pitch-diagnostic".into()],
                            diagnostics: vec![format!("estimated_hz={estimate:.9}"), format!("error_hz={error_hz:.9}")],
                        });
                        errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                        row_index += 1;
                    }
                }
            }
            finish_owner(owner, errors, 21, 9)
        }

        fn shortest_energy_width(samples: &[f32], fraction: f64) -> usize {
            let total = samples.iter().map(|sample| (*sample as f64).powi(2)).sum::<f64>();
            if total == 0.0 { return 0; }
            let target = total * fraction;
            let mut best = samples.len();
            let mut end = 0;
            let mut accumulated = 0.0;
            for start in 0..samples.len() {
                while end < samples.len() && accumulated < target {
                    accumulated += (samples[end] as f64).powi(2);
                    end += 1;
                }
                if accumulated >= target { best = best.min(end - start); }
                if start < end { accumulated -= (samples[start] as f64).powi(2); }
            }
            best
        }

        fn energy_centroid(samples: &[f32]) -> f64 {
            let mut weighted = 0.0;
            let mut total = 0.0;
            for (index, sample) in samples.iter().enumerate() {
                let energy = (*sample as f64).powi(2);
                weighted += index as f64 * energy;
                total += energy;
            }
            if total == 0.0 { 0.0 } else { weighted / total }
        }

        fn active_regions(samples: &[f32]) -> (usize, Option<f64>) {
            let windows = samples.windows(480).step_by(240).map(rms).collect::<Vec<_>>();
            let Some((peak_index, peak)) = windows.iter().copied().enumerate().max_by(|left, right| left.1.total_cmp(&right.1)) else { return (0, None); };
            if peak == 0.0 { return (0, None); }
            let threshold = peak * 10.0_f64.powf(-30.0 / 20.0);
            let active = windows.iter().enumerate().filter_map(|(index, value)| (*value >= threshold).then_some(index * 240)).collect::<Vec<_>>();
            let mut regions: Vec<(usize, usize, f64)> = Vec::new();
            for start in active {
                let value = windows[start / 240];
                if regions.last().is_none_or(|(_, last, _)| start.saturating_sub(*last) >= 2_400) {
                    regions.push((start, start, value));
                } else {
                    let region = regions.last_mut().unwrap();
                    region.1 = start;
                    region.2 = region.2.max(value);
                }
            }
            let peak_start = peak_index * 240;
            let primary_region = regions.iter().position(|(start, last, _)| *start <= peak_start && peak_start <= *last).unwrap_or(0);
            let secondary = regions.iter().enumerate().filter(|(index, _)| *index != primary_region).map(|(_, (_, _, value))| 20.0 * (value / peak).log10()).max_by(f64::total_cmp);
            (regions.len(), secondary)
        }

        fn owner_y03() -> Result<(), String> {
            let owner = "Y03";
            let mut errors = Vec::new();
            let mut row_index = 0;
            for source in [SourceKind::Impulse, SourceKind::ImpulseTrain] {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for ratio in RATIOS {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let mut row_errors = Vec::new();
                    if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                    let width = shortest_energy_width(&output, 0.95);
                    let centroid = energy_centroid(&output);
                    let expected = (48_000.5 * output.len() as f64 / SYNTHETIC_SOURCE_FRAMES as f64) - 0.5;
                    let centroid_error = (centroid - expected).abs();
                    let (regions, secondary) = active_regions(&output);
                    if !centroid.is_finite() || !centroid_error.is_finite() { row_errors.push("finite-impulse-diagnostic".into()); }
                    let output_hash = hash_f32(&output);
                    let row_id = format!("{}-{ratio}x", source.id());
                    write_receipt(Receipt {
                        owner, row_index, row_id: &row_id,
                        status: if row_errors.is_empty() { "pass" } else { "fail" }, render_count: 1,
                        output_frames: output.len(), input_hash: &input_hash, output_hash: &output_hash,
                        assertions: vec!["finite-impulse-diagnostics".into()],
                        diagnostics: vec![format!("width95={width}"), format!("centroid_error={centroid_error:.9}"), format!("active_regions={regions}"), format!("secondary_db={}", secondary.map_or("null".into(), |value| format!("{value:.9}")))],
                    });
                    errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                    row_index += 1;
                }
            }
            finish_owner(owner, errors, 6, 6)
        }

        fn linear_autocorrelation_max(samples: &[f32]) -> f64 {
            let mean = samples.iter().map(|sample| *sample as f64).sum::<f64>() / samples.len() as f64;
            let fft_len = (samples.len() * 2 - 1).next_power_of_two();
            let mut spectrum = vec![Complex64::new(0.0, 0.0); fft_len];
            for (bin, sample) in spectrum.iter_mut().zip(samples) { bin.re = *sample as f64 - mean; }
            let mut planner = FftPlanner::<f64>::new();
            planner.plan_fft_forward(fft_len).process(&mut spectrum);
            for bin in &mut spectrum { *bin = Complex64::new(bin.norm_sqr(), 0.0); }
            planner.plan_fft_inverse(fft_len).process(&mut spectrum);
            let lag_zero = spectrum[0].re;
            (960..=48_000).map(|lag| (spectrum[lag].re / lag_zero).abs()).fold(0.0, f64::max)
        }

        fn block_rms_cv(samples: &[f32]) -> f64 {
            let values = samples.windows(2_400).step_by(1_200).map(rms).collect::<Vec<_>>();
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / values.len() as f64;
            variance.sqrt() / mean
        }

        fn owner_y04() -> Result<(), String> {
            const AUTOCORRELATION: [f64; 3] = [0.017218163, 0.017727693, 0.017090511];
            const UNIFORM_CV: [f64; 3] = [0.387747959, 0.460013282, 0.492971808];
            const MID_CV: [f64; 3] = [0.617268653, 0.679139581, 0.708639858];
            let owner = "Y04";
            let mut errors = Vec::new();
            let mut row_index = 0;
            for source in [SourceKind::UniformNoise, SourceKind::MidTone, SourceKind::SilenceGap] {
                let input = source.generate();
                let input_hash = hash_f32(&input);
                for (ratio_index, ratio) in RATIOS.into_iter().enumerate() {
                    let output = render(mono_request(&input, ratio, ADMISSION_SEED)).map_err(|error| format!("{error:?}"))?;
                    let (start, end) = mapped_support(source, ratio);
                    let mut row_errors = Vec::new();
                    if let Err(error) = hard_integrity(&output, ratio, 1) { row_errors.push(error); }
                    let diagnostic = match source {
                        SourceKind::UniformNoise => {
                            let autocorrelation = linear_autocorrelation_max(&output[start..end]);
                            let cv = block_rms_cv(&output[start..end]);
                            if autocorrelation > AUTOCORRELATION[ratio_index] + 0.05 { row_errors.push("autocorrelation".into()); }
                            if cv > UNIFORM_CV[ratio_index] + 0.05 { row_errors.push("uniform-cv".into()); }
                            vec![format!("autocorrelation={autocorrelation:.9}"), format!("block_rms_cv={cv:.9}")]
                        }
                        SourceKind::MidTone => {
                            let cv = block_rms_cv(&output[start..end]);
                            if cv > MID_CV[ratio_index] + 0.05 { row_errors.push("mid-cv".into()); }
                            vec![format!("block_rms_cv={cv:.9}")]
                        }
                        SourceKind::SilenceGap => {
                            let gap = rms(&output[42_000 * ratio..54_000 * ratio]);
                            let active = rms(&output[start..end]);
                            let gap_db = 20.0 * (gap / active).log10();
                            if !gap_db.is_finite() { row_errors.push("finite-gap-rms".into()); }
                            vec![format!("gap_relative_db={gap_db:.9}")]
                        }
                        _ => unreachable!(),
                    };
                    let output_hash = hash_f32(&output);
                    let row_id = format!("{}-{ratio}x", source.id());
                    write_receipt(Receipt {
                        owner, row_index, row_id: &row_id,
                        status: if row_errors.is_empty() { "pass" } else { "fail" }, render_count: 1,
                        output_frames: output.len(), input_hash: &input_hash, output_hash: &output_hash,
                        assertions: vec!["periodicity-modulation-gap".into()], diagnostics: diagnostic,
                    });
                    errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
                    row_index += 1;
                }
            }
            finish_owner(owner, errors, 9, 9)
        }

        #[derive(Clone, Copy)]
        enum StereoFixture { Duplicate, Base, CommonNegated, Mixed, SwappedMixed, AntiPhase, DelayedPad }

        impl StereoFixture {
            fn id(self) -> &'static str {
                match self { Self::Duplicate => "duplicate", Self::Base => "base", Self::CommonNegated => "common-negated", Self::Mixed => "mixed", Self::SwappedMixed => "swapped-mixed", Self::AntiPhase => "anti-phase", Self::DelayedPad => "delayed-pad" }
            }

            fn generate(self) -> Vec<f32> {
                let mid = SourceKind::MidTone.generate();
                let pad = SourceKind::HarmonicPad.generate();
                let chord = SourceKind::Chord.generate();
                let noise = SourceKind::UniformNoise.generate();
                let mut output = Vec::with_capacity(SYNTHETIC_SOURCE_FRAMES * 2);
                for frame in 0..SYNTHETIC_SOURCE_FRAMES {
                    let delayed_pad = frame.checked_sub(37).map_or(0.0, |index| pad[index]);
                    let delayed_chord = frame.checked_sub(37).map_or(0.0, |index| chord[index]);
                    let pair = match self {
                        Self::Duplicate | Self::Base => (mid[frame], mid[frame]),
                        Self::CommonNegated => (-mid[frame], -mid[frame]),
                        Self::AntiPhase => (mid[frame], -mid[frame]),
                        Self::DelayedPad => (pad[frame], delayed_pad),
                        Self::Mixed => (chord[frame] + 0.2 * noise[frame], delayed_chord - 0.2 * noise[frame]),
                        Self::SwappedMixed => (delayed_chord - 0.2 * noise[frame], chord[frame] + 0.2 * noise[frame]),
                    };
                    output.extend_from_slice(&[pair.0, pair.1]);
                }
                output
            }
        }

        fn channel_pair(interleaved: &[f32]) -> (Vec<f32>, Vec<f32>) {
            let mut left = Vec::with_capacity(interleaved.len() / 2);
            let mut right = Vec::with_capacity(interleaved.len() / 2);
            for frame in interleaved.chunks_exact(2) { left.push(frame[0]); right.push(frame[1]); }
            (left, right)
        }

        fn band_energies(channel: &[f32]) -> [f64; 5] {
            let len = channel.len();
            let mut spectrum = channel.iter().map(|sample| Complex64::new(*sample as f64, 0.0)).collect::<Vec<_>>();
            FftPlanner::<f64>::new().plan_fft_forward(len).process(&mut spectrum);
            let mut bands = [0.0_f64; 5];
            for (bin, value) in spectrum.iter().enumerate().take(len / 2 + 1) {
                let frequency = bin as f64 * SAMPLE_RATE as f64 / len as f64;
                let weight = if bin == 0 || (len.is_multiple_of(2) && bin == len / 2) { 1.0 } else { 2.0 };
                let energy = weight * value.norm_sqr();
                bands[0] += energy;
                if frequency <= 80.0 { bands[1] += energy; }
                let band = if frequency < 250.0 { 2 } else if frequency < 1_500.0 { 3 } else { 4 };
                bands[band] += energy;
            }
            bands
        }

        fn stereo_metrics(interleaved: &[f32]) -> ([f64; 4], f64, f64) {
            let (left, right) = channel_pair(interleaved);
            let left_energy = band_energies(&left);
            let right_energy = band_energies(&right);
            let balances = std::array::from_fn(|index| {
                let energy_index = [0, 2, 3, 4][index];
                if left_energy[energy_index] == 0.0 && right_energy[energy_index] == 0.0 { 0.0 }
                else { 10.0 * (right_energy[energy_index] / left_energy[energy_index]).log10() }
            });
            let low_fraction = (left_energy[1] + right_energy[1]) / (left_energy[0] + right_energy[0]);
            let side_energy = interleaved.chunks_exact(2).map(|frame| ((frame[0] as f64 - frame[1] as f64) * 0.5).powi(2)).sum::<f64>();
            (balances, low_fraction, side_energy)
        }

        fn time_relation_residual(fixture: StereoFixture, output: &[f32], _space: f32) -> f64 {
            output.chunks_exact(2).map(|frame| match fixture {
                StereoFixture::Duplicate | StereoFixture::Base | StereoFixture::CommonNegated => (frame[0] - frame[1]).abs() as f64,
                StereoFixture::AntiPhase => (frame[0] + frame[1]).abs() as f64,
                _ => 0.0,
            }).fold(0.0, f64::max)
        }

        fn owner_y05() -> Result<(), String> {
            let owner = "Y05";
            let mut rows = Vec::new();
            for ratio in RATIOS { for space in [0.0_f32, 0.5, 1.0] { rows.push((StereoFixture::Duplicate, ratio, space)); } }
            rows.extend([(StereoFixture::Base, 8, 0.5), (StereoFixture::CommonNegated, 8, 0.5), (StereoFixture::Mixed, 8, 0.5), (StereoFixture::SwappedMixed, 8, 0.5)]);
            for space in [0.0_f32, 0.5, 1.0] { rows.push((StereoFixture::AntiPhase, 8, space)); }
            for ratio in RATIOS { for space in [0.0_f32, 1.0] { rows.push((StereoFixture::DelayedPad, ratio, space)); } }
            assert_eq!(rows.len(), 22);
            let mut errors = Vec::new();
            let mut duplicate_balances: Vec<(usize, f32, [f64; 4])> = Vec::new();
            let mut anti_balances: Vec<(f32, [f64; 4])> = Vec::new();
            for (row_index, (fixture, ratio, space)) in rows.into_iter().enumerate() {
                let input = fixture.generate();
                let input_hash = hash_f32(&input);
                let (source_balances, source_low_fraction, source_side_energy) = stereo_metrics(&input);
                let output = render(stereo_request(&input, ratio, ADMISSION_SEED, space)).map_err(|error| format!("{error:?}"))?;
                let (candidate_balances, candidate_low_fraction, candidate_side_energy) = stereo_metrics(&output);
                let relation_residual = time_relation_residual(fixture, &output, space);
                let mut row_errors = Vec::new();
                if let Err(error) = hard_integrity(&output, ratio, 2) { row_errors.push(error); }
                for band in 0..4 {
                    if !source_balances[band].is_finite() || !candidate_balances[band].is_finite() { row_errors.push(format!("finite-balance-{band}")); continue; }
                    if (candidate_balances[band] - source_balances[band]).abs() > 0.75 { row_errors.push(format!("balance-error-{band}")); }
                    if source_balances[band].abs() >= 0.5 && source_balances[band].signum() != candidate_balances[band].signum() { row_errors.push(format!("dominance-reversal-{band}")); }
                }
                if space == 0.0 && matches!(fixture, StereoFixture::Duplicate | StereoFixture::Base | StereoFixture::CommonNegated | StereoFixture::AntiPhase) && relation_residual > 1.0e-6 { row_errors.push("source-relation".into()); }
                if !source_low_fraction.is_finite() || !candidate_low_fraction.is_finite() || !source_side_energy.is_finite() || !candidate_side_energy.is_finite() { row_errors.push("finite-stereo-diagnostic".into()); }
                if matches!(fixture, StereoFixture::Duplicate) { duplicate_balances.push((ratio, space, candidate_balances)); }
                if matches!(fixture, StereoFixture::AntiPhase) { anti_balances.push((space, candidate_balances)); }
                let output_hash = hash_f32(&output);
                let row_id = format!("{}-{ratio}x-space-{space:.1}", fixture.id());
                write_receipt(Receipt {
                    owner, row_index, row_id: &row_id,
                    status: if row_errors.is_empty() { "pass" } else { "fail" }, render_count: 1,
                    output_frames: output.len() / 2, input_hash: &input_hash, output_hash: &output_hash,
                    assertions: vec!["stereo-integrity".into(), "balance-bands".into(), "dominance".into()],
                    diagnostics: vec![format!("source_balance_db={source_balances:?}"), format!("candidate_balance_db={candidate_balances:?}"), format!("relation_residual={relation_residual:.12}"), format!("source_low_fraction={source_low_fraction:.12}"), format!("candidate_low_fraction={candidate_low_fraction:.12}"), format!("source_side_energy={source_side_energy:.12}"), format!("candidate_side_energy={candidate_side_energy:.12}")],
                });
                errors.extend(row_errors.into_iter().map(|error| format!("{row_id}:{error}")));
            }
            for ratio in RATIOS {
                let trio = duplicate_balances.iter().filter(|(candidate_ratio, _, _)| *candidate_ratio == ratio).take(3).collect::<Vec<_>>();
                for band in 0..4 {
                    let minimum = trio.iter().map(|(_, _, value)| value[band]).fold(f64::INFINITY, f64::min);
                    let maximum = trio.iter().map(|(_, _, value)| value[band]).fold(f64::NEG_INFINITY, f64::max);
                    if maximum - minimum > 0.5 { errors.push(format!("duplicate-{ratio}x:balance-spread-{band}")); }
                }
            }
            for band in 0..4 {
                let minimum = anti_balances.iter().map(|(_, value)| value[band]).fold(f64::INFINITY, f64::min);
                let maximum = anti_balances.iter().map(|(_, value)| value[band]).fold(f64::NEG_INFINITY, f64::max);
                if maximum - minimum > 0.5 { errors.push(format!("anti-phase:balance-spread-{band}")); }
            }
            finish_owner(owner, errors, 22, 22)
        }
