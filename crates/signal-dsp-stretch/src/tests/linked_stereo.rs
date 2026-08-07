use super::support::*;
use super::*;

#[test]
fn offline_high_quality_linked_stereo_honors_output_length_contract() {
    let sample_rate = 48_000.0;
    let left = sine(440.0, sample_rate, 48_000);
    let right = sine(660.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }

    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let output = stretcher
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound");

        assert_eq!(
            output.len(),
            ((left.len() as f64 * ratio).round() as usize) * 2,
            "ratio {ratio}"
        );
    }
}

#[test]
fn offline_high_quality_linked_stereo_is_identity_passthrough() {
    let frames = [0.0, 0.1, 0.2, 0.3, 0.4];
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);

    assert_eq!(
        stretcher
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound"),
        frames[..4]
    );
}

#[test]
fn offline_high_quality_linked_stereo_is_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(330.0, sample_rate, 48_000);
    let right = sine(550.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }

    let mut first = OfflineHighQualityStretcher::new(1.5);
    let mut repeated = OfflineHighQualityStretcher::new(1.5);

    assert_eq!(
        first
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound"),
        repeated
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound")
    );
}
