use std::{fs, path::PathBuf};

use super::{configurations, objective_grid::audio::development_cases};
use crate::frequency_adaptive::{
    complete_phase_synthesis::render::{render_configured_with_layers, Mode},
    study_local_schedule::{
        schedule::build_schedule_with_strength,
        study::{analyze_with_geometry, select},
    },
    HASH_OFFSET,
};

const CANDIDATES: [&str; 3] = ["g512-sr-u0-rc-v1", "g512-sr-u1-rc-v1", "g512-sc-u1-rc-v0"];
const MODES: [Mode; 4] = [Mode::Ordinary, Mode::Event, Mode::Vertical, Mode::Both];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ModeEvidence {
    pub renders: usize,
    pub mean_pairwise_event_disagreement: f64,
    pub maximum_pairwise_event_disagreement: usize,
    pub mean_pairwise_correlation: f64,
    pub mean_layer_replica_count: f64,
    pub mean_combined_replica_count: f64,
}

impl Default for ModeEvidence {
    fn default() -> Self {
        Self {
            renders: 0,
            mean_pairwise_event_disagreement: 0.0,
            maximum_pairwise_event_disagreement: 0,
            mean_pairwise_correlation: 0.0,
            mean_layer_replica_count: 0.0,
            mean_combined_replica_count: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Direction {
    CrossResolutionRecombination,
    LayerTransport,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Review {
    pub configurations: usize,
    pub development_rows: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub maximum_layer_sum_error: f64,
    pub modes: [ModeEvidence; 4],
    pub hashes: [u64; 2],
    pub direction: Direction,
}

#[derive(Default)]
struct Accumulator {
    evidence: ModeEvidence,
    event_disagreement_sum: f64,
    event_disagreement_count: usize,
    correlation_sum: f64,
    correlation_count: usize,
    layer_replicas: usize,
    layer_replica_events: usize,
    combined_replicas: usize,
    combined_replica_events: usize,
}

pub(crate) fn smear_attribution_review() -> Review {
    let configurations = configurations()
        .into_iter()
        .filter(|configuration| CANDIDATES.contains(&configuration.stable_id().as_str()))
        .collect::<Vec<_>>();
    assert_eq!(configurations.len(), CANDIDATES.len());
    let cases = development_cases();
    let mut accumulators: [Accumulator; 4] = std::array::from_fn(|_| Accumulator::default());
    let mut maximum_layer_sum_error = 0.0_f64;
    let mut hashes = [HASH_OFFSET; 2];
    let mut report = String::from(
        "configuration\trow\tmode\tevent_disagreement\tcorrelation\tlayer_replicas\tcombined_replicas\tlayer_sum_error\n",
    );

    for configuration in configurations.iter().copied() {
        for (row, case) in cases.iter().enumerate() {
            let study = analyze_with_geometry(
                &case.channels,
                case.channels[0].len(),
                configuration.geometry,
            );
            let (threshold, agreement) = match configuration.sensitivity {
                super::Sensitivity::Responsive => (3.0, 2),
                super::Sensitivity::Conservative => (6.0, 3),
            };
            let points = select(&study, threshold, agreement);
            let schedule = build_schedule_with_strength(
                case.channels[0].len(),
                128,
                case.ratio,
                &points,
                configuration.unity_strength(),
            );
            for (mode_index, mode) in MODES.into_iter().enumerate() {
                let render = render_configured_with_layers(
                    &case.channels,
                    case.ratio,
                    &points,
                    &schedule,
                    mode,
                    configuration,
                );
                let layers = render
                    .layer_samples
                    .as_ref()
                    .expect("captured layer samples");
                let combined = &render.samples[0];
                let stems = [&layers[0][0], &layers[1][0], &layers[2][0]];
                let layer_sum_error = combined
                    .iter()
                    .enumerate()
                    .map(|(index, sample)| {
                        (sample - stems.iter().map(|stem| stem[index]).sum::<f64>()).abs()
                    })
                    .fold(0.0_f64, f64::max);
                maximum_layer_sum_error = maximum_layer_sum_error.max(layer_sum_error);
                let projected = points
                    .iter()
                    .filter_map(|point| schedule.positions.get(*point / 128).copied())
                    .filter(|point| *point >= 256 && *point + 257 < combined.len())
                    .collect::<Vec<_>>();
                let event_disagreement = event_disagreement(stems, &projected);
                let correlation = pairwise_correlation(stems);
                let layer_replicas = stems
                    .iter()
                    .map(|stem| replica_count(stem, &projected))
                    .sum::<usize>();
                let combined_replicas = replica_count(combined, &projected);
                accumulate(
                    &mut accumulators[mode_index],
                    event_disagreement,
                    correlation,
                    layer_replicas,
                    combined_replicas,
                    projected.len(),
                );
                mix(&mut hashes[0], render.output_hash);
                mix(&mut hashes[1], layer_sum_error.to_bits());
                report.push_str(&format!(
                    "{}\t{}\t{}\t{:.6}\t{:.6}\t{}\t{}\t{:.12e}\n",
                    configuration.stable_id(),
                    row,
                    mode_index,
                    event_disagreement.0,
                    correlation,
                    layer_replicas,
                    combined_replicas,
                    layer_sum_error,
                ));
            }
        }
    }
    fs::write(report_path(), report).expect("write smear attribution report");
    let modes = accumulators.map(finish);
    let complete = modes[3];
    let replica_growth = complete.mean_combined_replica_count - complete.mean_layer_replica_count;
    let direction = if complete.mean_pairwise_event_disagreement >= 8.0 && replica_growth >= 0.25 {
        Direction::CrossResolutionRecombination
    } else if complete.mean_layer_replica_count >= 1.5 {
        Direction::LayerTransport
    } else {
        Direction::Inconclusive
    };
    Review {
        configurations: configurations.len(),
        development_rows: cases.len(),
        renders: configurations.len() * cases.len() * MODES.len(),
        holdout_reads: 0,
        maximum_layer_sum_error,
        modes,
        hashes,
        direction,
    }
}

fn event_disagreement(stems: [&Vec<f64>; 3], events: &[usize]) -> (f64, usize, usize) {
    let mut sum = 0;
    let mut maximum = 0;
    let mut count = 0;
    for event in events {
        let positions = stems.map(|stem| derivative_peak(stem, *event, 256));
        for left in 0..positions.len() {
            for right in left + 1..positions.len() {
                let difference = positions[left].abs_diff(positions[right]);
                sum += difference;
                maximum = maximum.max(difference);
                count += 1;
            }
        }
    }
    (sum as f64, maximum, count)
}

fn derivative_peak(samples: &[f64], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius).max(1);
    let end = (center + radius + 1).min(samples.len());
    (start..end)
        .max_by(|left, right| {
            (samples[*left] - samples[*left - 1])
                .abs()
                .total_cmp(&(samples[*right] - samples[*right - 1]).abs())
        })
        .unwrap_or(center)
}

fn pairwise_correlation(stems: [&Vec<f64>; 3]) -> f64 {
    let mut sum = 0.0;
    let mut count = 0;
    for left in 0..stems.len() {
        for right in left + 1..stems.len() {
            let dot = stems[left]
                .iter()
                .zip(stems[right])
                .map(|(left, right)| left * right)
                .sum::<f64>();
            let energies = [stems[left], stems[right]]
                .map(|stem| stem.iter().map(|sample| sample * sample).sum::<f64>());
            sum += dot.abs() / (energies[0] * energies[1]).sqrt().max(1.0e-15);
            count += 1;
        }
    }
    sum / count as f64
}

fn replica_count(samples: &[f64], events: &[usize]) -> usize {
    events
        .iter()
        .map(|event| {
            let start = event.saturating_sub(256).max(1);
            let end = (event + 257).min(samples.len() - 1);
            let maximum = (start..end)
                .map(|index| (samples[index] - samples[index - 1]).abs())
                .fold(0.0_f64, f64::max);
            let threshold = maximum * 0.35;
            let mut last = None;
            let mut peaks = 0_usize;
            for index in start + 1..end - 1 {
                let value = (samples[index] - samples[index - 1]).abs();
                let previous = (samples[index - 1] - samples[index - 2]).abs();
                let next = (samples[index + 1] - samples[index]).abs();
                if value >= threshold
                    && value >= previous
                    && value > next
                    && last.is_none_or(|last| index - last >= 8)
                {
                    peaks += 1;
                    last = Some(index);
                }
            }
            peaks.saturating_sub(1)
        })
        .sum()
}

fn accumulate(
    accumulator: &mut Accumulator,
    event_disagreement: (f64, usize, usize),
    correlation: f64,
    layer_replicas: usize,
    combined_replicas: usize,
    event_count: usize,
) {
    accumulator.evidence.renders += 1;
    accumulator.event_disagreement_sum += event_disagreement.0;
    accumulator.event_disagreement_count += event_disagreement.2;
    accumulator.evidence.maximum_pairwise_event_disagreement = accumulator
        .evidence
        .maximum_pairwise_event_disagreement
        .max(event_disagreement.1);
    accumulator.correlation_sum += correlation;
    accumulator.correlation_count += 1;
    accumulator.layer_replicas += layer_replicas;
    accumulator.layer_replica_events += event_count * 3;
    accumulator.combined_replicas += combined_replicas;
    accumulator.combined_replica_events += event_count;
}

fn finish(mut accumulator: Accumulator) -> ModeEvidence {
    accumulator.evidence.mean_pairwise_event_disagreement =
        accumulator.event_disagreement_sum / accumulator.event_disagreement_count.max(1) as f64;
    accumulator.evidence.mean_pairwise_correlation =
        accumulator.correlation_sum / accumulator.correlation_count.max(1) as f64;
    accumulator.evidence.mean_layer_replica_count =
        accumulator.layer_replicas as f64 / accumulator.layer_replica_events.max(1) as f64;
    accumulator.evidence.mean_combined_replica_count =
        accumulator.combined_replicas as f64 / accumulator.combined_replica_events.max(1) as f64;
    accumulator.evidence
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-bm-smear-attribution.tsv")
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
