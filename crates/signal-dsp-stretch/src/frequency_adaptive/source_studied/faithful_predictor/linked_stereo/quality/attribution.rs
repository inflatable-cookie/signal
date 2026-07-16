use super::super::super::{coherent_representation, hash_samples, HASH_OFFSET};
use super::super::hash_values;
use super::{
    controls::{
        correlated_control, delay_control, tone_control, PHASE_OFFSETS, RATIOS, SAMPLE_RATE,
        TONE_FREQUENCIES,
    },
    measure, quality_review,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum LinkedStereoQualityAttributionDirection {
    PerChannelRecurrencePrimary,
    ReferenceProjectionResidual,
    AggregateModeContribution,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LinkedStereoQualityAttributionRow {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) linked: [f64; 4],
    pub(in crate::frequency_adaptive) independent: [f64; 4],
    pub(in crate::frequency_adaptive) linked_failure_mask: u8,
    pub(in crate::frequency_adaptive) independent_failure_mask: u8,
    pub(in crate::frequency_adaptive) independent_audio_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LinkedStereoQualityAttributionReview {
    pub(in crate::frequency_adaptive) rows: Vec<LinkedStereoQualityAttributionRow>,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: LinkedStereoQualityAttributionDirection,
}

pub(in crate::frequency_adaptive) fn attribution_review() -> LinkedStereoQualityAttributionReview {
    let first = run();
    let second = run();
    let repeated = first == second;
    let recurrence_primary = first
        .rows
        .iter()
        .all(|row| row.linked_failure_mask == row.independent_failure_mask);
    let reference_residual = first.rows.iter().all(|row| {
        row.linked_failure_mask != 0
            && row.linked_failure_mask & !row.independent_failure_mask == 0
            && row.linked_failure_mask != row.independent_failure_mask
    });
    LinkedStereoQualityAttributionReview {
        rows: first.rows,
        evidence_hash: first.evidence_hash,
        repeated,
        direction: if recurrence_primary {
            LinkedStereoQualityAttributionDirection::PerChannelRecurrencePrimary
        } else if reference_residual {
            LinkedStereoQualityAttributionDirection::ReferenceProjectionResidual
        } else {
            LinkedStereoQualityAttributionDirection::AggregateModeContribution
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<LinkedStereoQualityAttributionRow>,
    evidence_hash: u64,
}

fn run() -> Run {
    let linked = quality_review();
    let delay = delay_control();
    let correlated = correlated_control();
    let mut rows = Vec::with_capacity(RATIOS.len());
    let mut evidence_hash = HASH_OFFSET;

    for (&ratio, linked) in RATIOS.iter().zip(&linked.ratios) {
        let mut audio_hash = HASH_OFFSET;
        let mut independent_ipd = 0.0_f64;
        for phase_offset in PHASE_OFFSETS {
            for frequency in TONE_FREQUENCIES {
                let input = tone_control(phase_offset, frequency);
                let output = independent(&input, ratio);
                hash_channels(&mut audio_hash, &output);
                independent_ipd = independent_ipd.max(measure::maximum_ipd_error(
                    &input,
                    &output,
                    &[frequency],
                    SAMPLE_RATE,
                ));
            }
        }

        let delay_output = independent(&delay, ratio);
        hash_channels(&mut audio_hash, &delay_output);
        let input_delay = measure::best_delay(&delay[0], &delay[1], 32);
        let output_delay = measure::best_delay(&delay_output[0], &delay_output[1], 32);
        let independent_delay = input_delay.abs_diff(output_delay) as f64;

        let image_output = independent(&correlated, ratio);
        hash_channels(&mut audio_hash, &image_output);
        let image = measure::image_delta(&correlated, &image_output);
        let linked_metrics = [
            linked.maximum_ipd_error_radians,
            linked.delay_change_frames as f64,
            linked.correlated_image_delta[0],
            linked.correlated_image_delta[1],
        ];
        let independent_metrics = [
            independent_ipd,
            independent_delay,
            image.mid_side_ratio_db,
            image.correlation,
        ];
        let linked_failure_mask = failure_mask(linked_metrics);
        let independent_failure_mask = failure_mask(independent_metrics);
        hash_values(
            &mut evidence_hash,
            &[
                ratio.to_bits(),
                linked_metrics[0].to_bits(),
                linked_metrics[1].to_bits(),
                linked_metrics[2].to_bits(),
                linked_metrics[3].to_bits(),
                independent_metrics[0].to_bits(),
                independent_metrics[1].to_bits(),
                independent_metrics[2].to_bits(),
                independent_metrics[3].to_bits(),
                linked_failure_mask as u64,
                independent_failure_mask as u64,
                audio_hash,
            ],
        );
        rows.push(LinkedStereoQualityAttributionRow {
            ratio,
            linked: linked_metrics,
            independent: independent_metrics,
            linked_failure_mask,
            independent_failure_mask,
            independent_audio_hash: audio_hash,
        });
    }
    Run {
        rows,
        evidence_hash,
    }
}

fn independent(input: &[Vec<f64>; 2], ratio: f64) -> [Vec<f64>; 2] {
    std::array::from_fn(|channel| {
        coherent_representation::render(&input[channel], ratio, SAMPLE_RATE).samples
    })
}

fn hash_channels(hash: &mut u64, channels: &[Vec<f64>; 2]) {
    hash_values(
        hash,
        &[hash_samples(&channels[0]), hash_samples(&channels[1])],
    );
}

fn failure_mask(metrics: [f64; 4]) -> u8 {
    u8::from(metrics[0] > 1.0e-9)
        | (u8::from(metrics[1] > 1.0) << 1)
        | (u8::from(metrics[2] > 0.25) << 2)
        | (u8::from(metrics[3] > 0.02) << 3)
}
