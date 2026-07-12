use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::{
    hash_f64, hash_u64, input_hash, reflected, window, ANCHOR_HOP, FFT, HASH_OFFSET,
};
use super::{AnchorEvidence, ControlEvidence, FRAMES};

const WINDOW: usize = 2_048;
const FIRST_CENTER: isize = -(2 * ANCHOR_HOP as isize);
const LAST_CENTER: isize = FRAMES as isize + ANCHOR_HOP as isize;
pub(super) const MIXED_SCALE: f64 = std::f64::consts::TAU * (2 * ANCHOR_HOP) as f64 / FFT as f64;

pub(super) fn measure(control: usize, channels: &[&[f64]]) -> ControlEvidence {
    let (spectra, reflected_reads) = spectra(channels);
    let mut extended = Vec::with_capacity(FRAMES / ANCHOR_HOP + 2);
    let mut anchors = Vec::with_capacity(FRAMES / ANCHOR_HOP);
    let mut mask_hash = HASH_OFFSET;
    let mut ratio_hash = HASH_OFFSET;
    let mut non_finite = 0;
    for anchor in (-(ANCHOR_HOP as isize)..=FRAMES as isize).step_by(ANCHOR_HOP) {
        let frame = ((anchor - FIRST_CENTER) / ANCHOR_HOP as isize) as usize;
        let mut counts = [0; 2];
        let mut sums = [0.0; 2];
        for channel in 0..channels.len() {
            let current = &spectra[channel][frame];
            let before = &spectra[channel][frame - 1];
            let after = &spectra[channel][frame + 1];
            let frame_energy = current[1..=2046]
                .iter()
                .map(|value| value.norm_sqr())
                .sum::<f64>();
            let floor = frame_energy / (FFT * FFT) as f64;
            for bin in 1..=2046 {
                let energy = current[bin].norm_sqr();
                let eligible = frame_energy > 0.0 && energy > 0.0 && energy >= floor;
                let mut percussive = false;
                if eligible {
                    let cross =
                        after[bin + 1] * before[bin + 1].conj() * after[bin].conj() * before[bin];
                    let mixed = cross.arg() / MIXED_SCALE;
                    percussive = (mixed - 1.0).abs() <= mixed.abs();
                    let magnitude = energy.sqrt();
                    counts[0] += 1;
                    sums[1] += magnitude;
                    if percussive {
                        counts[1] += 1;
                        sums[0] += magnitude;
                    }
                    non_finite += usize::from(!mixed.is_finite());
                }
                if (0..FRAMES as isize).contains(&anchor) {
                    hash_u64(&mut mask_hash, eligible as u64);
                    hash_u64(&mut mask_hash, percussive as u64);
                }
            }
        }
        let occupancy = if sums[1] == 0.0 {
            0.0
        } else {
            sums[0] / sums[1]
        };
        non_finite += usize::from(!occupancy.is_finite());
        extended.push(occupancy);
        if (0..FRAMES as isize).contains(&anchor) {
            hash_f64(&mut ratio_hash, occupancy);
            anchors.push(AnchorEvidence {
                anchor: anchor as usize,
                cell_counts: counts,
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
        .filter(|anchor| *anchor < FRAMES)
        .collect::<Vec<_>>();
    let peak_hash = hash_peaks(&peaks);
    let mut evidence = ControlEvidence {
        control,
        anchors,
        peaks,
        event_offsets: Vec::new(),
        structural_counts: [reflected_reads, non_finite],
        hashes: [input_hash(channels), mask_hash, ratio_hash, peak_hash, 0],
    };
    evidence.hashes[4] = control_hash(&evidence);
    evidence
}

pub(super) fn spectra(channels: &[&[f64]]) -> (Vec<Vec<Vec<Complex64>>>, usize) {
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(FFT);
    let window = window(WINDOW);
    let mut all = Vec::with_capacity(channels.len());
    let mut reflected_reads = 0;
    for channel in channels {
        let mut frames = Vec::new();
        for center in (FIRST_CENTER..=LAST_CENTER).step_by(ANCHOR_HOP) {
            let mut buffer = vec![Complex64::new(0.0, 0.0); FFT];
            let offset = (FFT - WINDOW) / 2;
            for (index, weight) in window.iter().copied().enumerate() {
                let logical = center - WINDOW as isize / 2 + index as isize;
                reflected_reads += usize::from(logical < 0 || logical >= channel.len() as isize);
                buffer[offset + index].re = reflected(channel, logical) * weight;
            }
            fft.process(&mut buffer);
            frames.push(buffer);
        }
        all.push(frames);
    }
    (all, reflected_reads)
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
    for value in &control.hashes[..4] {
        hash_u64(&mut hash, *value);
    }
    for anchor in &control.anchors {
        hash_u64(&mut hash, anchor.cell_counts[0] as u64);
        hash_u64(&mut hash, anchor.cell_counts[1] as u64);
        hash_f64(&mut hash, anchor.magnitude_sums[0]);
        hash_f64(&mut hash, anchor.magnitude_sums[1]);
    }
    hash
}
