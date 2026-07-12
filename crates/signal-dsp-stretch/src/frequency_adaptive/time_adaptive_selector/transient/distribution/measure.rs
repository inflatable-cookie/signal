use super::super::super::super::types::{
    StretchMixedPhaseBandEvidence as BandEvidence,
    StretchMixedPhaseControlEvidence as ControlEvidence,
};
use super::super::super::{hash_f64, hash_u64, input_hash, ANCHOR_HOP, HASH_OFFSET};
use super::super::measure::{spectra, MIXED_SCALE};
use super::super::FRAMES;
use super::{CUTOFFS, RADII};

const BINS: std::ops::RangeInclusive<usize> = 1..=2046;
const BAND_EDGES: [f64; 4] = [0.001, 0.003, 0.01, 0.03];
const QUANTILES: [f64; 9] = [0.0, 0.01, 0.05, 0.25, 0.5, 0.75, 0.95, 0.99, 1.0];

#[derive(Clone)]
pub(super) struct Cell {
    pub(super) normalized_magnitude: f64,
    pub(super) mixed_phase: f64,
    pub(super) magnitude: f64,
    pub(super) event: bool,
}

pub(super) struct Distribution {
    pub(super) evidence: ControlEvidence,
    pub(super) cells: Vec<Cell>,
    pub(super) signature: Vec<f64>,
}

pub(super) fn distribute(control: usize, perturbed: bool, channels: &[&[f64]]) -> Distribution {
    let (spectra, reflected_reads) = spectra(channels);
    let mut cells = Vec::new();
    let mut nonzero_cells = 0;
    let mut non_finite = 0;
    for anchor in (0..FRAMES).step_by(ANCHOR_HOP) {
        let frame = anchor / ANCHOR_HOP + 2;
        for channel in 0..channels.len() {
            let current = &spectra[channel][frame];
            let before = &spectra[channel][frame - 1];
            let after = &spectra[channel][frame + 1];
            let energy = BINS.clone().map(|bin| current[bin].norm_sqr()).sum::<f64>();
            if energy == 0.0 {
                continue;
            }
            let norm = energy.sqrt();
            for bin in BINS.clone() {
                let magnitude = current[bin].norm();
                if magnitude == 0.0 {
                    continue;
                }
                nonzero_cells += 1;
                let cross =
                    after[bin + 1] * before[bin + 1].conj() * after[bin].conj() * before[bin];
                let cell = Cell {
                    normalized_magnitude: magnitude / norm,
                    mixed_phase: cross.arg() / MIXED_SCALE,
                    magnitude,
                    event: event_anchor(control, anchor),
                };
                non_finite += usize::from(
                    !cell.normalized_magnitude.is_finite()
                        || !cell.mixed_phase.is_finite()
                        || !cell.magnitude.is_finite(),
                );
                if cell.normalized_magnitude.is_finite()
                    && cell.mixed_phase.is_finite()
                    && cell.magnitude.is_finite()
                {
                    cells.push(cell);
                }
            }
        }
    }
    let bands = summarize(&cells);
    let signature = signature(&cells, &bands);
    let mut evidence = ControlEvidence {
        control,
        perturbed,
        bands,
        structural_counts: [nonzero_cells, cells.len(), reflected_reads, non_finite],
        hashes: [input_hash(channels), 0],
    };
    evidence.hashes[1] = control_hash(&evidence);
    Distribution {
        evidence,
        cells,
        signature,
    }
}

pub(super) fn selection_ratio(
    cells: &[Cell],
    cutoff: f64,
    radius: f64,
    region: impl Fn(&Cell) -> bool,
) -> f64 {
    let denominator = cells
        .iter()
        .filter(|cell| region(cell))
        .map(|cell| cell.magnitude)
        .sum::<f64>();
    if denominator == 0.0 {
        return 0.0;
    }
    cells
        .iter()
        .filter(|cell| {
            region(cell)
                && cell.normalized_magnitude >= cutoff
                && (cell.mixed_phase - 1.0).abs() <= radius
        })
        .map(|cell| cell.magnitude)
        .sum::<f64>()
        / denominator
}

fn summarize(cells: &[Cell]) -> Vec<BandEvidence> {
    let mut evidence = Vec::with_capacity(10);
    for event in [false, true] {
        for band in 0..5 {
            let selected = cells
                .iter()
                .filter(|cell| cell.event == event && band_index(cell.normalized_magnitude) == band)
                .collect::<Vec<_>>();
            let mut phases = selected
                .iter()
                .map(|cell| cell.mixed_phase)
                .collect::<Vec<_>>();
            phases.sort_by(f64::total_cmp);
            evidence.push(BandEvidence {
                band,
                event,
                cell_count: selected.len(),
                magnitude_sum: selected.iter().map(|cell| cell.magnitude).sum(),
                quantiles: quantiles(&phases),
            });
        }
    }
    evidence
}

fn signature(cells: &[Cell], bands: &[BandEvidence]) -> Vec<f64> {
    let count = cells.len() as f64;
    let magnitude = cells.iter().map(|cell| cell.magnitude).sum::<f64>();
    let mut values = Vec::with_capacity(45);
    for band in bands {
        values.push(if count == 0.0 {
            0.0
        } else {
            band.cell_count as f64 / count
        });
        values.push(if magnitude == 0.0 {
            0.0
        } else {
            band.magnitude_sum / magnitude
        });
    }
    for cutoff in CUTOFFS {
        for radius in RADII {
            values.push(selection_ratio(cells, cutoff, radius, |_| true));
        }
    }
    values
}

fn event_anchor(control: usize, anchor: usize) -> bool {
    let events: &[usize] = match control {
        5 | 11 => &[FRAMES / 2],
        6 => &[FRAMES / 2 - 128, FRAMES / 2 + 128],
        7 => &[0, FRAMES - 1],
        _ => &[],
    };
    events.iter().any(|event| anchor.abs_diff(*event) <= 256)
}

fn band_index(value: f64) -> usize {
    BAND_EDGES.partition_point(|edge| value >= *edge)
}

fn quantiles(values: &[f64]) -> [f64; 9] {
    if values.is_empty() {
        return [0.0; 9];
    }
    QUANTILES.map(|probability| {
        let index = (probability * (values.len() - 1) as f64).floor() as usize;
        values[index]
    })
}

fn control_hash(evidence: &ControlEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, evidence.control as u64);
    hash_u64(&mut hash, evidence.perturbed as u64);
    for band in &evidence.bands {
        hash_u64(&mut hash, band.band as u64);
        hash_u64(&mut hash, band.event as u64);
        hash_u64(&mut hash, band.cell_count as u64);
        hash_f64(&mut hash, band.magnitude_sum);
        for value in band.quantiles {
            hash_f64(&mut hash, value);
        }
    }
    for value in evidence.structural_counts {
        hash_u64(&mut hash, value as u64);
    }
    hash
}
