use super::super::SAMPLE_RATE;
use super::{add_render_hashes, specimen::RubberBand};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::mechanics;

pub(super) fn prepare() -> [Vec<f64>; 3] {
    let primary = mechanics::primary_control(SAMPLE_RATE);
    let secondary = mechanics::secondary_control(SAMPLE_RATE);
    let silence = vec![0.0; primary.len()];
    [primary, secondary, silence]
}

pub(super) fn measure(
    specimen: &RubberBand,
    root: &std::path::Path,
    inputs: &[Vec<f64>; 3],
    hashes: &mut [u64; 4],
) -> [f64; 6] {
    let [primary, secondary, silence] = inputs;
    let mut errors = [0.0_f64; 6];
    for ratio in super::super::RATIOS {
        let ordinary = measured_render(
            specimen,
            root,
            &format!("ordinary-{ratio:.2}"),
            &[primary, secondary],
            ratio,
            hashes,
        );
        let duplicate = measured_render(
            specimen,
            root,
            &format!("duplicate-{ratio:.2}"),
            &[primary, primary],
            ratio,
            hashes,
        );
        errors[0] = errors[0].max(maximum_error(&duplicate[0], &duplicate[1], 1.0));
        let mono = measured_render(
            specimen,
            root,
            &format!("mono-{ratio:.2}"),
            &[primary],
            ratio,
            hashes,
        );
        errors[1] = errors[1].max(maximum_error(&duplicate[0], &mono[0], 1.0));
        let pan = measured_render(
            specimen,
            root,
            &format!("pan-{ratio:.2}"),
            &[primary, silence],
            ratio,
            hashes,
        );
        errors[2] = errors[2].max(pan[1].iter().map(|sample| sample.abs()).fold(0.0, f64::max));
        let swapped = measured_render(
            specimen,
            root,
            &format!("swap-{ratio:.2}"),
            &[secondary, primary],
            ratio,
            hashes,
        );
        errors[3] = errors[3]
            .max(maximum_error(&swapped[0], &ordinary[1], 1.0))
            .max(maximum_error(&swapped[1], &ordinary[0], 1.0));
        let negative = measured_render(
            specimen,
            root,
            &format!("polarity-{ratio:.2}"),
            &[
                &mechanics::scaled(primary, -1.0),
                &mechanics::scaled(secondary, -1.0),
            ],
            ratio,
            hashes,
        );
        errors[4] = errors[4]
            .max(maximum_error(&negative[0], &ordinary[0], -1.0))
            .max(maximum_error(&negative[1], &ordinary[1], -1.0));
        let gained = measured_render(
            specimen,
            root,
            &format!("gain-{ratio:.2}"),
            &[
                &mechanics::scaled(primary, 0.25),
                &mechanics::scaled(secondary, 0.25),
            ],
            ratio,
            hashes,
        );
        errors[5] = errors[5]
            .max(maximum_error(&gained[0], &ordinary[0], 0.25))
            .max(maximum_error(&gained[1], &ordinary[1], 0.25));
    }
    errors
}

fn measured_render(
    specimen: &RubberBand,
    root: &std::path::Path,
    stem: &str,
    channels: &[&[f64]],
    ratio: f64,
    hashes: &mut [u64; 4],
) -> Vec<Vec<f64>> {
    let rendered = specimen.render(root, stem, channels, ratio, SAMPLE_RATE);
    add_render_hashes(hashes, &rendered);
    rendered.channels
}

fn maximum_error(actual: &[f64], expected: &[f64], gain: f64) -> f64 {
    actual
        .iter()
        .zip(expected)
        .map(|(actual, expected)| (actual - expected * gain).abs())
        .fold(0.0, f64::max)
}
