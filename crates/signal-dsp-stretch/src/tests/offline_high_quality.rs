use super::support::{self, boundary_content_probe, sine};
use super::*;

#[test]
fn offline_high_quality_reports_target_quality() {
    let stretcher = OfflineHighQualityStretcher::new(1.25);

    assert_eq!(stretcher.quality(), StretchQuality::OfflineHighQuality);
    assert_eq!(stretcher.ratio(), 1.25);
    assert_eq!(stretcher.path(), OfflineHighQualityPath::Default);
}

#[test]
fn offline_high_quality_path_can_be_selected_explicitly() {
    let mut stretcher = OfflineHighQualityStretcher::with_path(
        0.75,
        OfflineHighQualityPath::CompressionShortWindowSelector,
    );

    assert_eq!(
        stretcher.path(),
        OfflineHighQualityPath::CompressionShortWindowSelector
    );
    stretcher.set_path(OfflineHighQualityPath::Default);
    assert_eq!(stretcher.path(), OfflineHighQualityPath::Default);
    stretcher.set_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
    assert_eq!(
        stretcher.path(),
        OfflineHighQualityPath::ExpansionShortWindowSelector
    );
}

#[test]
fn offline_high_quality_is_deterministic_and_honors_output_length() {
    let input = sine(440.0, 48_000.0, 48_000);
    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let repeated_output = repeated
            .stretch_mono(&input)
            .expect("render fits the offline output bound");

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize,
            "ratio {ratio}"
        );
        assert_eq!(first_output, repeated_output, "ratio {ratio}");
    }
}

#[test]
fn offline_high_quality_boundary_preserves_endpoint_content() {
    let input = boundary_content_probe(48_000, 384);
    for ratio in [0.5, 2.0] {
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let edge_span = 2_048.min(output.len());

        assert_eq!(output.len(), (input.len() as f64 * ratio).round() as usize);
        assert!(
            support::rms(&output[..edge_span]) > 0.01,
            "ratio {ratio}: silent head"
        );
        assert!(
            support::rms(&output[output.len() - edge_span..]) > 0.01,
            "ratio {ratio}: silent tail"
        );
    }
}
