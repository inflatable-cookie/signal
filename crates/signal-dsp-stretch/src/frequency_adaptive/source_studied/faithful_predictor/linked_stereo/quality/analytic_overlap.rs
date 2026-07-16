use super::super::super::{coherent_representation, HASH_OFFSET};
use super::super::{
    hash_values,
    mechanics::{
        mismatch_count, primary_control, scaled, secondary_control, signed_mismatch_count,
    },
    render,
};
use super::{controls::*, measure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum AnalyticOverlapDirection {
    AdoptionContract,
    Reject,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AnalyticOverlapRow {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 4],
    pub(in crate::frequency_adaptive) duplicate_mismatches: usize,
    pub(in crate::frequency_adaptive) hard_pan_mismatches: usize,
    pub(in crate::frequency_adaptive) maximum_mono_difference: f64,
    pub(in crate::frequency_adaptive) silent_peer_peak: f64,
    pub(in crate::frequency_adaptive) swap_mismatches: usize,
    pub(in crate::frequency_adaptive) polarity_mismatches: usize,
    pub(in crate::frequency_adaptive) current_ipd_error: f64,
    pub(in crate::frequency_adaptive) analytic_ipd_error: f64,
    pub(in crate::frequency_adaptive) current_oracle_ipd_error: f64,
    pub(in crate::frequency_adaptive) analytic_oracle_ipd_error: f64,
    pub(in crate::frequency_adaptive) current_image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) analytic_image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) current_tone_hash: u64,
    pub(in crate::frequency_adaptive) analytic_tone_hash: u64,
    pub(in crate::frequency_adaptive) analytic_oracle_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AnalyticOverlapReview {
    pub(in crate::frequency_adaptive) rows: Vec<AnalyticOverlapRow>,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: AnalyticOverlapDirection,
}

