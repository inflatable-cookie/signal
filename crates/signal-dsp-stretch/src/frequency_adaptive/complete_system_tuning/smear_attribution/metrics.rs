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

#[derive(Default)]
pub(crate) struct Accumulator {
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

pub(crate) fn event_disagreement(stems: [&Vec<f64>; 3], events: &[usize]) -> (f64, usize, usize) {
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

pub(crate) fn pairwise_correlation(stems: [&Vec<f64>; 3]) -> f64 {
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

pub(crate) fn replica_count(samples: &[f64], events: &[usize]) -> usize {
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

pub(crate) fn accumulate(
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

pub(crate) fn finish(mut accumulator: Accumulator) -> ModeEvidence {
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
