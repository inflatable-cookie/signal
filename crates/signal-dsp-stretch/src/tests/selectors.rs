use super::support::*;
use super::*;

#[test]
fn compression_short_window_selector_matches_gate_decision() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 0.75;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut short_window = OfflineHighQualityStretcher::with_window(
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    let short_window_output = short_window
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::CompressionShortWindowSelector,
    );
    let selector_output = selector
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let default_smear = measure_transient_smear(
        &input,
        &default_output,
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        StretchTransientSmearPolicies::production(),
    );
    let accepted = default_smear.missed_transients
        >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES
        || default_smear.max_smear_frames
            >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES;

    let expected = if accepted {
        &short_window_output
    } else {
        &default_output
    };
    assert_eq!(selector_output, *expected);
    assert_eq!(
        selector_output.len(),
        (input.len() as f64 * ratio).round() as usize
    );
}

#[test]
fn compression_short_window_selector_does_not_switch_expansion_ratios() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 1.25;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::CompressionShortWindowSelector,
    );

    assert_eq!(
        selector
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        default_output
    );
}

#[test]
fn expansion_short_window_selector_matches_gate_decision() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 1.25;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut short_window = OfflineHighQualityStretcher::with_window(
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    let short_window_output = short_window
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::ExpansionShortWindowSelector,
    );
    let selector_output = selector
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let accepted = should_select_expansion_short_window(&input, &default_output, ratio);

    let expected = if accepted {
        &short_window_output
    } else {
        &default_output
    };
    assert_eq!(selector_output, *expected);
    assert_eq!(
        selector_output.len(),
        (input.len() as f64 * ratio).round() as usize
    );
}

#[test]
fn expansion_short_window_selector_rejects_compression_ratios() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 0.75;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::ExpansionShortWindowSelector,
    );

    assert_eq!(
        selector
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        default_output
    );
}

#[test]
fn expansion_short_window_gate_accepts_current_misses() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 1.25;
    let silent_current = vec![0.0; (input.len() as f64 * ratio).round() as usize];

    assert!(should_select_expansion_short_window(
        &input,
        &silent_current,
        ratio
    ));
    assert!(!should_select_expansion_short_window(
        &input,
        &silent_current,
        0.75
    ));
}
