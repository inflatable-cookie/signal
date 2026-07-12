use rustfft::num_complex::Complex64;

use super::super::super::study_local_schedule::schedule::Schedule;
use super::Frame;

pub(super) fn frames(
    source_len: usize,
    ratio: f64,
    schedule: &Schedule,
    layers: [usize; 3],
) -> Vec<Frame> {
    let mut result = Vec::new();
    for (layer, length) in layers.into_iter().enumerate() {
        let hop = (length / 4) as isize;
        let mut source = -(length as isize / 2);
        while source < source_len as isize + length as isize / 2 {
            result.push(Frame {
                layer,
                source,
                output: project(source, source_len, ratio, schedule),
            });
            source += hop;
        }
    }
    result.sort_by_key(|frame| (frame.source, frame.layer));
    result
}

fn project(source: isize, source_len: usize, ratio: f64, schedule: &Schedule) -> isize {
    if source < 0 || source > source_len as isize {
        (ratio * source as f64).round() as isize
    } else {
        schedule.positions[source as usize / 128] as isize
    }
}

pub(super) fn window(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect()
}

pub(super) fn reflected(input: &[f64], mut index: isize) -> f64 {
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

pub(super) fn mirror(spectrum: &mut [Complex64]) {
    let length = spectrum.len();
    for bin in 1..length / 2 {
        spectrum[length - bin] = spectrum[bin].conj();
    }
}

pub(super) fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
