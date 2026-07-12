use rustfft::num_complex::Complex64;

use super::{Frame, FFT_FRAMES, GUARD_FRAMES, WINDOW_LENGTHS};

pub(super) fn propagate_phases(
    frame_index: usize,
    frames: &[Frame],
    spectrum: &mut [Complex64],
    previous: &mut [f64],
    synthesis: &mut [f64],
    phase_count: &mut usize,
) {
    let bins = FFT_FRAMES / 2 + 1;
    let mut analysis_phase = vec![0.0; bins];
    let mut magnitudes = vec![0.0; bins];
    for bin in 0..bins {
        analysis_phase[bin] = spectrum[bin].arg();
        magnitudes[bin] = spectrum[bin].norm();
        if frame_index == 0 {
            synthesis[bin] = analysis_phase[bin];
        } else {
            let source_hop =
                (frames[frame_index].source_center - frames[frame_index - 1].source_center) as f64;
            let output_hop =
                (frames[frame_index].output_center - frames[frame_index - 1].output_center) as f64;
            let omega = std::f64::consts::TAU * bin as f64 / FFT_FRAMES as f64;
            let deviation = wrap(analysis_phase[bin] - previous[bin] - omega * source_hop);
            synthesis[bin] = wrap(synthesis[bin] + (omega + deviation / source_hop) * output_hop);
        }
        previous[bin] = analysis_phase[bin];
        *phase_count += 1;
    }
    lock_to_peaks(&analysis_phase, &magnitudes, synthesis);
    for bin in 0..bins {
        spectrum[bin] = Complex64::from_polar(magnitudes[bin], synthesis[bin]);
    }
    spectrum[0].im = 0.0;
    spectrum[FFT_FRAMES / 2].im = 0.0;
    for bin in 1..FFT_FRAMES / 2 {
        spectrum[FFT_FRAMES - bin] = spectrum[bin].conj();
    }
}

fn lock_to_peaks(analysis: &[f64], magnitudes: &[f64], synthesis: &mut [f64]) {
    let peaks = (1..magnitudes.len() - 1)
        .filter(|bin| {
            magnitudes[*bin] > 1.0e-9
                && magnitudes[*bin] > magnitudes[*bin - 1]
                && magnitudes[*bin] >= magnitudes[*bin + 1]
        })
        .collect::<Vec<_>>();
    for (index, peak) in peaks.iter().copied().enumerate() {
        let left = index
            .checked_sub(1)
            .map(|prior| (peaks[prior] + peak) / 2 + 1)
            .unwrap_or(0);
        let right = peaks
            .get(index + 1)
            .map(|next| (peak + *next) / 2 + 1)
            .unwrap_or(magnitudes.len());
        let peak_phase = synthesis[peak];
        for bin in left..right {
            synthesis[bin] = wrap(peak_phase + wrap(analysis[bin] - analysis[peak]));
        }
    }
}

pub(super) fn schedule(source_len: usize, ratio: f64, events: &[usize]) -> Vec<Frame> {
    let mut frames = Vec::new();
    let end = source_len as isize + GUARD_FRAMES;
    let mut center = -GUARD_FRAMES - FFT_FRAMES as isize / 2;
    let mut length = desired_length(center, events);
    while center <= end + FFT_FRAMES as isize / 2 {
        frames.push(Frame {
            source_center: center,
            output_center: (ratio * center as f64).round() as isize,
            length,
        });
        let proposed = center + length as isize / 4;
        let desired = level(desired_length(proposed, events));
        let next = WINDOW_LENGTHS
            [desired.clamp(level(length).saturating_sub(1), (level(length) + 1).min(3))];
        center += length.min(next) as isize / 4;
        length = next;
    }
    frames
}

fn desired_length(center: isize, events: &[usize]) -> usize {
    events
        .iter()
        .map(|event| match center.abs_diff(*event as isize) {
            0..=256 => 512,
            257..=768 => 1_024,
            769..=1_792 => 2_048,
            _ => 4_096,
        })
        .min()
        .unwrap_or(4_096)
}

pub(super) fn level(length: usize) -> usize {
    WINDOW_LENGTHS
        .iter()
        .position(|candidate| *candidate == length)
        .unwrap()
}

pub(super) fn window(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect()
}

pub(super) fn reflected_sample(input: &[f64], logical: isize) -> f64 {
    let mut index = logical;
    let length = input.len() as isize;
    while index < 0 || index >= length {
        index = if index < 0 {
            -index - 1
        } else {
            2 * length - index - 1
        };
    }
    input[index as usize]
}

pub(super) fn hop_extrema(frames: &[Frame], field: impl Fn(&Frame) -> isize) -> [usize; 2] {
    let hops = frames
        .windows(2)
        .map(|pair| (field(&pair[1]) - field(&pair[0])) as usize);
    [hops.clone().min().unwrap_or(0), hops.max().unwrap_or(0)]
}

fn wrap(phase: f64) -> f64 {
    phase - std::f64::consts::TAU * (phase / std::f64::consts::TAU).round()
}
