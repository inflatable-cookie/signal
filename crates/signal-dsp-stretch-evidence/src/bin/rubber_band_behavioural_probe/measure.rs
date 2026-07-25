use signal_dsp_stretch::{measure_stereo_image_delta, measure_tonal_texture};

use super::controls::Control;

mod coherence;
use coherence::vertical_phase_coherence;
mod hash;
pub(crate) use hash::hash_samples;
use hash::hash_u64;

pub(super) struct Evidence {
    pub output_frames: usize,
    pub expected_frames: usize,
    pub length_error: isize,
    pub peak: f32,
    pub non_finite: usize,
    pub clipped: usize,
    pub event_offsets: String,
    pub mean_abs_event_offset: f64,
    pub crest_db: f64,
    pub replica_ratio: f64,
    pub endpoint_energy: f64,
    pub added_silence: usize,
    pub vertical_coherence: f64,
    pub mean_spectral_residual: f64,
    pub tonal_movement_delta: f64,
    pub unsupported_mass: f64,
    pub stereo_image_delta: f64,
    pub output_hash: u64,
    pub measurement_hash: u64,
}

pub(super) fn measure(control: &Control, output: &[f32], ratio: f64) -> Evidence {
    let output_frames = output.len() / control.channels;
    let expected_frames = (control.frames() as f64 * ratio).round() as usize;
    let mono_source = channel(&control.samples, control.channels, 0);
    let mono_output = channel(output, control.channels, 0);
    let projected_events = control
        .events
        .iter()
        .map(|event| (*event as f64 * ratio).round() as usize)
        .collect::<Vec<_>>();
    let measured_events = projected_events
        .iter()
        .enumerate()
        .map(|(index, expected)| {
            let radius = event_radius(&projected_events, index);
            if control.id == "soft-onset" {
                onset_near(&mono_output, *expected, radius)
            } else {
                peak_near(&mono_output, *expected, radius)
            }
        })
        .collect::<Vec<_>>();
    let offsets = projected_events
        .iter()
        .zip(&measured_events)
        .map(|(expected, measured)| *measured as isize - *expected as isize)
        .collect::<Vec<_>>();
    let mean_abs_event_offset = if offsets.is_empty() {
        f64::NAN
    } else {
        offsets
            .iter()
            .map(|offset| offset.abs() as f64)
            .sum::<f64>()
            / offsets.len() as f64
    };
    let tonal = measure_tonal_texture(&mono_source, &mono_output, ratio);
    let peak = output
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max);
    let non_finite = output.iter().filter(|sample| !sample.is_finite()).count();
    let clipped = output.iter().filter(|sample| sample.abs() > 1.0).count();
    let crest_db = event_crest(&mono_output, &measured_events);
    let replica_ratio = replica_ratio(&mono_output, &measured_events);
    let endpoint_energy = endpoint_energy(&mono_output);
    let added_silence = added_silence(&mono_source, &mono_output);
    let vertical_coherence = vertical_phase_coherence(&mono_output);
    let stereo_image_delta = if control.channels == 2 {
        measure_stereo_image_delta(&control.samples, output, ratio).image_delta
    } else {
        f64::NAN
    };
    let output_hash = hash_samples(output);
    let event_offsets = offsets
        .iter()
        .map(isize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let unsupported_mass = tonal.mean_added_sideband_ratio;
    let mut measurement_hash = 0xcbf2_9ce4_8422_2325;
    for value in [
        output_frames as u64,
        expected_frames as u64,
        length_error(output_frames, expected_frames) as i64 as u64,
        output_hash,
        mean_abs_event_offset.to_bits(),
        crest_db.to_bits(),
        replica_ratio.to_bits(),
        endpoint_energy.to_bits(),
        vertical_coherence.to_bits(),
        tonal.mean_spectral_residual_ratio.to_bits(),
        tonal.spectral_modulation_delta.to_bits(),
        unsupported_mass.to_bits(),
        stereo_image_delta.to_bits(),
    ] {
        hash_u64(&mut measurement_hash, value);
    }
    Evidence {
        output_frames,
        expected_frames,
        length_error: length_error(output_frames, expected_frames),
        peak,
        non_finite,
        clipped,
        event_offsets,
        mean_abs_event_offset,
        crest_db,
        replica_ratio,
        endpoint_energy,
        added_silence,
        vertical_coherence,
        mean_spectral_residual: tonal.mean_spectral_residual_ratio,
        tonal_movement_delta: tonal.spectral_modulation_delta,
        unsupported_mass,
        stereo_image_delta,
        output_hash,
        measurement_hash,
    }
}

