use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::super::super::types::{
    StretchMedianHpssAnchorEvidence as AnchorEvidence,
    StretchMedianHpssControlEvidence as ControlEvidence,
};
use super::super::super::{
    hash_f64, hash_u64, input_hash, reflected, window, ANCHOR_HOP, FFT, HASH_OFFSET,
};
use super::super::FRAMES;

const WINDOW: usize = 2_048;
const FIRST_MAGNITUDE_FRAME: isize = -75;
const LAST_MAGNITUDE_FRAME: isize = FRAMES as isize / ANCHOR_HOP as isize + 74;
const TIME_RADIUS: isize = 74;
const FREQUENCY_RADIUS: isize = 8;
const FIRST_BIN: isize = 1;
const LAST_BIN: isize = 2_046;

pub(super) fn measure(control: usize, channels: &[&[f64]]) -> ControlEvidence {
    let (magnitudes, sample_reflections, magnitude_hash) = linked_magnitudes(channels);
    let mut extended = Vec::with_capacity(FRAMES / ANCHOR_HOP + 2);
    let mut anchors = Vec::with_capacity(FRAMES / ANCHOR_HOP);
    let mut median_hash = HASH_OFFSET;
    let mut mask_hash = HASH_OFFSET;
    let mut occupancy_hash = HASH_OFFSET;
    let mut median_reflections = 0;
    let mut non_finite = 0;
    for logical_frame in -1..=FRAMES as isize / ANCHOR_HOP as isize {
        let frame = (logical_frame - FIRST_MAGNITUDE_FRAME) as usize;
        let mut sums = [0.0; 4];
        for bin in FIRST_BIN..=LAST_BIN {
            let linked = magnitudes[frame][bin as usize];
            let mut frequency_values = [0.0; 17];
            for (slot, offset) in (-FREQUENCY_RADIUS..=FREQUENCY_RADIUS).enumerate() {
                let requested = bin + offset;
                let reflected_bin = reflect_bin(requested);
                median_reflections += usize::from(requested != reflected_bin);
                frequency_values[slot] = magnitudes[frame][reflected_bin as usize];
            }
            frequency_values.sort_by(f64::total_cmp);
            let percussive = frequency_values[FREQUENCY_RADIUS as usize];
            let mut time_values = [0.0; 149];
            for (slot, offset) in (-TIME_RADIUS..=TIME_RADIUS).enumerate() {
                time_values[slot] = magnitudes[(frame as isize + offset) as usize][bin as usize];
            }
            time_values.sort_by(f64::total_cmp);
            let harmonic = time_values[TIME_RADIUS as usize];
            let denominator = percussive * percussive + harmonic * harmonic;
            let mask = if denominator == 0.0 {
                0.0
            } else {
                percussive * percussive / denominator
            };
            let masked = linked * mask;
            sums[0] += linked;
            sums[1] += harmonic;
            sums[2] += percussive;
            sums[3] += masked;
            non_finite += usize::from(
                !linked.is_finite()
                    || !harmonic.is_finite()
                    || !percussive.is_finite()
                    || !mask.is_finite(),
            );
            hash_f64(&mut median_hash, harmonic);
            hash_f64(&mut median_hash, percussive);
            hash_f64(&mut mask_hash, mask);
        }
        let occupancy = if sums[0] == 0.0 {
            0.0
        } else {
            sums[3] / sums[0]
        };
        non_finite += usize::from(!occupancy.is_finite());
        extended.push(occupancy);
        if (0..FRAMES as isize / ANCHOR_HOP as isize).contains(&logical_frame) {
            hash_f64(&mut occupancy_hash, occupancy);
            anchors.push(AnchorEvidence {
                anchor: logical_frame as usize * ANCHOR_HOP,
                magnitude_sums: sums,
                occupancy,
            });
        }
    }
    let peaks = (1..extended.len() - 1)
        .filter(|index| {
            extended[*index] >= 0.5
                && extended[*index] > extended[*index - 1]
                && extended[*index] >= extended[*index + 1]
        })
        .map(|index| (index - 1) * ANCHOR_HOP)
        .collect::<Vec<_>>();
    let peak_hash = hash_peaks(&peaks);
    let mut evidence = ControlEvidence {
        control,
        anchors,
        peaks,
        event_offsets: Vec::new(),
        structural_counts: [sample_reflections, median_reflections, non_finite],
        hashes: [
            input_hash(channels),
            magnitude_hash,
            median_hash,
            mask_hash,
            occupancy_hash,
            peak_hash,
            0,
        ],
    };
    evidence.hashes[6] = control_hash(&evidence);
    evidence
}

fn linked_magnitudes(channels: &[&[f64]]) -> (Vec<Vec<f64>>, usize, u64) {
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(FFT);
    let window = window(WINDOW);
    let mut linked = Vec::new();
    let mut reflected_reads = 0;
    let mut hash = HASH_OFFSET;
    for logical_frame in FIRST_MAGNITUDE_FRAME..=LAST_MAGNITUDE_FRAME {
        let center = logical_frame * ANCHOR_HOP as isize;
        let mut energy = vec![0.0; FFT / 2];
        for channel in channels {
            let mut buffer = vec![Complex64::new(0.0, 0.0); FFT];
            let offset = (FFT - WINDOW) / 2;
            for (index, weight) in window.iter().copied().enumerate() {
                let logical = center - WINDOW as isize / 2 + index as isize;
                reflected_reads += usize::from(logical < 0 || logical >= channel.len() as isize);
                buffer[offset + index].re = reflected(channel, logical) * weight;
            }
            fft.process(&mut buffer);
            for bin in FIRST_BIN as usize..=LAST_BIN as usize {
                energy[bin] += buffer[bin].norm_sqr();
            }
        }
        for value in &mut energy {
            *value = value.sqrt();
            hash_f64(&mut hash, *value);
        }
        linked.push(energy);
    }
    (linked, reflected_reads, hash)
}

fn reflect_bin(mut bin: isize) -> isize {
    while bin < FIRST_BIN || bin > LAST_BIN {
        if bin < FIRST_BIN {
            bin = 2 * FIRST_BIN - bin - 1;
        } else {
            bin = 2 * LAST_BIN - bin + 1;
        }
    }
    bin
}

fn hash_peaks(peaks: &[usize]) -> u64 {
    let mut hash = HASH_OFFSET;
    for peak in peaks {
        hash_u64(&mut hash, *peak as u64);
    }
    hash
}

fn control_hash(control: &ControlEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in &control.hashes[..6] {
        hash_u64(&mut hash, *value);
    }
    for anchor in &control.anchors {
        for value in anchor.magnitude_sums {
            hash_f64(&mut hash, value);
        }
        hash_f64(&mut hash, anchor.occupancy);
    }
    for value in control.structural_counts {
        hash_u64(&mut hash, value as u64);
    }
    hash
}