pub(in crate::frequency_adaptive) fn analytic_overlap_review() -> AnalyticOverlapReview {
    let first = run();
    let second = run();
    let repeated = first == second;
    let passed = repeated
        && first.rows.iter().all(|row| {
            row.structural_failures == [0; 4]
                && row.duplicate_mismatches == 0
                && row.hard_pan_mismatches == 0
                && row.silent_peer_peak == 0.0
                && row.swap_mismatches == 0
                && row.polarity_mismatches == 0
                && row.analytic_ipd_error < row.current_ipd_error
                && (row.current_image_delta[0] <= 0.25
                    || row.analytic_image_delta[0] < row.current_image_delta[0])
        });
    AnalyticOverlapReview {
        rows: first.rows,
        evidence_hash: first.evidence_hash,
        repeated,
        direction: if passed {
            AnalyticOverlapDirection::AdoptionContract
        } else {
            AnalyticOverlapDirection::Reject
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<AnalyticOverlapRow>,
    evidence_hash: u64,
}

fn run() -> Run {
    let primary = primary_control(SAMPLE_RATE);
    let secondary = secondary_control(SAMPLE_RATE);
    let correlated = correlated_control();
    let silence = vec![0.0; primary.len()];
    let mut evidence_hash = HASH_OFFSET;
    let mut rows = Vec::with_capacity(RATIOS.len());

    for ratio in RATIOS {
        let mono = coherent_representation::render(&primary, ratio, SAMPLE_RATE);
        let duplicate = render::linked_analytic([&primary, &primary], ratio, SAMPLE_RATE);
        let hard_pan = render::linked_analytic([&primary, &silence], ratio, SAMPLE_RATE);
        let ordinary = render::linked_analytic([&primary, &secondary], ratio, SAMPLE_RATE);
        let swapped = render::linked_analytic([&secondary, &primary], ratio, SAMPLE_RATE);
        let negative_primary = scaled(&primary, -1.0);
        let negative_secondary = scaled(&secondary, -1.0);
        let polarity =
            render::linked_analytic([&negative_primary, &negative_secondary], ratio, SAMPLE_RATE);
        let target_length = mono.samples.len();
        let structural_failures = [
            usize::from(
                duplicate
                    .channels
                    .iter()
                    .chain(&hard_pan.channels)
                    .chain(&ordinary.channels)
                    .any(|channel| channel.len() != target_length),
            ),
            duplicate.uncovered + hard_pan.uncovered + ordinary.uncovered,
            duplicate.non_finite + hard_pan.non_finite + ordinary.non_finite,
            duplicate.boundary_failures + hard_pan.boundary_failures + ordinary.boundary_failures,
        ];
        let duplicate_mismatches = mismatch_count(&duplicate.channels[0], &mono.samples)
            + mismatch_count(&duplicate.channels[1], &mono.samples);
        let hard_pan_mismatches = mismatch_count(&hard_pan.channels[0], &mono.samples);
        let maximum_mono_difference = maximum_difference(&duplicate.channels[0], &mono.samples)
            .max(maximum_difference(&duplicate.channels[1], &mono.samples))
            .max(maximum_difference(&hard_pan.channels[0], &mono.samples));
        let silent_peer_peak = hard_pan.channels[1]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0, f64::max);
        let swap_mismatches = mismatch_count(&swapped.channels[0], &ordinary.channels[1])
            + mismatch_count(&swapped.channels[1], &ordinary.channels[0]);
        let polarity_mismatches =
            signed_mismatch_count(&polarity.channels[0], &ordinary.channels[0], -1.0)
                + signed_mismatch_count(&polarity.channels[1], &ordinary.channels[1], -1.0);

        let mut current_ipd_error = 0.0_f64;
        let mut analytic_ipd_error = 0.0_f64;
        let mut current_oracle_ipd_error = 0.0_f64;
        let mut analytic_oracle_ipd_error = 0.0_f64;
        let mut current_tone_hash = HASH_OFFSET;
        let mut analytic_tone_hash = HASH_OFFSET;
        let mut analytic_oracle_hash = HASH_OFFSET;
        for frequency in TONE_FREQUENCIES {
            let input = tone_control(std::f64::consts::FRAC_PI_2, frequency);
            let current = render::linked([&input[0], &input[1]], ratio, SAMPLE_RATE);
            let analytic = render::linked_analytic([&input[0], &input[1]], ratio, SAMPLE_RATE);
            let current_oracle = render::linked_with_relation_oracle(
                [&input[0], &input[1]],
                ratio,
                SAMPLE_RATE,
                std::f64::consts::FRAC_PI_2,
            );
            let analytic_oracle = render::linked_analytic_with_relation_oracle(
                [&input[0], &input[1]],
                ratio,
                SAMPLE_RATE,
                std::f64::consts::FRAC_PI_2,
            );
            current_ipd_error = current_ipd_error.max(measure::maximum_ipd_error(
                &input,
                &current.channels,
                &[frequency],
                SAMPLE_RATE,
            ));
            analytic_ipd_error = analytic_ipd_error.max(measure::maximum_ipd_error(
                &input,
                &analytic.channels,
                &[frequency],
                SAMPLE_RATE,
            ));
            current_oracle_ipd_error = current_oracle_ipd_error.max(measure::maximum_ipd_error(
                &input,
                &current_oracle.channels,
                &[frequency],
                SAMPLE_RATE,
            ));
            analytic_oracle_ipd_error = analytic_oracle_ipd_error.max(measure::maximum_ipd_error(
                &input,
                &analytic_oracle.channels,
                &[frequency],
                SAMPLE_RATE,
            ));
            hash_values(&mut current_tone_hash, &[current.hash]);
            hash_values(&mut analytic_tone_hash, &[analytic.hash]);
            hash_values(&mut analytic_oracle_hash, &[analytic_oracle.hash]);
        }

        let current_image = render::linked([&correlated[0], &correlated[1]], ratio, SAMPLE_RATE);
        let analytic_image =
            render::linked_analytic([&correlated[0], &correlated[1]], ratio, SAMPLE_RATE);
        let current_image = measure::image_delta(&correlated, &current_image.channels);
        let analytic_image = measure::image_delta(&correlated, &analytic_image.channels);
        let row = AnalyticOverlapRow {
            ratio,
            structural_failures,
            duplicate_mismatches,
            hard_pan_mismatches,
            maximum_mono_difference,
            silent_peer_peak,
            swap_mismatches,
            polarity_mismatches,
            current_ipd_error,
            analytic_ipd_error,
            current_oracle_ipd_error,
            analytic_oracle_ipd_error,
            current_image_delta: [current_image.mid_side_ratio_db, current_image.correlation],
            analytic_image_delta: [analytic_image.mid_side_ratio_db, analytic_image.correlation],
            current_tone_hash,
            analytic_tone_hash,
            analytic_oracle_hash,
        };
        hash_row(&mut evidence_hash, &row);
        rows.push(row);
    }
    Run {
        rows,
        evidence_hash,
    }
}

fn hash_row(hash: &mut u64, row: &AnalyticOverlapRow) {
    hash_values(
        hash,
        &[
            row.ratio.to_bits(),
            row.duplicate_mismatches as u64,
            row.hard_pan_mismatches as u64,
            row.maximum_mono_difference.to_bits(),
            row.silent_peer_peak.to_bits(),
            row.swap_mismatches as u64,
            row.polarity_mismatches as u64,
            row.current_ipd_error.to_bits(),
            row.analytic_ipd_error.to_bits(),
            row.current_oracle_ipd_error.to_bits(),
            row.analytic_oracle_ipd_error.to_bits(),
            row.current_image_delta[0].to_bits(),
            row.current_image_delta[1].to_bits(),
            row.analytic_image_delta[0].to_bits(),
            row.analytic_image_delta[1].to_bits(),
            row.current_tone_hash,
            row.analytic_tone_hash,
            row.analytic_oracle_hash,
        ],
    );
    hash_values(hash, &row.structural_failures.map(|value| value as u64));
}

fn maximum_difference(actual: &[f64], expected: &[f64]) -> f64 {
    actual
        .iter()
        .zip(expected)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0, f64::max)
}
