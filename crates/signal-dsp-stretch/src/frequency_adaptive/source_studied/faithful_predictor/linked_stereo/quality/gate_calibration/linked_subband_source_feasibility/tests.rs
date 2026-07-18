use super::*;

#[test]
#[ignore = "requires pinned SBSMS 2.3.0 source, adapter, and frozen development pack"]
#[cfg(not(debug_assertions))]
fn source_studied_linked_subband_sinusoidal_source_feasibility() {
    let result = review();
    eprintln!("linked_subband_source_feasibility {result:#?}");
    assert_eq!(result.revision, PINNED_REVISION);
    assert!(result.repeated);
    assert_eq!(result.stereo_rows, 48);
    assert!(result.maximum_tracks_per_time > 0);
    assert!(result.maximum_track_visits_per_output_read > 0);
    assert!(result.maximum_peak_rss_bytes > 0);
    assert_eq!(result.stereo_failures, 0);
    assert_eq!(result.local_consistency_failures, 6);
    assert_eq!(result.mono_hard_failures, 7);
    assert_eq!(result.mono_row_complete_regressions, 2);
    assert_eq!(result.metrics_worse_than_both_controls, 21);
    assert_eq!(result.maximum_tracks_per_time, 66);
    assert_eq!(result.maximum_track_visits_per_output_read, 10_728);
    assert_eq!(result.evidence_hash, 0x79b5_f7c1_4692_b8f5);
    assert_eq!(
        result.direction,
        LinkedSubbandFeasibilityDirection::CloseCandidate
    );
}
