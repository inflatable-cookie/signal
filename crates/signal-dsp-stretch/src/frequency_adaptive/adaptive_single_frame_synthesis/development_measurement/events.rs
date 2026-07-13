use crate::{
    detect_stretch_transients_with_policy, measure_transient_detail, StretchTransientDetectorPolicy,
};

const TRANSIENT_WINDOW: usize = 1_024;
const TRANSIENT_HOP: usize = 128;

pub(super) struct EventEvidence {
    pub matched: usize,
    pub fallback: bool,
    pub mean_offset: f64,
    pub max_offset: f64,
    pub crest_growth: f64,
    pub replica_ratio: f64,
}

pub(super) fn measure_events(source: &[f32], output: &[f32], ratio: f64) -> EventEvidence {
    let detail = measure_transient_detail(source, output, ratio, TRANSIENT_WINDOW, TRANSIENT_HOP);
    if detail.matched_transients > 0 {
        return EventEvidence {
            matched: detail.matched_transients,
            fallback: false,
            mean_offset: detail.mean_absolute_timing_offset_frames,
            max_offset: detail.max_absolute_timing_offset_frames,
            crest_growth: detail.max_transient_crest_growth_db,
            replica_ratio: replica_at(output, detail.max_crest_output_frame),
        };
    }
    let mut events = detect_stretch_transients_with_policy(
        source,
        TRANSIENT_WINDOW,
        TRANSIENT_HOP,
        StretchTransientDetectorPolicy::production(),
    )
    .into_iter()
    .map(|event| event.frame_index)
    .collect::<Vec<_>>();
    if events.is_empty() {
        events.push(strongest_onset(source));
    }
    let mut offset_sum = 0.0;
    let mut max_offset = 0.0_f64;
    let mut crest_growth = f64::NEG_INFINITY;
    let mut replica_ratio = 0.0_f64;
    for source_event in &events {
        let expected = (*source_event as f64 * ratio).round() as usize;
        let output_event = peak_index(output, expected, 512);
        let offset = output_event.abs_diff(expected) as f64;
        offset_sum += offset;
        max_offset = max_offset.max(offset);
        crest_growth = crest_growth.max(
            20.0 * (local_crest(output, output_event) / local_crest(source, *source_event)).log10(),
        );
        replica_ratio = replica_ratio.max(replica_at(output, output_event));
    }
    EventEvidence {
        matched: events.len(),
        fallback: true,
        mean_offset: offset_sum / events.len() as f64,
        max_offset,
        crest_growth,
        replica_ratio,
    }
}

fn replica_at(output: &[f32], event: usize) -> f64 {
    let secondary_start = (event + 65).min(output.len());
    let secondary_end = (event + 513).min(output.len());
    peak(&output[secondary_start..secondary_end]) / f64::from(output[event].abs()).max(1.0e-12)
}

fn strongest_onset(samples: &[f32]) -> usize {
    (1..samples.len())
        .max_by(|left, right| {
            let left_rise = (samples[*left] - samples[*left - 1]).abs();
            let right_rise = (samples[*right] - samples[*right - 1]).abs();
            left_rise.total_cmp(&right_rise)
        })
        .unwrap_or(0)
}

fn peak_index(samples: &[f32], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    (start..end)
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start)
}

fn local_crest(samples: &[f32], center: usize) -> f64 {
    let start = center.saturating_sub(128);
    let end = (center + 129).min(samples.len());
    let span = &samples[start..end];
    let rms = (span
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>()
        / span.len().max(1) as f64)
        .sqrt();
    peak(span) / rms.max(1.0e-12)
}

fn peak(samples: &[f32]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs() as f64)
        .fold(0.0, f64::max)
}
