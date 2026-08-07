use super::*;

#[test]
fn output_length_drift_tracks_fixed_ratio_contract() {
    assert_eq!(output_length_drift_samples(1_000, 1_500, 1.5), 0.0);
    assert_eq!(output_length_drift_samples(1_001, 1_502, 1.5), 0.0);
    assert_eq!(output_length_drift_samples(1_001, 1_503, 1.5), 1.0);
    assert!(output_length_drift_samples(1_000, 1_000, f64::NAN).is_nan());
}
