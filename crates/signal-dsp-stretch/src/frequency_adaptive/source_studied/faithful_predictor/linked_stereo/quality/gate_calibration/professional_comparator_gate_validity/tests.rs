use super::*;

#[test]
#[ignore = "requires pinned Rubber Band 4.0.0 CLI"]
#[cfg(not(debug_assertions))]
fn source_studied_professional_comparator_gate_validity() {
    let result = review();
    eprintln!("professional_comparator_gate_validity {result:#?}");
    assert_eq!(result.rubber_band_version, "4.0.0");
    assert!(result.repeated);
    assert_eq!(result.stereo_rows, 48);
    assert_eq!(result.calibrated_failures, 0);
    assert_eq!(result.signal_relative_local_failures, 13);
    assert_eq!(
        result.mechanics_errors,
        [0.0, 0.0, 0.0, 0.0, 0.950164794921875, 0.04590606689453125]
    );
    assert_eq!(result.exact_mechanics_failures, 2);
    assert_eq!(result.binary_hash, 0x1c4b_0c5b_9f8f_b803);
    assert_eq!(result.input_hash, 0x4712_bef6_ac17_870e);
    assert_eq!(result.output_hash, 0x9575_2edc_43fc_6997);
    assert_eq!(result.command_hash, 0x628f_977d_4361_ad21);
    assert_eq!(result.measurement_hash, 0x8ec1_d715_8d12_09ca);
    assert_eq!(result.comparator_envelope_hash, 0x9574_e5e2_e53d_1a63);
    assert_eq!(result.evidence_hash, 0xb933_1f08_5832_6f19);
    assert_eq!(
        result.direction,
        ProfessionalComparatorGateDirection::ReviseLocalAndExactMechanics
    );
}