fn channel(samples: &[f32], channels: usize, channel: usize) -> Vec<f32> {
    samples
        .iter()
        .skip(channel)
        .step_by(channels)
        .copied()
        .collect()
}

fn peak_near(samples: &[f32], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    samples[start..end]
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .map(|(index, _)| start + index)
        .unwrap_or(center.min(samples.len().saturating_sub(1)))
}

fn event_radius(events: &[usize], index: usize) -> usize {
    let left = index
        .checked_sub(1)
        .map(|prior| events[index].abs_diff(events[prior]));
    let right = events
        .get(index + 1)
        .map(|next| events[index].abs_diff(*next));
    left.into_iter()
        .chain(right)
        .min()
        .map(|distance| (distance / 2).saturating_sub(1).max(16))
        .unwrap_or(1_024)
}

fn event_crest(samples: &[f32], events: &[usize]) -> f64 {
    events
        .iter()
        .map(|center| {
            let start = center.saturating_sub(128);
            let end = (*center + 129).min(samples.len());
            let span = &samples[start..end];
            let peak = span
                .iter()
                .map(|sample| sample.abs() as f64)
                .fold(0.0, f64::max);
            let rms = (span
                .iter()
                .map(|sample| f64::from(*sample) * f64::from(*sample))
                .sum::<f64>()
                / span.len().max(1) as f64)
                .sqrt();
            20.0 * (peak.max(1.0e-12) / rms.max(1.0e-12)).log10()
        })
        .fold(f64::NAN, f64::max)
}

fn replica_ratio(samples: &[f32], events: &[usize]) -> f64 {
    events
        .iter()
        .map(|center| {
            let primary = samples.get(*center).copied().unwrap_or(0.0).abs() as f64;
            let start = (*center + 32).min(samples.len());
            let end = (*center + 512).min(samples.len());
            let secondary = samples[start..end]
                .iter()
                .map(|sample| sample.abs() as f64)
                .fold(0.0, f64::max);
            secondary / primary.max(1.0e-12)
        })
        .fold(f64::NAN, f64::max)
}

fn endpoint_energy(samples: &[f32]) -> f64 {
    let span = 256.min(samples.len());
    samples[..span]
        .iter()
        .chain(&samples[samples.len() - span..])
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>()
}

fn added_silence(source: &[f32], output: &[f32]) -> usize {
    if source.iter().all(|sample| sample.abs() <= 1.0e-12) {
        return output
            .iter()
            .filter(|sample| sample.abs() > 1.0e-12)
            .count();
    }
    longest_zero_run(output).saturating_sub(longest_zero_run(source))
}

fn longest_zero_run(samples: &[f32]) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for sample in samples {
        if sample.abs() <= 1.0e-12 {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

fn onset_near(samples: &[f32], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius).max(64);
    let end = (center + radius).min(samples.len().saturating_sub(64));
    (start..=end)
        .max_by(|a, b| energy_rise(samples, *a).total_cmp(&energy_rise(samples, *b)))
        .unwrap_or(center)
}

fn energy_rise(samples: &[f32], center: usize) -> f64 {
    let before = samples[center - 64..center]
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>();
    let after = samples[center..center + 64]
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>();
    after - before
}

fn length_error(actual: usize, expected: usize) -> isize {
    actual as isize - expected as isize
}
