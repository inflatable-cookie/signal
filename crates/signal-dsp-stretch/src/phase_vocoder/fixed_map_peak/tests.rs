use super::*;

#[test]
fn fixed_map_peak_transient_region_stops_at_nearest_minima() {
    let magnitudes = [3.0, 1.0, 2.0, 5.0, 3.0, 1.5, 2.0];

    assert_eq!(peak_minimum_region_bounds(3, &magnitudes), (1, 6));
}

#[test]
fn fixed_map_peak_transient_reference_threshold_is_window_derived() {
    let window = (0..2_048)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / 2_048.0).cos())
        .collect::<Vec<_>>();
    let threshold = reference_ramp_energy_position(&window);

    assert!(threshold.is_finite());
    assert!(threshold > 0.0);
    assert!(threshold < 1_024.0);
}
