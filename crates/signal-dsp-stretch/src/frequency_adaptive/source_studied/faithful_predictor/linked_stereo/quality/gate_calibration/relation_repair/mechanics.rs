use super::{coherent_representation, render, repair, SAMPLE_RATE};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::mechanics::{
    primary_control, scaled, secondary_control,
};

pub(super) fn review() -> ([f64; 5], f64) {
    let primary = primary_control(SAMPLE_RATE);
    let secondary = secondary_control(SAMPLE_RATE);
    let silence = vec![0.0; primary.len()];
    let mut errors = [0.0_f64; 5];
    let mut silent_peer_peak = 0.0_f64;
    for ratio in super::RATIOS {
        let ordinary = repaired([&primary, &secondary], ratio);
        let duplicate_before = render::linked([&primary, &primary], ratio, SAMPLE_RATE).channels;
        let duplicate = repair(
            &[primary.clone(), primary.clone()],
            duplicate_before.clone(),
        );
        errors[0] = errors[0].max(maximum_error(&duplicate.channels, &duplicate_before, 1.0));

        let hard_pan_before = render::linked([&primary, &silence], ratio, SAMPLE_RATE).channels;
        let hard_pan = repair(&[primary.clone(), silence.clone()], hard_pan_before.clone());
        errors[1] = errors[1].max(maximum_error(&hard_pan.channels, &hard_pan_before, 1.0));
        silent_peer_peak = silent_peer_peak.max(
            hard_pan.channels[1]
                .iter()
                .map(|sample| sample.abs())
                .fold(0.0, f64::max),
        );

        let swapped = repaired([&secondary, &primary], ratio);
        let swapped_expected = [ordinary[1].clone(), ordinary[0].clone()];
        errors[2] = errors[2].max(maximum_error(&swapped, &swapped_expected, 1.0));

        let negative_primary = scaled(&primary, -1.0);
        let negative_secondary = scaled(&secondary, -1.0);
        let negative = repaired([&negative_primary, &negative_secondary], ratio);
        errors[3] = errors[3].max(maximum_error(&negative, &ordinary, -1.0));

        for gain in [0.25, 4.0] {
            let gained_primary = scaled(&primary, gain);
            let gained = repaired([&gained_primary, &gained_primary], ratio);
            let mono = coherent_representation::render(&gained_primary, ratio, SAMPLE_RATE).samples;
            let expected = [mono.clone(), mono];
            errors[4] = errors[4].max(maximum_error(&gained, &expected, 1.0));
        }
    }
    (errors, silent_peer_peak)
}

fn repaired(inputs: [&[f64]; 2], ratio: f64) -> [Vec<f64>; 2] {
    let source = [inputs[0].to_vec(), inputs[1].to_vec()];
    let output = render::linked(inputs, ratio, SAMPLE_RATE);
    repair(&source, output.channels).channels
}

fn maximum_error(actual: &[Vec<f64>; 2], expected: &[Vec<f64>; 2], gain: f64) -> f64 {
    actual
        .iter()
        .zip(expected)
        .flat_map(|(actual, expected)| actual.iter().zip(expected))
        .map(|(actual, expected)| (actual - expected * gain).abs())
        .fold(0.0, f64::max)
}
